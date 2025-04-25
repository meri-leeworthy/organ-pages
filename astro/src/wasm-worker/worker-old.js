// Web Worker for communicating with the WASM Actor

// We'll need to initialize the WASM module
import init, { Actor } from "../wasm/minissg/minissg.js"

// Global variables
let actor = null
let initialized = false
let initPromise = null

// Constants for IndexedDB
const DB_NAME = "organ-static-projects"
const STORE_NAME = "projects"
const META_STORE_NAME = "metadata"
const DB_VERSION = 1

/**
 * Initialize the WASM module
 * @returns {Promise<void>} - A promise that resolves when the WASM module is initialized
 */
async function initWasm() {
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
async function openDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)

    request.onerror = event => {
      console.error("[Worker] Error opening database:", event.target.error)
      reject("Error opening database")
    }

    request.onsuccess = event => {
      resolve(event.target.result)
    }

    request.onupgradeneeded = event => {
      const db = event.target.result
      // Create projects store if it doesn't exist
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: "id" })
      }

      // Create metadata store for active project IDs and project list
      if (!db.objectStoreNames.contains(META_STORE_NAME)) {
        db.createObjectStore(META_STORE_NAME)
      }
    }
  })
}

/**
 * Save project to IndexedDB
 * @param {string} id - The ID of the project
 * @param {string} binary - The binary data of the project
 * @param {object} metadata - The metadata of the project
 */
async function saveProjectToIDB(id, binary, metadata) {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([STORE_NAME], "readwrite")
    const store = transaction.objectStore(STORE_NAME)

    const project = {
      id,
      binary,
      metadata,
      updatedAt: Date.now(),
    }

    const request = store.put(project)

    request.onsuccess = () => resolve()
    request.onerror = event => {
      console.error("[Worker] Error saving project:", event.target.error)
      reject("Error saving project")
    }
  })
}

/**
 * Load project from IndexedDB
 * @param {string} id - The ID of the project to load
 * @returns {Promise<object>} - A promise that resolves to the project object
 */
async function loadProjectFromIDB(id) {
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
      console.error("[Worker] Error loading project:", event.target.error)
      reject("Error loading project")
    }
  })
}

/**
 * Get all project IDs from IndexedDB
 * @returns {Promise<string[]>} - A promise that resolves to an array of project IDs
 */
async function getAllProjectIds() {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([STORE_NAME], "readonly")
    const store = transaction.objectStore(STORE_NAME)

    const request = store.getAllKeys()

    request.onsuccess = () => {
      resolve(request.result)
    }

    request.onerror = event => {
      console.error("[Worker] Error getting project IDs:", event.target.error)
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
async function saveUIState(projects, siteId, themeId, activeProjectType) {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([META_STORE_NAME], "readwrite")
    const store = transaction.objectStore(META_STORE_NAME)

    store.put(projects, "projectList")
    store.put(siteId, "activeSiteId")
    store.put(themeId, "activeThemeId")
    store.put(activeProjectType, "activeProjectType")

    transaction.oncomplete = () => resolve()
    transaction.onerror = event => {
      console.error("[Worker] Error saving metadata:", event.target.error)
      reject("Error saving metadata")
    }
  })
}

/**
 * Load project metadata from IndexedDB
 * @returns {Promise<object>} - A promise that resolves to the project metadata object
 */
async function loadProjectMetadata() {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction([META_STORE_NAME], "readonly")
    const store = transaction.objectStore(META_STORE_NAME)

    const projectListRequest = store.get("projectList")
    const activeSiteIdRequest = store.get("activeSiteId")
    const activeThemeIdRequest = store.get("activeThemeId")
    const activeProjectTypeRequest = store.get("activeProjectType")

    const result = {
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
      console.error("[Worker] Error loading metadata:", event.target.error)
      reject("Error loading metadata")
    }
  })
}

/**
 * Handle the SaveState message
 * @param {object} payload - The payload containing the project ID and type
 * @returns {Promise<object>} - A promise that resolves to the result of the save operation
 */
async function handleSaveState(payload) {
  try {
    const { site_id, theme_id, active_project_type } = payload || {}

    console.log(
      `[Worker] Saving state with siteId: ${site_id}, themeId: ${theme_id}, activeProjectType: ${active_project_type}`
    )

    // Export projects using WASM
    // Convert the exported data to a format suitable for storage
    const siteExport = actor.process_message(
      JSON.stringify({ ExportProject: { project_type: "site" } })
    )

    const themeExport = actor.process_message(
      JSON.stringify({ ExportProject: { project_type: "theme" } })
    )

    console.log("siteExport", siteExport)
    console.log("themeExport", themeExport)

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
    const siteMetadata = {
      id: site_id,
      type: "site",
      createdAt: now,
      updatedAt: now,
    }

    const themeMetadata = {
      id: theme_id,
      type: "theme",
      createdAt: now,
      updatedAt: now,
    }

    // Save to IndexedDB
    await saveProjectToIDB(site_id, siteBinary, siteMetadata)
    await saveProjectToIDB(theme_id, themeBinary, themeMetadata)

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
    return { success: false, error: error.message }
  }
}

/**
 * Handle the LoadState message
 * @param {object} payload - The payload containing the project ID and type
 * @returns {Promise<object>} - A promise that resolves to the result of the load operation
 */
async function handleLoadState(payload) {
  try {
    console.log("[Worker] Loading state", payload)

    // Get project IDs
    let { siteId, themeId } = payload || {}

    // If no specific IDs provided, load from metadata
    if (!siteId || !themeId) {
      const storedIds = await loadProjectMetadata()
      siteId = siteId || storedIds.activeSiteId
      themeId = themeId || storedIds.activeThemeId

      console.log("[Worker] Loaded IDs from metadata:", storedIds)
    }

    console.log(
      `[Worker] Loading state with siteId: ${siteId}, themeId: ${themeId}`
    )

    // If no IDs are available, we can't load anything
    if (!siteId && !themeId) {
      return { success: false, error: "No projects to load" }
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
    await saveUIState(allProjectIds, siteProject?.id, themeProject?.id, "site")

    // Notify the main thread about successful load
    self.postMessage({
      type: "state_loaded",
      siteId: siteProject?.id,
      themeId: themeProject?.id,
    })

    return {
      success: true,
      siteId: siteProject?.id,
      themeId: themeProject?.id,
    }
  } catch (error) {
    console.error("[Worker] Error loading state:", error)
    return { success: false, error: error.message }
  }
}

/**
 * Handle messages from the main thread
 * @param {object} event - The event object containing the message data
 */
self.onmessage = async function (event) {
  const { id, action, payload } = event.data
  console.log(`[Worker] Received hi message - ID: ${id}, Action: ${action}`)

  try {
    // Make sure WASM is initialized
    console.log("[Worker] Ensuring WASM is initialized...")
    await initWasm()

    let response

    // Process the message based on the action type
    if (action === "process_message") {
      console.log("[Worker] Processing message with payload:", payload)

      // Check if this is a save/load state message that we handle directly
      if (payload && "SaveState" in payload) {
        console.log("[Worker] Handling SaveState message")
        response = await handleSaveState(payload.SaveState)
      } else if (payload && "LoadState" in payload) {
        console.log("[Worker] Handling LoadState message")
        response = await handleLoadState(payload.LoadState)
      } else {
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
      error: error.message || "Unknown error in worker",
    })
  }
}

// Report that the worker is ready
console.log("[Worker] Worker script loaded and ready")
self.postMessage({ type: "ready" })
