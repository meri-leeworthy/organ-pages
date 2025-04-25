// Web Worker for communicating with the WASM Store
import init, { Store } from "/wasm/minissg/minissg.js"

// Global variables
let store = null
let initialized = false
let initPromise = null

// Constants for IndexedDB
const DB_NAME = "organ_db"
const STORE_NAME = "projects"
const META_STORE_NAME = "metadata" // or "doc"?
const FILES_STORE_NAME = "files"
const DB_VERSION = 2 // Increased version for document store

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
      store = new Store()
      initialized = true
    })
    .catch(error => {
      console.error("[Worker] Failed to initialize WASM:", error)
      throw error
    })

  return initPromise
}

/**
 * Handle messages from the main thread
 */
self.onmessage = async function (event) {
  const { id, action, payload } = event.data
  console.log(`[Worker] Received message - ID: ${id}, Action: ${action}`)

  try {
    // Make sure WASM is initialized
    console.log("[Worker] Ensuring WASM is initialized...")
    await initWasm()

    let response

    // Process the message
    if (action === "process_message") {
      if (!store) {
        throw new Error("WASM Store not initialized")
      }

      // The payload should be a serialized message for the Store
      const messageJson = JSON.stringify(payload)
      console.log("[Worker] Sending to Store:", messageJson)
      const responseJson = await store.process_message(messageJson)
      response = JSON.parse(responseJson)
      console.log("[Worker] Received response from Store:", response)
    } else {
      // Unknown action
      throw new Error(`Unknown action: ${action}`)
    }

    // Send success response
    self.postMessage({
      id,
      success: true,
      data: response,
    })
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
