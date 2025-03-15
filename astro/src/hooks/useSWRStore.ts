import useSWR from "swr"
import type {
  Collection,
  File,
  ProjectType,
  ActiveFile,
} from "@/wasm-worker/types"
import { useStore } from "@/components/StoreProvider"

// SWR fetcher functions that use Store functions
const createFetcher =
  <T, A extends any[]>(fn: (...args: A) => Promise<T>) =>
  (...args: A): Promise<T> =>
    fn(...args)

// Hook to fetch collections for the current active context
export function useCollections(projectType?: ProjectType) {
  const { state, listCollections } = useStore()

  // If no context is provided, use the active context from the store
  const targetProjectType = projectType || state.activeProjectType

  // Only run the query if we have a context type
  const enabled = !!targetProjectType

  // SWR hook for collections
  const { data, error, isLoading, mutate } = useSWR(
    enabled ? ["collections", targetProjectType] : null,
    enabled ? () => listCollections(targetProjectType as ProjectType) : null,
    {
      // Revalidate when window gets focus
      revalidateOnFocus: true,
      // Keep data even when window is unfocused
      revalidateOnReconnect: true,
      // Refresh data with the specified interval to ensure it stays fresh
      refreshInterval: 0, // Disable automatic refreshing
    }
  )

  return {
    collections: data || [],
    isLoading,
    error,
    refreshCollections: mutate,
  }
}

export function useCollection(
  projectType?: ProjectType,
  collectionName?: string
) {
  const { state, getCollection } = useStore()
  const targetProjectType = projectType || state.activeProjectType

  const enabled = !!targetProjectType && !!collectionName

  const { data, error, isLoading, mutate } = useSWR(
    enabled ? ["collection", targetProjectType, collectionName] : null,
    enabled ? () => getCollection(targetProjectType, collectionName) : null,
    {
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      refreshInterval: 0, // Disable automatic refreshing
    }
  )

  return {
    collection: data || null,
    isLoading,
    error,
    refreshCollection: mutate,
  }
}

// Hook to fetch files for a specific collection
export function useFiles(projectType?: ProjectType, collectionName?: string) {
  const { state, listFiles } = useStore()

  // If no context is provided, use the active context from the store
  const targetProjectType = projectType || state.activeProjectType

  // Only run the query if we have a context type and collection name
  const enabled = !!targetProjectType && !!collectionName

  // SWR hook for files
  const { data, error, isLoading, mutate } = useSWR(
    enabled ? ["files", targetProjectType, collectionName] : null,
    enabled
      ? () =>
          listFiles(targetProjectType as ProjectType, collectionName as string)
      : null,
    {
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      refreshInterval: 0, // Disable automatic refreshing
    }
  )

  return {
    files: data || [],
    isLoading,
    error,
    refreshFiles: mutate,
  }
}

export function useFile(
  fileId?: string,
  collectionName?: string,
  projectType?: ProjectType
): {
  file: File | null
  isLoading: boolean
  error: Error | null
  refreshFile: () => void
} {
  const { state, getFile } = useStore()
  const targetProjectType = projectType || state.activeProjectType

  const enabled = !!targetProjectType && !!collectionName && !!fileId

  const { data, error, isLoading, mutate } = useSWR(
    enabled ? [fileId, targetProjectType, collectionName] : null,
    enabled ? () => getFile(targetProjectType, collectionName, fileId) : null,
    {
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      refreshInterval: 0, // Disable automatic refreshing
    }
  )

  return {
    file: data || null,
    isLoading,
    error,
    refreshFile: mutate,
  }
}

// Hook for the active file in the current context
export function useActiveFile() {
  const { state, setSiteActiveFile, setThemeActiveFile, updateFile } =
    useStore()

  // Get the appropriate active file based on the current context
  const activeFileId =
    state.activeProjectType === "site"
      ? state.siteActiveFile
      : state.themeActiveFile

  // Function to set the active file in the current context
  const setActiveFile = (file: ActiveFile | undefined) => {
    if (state.activeProjectType === "site") {
      setSiteActiveFile(file)
    } else if (state.activeProjectType === "theme") {
      setThemeActiveFile(file)
    }
  }

  const { file, isLoading, error, refreshFile } = useFile(
    activeFileId?.fileId,
    activeFileId?.collectionName,
    activeFileId?.projectType
  )

  // Function to update the active file's fields
  const updateActiveFile = async (fieldName: string, value: any) => {
    if (!activeFileId || !file) return

    try {
      // Handle special case for body field
      if (fieldName === "body") {
        await updateFile(
          activeFileId.projectType,
          activeFileId.collectionName,
          activeFileId.fileId,
          {
            SetField: {
              name: fieldName,
              value: JSON.stringify({ type: "html", content: value }),
            },
          }
        )
      } else if (typeof value === "string") {
        await updateFile(
          activeFileId.projectType,
          activeFileId.collectionName,
          activeFileId.fileId,
          { SetField: { name: fieldName, value } }
        )
      } else {
        await updateFile(
          activeFileId.projectType,
          activeFileId.collectionName,
          activeFileId.fileId,
          { SetField: { name: fieldName, value: JSON.stringify(value) } }
        )
      }

      // Refresh the file data
      refreshFile()
      return true
    } catch (error) {
      console.error("Failed to update file:", error)
      throw error
    }
  }

  return {
    file,
    activeFileId,
    setActiveFile,
    updateActiveFile,
    isLoading,
    error,
    refreshFile,
  }
}
