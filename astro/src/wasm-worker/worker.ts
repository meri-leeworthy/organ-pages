// Web Worker for communicating with the WASM Actor
import init, { Actor } from "../wasm/minissg/minissg.js"
import type { Message, ProjectType } from "./types"

// Global variables
let actor: Actor | null = null
let initialized = false
let initPromise: Promise<void> | null = null

// Constants for IndexedDB
const DB_NAME = "organ-static-projects"
const STORE_NAME = "projects"
const META_STORE_NAME = "metadata"
const DOCUMENT_STORE_NAME = "documents"
const DB_VERSION = 2 // Increased version for document store

// Type definitions for IndexedDB
interface ProjectData {
  id: string
  binary: Uint8Array
  metadata: ProjectMetadata
  updatedAt: number
}

interface ProjectMetadata {
  id: string
  type: string
  createdAt: number
  updatedAt: number
}

interface ProjectMetadataStore {
  projectList: string[]
  activeSiteId: string | null
  activeThemeId: string | null
  activeProjectType: string | null
}

/**
 * Initialize the WASM module
 * @returns {Promise<void>} - A promise that resolves when the WASM module is initialized
 */
async function initWasm(): Promise<void> {
  if (initialized) {
    return Promise.resolve()
  }

  if (initPromise) {
    return initPromise
  }

  initPromise = init()
    .then(() => {
      actor = new Actor()
      initialized = true
    })
    .catch(error => {
      console.error("[Worker] Failed to initialize WASM:", error)
      throw error
    })

  return initPromise
}

/**
 * Open and initialize IndexedDB
 * @returns {Promise<IDBDatabase>} - A promise that resolves to the IndexedDB database
 */
async function openDatabase(): Promise<IDBDatabase> {
  // For development only - force clear database to ensure new schema
  // const isDevelopment = true

  // if (isDevelopment) {
  //   try {
  //     console.log("[Worker] Development mode: clearing IndexedDB")
  //     await clearIndexedDB()
  //   } catch (err) {
  //     console.warn("[Worker] Error clearing IndexedDB:", err)
  //   }
  // }

  return new Promise((resolve, reject) => {
    console.log(
      `[Worker] Opening database ${DB_NAME} with version ${DB_VERSION}`
    )
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onerror = event => {
      console.error(
        "[Worker] Error opening database:",
        (event.target as IDBRequest).error
      )
      reject("Error opening database")
    }

    request.onsuccess = event => {
      const db = (event.target as IDBOpenDBRequest).result
      console.log(
        "[Worker] Database opened successfully. Object stores:",
        Array.from(db.objectStoreNames)
      )
      resolve(db)
    }

    request.onupgradeneeded = event => {
      console.log("[Worker] Database upgrade needed, creating stores")
      const db = (event.target as IDBOpenDBRequest).result
      const oldVersion = event.oldVersion

      // First-time creation
      if (oldVersion < 1) {
        console.log("[Worker] Creating initial stores (v1)")
        db.createObjectStore(STORE_NAME, { keyPath: "id" })
        db.createObjectStore(META_STORE_NAME)
      }

      // Upgrade to version 2: add document store
      if (oldVersion < 2) {
        console.log("[Worker] Upgrading to v2: adding document store")
        if (!db.objectStoreNames.contains(DOCUMENT_STORE_NAME)) {
          db.createObjectStore(DOCUMENT_STORE_NAME, { keyPath: "id" })
        }
      }

      console.log("[Worker] Database upgrade complete")
    }
  })
}

/**
 * Save project to IndexedDB
 * @param {string} id - The ID of the project
 * @param {string} binary - The binary data of the project
 * @param {object} metadata - The metadata of the project
 */
async function saveProjectToIDB(
  id: string,
  binary: Uint8Array,
  metadata: ProjectMetadata
): Promise<void> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([STORE_NAME], "readwrite")
    const store = transaction.objectStore(STORE_NAME)

    const project: ProjectData = {
      id,
      binary,
      metadata,
      updatedAt: Date.now(),
    }

    const request = store.put(project)

    request.onsuccess = () => resolve()
    request.onerror = event => {
      console.error(
        "[Worker] Error saving project:",
        (event.target as IDBRequest).error
      )
      reject("Error saving project")
    }
  })
}

/**
 * Load project from IndexedDB
 * @param {string} id - The ID of the project to load
 * @returns {Promise<object>} - A promise that resolves to the project object
 */
async function loadProjectFromIDB(id: string): Promise<ProjectData | null> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([STORE_NAME], "readonly")
    const store = transaction.objectStore(STORE_NAME)

    const request = store.get(id)

    request.onsuccess = () => {
      if (request.result) {
        // Convert binary data to Uint8Array if it's not already
        const result = request.result
        if (result.binary && !(result.binary instanceof Uint8Array)) {
          result.binary = new Uint8Array(result.binary)
        }
        resolve(result)
      } else {
        resolve(null) // Project not found
      }
    }

    request.onerror = event => {
      console.error(
        "[Worker] Error loading project:",
        (event.target as IDBRequest).error
      )
      reject("Error loading project")
    }
  })
}

/**
 * Get all project IDs from IndexedDB
 * @returns {Promise<string[]>} - A promise that resolves to an array of project IDs
 */
async function getAllProjectIds(): Promise<string[]> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([STORE_NAME], "readonly")
    const store = transaction.objectStore(STORE_NAME)

    const request = store.getAllKeys()

    request.onsuccess = () => {
      resolve(request.result as string[])
    }

    request.onerror = event => {
      console.error(
        "[Worker] Error getting project IDs:",
        (event.target as IDBRequest).error
      )
      reject("Error getting project IDs")
    }
  })
}

/**
 * Save project list and active IDs to metadata store in IndexedDB
 * @param {string[]} projects - The list of project IDs
 * @param {string} activeSiteId - The ID of the active site
 * @param {string} activeThemeId - The ID of the active theme
 * @param {string} activeProjectType - The type of the active project
 */
async function saveUIState(
  projects: string[],
  siteId?: string,
  themeId?: string,
  activeProjectType?: string
): Promise<void> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([META_STORE_NAME], "readwrite")
    const store = transaction.objectStore(META_STORE_NAME)

    store.put(projects, "projectList")
    if (siteId) store.put(siteId, "activeSiteId")
    if (themeId) store.put(themeId, "activeThemeId")
    if (activeProjectType) store.put(activeProjectType, "activeProjectType")

    transaction.oncomplete = () => resolve()
    transaction.onerror = event => {
      console.error(
        "[Worker] Error saving metadata:",
        (event.target as IDBTransaction).error
      )
      reject("Error saving metadata")
    }
  })
}

/**
 * Load project metadata from IndexedDB
 * @returns {Promise<object>} - A promise that resolves to the project metadata object
 */
async function loadProjectMetadata(): Promise<ProjectMetadataStore> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([META_STORE_NAME], "readonly")
    const store = transaction.objectStore(META_STORE_NAME)

    const projectListRequest = store.get("projectList")
    const activeSiteIdRequest = store.get("activeSiteId")
    const activeThemeIdRequest = store.get("activeThemeId")
    const activeProjectTypeRequest = store.get("activeProjectType")

    const result: ProjectMetadataStore = {
      projectList: [],
      activeSiteId: null,
      activeThemeId: null,
      activeProjectType: null,
    }

    projectListRequest.onsuccess = () => {
      if (projectListRequest.result) {
        result.projectList = projectListRequest.result
      }
    }

    activeSiteIdRequest.onsuccess = () => {
      if (activeSiteIdRequest.result) {
        result.activeSiteId = activeSiteIdRequest.result
      }
    }

    activeThemeIdRequest.onsuccess = () => {
      if (activeThemeIdRequest.result) {
        result.activeThemeId = activeThemeIdRequest.result
      }
    }

    activeProjectTypeRequest.onsuccess = () => {
      if (activeProjectTypeRequest.result) {
        result.activeProjectType = activeProjectTypeRequest.result
      }
    }

    transaction.oncomplete = () => resolve(result)
    transaction.onerror = event => {
      console.error(
        "[Worker] Error loading metadata:",
        (event.target as IDBTransaction).error
      )
      reject("Error loading metadata")
    }
  })
}

interface SaveStatePayload {
  site_id?: string
  theme_id?: string
  active_project_type?: ProjectType
}

interface LoadStatePayload {
  siteId?: string
  themeId?: string
}

interface SaveStateResult {
  success: boolean
  error?: string
}

interface LoadStateResult {
  success: boolean
  siteId?: string
  themeId?: string
  error?: string
}

/**
 * Handle the SaveState message
 * @param {object} payload - The payload containing the project ID and type
 * @returns {Promise<object>} - A promise that resolves to the result of the save operation
 */
async function handleSaveState(
  payload?: SaveStatePayload
): Promise<SaveStateResult> {
  try {
    const { site_id, theme_id, active_project_type } = payload || {}

    console.log(
      `[Worker] Saving state with siteId: ${site_id}, themeId: ${theme_id}, activeProjectType: ${active_project_type}`
    )

    if (!actor) {
      throw new Error("WASM actor not initialized")
    }

    // Export projects using WASM
    // Convert the exported data to a format suitable for storage
    const siteExport = actor.process_message(
      JSON.stringify({ ExportProject: { project_type: "site" } })
    )

    const themeExport = actor.process_message(
      JSON.stringify({ ExportProject: { project_type: "theme" } })
    )

    // console.log("[Worker] siteExport", siteExport)
    // console.log("[Worker] themeExport", themeExport)

    // Parse the responses
    const siteResponse = JSON.parse(siteExport)
    const themeResponse = JSON.parse(themeExport)
    if (siteResponse.Error) {
      throw new Error(siteResponse.Error)
    }

    if (themeResponse.Error) {
      throw new Error(themeResponse.Error)
    }

    // Extract the binary data
    const siteBinary = siteResponse.Success
    const themeBinary = themeResponse.Success
    // Create metadata
    const now = Date.now()
    const siteMetadata: ProjectMetadata = {
      id: site_id || "",
      type: "site",
      createdAt: now,
      updatedAt: now,
    }

    const themeMetadata: ProjectMetadata = {
      id: theme_id || "",
      type: "theme",
      createdAt: now,
      updatedAt: now,
    }

    // Save to IndexedDB
    if (site_id) {
      await saveProjectToIDB(site_id, siteBinary, siteMetadata)
    }

    if (theme_id) {
      await saveProjectToIDB(theme_id, themeBinary, themeMetadata)
    }

    // Update project list in metadata store
    const allProjectIds = await getAllProjectIds()
    await saveUIState(allProjectIds, site_id, theme_id, active_project_type)

    // Notify the main thread about successful save
    self.postMessage({
      type: "state_saved",
      siteId: site_id,
      themeId: theme_id,
    })

    return { success: true }
  } catch (error) {
    console.error("[Worker] Error saving state:", error)
    return {
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
    }
  }
}

/**
 * Handle the LoadState message
 * @param {object} payload - The payload containing the project ID and type
 * @returns {Promise<object>} - A promise that resolves to the result of the load operation
 */
async function handleLoadState(
  payload?: LoadStatePayload
): Promise<LoadStateResult> {
  try {
    console.log("[Worker] Loading state", payload)

    // Get project IDs
    let { siteId, themeId } = payload || {}

    // If no specific IDs provided, load from metadata
    if (!siteId || !themeId) {
      const storedIds = await loadProjectMetadata()
      siteId = siteId || storedIds.activeSiteId || undefined
      themeId = themeId || storedIds.activeThemeId || undefined

      console.log("[Worker] Loaded IDs from metadata:", storedIds)
    }

    console.log(
      `[Worker] Loading state with siteId: ${siteId}, themeId: ${themeId}`
    )

    // If no IDs are available, we can't load anything
    if (!siteId && !themeId) {
      return { success: false, error: "No projects to load" }
    }

    if (!actor) {
      throw new Error("WASM actor not initialized")
    }

    // Load projects from IndexedDB
    const siteProject = siteId ? await loadProjectFromIDB(siteId) : null
    const themeProject = themeId ? await loadProjectFromIDB(themeId) : null

    if (!siteProject && !themeProject) {
      return { success: false, error: "Projects not found in database" }
    }

    // Import projects using WASM
    if (siteProject) {
      const response = actor.process_message(
        JSON.stringify({
          ImportProject: {
            data: Array.from(new Uint8Array(siteProject.binary)), // Ensure binary data is Uint8Array and convert to array
            id: siteProject.metadata.id,
            project_type: "site",
            created: siteProject.metadata.createdAt,
            updated: siteProject.metadata.updatedAt,
          },
        })
      )
      const parsedResponse = JSON.parse(response)
      if (parsedResponse.Error) {
        throw new Error(`Error importing site: ${parsedResponse.Error}`)
      }
    }

    if (themeProject) {
      const response = actor.process_message(
        JSON.stringify({
          ImportProject: {
            data: Array.from(new Uint8Array(themeProject.binary)), // Ensure binary data is Uint8Array and convert to array
            id: themeProject.metadata.id,
            project_type: "theme",
            created: themeProject.metadata.createdAt,
            updated: themeProject.metadata.updatedAt,
          },
        })
      )
      const parsedResponse = JSON.parse(response)
      if (parsedResponse.Error) {
        throw new Error(`Error importing theme: ${parsedResponse.Error}`)
      }
    }

    // Update active project IDs in metadata store
    const allProjectIds = await getAllProjectIds()
    await saveUIState(
      allProjectIds,
      siteProject?.metadata.id,
      themeProject?.metadata.id,
      "site"
    )

    // Notify the main thread about successful load
    self.postMessage({
      type: "state_loaded",
      siteId: siteProject?.metadata.id,
      themeId: themeProject?.metadata.id,
    })

    return {
      success: true,
      siteId: siteProject?.metadata.id,
      themeId: themeProject?.metadata.id,
    }
  } catch (error) {
    console.error("[Worker] Error loading state:", error)
    return {
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
    }
  }
}

interface WorkerMessage {
  id: number
  action: string
  payload: Message | any
}

/**
 * Handle messages from the main thread
 * @param {MessageEvent<WorkerMessage>} event - The event object containing the message data
 */
self.onmessage = async function (event: MessageEvent<WorkerMessage>) {
  const { id, action, payload } = event.data
  console.log(`[Worker] Received message - ID: ${id}, Action: ${action}`)

  try {
    // Make sure WASM is initialized
    console.log("[Worker] Ensuring WASM is initialized...")
    await initWasm()

    let response: any

    // Process the message based on the action type
    if (action === "process_message") {
      console.log("[Worker] Processing message with payload:", payload)

      // Check if this is a document-related message
      if (payload && "InitializeDocument" in payload) {
        console.log("[Worker] Handling InitializeDocument message")
        const { document_id, schema } = payload.InitializeDocument

        if (!actor) {
          throw new Error("[Worker] WASM actor not initialized")
        }

        // Pass the initialization to the WASM actor
        const messageJson = JSON.stringify({
          InitializeDocument: {
            document_id: document_id,
            schema: schema,
          },
        })

        console.log(
          "[Worker] Sending InitializeDocument to Actor:",
          messageJson
        )

        // Check if WASM actor is properly initialized
        if (typeof actor.process_message !== "function") {
          console.error(
            "[Worker] Actor not properly initialized, process_message is not a function"
          )
          throw new Error("WASM Actor not properly initialized")
        }

        const responseJson = actor.process_message(messageJson)
        console.log(
          "[Worker] Actor response for InitializeDocument:",
          responseJson
        )

        response = JSON.parse(responseJson)

        // If WASM actor returns an error, fall back to temporary implementation
        if (response.Error) {
          throw new Error("[Worker] WASM actor returned error")
        }

        // If successful, store a reference to the document ID
        if (response && response.Success) {
          // We could track active documents here if needed
          console.log(
            "[Worker] Document initialized successfully:",
            document_id
          )
        }
      } else if (payload && "GetDocument" in payload) {
        console.log("[Worker] Handling GetDocument message")
        const { document_id } = payload.GetDocument

        if (!actor) {
          throw new Error("[Worker] WASM actor not initialized")
        }

        // Try using the WASM actor
        // try {
        const messageJson = JSON.stringify({
          GetDocument: {
            document_id: document_id,
          },
        })

        console.log("[Worker] Sending GetDocument to Actor:", messageJson)

        const responseJson = actor.process_message(messageJson)
        console.log("[Worker] Actor response for GetDocument:", responseJson)

        response = JSON.parse(responseJson)

        // If WASM actor returns an error, fall back to temporary implementation
        if (response.Error) {
          throw new Error("[Worker] WASM actor returned error")
        }
      } else if (payload && "GetFile" in payload) {
      } else if (payload && "GetFile" in payload) {
        console.log("[Worker] Handling GetFile message", payload)
        const { file_id, project_type, collection_name } = payload.GetFile
        console.log(
          "[Worker] Getting file",
          file_id,
          project_type,
          collection_name
        )

        if (!actor) {
          throw new Error("WASM actor not initialized")
        }

        // Pass the get document request to the WASM actor
        try {
          const messageJson = JSON.stringify({
            GetFile: {
              file_id: file_id,
              project_type: project_type,
              collection_name: collection_name,
            },
          })

          console.log("[Worker] Sending GetFile to Actor:", messageJson)
          const responseJson = actor.process_message(messageJson)
          console.log("[Worker] Actor response for GetFile:", responseJson)

          response = JSON.parse(responseJson)
        } catch (err: any) {
          console.error("[Worker] Error in GetFile:", err)
          throw new Error(`Error getting file: ${err.message || err}`)
        }
      } else if (payload && "ApplySteps" in payload) {
        console.log("[Worker] Handling ApplySteps message")
        const { document_id, steps, client_id, version } = payload.ApplySteps

        if (!actor) {
          throw new Error("[Worker] WASM actor not initialized")
        }

        // Try using the WASM actor
        const messageJson = JSON.stringify({
          ApplySteps: {
            document_id,
            steps,
            client_id,
            version,
          },
        })

        console.log("[Worker] Sending ApplySteps to Actor:", messageJson)
        const responseJson = actor.process_message(messageJson)
        console.log("[Worker] Actor response for ApplySteps:", responseJson)

        response = JSON.parse(responseJson)

        // If WASM actor returns an error, fall back to temporary implementation
        if (response.Error) {
          throw new Error("[Worker] WASM actor returned error")
        }
      }
      // Check if this is a save/load state message that we handle directly
      else if (payload && "SaveState" in payload) {
        console.log("[Worker] Handling SaveState message")
        response = await handleSaveState(payload.SaveState)
      } else if (payload && "LoadState" in payload) {
        console.log("[Worker] Handling LoadState message")
        response = await handleLoadState(payload.LoadState)
      } else {
        if (!actor) {
          throw new Error("WASM actor not initialized")
        }
        // The payload should be a serialized message for the Actor
        const messageJson = JSON.stringify(payload)
        console.log("[Worker] Sending to Actor:", messageJson)
        const responseJson = actor.process_message(messageJson)
        response = JSON.parse(responseJson)
        console.log("[Worker] Received response from Actor:", response)
      }
    } else {
      // Unknown action
      throw new Error(`Unknown action: ${action}`)
    }

    // Check if the response is an error
    if (response && typeof response === "object" && "Error" in response) {
      console.log(
        "[Worker] Sending error response back to main thread:",
        response.Error
      )
      self.postMessage({
        id,
        success: false,
        error: response.Error,
      })
    } else {
      // It's a success response
      console.log(
        "[Worker] Sending success response back to main thread:",
        response
      )
      self.postMessage({
        id,
        success: true,
        data: response,
      })
    }
  } catch (error) {
    // Send error back to the main thread
    console.error("[Worker] Error processing message:", error)
    self.postMessage({
      id,
      success: false,
      error: error instanceof Error ? error.message : "Unknown error in worker",
    })
  }
}

// Report that the worker is ready
console.log("[Worker] Worker script loaded and ready")
self.postMessage({ type: "ready" })
