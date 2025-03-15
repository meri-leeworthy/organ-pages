import React, {
  createContext,
  useContext,
  useReducer,
  useEffect,
  type ReactNode,
} from "react"
import { wasmClient } from "../wasm-worker/client"
import type {
  StoreState,
  StoreContextValue,
  Site,
  Theme,
  Collection,
  File,
  FieldDefinition,
  FileUpdate,
  ProjectType,
  ActiveFile,
} from "../wasm-worker/types"
import { Alert } from "@/components/ui/alert"

// LocalStorage keys
const LOCAL_STORAGE_KEYS = {
  ACTIVE_PROJECT_TYPE: "organ_active_project_type",
  SITE_ACTIVE_FILE: "organ_site_active_file",
  SITE_ACTIVE_PAGE: "organ_site_active_page",
  THEME_ACTIVE_FILE: "organ_theme_active_file",
}

// Helper to load from localStorage
const loadFromLocalStorage = <T,>(key: string): T | undefined => {
  if (typeof window === "undefined") return undefined

  const value = localStorage.getItem(key)
  if (!value) return undefined

  try {
    return JSON.parse(value) as T
  } catch (e) {
    console.error(`Error loading ${key} from localStorage:`, e)
    return undefined
  }
}

// Initial state with localStorage values
const initialState: StoreState = {
  initialized: false,
  loading: false,
  site: undefined,
  theme: undefined,
  collections: [],
  files: {},

  // Load UI state from localStorage
  activeProjectType:
    loadFromLocalStorage<ProjectType>(LOCAL_STORAGE_KEYS.ACTIVE_PROJECT_TYPE) ||
    "site",
  siteActiveFile: loadFromLocalStorage<ActiveFile>(
    LOCAL_STORAGE_KEYS.SITE_ACTIVE_FILE
  ),
  siteActivePage: loadFromLocalStorage<ActiveFile>(
    LOCAL_STORAGE_KEYS.SITE_ACTIVE_PAGE
  ),
  themeActiveFile: loadFromLocalStorage<ActiveFile>(
    LOCAL_STORAGE_KEYS.THEME_ACTIVE_FILE
  ),
}

console.log("initialState", initialState)

// Action types
type Action =
  | { type: "SET_LOADING"; payload: boolean }
  | { type: "SET_ERROR"; payload: string }
  | { type: "INITIALIZE_SUCCESS" }
  | { type: "SET_SITE"; payload: Site }
  | { type: "SET_THEME"; payload: Theme }
  | {
      type: "SET_COLLECTIONS"
      payload: { projectType: ProjectType; collections: Collection[] }
    }
  | {
      type: "ADD_COLLECTION"
      payload: { projectType: ProjectType; collection: Collection }
    }
  | {
      type: "SET_FILES"
      payload: {
        projectType: ProjectType
        collectionName: string
        files: File[]
      }
    }
  | {
      type: "ADD_FILE"
      payload: { projectType: ProjectType; collectionName: string; file: File }
    }
  | {
      type: "UPDATE_FILE"
      payload: {
        projectType: ProjectType
        collectionName: string
        fileId: string
        updates: any
      }
    }
  // UI state actions
  | { type: "SET_ACTIVE_PROJECT_TYPE"; payload: ProjectType }
  | { type: "SET_SITE_ACTIVE_FILE"; payload: ActiveFile | undefined }
  | { type: "SET_SITE_ACTIVE_PAGE"; payload: ActiveFile | undefined }
  | { type: "SET_THEME_ACTIVE_FILE"; payload: ActiveFile | undefined }

// Helper to save to localStorage
const saveToLocalStorage = <T,>(key: string, value: T | undefined): void => {
  if (typeof window === "undefined") return

  if (value === undefined) {
    localStorage.removeItem(key)
  } else {
    localStorage.setItem(key, JSON.stringify(value))
  }
}

// Reducer
function storeReducer(state: StoreState, action: Action): StoreState {
  switch (action.type) {
    case "SET_LOADING":
      return { ...state, loading: action.payload }

    case "SET_ERROR":
      return { ...state, error: action.payload, loading: false }

    case "INITIALIZE_SUCCESS":
      return {
        ...state,
        initialized: true,
        loading: false,
        error: undefined,
      }

    case "SET_SITE":
      return { ...state, site: action.payload }

    case "SET_THEME":
      return { ...state, theme: action.payload }

    case "SET_COLLECTIONS":
      return { ...state, collections: action.payload.collections }

    case "ADD_COLLECTION":
      return {
        ...state,
        collections: [...state.collections, action.payload.collection],
      }

    case "SET_FILES":
      return {
        ...state,
        files: {
          ...state.files,
          [action.payload.collectionName]: action.payload.files,
        },
      }

    case "ADD_FILE":
      return {
        ...state,
        files: {
          ...state.files,
          [action.payload.collectionName]: [
            ...(state.files[action.payload.collectionName] || []),
            action.payload.file,
          ],
        },
      }

    case "UPDATE_FILE":
      console.log("update file", action.payload)
      return {
        ...state,
        files: {
          ...state.files,
          [action.payload.collectionName]:
            state.files[action.payload.collectionName]?.map(file =>
              file.id === action.payload.fileId
                ? { ...file, ...action.payload.updates.Success }
                : file
            ) || [],
        },
      }

    // UI state actions
    case "SET_ACTIVE_PROJECT_TYPE": {
      // saveToLocalStorage(LOCAL_STORAGE_KEYS.ACTIVE_PROJECT_TYPE, action.payload)
      return { ...state, activeProjectType: action.payload }
    }

    case "SET_SITE_ACTIVE_FILE": {
      // saveToLocalStorage(LOCAL_STORAGE_KEYS.SITE_ACTIVE_FILE, action.payload)
      return { ...state, siteActiveFile: action.payload }
    }

    case "SET_SITE_ACTIVE_PAGE": {
      // saveToLocalStorage(LOCAL_STORAGE_KEYS.SITE_ACTIVE_PAGE, action.payload)
      return { ...state, siteActivePage: action.payload }
    }

    case "SET_THEME_ACTIVE_FILE": {
      // saveToLocalStorage(LOCAL_STORAGE_KEYS.THEME_ACTIVE_FILE, action.payload)
      return { ...state, themeActiveFile: action.payload }
    }

    default:
      return state
  }
}

// Create context
const StoreContext = createContext<StoreContextValue | undefined>(undefined)

// Provider component
export function WasmStoreProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(storeReducer, initialState)

  // Initialize the store
  const initialize = async () => {
    console.log("[StoreProvider] Starting initialization...")
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      // Initialize client and WASM module
      console.log("[StoreProvider] Initializing WASM client...")
      await wasmClient.init()
      console.log("[StoreProvider] WASM client initialized successfully")

      // Initialize default structure first (sets up the store internals)
      console.log("[StoreProvider] Initializing default structure...")
      const initResponse = await wasmClient.initDefault()
      console.log(
        "[StoreProvider] Default initialization response:",
        initResponse
      )

      // Try to load state from persistent storage
      console.log("[StoreProvider] Attempting to load state from IndexedDB...")
      const stateLoaded = await loadState()

      if (stateLoaded) {
        console.log("[StoreProvider] State loaded successfully, setup complete")
        dispatch({ type: "INITIALIZE_SUCCESS" })
        return // Early return since we successfully loaded from storage
      }

      console.log(
        "[StoreProvider] No saved state found, continuing with default state"
      )

      // If we reach here, there was no saved state
      // We have two options:
      // 1. We already have a site and theme in memory from initDefault()
      // 2. We need to create a new site and theme

      let siteId, themeId
      // Check if we have a theme
      try {
        console.log("[StoreProvider] Checking for theme...")
        const theme = await wasmClient.getTheme()

        if ("Error" in theme) {
          console.log(
            "[StoreProvider] No theme found, will create a default theme"
          )
          // Create a default theme
          const newTheme = await wasmClient.createTheme("Default Theme")
          if ("Error" in newTheme) throw newTheme
          themeId = newTheme.Success.id
          dispatch({ type: "SET_THEME", payload: newTheme.Success })
        } else {
          console.log("[StoreProvider] Found existing theme:", theme.Success)
          themeId = theme.Success.id
          dispatch({ type: "SET_THEME", payload: theme.Success })
        }
      } catch (err) {
        console.error("[StoreProvider] Error handling theme:", err)
        throw err
      }

      // Check if we have a site (requires a theme)
      if (themeId) {
        try {
          console.log("[StoreProvider] Checking for site...")
          const site = await wasmClient.getSite()

          if ("Error" in site) {
            console.log(
              "[StoreProvider] No site found, will create a default site"
            )
            // Create a default site
            const newSite = await wasmClient.createSite("Default Site", themeId)
            if ("Error" in newSite) throw newSite
            siteId = newSite.Success.id
            dispatch({ type: "SET_SITE", payload: newSite.Success })
          } else {
            console.log("[StoreProvider] Found existing site:", site.Success)
            siteId = site.Success.id
            dispatch({ type: "SET_SITE", payload: site.Success })
          }
        } catch (err) {
          console.error("[StoreProvider] Error handling site:", err)
          throw err
        }
      }

      // Save state if we have both site and theme
      if (state.site && state.theme) {
        const activeProjectId =
          state.activeProjectType === "site" ? state.site.id : state.theme.id
        try {
          console.log("[StoreProvider] Saving initial state...")
          await saveState(
            state.site.id,
            state.theme.id,
            state.activeProjectType
          )
        } catch (err) {
          console.error("[StoreProvider] Error saving initial state:", err)
          // Continue anyway - saving state is not critical for initialization
        }
      }

      dispatch({ type: "INITIALIZE_SUCCESS" })
    } catch (error) {
      console.error("[StoreProvider] Initialization error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error
            ? error.message
            : "Unknown error during initialization",
      })
    }
  }

  // Create a new site
  const createSite = async (name: string, themeId: string) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const site = await wasmClient.createSite(name, themeId)

      if ("Error" in site) throw site

      if (site) {
        dispatch({ type: "SET_SITE", payload: site.Success })
        dispatch({ type: "SET_LOADING", payload: false })
        return site.Success
      } else {
        throw new Error("Failed to create site")
      }
    } catch (error) {
      console.error("Create site error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to create site",
      })
      throw error
    }
  }

  // Get current site
  const getSite = async () => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })
      const site = await wasmClient.getSite()
      if ("Error" in site) throw site
      dispatch({ type: "SET_SITE", payload: site.Success })
      dispatch({ type: "SET_LOADING", payload: false })
      return site.Success
    } catch (error) {
      console.error("Get site error:", error)
      dispatch({
        type: "SET_ERROR",
        payload: error instanceof Error ? error.message : "Failed to get site",
      })
      throw error
    }
  }

  // Create a new theme
  const createTheme = async (name: string) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const theme = await wasmClient.createTheme(name)
      if ("Error" in theme) throw theme

      if (theme) {
        dispatch({ type: "SET_THEME", payload: theme.Success })
        dispatch({ type: "SET_LOADING", payload: false })
        return theme.Success
      } else {
        throw new Error("Failed to create theme")
      }
    } catch (error) {
      console.error("Create theme error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to create theme",
      })
      throw error
    }
  }

  // Get current theme
  const getTheme = async () => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })
      const theme = await wasmClient.getTheme()
      if ("Error" in theme) throw theme
      dispatch({ type: "SET_THEME", payload: theme.Success })
      dispatch({ type: "SET_LOADING", payload: false })
      return theme.Success
    } catch (error) {
      console.error("Get theme error:", error)
      dispatch({
        type: "SET_ERROR",
        payload: error instanceof Error ? error.message : "Failed to get theme",
      })
      throw error
    }
  }

  // Add a collection
  const addCollection = async (
    projectType: ProjectType,
    name: string,
    fields: FieldDefinition[]
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const collection = await wasmClient.addCollection(
        projectType,
        name,
        fields
      )

      if ("Error" in collection) throw collection

      if (collection) {
        // Add to state
        dispatch({
          type: "ADD_COLLECTION",
          payload: { projectType, collection: collection.Success },
        })

        dispatch({ type: "SET_LOADING", payload: false })
        return collection.Success
      } else {
        throw new Error("Failed to add collection")
      }
    } catch (error) {
      console.error("Add collection error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to add collection",
      })
      throw error
    }
  }

  // Get a collection
  const getCollection = async (projectType: ProjectType, name: string) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const collection = await wasmClient.getCollection(projectType, name)
      if ("Error" in collection) throw collection

      dispatch({ type: "SET_LOADING", payload: false })
      return collection.Success
    } catch (error) {
      console.error("Get collection error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to get collection",
      })
      throw error
    }
  }

  // List collections
  const listCollections = async (projectTypeParam?: ProjectType) => {
    const projectType = projectTypeParam || state.activeProjectType
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const collections = await wasmClient.listCollections(projectType)

      if ("Error" in collections) throw collections

      dispatch({
        type: "SET_COLLECTIONS",
        payload: { projectType, collections: collections.Success },
      })

      dispatch({ type: "SET_LOADING", payload: false })
      return collections.Success
    } catch (error) {
      console.error("List collections error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to list collections",
      })
      throw error
    }
  }

  // Create a file
  const createFile = async (
    projectType: ProjectType,
    collectionName: string,
    name: string
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const file = await wasmClient.createFile(
        projectType,
        collectionName,
        name
      )

      if ("Error" in file) throw file

      // Add to state
      dispatch({
        type: "ADD_FILE",
        payload: { projectType, collectionName, file: file.Success },
      })

      dispatch({ type: "SET_LOADING", payload: false })
      return file.Success
    } catch (error) {
      console.error("Create file error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to create file",
      })
      throw error
    }
  }

  // Update a file
  const updateFile = async (
    projectType: ProjectType,
    collectionName: string,
    fileId: string,
    update: FileUpdate
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      console.log("update", update)

      await wasmClient.updateFile(projectType, collectionName, fileId, update)
      if ("Error" in update) throw update

      // Get updated file
      const file = await wasmClient.getFile(projectType, collectionName, fileId)
      if ("Error" in file) throw file
      // Update in state
      dispatch({
        type: "UPDATE_FILE",
        payload: {
          projectType,
          collectionName,
          fileId,
          updates: file,
        },
      })

      dispatch({ type: "SET_LOADING", payload: false })
    } catch (error) {
      console.error("Update file error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to update file",
      })
      throw error
    }
  }

  // Get a file
  const getFile = async (
    projectType: ProjectType,
    collectionName: string,
    fileId: string
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const file = await wasmClient.getFile(projectType, collectionName, fileId)
      if ("Error" in file) throw file

      dispatch({ type: "SET_LOADING", payload: false })
      return file.Success
    } catch (error) {
      console.error("Get file error:", error)
      dispatch({
        type: "SET_ERROR",
        payload: error instanceof Error ? error.message : "Failed to get file",
      })
      throw error
    }
  }

  // List files
  const listFiles = async (
    projectType: ProjectType,
    collectionName: string
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })

      const files = await wasmClient.listFiles(projectType, collectionName)

      if ("Error" in files) throw files

      dispatch({
        type: "SET_FILES",
        payload: { projectType, collectionName, files: files.Success },
      })

      dispatch({ type: "SET_LOADING", payload: false })
      return files.Success
    } catch (error) {
      console.error("List files error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to list files",
      })
      throw error
    }
  }

  // Render a file
  const renderFile = async (fileId: string, context: any) => {
    try {
      const renderedFile = await wasmClient.renderFile(fileId, context)
      if ("Error" in renderedFile) throw renderedFile
      return renderedFile.Success
    } catch (error) {
      console.error("Render file error:", error)
      throw error
    }
  }

  // Save state
  const saveState = async (
    siteId: string,
    themeId: string,
    activeProjectType: ProjectType
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })
      console.log(
        "[StoreProvider] Saving state - activeProjectType:",
        activeProjectType
      )

      const response = await wasmClient.saveState(
        siteId,
        themeId,
        activeProjectType
      )
      if ("Error" in response) throw response

      console.log("[StoreProvider] State saved successfully")
      dispatch({ type: "SET_LOADING", payload: false })

      return response
    } catch (error) {
      console.error("Save state error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to save state",
      })
      throw error
    }
  }

  // Load state
  const loadState = async (
    specificSiteId?: string,
    specificThemeId?: string
  ) => {
    try {
      dispatch({ type: "SET_LOADING", payload: true })
      console.log(
        "[StoreProvider] Loading state",
        specificSiteId
          ? `with specific siteId: ${specificSiteId}`
          : "with stored Ids if available",
        specificThemeId ? `with specific themeId: ${specificThemeId}` : ""
      )

      const response = await wasmClient.loadState(
        specificSiteId,
        specificThemeId
      )

      // If we get an error response, it means no state was found in storage
      // This is not necessarily an error condition - it might just mean this is the first run
      if ("Error" in response) {
        console.log(
          "[StoreProvider] No state found or error loading state:",
          response.Error
        )
        dispatch({ type: "SET_LOADING", payload: false })
        return false
      }

      console.log(
        "[StoreProvider] State loaded successfully:",
        response.Success
      )

      // Refresh site and theme
      try {
        const site = await wasmClient.getSite()
        if ("Error" in site) {
          console.log("[StoreProvider] No site found after loading state")
        } else {
          console.log(
            "[StoreProvider] Retrieved site after state load:",
            site.Success
          )
          dispatch({ type: "SET_SITE", payload: site.Success })
        }
      } catch (err) {
        console.log(
          "[StoreProvider] Error getting site after loading state:",
          err
        )
      }

      try {
        const theme = await wasmClient.getTheme()
        if ("Error" in theme) {
          console.log("[StoreProvider] No theme found after loading state")
        } else {
          console.log(
            "[StoreProvider] Retrieved theme after state load:",
            theme.Success
          )
          dispatch({ type: "SET_THEME", payload: theme.Success })
        }
      } catch (err) {
        console.log(
          "[StoreProvider] Error getting theme after loading state:",
          err
        )
      }

      dispatch({ type: "SET_LOADING", payload: false })
      return true
    } catch (error) {
      console.error("[StoreProvider] Load state error:", error)
      dispatch({
        type: "SET_ERROR",
        payload:
          error instanceof Error ? error.message : "Failed to load state",
      })
      return false
    }
  }

  // Initialize on mount
  useEffect(() => {
    if (!state.initialized) {
      initialize()
    }
  }, [])

  // UI state setters
  const setActiveProjectType = (projectType: ProjectType) => {
    dispatch({ type: "SET_ACTIVE_PROJECT_TYPE", payload: projectType })
  }

  const setSiteActiveFile = (activeFile: ActiveFile | undefined) => {
    dispatch({ type: "SET_SITE_ACTIVE_FILE", payload: activeFile })
  }

  const setSiteActivePage = (
    activePage: Omit<ActiveFile, "projectType"> | undefined
  ) => {
    dispatch({
      type: "SET_SITE_ACTIVE_PAGE",
      payload: activePage ? { ...activePage, projectType: "site" } : undefined,
    })
  }

  const setThemeActiveFile = (activeFile: ActiveFile | undefined) => {
    dispatch({ type: "SET_THEME_ACTIVE_FILE", payload: activeFile })
  }

  // Value object
  const value: StoreContextValue = {
    state,
    initialize,
    createSite,
    getSite,
    createTheme,
    getTheme,
    addCollection,
    getCollection,
    listCollections,
    createFile,
    updateFile,
    getFile,
    listFiles,
    renderFile,
    saveState,
    loadState,
    // UI state setters
    setActiveProjectType,
    setSiteActiveFile,
    setSiteActivePage,
    setThemeActiveFile,
  }

  // Render loading/error states
  if (state.loading && !state.initialized) {
    return (
      <div className="flex items-center justify-center w-screen h-screen gap-2 bg-zinc-800">
        <Alert className="w-64">Loading Organ...</Alert>
      </div>
    )
  }

  if (state.error) {
    return (
      <div className="flex items-center justify-center w-screen h-screen gap-2 bg-zinc-800">
        <Alert variant="destructive" className="w-64">
          Error initializing Organ: {state.error}
        </Alert>
      </div>
    )
  }

  return <StoreContext.Provider value={value}>{children}</StoreContext.Provider>
}

// Hook for using the store context
export function useStore() {
  const context = useContext(StoreContext)
  if (context === undefined) {
    throw new Error("useStore must be used within a WasmStoreProvider")
  }
  return context
}

// Hook for accessing and changing the active page for rendering
export function useActivePage() {
  const { state, setSiteActivePage } = useStore()
  return {
    activePage: state.siteActivePage,
    setActivePage: setSiteActivePage,
  }
}
