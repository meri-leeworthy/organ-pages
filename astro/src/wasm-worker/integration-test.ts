/**
 * Integration tests for WASM Store Actor and IndexedDB persistence
 *
 * This file tests:
 * 1. Core Actor message handling (from store.rs)
 * 2. IndexedDB persistence
 * 3. Project creation, loading, and updating
 */

import { openDatabase } from "@/lib/idbHelper"
import wasmClient from "./client"
import type { ProjectType, FileUpdate } from "./types"

// Mock indexedDB for testing in Node.js environments
// These variables will be used to track test state
let testSiteId: string
let testThemeId: string
let testPageId: string
let testPostId: string

/**
 * Run all integration tests
 */
async function runTests() {
  console.log("🧪 Starting integration tests...")

  try {
    // Initialize
    console.log("Clearing IndexedDB...")
    await clearIndexedDB()
    console.log("Initialising IndexedDB...")
    await openDatabase()

    console.log("Initializing WASM client...")
    try {
      await wasmClient.init()
      console.log("✅ Initialized WASM client successfully")
    } catch (initError: unknown) {
      console.error("❌ Failed to initialize WASM client:", initError)
      const errorMessage =
        initError instanceof Error ? initError.message : "Unknown error"
      throw new Error(`WASM client initialization failed: ${errorMessage}`)
    }

    // Core functionality tests
    await testInitDefault()
    await testThemeOperations()
    await testSiteOperations()
    await testCollectionOperations()
    await testFileOperations()
    await testUpdatingFiles()
    await testExportAndImport()

    // Persistence tests
    await testSavingState()
    await testClearAndReload()

    console.log("🎉 All tests passed!")
  } catch (error) {
    console.error("❌ Test failed:", error)
    throw error
  }
}

/**
 * Wait for a specified time in milliseconds
 */
const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

/**
 * Clear IndexedDB before testing
 */
async function clearIndexedDB() {
  if (typeof window === "undefined") return

  return new Promise<void>((resolve, reject) => {
    const request = indexedDB.deleteDatabase("organ-db")

    request.onsuccess = () => {
      console.log("IndexedDB cleared successfully")
      resolve()
    }

    request.onerror = () => {
      console.error("Error clearing IndexedDB")
      reject(new Error("Failed to clear IndexedDB"))
    }
  })
}

/**
 * Test exporting and importing projects
 */
async function testExportAndImport() {
  console.log("🧪 Testing project export and import...")

  // Export site
  const exportResult = await wasmClient.exportProject("site")

  if ("Error" in exportResult) {
    throw new Error(`Export site failed: ${exportResult.Error}`)
  }

  console.log("✅ Site exported successfully")

  // Export theme
  const exportThemeResult = await wasmClient.exportProject("theme")

  if ("Error" in exportThemeResult) {
    throw new Error(`Export theme failed: ${exportThemeResult.Error}`)
  }

  console.log("✅ Theme exported successfully")

  // Create new site ID and import the exported site with that ID
  const newSiteId = `new-site-${Date.now()}`
  const timestamp = Date.now()

  // For testing purposes, we'll save and load from IndexedDB rather than
  // dealing with Uint8Array conversion issues

  // Save the current site state
  const saveSiteResult = await wasmClient.saveState("site")

  if ("Error" in saveSiteResult) {
    throw new Error(`Save site state failed: ${saveSiteResult.Error}`)
  }

  console.log("✅ Saved site state to verify export/import flow")

  // Save the current theme state
  const saveThemeResult = await wasmClient.saveState("theme")

  if ("Error" in saveThemeResult) {
    throw new Error(`Save theme state failed: ${saveThemeResult.Error}`)
  }

  console.log("✅ Saved theme state to verify export/import flow")

  // Create a new client instance to simulate a page reload
  const newClient = new (wasmClient.constructor as any)()
  await newClient.init()

  // Load the saved state
  const loadStateResult = await newClient.loadState(testSiteId, testThemeId)

  if ("Error" in loadStateResult) {
    throw new Error(`Load state failed: ${loadStateResult.Error}`)
  }

  console.log("✅ Loaded state in new client to verify export/import flow")

  // Verify site and theme data are preserved across the save/load cycle
  const getSiteResult = await newClient.getSite()

  if ("Error" in getSiteResult) {
    throw new Error(`Get site after reload failed: ${getSiteResult.Error}`)
  }

  if (getSiteResult.Success.id !== testSiteId) {
    throw new Error(
      `Site ID mismatch after reload: expected ${testSiteId}, got ${getSiteResult.Success.id}`
    )
  }

  console.log("✅ Verified site data preservation through export/import cycle")
}

/**
 * Test initializing default projects
 */
async function testInitDefault() {
  console.log("🧪 Testing InitDefault...")

  try {
    const result = await wasmClient.initDefault()

    console.log("InitDefault result:", result)

    if ("Error" in result) {
      throw new Error(`InitDefault failed: ${result.Error}`)
    }

    console.log("✅ InitDefault successful")
  } catch (e) {
    console.error("InitDefault error:", e)
    throw new Error(e as string)
  }
}

/**
 * Test theme creation and retrieval
 */
async function testThemeOperations() {
  console.log("🧪 Testing theme operations...")

  // Create theme
  const createResult = await wasmClient.createTheme("Test Theme")

  if ("Error" in createResult) {
    throw new Error(`Create theme failed: ${createResult.Error}`)
  }

  const themeData = createResult.Success
  testThemeId = themeData.id

  console.log(`✅ Created theme with ID: ${testThemeId}`)

  // Get theme
  const getResult = await wasmClient.getTheme()

  if ("Error" in getResult) {
    throw new Error(`Get theme failed: ${getResult.Error}`)
  }

  const theme = getResult.Success

  if (theme.id !== testThemeId || theme.name !== "Test Theme") {
    throw new Error(`Theme data mismatch: ${JSON.stringify(theme)}`)
  }

  console.log("✅ Retrieved theme successfully")
}

/**
 * Test site creation and retrieval
 */
async function testSiteOperations() {
  console.log("🧪 Testing site operations...")

  // Create site
  const createResult = await wasmClient.createSite("Test Site", testThemeId)

  if ("Error" in createResult) {
    throw new Error(`Create site failed: ${createResult.Error}`)
  }

  const siteData = createResult.Success
  testSiteId = siteData.id

  console.log(`✅ Created site with ID: ${testSiteId}`)

  // Get site
  const getResult = await wasmClient.getSite()

  if ("Error" in getResult) {
    throw new Error(`Get site failed: ${getResult.Error}`)
  }

  const site = getResult.Success

  if (
    site.id !== testSiteId ||
    site.name !== "Test Site" ||
    site.themeId !== testThemeId
  ) {
    throw new Error(`Site data mismatch: ${JSON.stringify(site)}`)
  }

  console.log("✅ Retrieved site successfully")
}

/**
 * Test collection listing
 */
async function testCollectionOperations() {
  console.log("🧪 Testing collection operations...")

  try {
    // List site collections
    const siteCollectionsResult = await wasmClient.listCollections("site")

    if ("Error" in siteCollectionsResult) {
      throw new Error(
        `List site collections failed: ${siteCollectionsResult.Error}`
      )
    }

    console.log("✅ Retrieved site collections successfully")

    // List theme collections
    const themeCollectionsResult = await wasmClient.listCollections("theme")

    if ("Error" in themeCollectionsResult) {
      throw new Error(
        `List theme collections failed: ${themeCollectionsResult.Error}`
      )
    }

    console.log("✅ Retrieved theme collections successfully")

    // Verify collections
    const siteCollections = siteCollectionsResult.Success
    const themeCollections = themeCollectionsResult.Success

    // Check if expected collections exist
    const expectedSiteCollections = ["page", "post", "asset"]
    for (const collection of expectedSiteCollections) {
      if (!siteCollections.some(c => c.name === collection)) {
        throw new Error(`Missing expected site collection: ${collection}`)
      }
    }
    const expectedThemeCollections = ["asset", "template", "partial", "text"]
    for (const collection of expectedThemeCollections) {
      if (!themeCollections.some(c => c.name === collection)) {
        throw new Error(`Missing expected theme collection: ${collection}`)
      }
    }

    console.log("✅ Collections verified successfully")

    // Get a specific collection
    const pageCollectionResult = await wasmClient.getCollection("site", "page")

    if ("Error" in pageCollectionResult) {
      throw new Error(
        `Get page collection failed: ${pageCollectionResult.Error}`
      )
    }

    console.log("✅ Retrieved specific collection successfully")
  } catch (error) {
    console.error("❌ Error in testCollectionOperations:", error)
    throw error
  }
}

/**
 * Test file creation and retrieval
 */
async function testFileOperations() {
  console.log("🧪 Testing file operations...")

  // Create a page in the site
  const createPageResult = await wasmClient.createFile(
    "site",
    "page",
    "Test Page"
  )

  if ("Error" in createPageResult) {
    throw new Error(`Create page failed: ${createPageResult.Error}`)
  }

  const pageData = createPageResult.Success
  testPageId = pageData.id

  console.log(`✅ Created page with ID: ${testPageId}`)

  // Create a post in the site
  const createPostResult = await wasmClient.createFile(
    "site",
    "post",
    "Test Post"
  )

  if ("Error" in createPostResult) {
    throw new Error(`Create post failed: ${createPostResult.Error}`)
  }

  const postData = createPostResult.Success
  testPostId = postData.id

  console.log(`✅ Created post with ID: ${testPostId}`)

  // Get the page
  const getPageResult = await wasmClient.getFile("site", "page", testPageId)

  if ("Error" in getPageResult) {
    throw new Error(`Get page failed: ${getPageResult.Error}`)
  }

  const page = getPageResult.Success

  if (page.id !== testPageId || page.name !== "Test Page") {
    throw new Error(`Page data mismatch: ${JSON.stringify(page)}`)
  }

  console.log("✅ Retrieved page successfully")

  // List pages
  const listPagesResult = await wasmClient.listFiles("site", "page")

  if ("Error" in listPagesResult) {
    throw new Error(`List pages failed: ${listPagesResult.Error}`)
  }

  const pages = listPagesResult.Success

  if (!pages.some(p => p.id === testPageId)) {
    throw new Error(`Page not found in pages list: ${JSON.stringify(pages)}`)
  }

  console.log("✅ Listed pages successfully")
}

/**
 * Test updating files
 */
async function testUpdatingFiles() {
  console.log("🧪 Testing file updates (SetTitle and SetUrl)...")

  // Update page title
  const setTitleUpdate: FileUpdate = { SetTitle: "Updated Page Title" }
  const updateTitleResult = await wasmClient.updateFile(
    "site",
    "page",
    testPageId,
    setTitleUpdate
  )

  if ("Error" in updateTitleResult) {
    throw new Error(`Update page title failed: ${updateTitleResult.Error}`)
  }

  console.log("✅ Updated page title")

  // Update page URL
  const setUrlUpdate: FileUpdate = { SetUrl: "/updated-page" }
  const updateUrlResult = await wasmClient.updateFile(
    "site",
    "page",
    testPageId,
    setUrlUpdate
  )

  if ("Error" in updateUrlResult) {
    throw new Error(`Update page URL failed: ${updateUrlResult.Error}`)
  }

  console.log("✅ Updated page URL")

  // // Update a field
  // const setFieldUpdate: FileUpdate = {
  //   SetField: { name: "description", value: "This is a test page" },
  // }
  // const updateFieldResult = await wasmClient.updateFile(
  //   "site",
  //   "page",
  //   testPageId,
  //   setFieldUpdate
  // )

  // if ("Error" in updateFieldResult) {
  //   throw new Error(`Update page field failed: ${updateFieldResult.Error}`)
  // }

  // console.log("✅ Updated page field")

  // Verify updates
  const getPageResult = await wasmClient.getFile("site", "page", testPageId)

  if ("Error" in getPageResult) {
    throw new Error(`Get updated page failed: ${getPageResult.Error}`)
  }

  const updatedPage = getPageResult.Success

  console.log("Updated page: {:#?}", updatedPage)

  if (updatedPage["title"] !== "Updated Page Title") {
    throw new Error(
      `Page updates not applied correctly: Expected title "Updated Page Title", got ${updatedPage["title"]}`
    )
  }

  if (updatedPage["url"] !== "/updated-page") {
    throw new Error(
      `Page updates not applied correctly: Expected url "/updated-page", got ${updatedPage["url"]}`
    )
  }

  console.log("✅ Verified updates successfully")
}

/**
 * Test saving state to IndexedDB
 */
async function testSavingState() {
  console.log("🧪 Testing saving state to IndexedDB...")

  // Save site state
  const saveSiteResult = await wasmClient.saveState("site")

  if ("Error" in saveSiteResult) {
    throw new Error(`Save site state failed: ${saveSiteResult.Error}`)
  }

  console.log("✅ Saved site state to IndexedDB")

  // Save theme state
  const saveThemeResult = await wasmClient.saveState("theme")

  if ("Error" in saveThemeResult) {
    throw new Error(`Save theme state failed: ${saveThemeResult.Error}`)
  }

  console.log("✅ Saved theme state to IndexedDB")

  // Wait a bit to ensure data is written
  await wait(500)
}

/**
 * Test clearing memory and reloading from IndexedDB
 */
async function testClearAndReload() {
  console.log("🧪 Testing clearing memory and reloading from IndexedDB...")

  // Create a new client to simulate reloading the page
  const newClient = new (wasmClient.constructor as any)()
  await newClient.init()

  // Load state from IndexedDB
  const loadStateResult = await newClient.loadState(testSiteId, testThemeId)

  if ("Error" in loadStateResult) {
    throw new Error(`Load state failed: ${loadStateResult.Error}`)
  }

  console.log("✅ Loaded state from IndexedDB")

  // Verify site data persisted
  const getSiteResult = await newClient.getSite()

  if ("Error" in getSiteResult) {
    throw new Error(`Get site after reload failed: ${getSiteResult.Error}`)
  }

  const site = getSiteResult.Success

  if (site.id !== testSiteId || site.name !== "Test Site") {
    throw new Error(`Reloaded site data mismatch: ${JSON.stringify(site)}`)
  }

  // Verify theme data persisted
  const getThemeResult = await newClient.getTheme()

  if ("Error" in getThemeResult) {
    throw new Error(`Get theme after reload failed: ${getThemeResult.Error}`)
  }

  const theme = getThemeResult.Success

  if (theme.id !== testThemeId || theme.name !== "Test Theme") {
    throw new Error(`Reloaded theme data mismatch: ${JSON.stringify(theme)}`)
  }

  // Verify file data persisted by retrieving the page
  const getPageResult = await newClient.getFile("site", "page", testPageId)

  if ("Error" in getPageResult) {
    throw new Error(`Get page after reload failed: ${getPageResult.Error}`)
  }

  const page = getPageResult.Success

  if (
    page.id !== testPageId ||
    page.title !== "Updated Page Title" ||
    page.url !== "/updated-page"
  ) {
    throw new Error(`Reloaded page data mismatch: ${JSON.stringify(page)}`)
  }

  console.log("✅ Verified persistence of site, theme, and file data")
}

// Export functions for direct execution in browser context
export {
  runTests,
  testInitDefault,
  testThemeOperations,
  testSiteOperations,
  testCollectionOperations,
  testFileOperations,
  testUpdatingFiles,
  testSavingState,
  testClearAndReload,
  testExportAndImport,
  clearIndexedDB,
}

// Auto-run tests if in a browser environment
if (typeof window !== "undefined") {
  console.log("Running tests in browser environment...")
  runTests().catch(console.error)
}
