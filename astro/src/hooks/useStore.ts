// hooks/useSql.ts
import { useEffect, useState, useRef, useMemo } from "react"
import { isBrowser, loadFromIndexedDB, saveToIndexedDB } from "../lib/idbHelper"
import { useDebounce } from "./useDebounce"
import { Store } from "@/lib/Store"

export interface UseStoreResult {
  loading: boolean
  error: Error | null
  store: Store
}

const useStore = (): UseStoreResult => {
  const [store, setStore] = useState<Store>(useMemo(() => new Store(), []))
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<Error | null>(null)
  const [saving, setSaving] = useState<number>(0)
  const debouncedSaving = useDebounce(saving, 2000)

  // Reference to track if the component is mounted
  const isMountedRef = useRef(true)

  // Debounced save function for saving to IndexedDB
  useEffect(() => {
    async function save() {
      if (isBrowser()) {
        try {
          store.export()
          console.log("Database saved to IndexedDB.")
        } catch (saveError) {
          console.error("Failed to save database to IndexedDB:", saveError)
        }
      }
    }
    save()
  }, [debouncedSaving])

  useEffect(() => {
    if (!isBrowser()) {
      // If not in the browser, skip document initialization
      console.warn(
        "Skipping document initialization in non-browser environment."
      )
      setLoading(false)
      return
    }

    async function load() {
      try {
        // Load database from IndexedDB if available
        const data = await loadFromIndexedDB()
        if (data) {
          console.log("Importing data from IndexedDB", data)
          store.import(data)
        } else {
          // should check remote here in case there are projects that need to be downloaded
          // and then initialize demo
          console.log("Initializing default store")
          store.initDefault()
        }

        console.log("Document initialized.")
        if (isMountedRef.current) {
          setLoading(false)
        }
      } catch (err) {
        if (isMountedRef.current) {
          setError(err as Error)
          setLoading(false)
        }
      }
    }
    load()
  }, [])

  return { loading, error, store }
}

export default useStore
