import type {
  Message,
  Response,
  Site,
  Theme,
  Collection,
  File,
  FieldDefinition,
  FileUpdate,
  ProjectType,
  DocumentData,
} from "./types"

/**
 * Client API for communicating with the WASM Actor via Web Worker
 */
class WasmClient {
  private worker: Worker | null = null
  private messageCounter: number = 0
  private callbacks: Map<
    number,
    {
      resolve: (value: unknown) => void
      reject: (reason: Error) => void
    }
  > = new Map()
  private isReady: boolean = false
  private readyPromise: Promise<void> | null = null
  private readyResolver: (() => void) | null = null

  // Event listeners for state changes
  private stateChangeListeners: Map<string, Set<Function>> = new Map()

  /**
   * Initialize the worker and wait for it to be ready
   * //TODO: maybe this should also initialise the IDB store?
   * @returns Promise that resolves when the worker is ready
   */
  public async init(): Promise<void> {
    if (this.isReady) return
    if (this.readyPromise) return this.readyPromise

    this.readyPromise = new Promise<void>(resolve => {
      this.readyResolver = resolve
    })

    // Create a new worker
    if (typeof window !== "undefined") {
      // Use the worker from the public directory with .js extension
      this.worker = new Worker(
        new URL("/wasm-worker/worker.js", window.location.origin),
        {
          type: "module",
        }
      )

      // Handle errors
      this.worker.onerror = (error: ErrorEvent) => {
        console.error("Worker error:", error)
      }

      // Set up message handler
      this.worker.onmessage = this.handleWorkerMessage.bind(this)
    }

    return this.readyPromise
  }

  /**
   * Send a message to the actor via the worker
   * @param message Message object to send to the WASM actor
   * @returns Promise that resolves with the response data
   */
  public async sendMessage<T = unknown>(
    message: Message
  ): Promise<Response<T>> {
    await this.init()

    return new Promise<Response<T>>((resolve, reject) => {
      if (!this.worker) {
        reject(new Error("Worker not initialized"))
        return
      }

      // Assign a unique ID to this message
      const id = this.messageCounter++

      // Store the callbacks
      this.callbacks.set(id, {
        resolve: (value: unknown) => resolve(value as Response<T>),
        reject,
      })

      try {
        // Ensure the message is cloneable
        const cloneableMessage = JSON.parse(JSON.stringify(message))

        // Send the message to the worker
        this.worker.postMessage({
          id,
          action: "process_message",
          payload: cloneableMessage,
        })
      } catch (error) {
        console.error("Error cloning message:", error)
        this.callbacks.delete(id)
        reject(
          new Error(
            `Failed to clone message: ${error instanceof Error ? error.message : "Unknown error"}`
          )
        )
      }
    })
  }

  /**
   * Handle messages from the worker
   * @param event MessageEvent containing the worker's response
   */
  private handleWorkerMessage(
    event: MessageEvent<{
      id?: number
      success?: boolean
      data?: unknown
      error?: string
      type?: string
      siteId?: string
      themeId?: string
      documentId?: string
      steps?: any[]
      source?: string
      version?: number
    }>
  ) {
    const {
      id,
      success,
      data,
      error,
      type,
      siteId,
      themeId,
      documentId,
      steps,
      source,
      version,
    } = event.data

    // Handle ready message
    if (type === "ready") {
      this.isReady = true
      if (this.readyResolver) {
        this.readyResolver()
        this.readyResolver = null
      }
      return
    }

    // Handle state changed events
    if (type === "state_saved" || type === "state_loaded") {
      this.notifyStateChangeListeners(type, { siteId, themeId })
      return
    }

    // Handle document change events
    if (type === "document_changed" && documentId) {
      const eventType = `document_changed:${documentId}`
      this.notifyStateChangeListeners(eventType, {
        steps,
        source: source || "unknown",
        version: version || 0,
      })
      return
    }

    // Skip processing if no id (likely an event notification)
    if (id === undefined) return

    // Handle response messages
    const callbacks = this.callbacks.get(id)
    if (callbacks) {
      this.callbacks.delete(id)

      if (success) {
        callbacks.resolve(data)
      } else {
        callbacks.reject(new Error(error || "Unknown error"))
      }
    }
  }

  /**
   * Subscribe to state change events
   * @param eventType The event type to listen for ('state_saved' or 'state_loaded')
   * @param callback Function to call when the event occurs
   * @returns Function to unsubscribe
   */
  public onStateChange(
    eventType: "state_saved" | "state_loaded",
    callback: (data: { siteId?: string; themeId?: string }) => void
  ): () => void {
    if (!this.stateChangeListeners.has(eventType)) {
      this.stateChangeListeners.set(eventType, new Set())
    }

    this.stateChangeListeners.get(eventType)!.add(callback)

    // Return unsubscribe function
    return () => {
      const listeners = this.stateChangeListeners.get(eventType)
      if (listeners) {
        listeners.delete(callback)
      }
    }
  }

  /**
   * Subscribe to document change events
   * @param documentId The ID of the document to listen for changes
   * @param callback Function to call when document changes occur
   * @returns Function to unsubscribe
   */
  public onDocumentChange(
    documentId: string,
    callback: (data: { steps: any[]; source: string; version: number }) => void
  ): () => void {
    const eventType = `document_changed:${documentId}`

    if (!this.stateChangeListeners.has(eventType)) {
      this.stateChangeListeners.set(eventType, new Set())
    }

    this.stateChangeListeners.get(eventType)!.add(callback)

    // Return unsubscribe function
    return () => {
      const listeners = this.stateChangeListeners.get(eventType)
      if (listeners) {
        listeners.delete(callback)
        if (listeners.size === 0) {
          // If no more listeners, we could notify the worker to stop sending updates
          // This is an optimization we can add later
        }
      }
    }
  }

  /**
   * Notify all listeners of a state change event
   * @param eventType The event type
   * @param data The event data
   */
  private notifyStateChangeListeners(eventType: string, data: any): void {
    const listeners = this.stateChangeListeners.get(eventType)
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          console.error(
            `Error in state change listener for ${eventType}:`,
            error
          )
        }
      })
    }
  }

  /**
   * High-level methods for common operations
   */

  // Project operations
  /**
   * Initialize default project structure
   * @returns Promise that resolves when initialization is complete
   */
  public async initDefault(): Promise<Response<void>> {
    const result = await this.sendMessage<void>({ InitDefault: null })
    console.log("initDefault result:", result)
    return result
  }

  /**
   * Create a new site
   * @param name Name of the site
   * @param themeId ID of the theme to use
   * @returns Promise resolving to the created Site object
   */
  public async createSite(
    name: string,
    themeId: string
  ): Promise<Response<Site>> {
    return this.sendMessage<Site>({
      CreateSite: {
        name,
        theme_id: themeId,
      },
    })
  }

  /**
   * Get the current site
   * @returns Promise resolving to the Site object
   */
  public async getSite(): Promise<Response<Site>> {
    return this.sendMessage<Site>({ GetSite: null })
  }

  /**
   * Create a new theme
   * @param name Name of the theme
   * @returns Promise resolving to the created Theme object
   */
  public async createTheme(name: string): Promise<Response<Theme>> {
    return this.sendMessage<Theme>({
      CreateTheme: {
        name,
      },
    })
  }

  /**
   * Get the current theme
   * @returns Promise resolving to the Theme object
   */
  public async getTheme(): Promise<Response<Theme>> {
    return this.sendMessage<Theme>({ GetTheme: null })
  }

  // Collection operations
  /**
   * Add a new collection
   * @param projectType Whether to add to 'site' or 'theme'
   * @param name Name of the collection
   * @param fields Array of field definitions for the collection
   * @returns Promise resolving to the created Collection object
   */
  public async addCollection(
    projectType: ProjectType,
    name: string,
    fields: FieldDefinition[]
  ): Promise<Response<Collection>> {
    return this.sendMessage<Collection>({
      AddCollection: {
        project_type: projectType,
        name,
        fields,
      },
    })
  }

  /**
   * Get a collection by name
   * @param projectType Whether to get from 'site' or 'theme'
   * @param name Name of the collection
   * @returns Promise resolving to the Collection object
   */
  public async getCollection(
    projectType: ProjectType,
    name: string
  ): Promise<Response<Collection>> {
    return this.sendMessage<Collection>({
      GetCollection: {
        project_type: projectType,
        name,
      },
    })
  }

  /**
   * List all collections
   * @param projectType Whether to list from 'site' or 'theme'
   * @returns Promise resolving to an array of Collection objects
   */
  public async listCollections(
    projectType: ProjectType
  ): Promise<Response<Collection[]>> {
    return this.sendMessage<Collection[]>({
      ListCollections: {
        project_type: projectType,
      },
    })
  }

  // File operations
  /**
   * Create a new file in a collection
   * @param projectType Whether to add to 'site' or 'theme'
   * @param collectionName Name of the collection
   * @param name Name of the file
   * @returns Promise resolving to the created File object
   */
  public async createFile(
    projectType: ProjectType,
    collectionName: string,
    name: string
  ): Promise<Response<File>> {
    return this.sendMessage<File>({
      CreateFile: {
        project_type: projectType,
        collection_name: collectionName,
        name,
      },
    })
  }

  /**
   * Update a file in a collection
   * @param projectType Whether to update in 'site' or 'theme'
   * @param collectionName Name of the collection
   * @param fileId ID of the file to update
   * @param update FileUpdate object containing the update to apply
   * @returns Promise that resolves when the update is complete
   */
  public async updateFile(
    projectType: ProjectType,
    collectionName: string,
    fileId: string,
    update: FileUpdate
  ): Promise<Response<void>> {
    return this.sendMessage<void>({
      UpdateFile: {
        project_type: projectType,
        collection_name: collectionName,
        file_id: fileId,
        updates: update,
      },
    })
  }

  /**
   * Get a file by ID
   * @param projectType Whether to get from 'site' or 'theme'
   * @param collectionName Name of the collection
   * @param fileId ID of the file
   * @returns Promise resolving to the File object
   */
  public async getFile(
    projectType: ProjectType,
    collectionName: string,
    fileId: string
  ): Promise<Response<File>> {
    return this.sendMessage<File>({
      GetFile: {
        project_type: projectType,
        collection_name: collectionName,
        file_id: fileId,
      },
    })
  }

  /**
   * List all files in a collection
   * @param projectType Whether to list from 'site' or 'theme'
   * @param collectionName Name of the collection
   * @returns Promise resolving to an array of File objects
   */
  public async listFiles(
    projectType: ProjectType,
    collectionName: string
  ): Promise<Response<File[]>> {
    return this.sendMessage<File[]>({
      ListFiles: {
        project_type: projectType,
        collection_name: collectionName,
      },
    })
  }

  public async initializeDocument(
    documentId: string,
    schema: string
  ): Promise<Response<void>> {
    return this.sendMessage<void>({
      InitializeDocument: { document_id: documentId, schema },
    })
  }

  public async getDocument(
    documentId: string
  ): Promise<Response<DocumentData>> {
    return this.sendMessage<DocumentData>({
      GetDocument: { document_id: documentId },
    })
  }

  public async applySteps(
    documentId: string,
    steps: any[],
    version: number
  ): Promise<Response<void>> {
    return this.sendMessage<void>({
      ApplySteps: {
        document_id: documentId,
        steps,
        version,
      },
    })
  }

  // Storage operations
  /**
   * Save the current state to persistent storage
   * @param siteId The ID of the active site project
   * @param themeId The ID of the active theme project
   * @returns Promise that resolves when the state is saved
   */
  public async saveState(projectType?: ProjectType): Promise<Response<void>> {
    return this.sendMessage<void>({
      SaveState: {
        project_type: projectType ? projectType : undefined,
      },
    })
  }

  /**
   * Load state from persistent storage
   * @param siteId Optional ID of a specific site to load
   * @param themeId Optional ID of a specific theme to load
   * @returns Promise that resolves when the state is loaded
   */
  public async loadState(
    siteId?: string,
    themeId?: string
  ): Promise<Response<void>> {
    return this.sendMessage<void>({
      LoadState: {
        site_id: siteId ? siteId : undefined,
        theme_id: themeId ? themeId : undefined,
      },
    })
  }

  /**
   * Export a project to a serialized string
   * @param id ID of the project to export
   * @returns Promise resolving to the exported project data string
   */
  public async exportProject(
    projectType: ProjectType
  ): Promise<Response<string>> {
    return this.sendMessage<string>({
      ExportProject: { project_type: projectType },
    })
  }

  /**
   * Import a project from binary data
   * @param data Binary project data (array of numbers or Uint8Array)
   * @param id ID of the project
   * @param projectType Type of the project
   * @param created Creation timestamp
   * @param updated Update timestamp
   * @returns Promise resolving to the imported Site or Theme object
   */
  public async importProject(
    data: number[] | Uint8Array,
    id: string,
    projectType: ProjectType,
    created: number,
    updated: number
  ): Promise<Response<Site | Theme>> {
    // Ensure data is an array of numbers
    const dataArray = data instanceof Uint8Array ? Array.from(data) : data
    return this.sendMessage<Site | Theme>({
      ImportProject: {
        data: dataArray,
        id,
        project_type: projectType,
        created,
        updated,
      },
    })
  }

  // Rendering operations
  /**
   * Render a file with the provided context
   * @param fileId ID of the file to render
   * @param context Context object to use for rendering
   * @returns Promise resolving to the rendered content string
   */
  public async renderFile(
    fileId: string,
    context: Record<string, unknown>
  ): Promise<Response<string>> {
    return this.sendMessage<string>({
      RenderFile: {
        file_id: fileId,
        context,
      },
    })
  }
}

// Create and export singleton instance
export const wasmClient = new WasmClient()
export default wasmClient
