/**
 * Example implementation of using the WASM persistence system
 * This file demonstrates how to use the wasmClient to save and load projects.
 */

import wasmClient from "../wasm-worker/client"
import type { Response, Site, Theme } from "../wasm-worker/types"

/**
 * Initialize the persistence system and register event listeners
 */
export async function setupPersistence() {
  // Wait for the WASM and worker to be initialized
  await wasmClient.init()

  // Register event listeners for state changes
  const savedListener = wasmClient.onStateChange(
    "state_saved",
    ({ siteId, themeId }) => {
      console.log(
        `State saved successfully! Site: ${siteId}, Theme: ${themeId}`
      )
    }
  )

  const loadedListener = wasmClient.onStateChange(
    "state_loaded",
    ({ siteId, themeId }) => {
      console.log(
        `State loaded successfully! Site: ${siteId}, Theme: ${themeId}`
      )
    }
  )

  // Return unsubscribe functions for cleanup
  return {
    unsubscribe: () => {
      savedListener()
      loadedListener()
    },
  }
}

/**
 * Save the current state of the application
 * @returns Promise that resolves when the state is saved
 */
// export async function saveCurrentState(): Promise<Response<void>> {
//   try {
//     // Get current project id and type
//     const activeProjectId =
//       state.activeProjectType === "site" ? state.site.id : state.theme.id
//     if (!activeProjectId) return

//     if ('Error' in siteResponse || 'Error' in themeResponse) {
//       const error = 'Error' in siteResponse ? siteResponse.Error : themeResponse.Error
//       throw new Error(`Failed to get projects: ${error}`)
//     }

//     const site = siteResponse.Success as Site
//     const theme = themeResponse.Success as Theme

//     console.log(`Saving state - Site: ${site.id}, Theme: ${theme.id}`)

//     // Save state with the project IDs
//     return await wasmClient.saveState()
//   } catch (error) {
//     console.error('Error saving state:', error)
//     return { Error: error instanceof Error ? error.message : String(error) }
//   }
// }

/**
 * Load the state of the application
 * @param siteId Optional ID of the site to load
 * @param themeId Optional ID of the theme to load
 * @returns Promise that resolves when the state is loaded
 */
export async function loadState(
  siteId?: string,
  themeId?: string
): Promise<Response<void>> {
  try {
    console.log(
      `Loading state - Site: ${siteId || "latest"}, Theme: ${themeId || "latest"}`
    )

    // Load state, either with specific IDs or the latest active projects
    return await wasmClient.loadState(siteId, themeId)
  } catch (error) {
    console.error("Error loading state:", error)
    return { Error: error instanceof Error ? error.message : String(error) }
  }
}

/**
 * Example of using the persistence system
 * Demonstrates a complete workflow of saving and loading state
 */
export async function demonstratePersistence() {
  try {
    // 1. Set up persistence and event listeners
    const { unsubscribe } = await setupPersistence()

    // 2. Create initial projects if needed
    await wasmClient.initDefault()

    // 3. Save the current state
    // const saveResult = await saveCurrentState()
    // if ("Error" in saveResult) {
    //   throw new Error(`Failed to save state: ${saveResult.Error}`)
    // }

    // 4. Simulate an application restart
    console.log("Simulating application restart...")

    // 5. Load the state (latest active projects)
    const loadResult = await loadState()
    if ("Error" in loadResult) {
      throw new Error(`Failed to load state: ${loadResult.Error}`)
    }

    // 6. Clean up event listeners when no longer needed
    unsubscribe()

    console.log("Persistence demonstration completed successfully!")
  } catch (error) {
    console.error("Persistence demonstration failed:", error)
  }
}
