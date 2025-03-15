// hooks/useStore.ts
import { useEffect, useState, useRef } from "react"
import { isBrowser, loadFromIndexedDB } from "../lib/idbHelper"
import { useDebounce } from "./useDebounce"
import { createStore } from "@/lib/LoroModel"
import { StoreAdapter, createStoreAdapter } from "@/lib/StoreAdapter"

export interface UseStoreResult {
  loading: boolean
  error: Error | null
  store: StoreAdapter
}

const useStore = (): UseStoreResult => {
  const [storeAdapter, setStoreAdapter] = useState<StoreAdapter | null>(null)
  const [loading, setLoading] = useState<boolean>(true)
  const [error, setError] = useState<Error | null>(null)
  const [saving, setSaving] = useState<number>(0)
  const debouncedSaving = useDebounce(saving, 2000)

  // Reference to track if the component is mounted
  const isMountedRef = useRef(true)

  // Initialize store
  useEffect(() => {
    async function initStore() {
      try {
        const wasmStore = await createStore();
        const adapter = createStoreAdapter(wasmStore);
        setStoreAdapter(adapter);
      } catch (err) {
        setError(err as Error);
      }
    }
    
    initStore();
    
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  // Debounced save function for saving to IndexedDB
  useEffect(() => {
    if (!storeAdapter) return;
    
    async function save() {
      if (isBrowser()) {
        try {
          storeAdapter.export();
          console.log("Database saved to IndexedDB.");
        } catch (saveError) {
          console.error("Failed to save database to IndexedDB:", saveError);
        }
      }
    }
    
    save();
  }, [debouncedSaving, storeAdapter]);

  // Load data or initialize default
  useEffect(() => {
    if (!storeAdapter || !isBrowser()) {
      if (!isBrowser()) {
        console.warn("Skipping document initialization in non-browser environment.");
        setLoading(false);
      }
      return;
    }

    async function load() {
      try {
        // Load database from IndexedDB if available
        const data = await loadFromIndexedDB();
        if (data) {
          console.log("Importing data from IndexedDB");
          storeAdapter.import(data);
        } else {
          // Initialize default store
          console.log("Initializing default store");
          storeAdapter.initDefault();
        }

        console.log("Document initialized.");
        if (isMountedRef.current) {
          setLoading(false);
        }
      } catch (err) {
        if (isMountedRef.current) {
          setError(err as Error);
          setLoading(false);
        }
      }
    }
    
    load();
  }, [storeAdapter]);

  // Register for updates from the store
  useEffect(() => {
    if (!storeAdapter) return;
    
    const callback = () => {
      setSaving(prev => prev + 1);
    };
    
    const listenerFunc = storeAdapter.on("update", callback);
    
    return () => {
      storeAdapter.off("update", listenerFunc);
    };
  }, [storeAdapter]);

  if (!storeAdapter) {
    return { 
      loading: true, 
      error: error || new Error("Store failed to initialize"), 
      store: {} as StoreAdapter 
    };
  }

  return { loading, error, store: storeAdapter };
}

export default useStore
