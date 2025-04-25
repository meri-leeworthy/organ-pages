export const isBrowser = (): boolean =>
  typeof window !== "undefined" && typeof indexedDB !== "undefined"

export const openDatabase = () => {
  if (!isBrowser()) {
    console.warn("IndexedDB is not available in this environment.")
    return Promise.reject(new Error("IndexedDB is not available"))
  }

  return new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open("organ_db", 2) // Ensure this version is higher if needed for idb schema changes

    request.onupgradeneeded = event => {
      const db = request.result
      if (!db.objectStoreNames.contains("doc")) {
        db.createObjectStore("doc") // Create the 'doc' object store // what is this?
      }
      if (!db.objectStoreNames.contains("projects")) {
        db.createObjectStore("projects")
      }
      if (!db.objectStoreNames.contains("files")) {
        db.createObjectStore("files")
      }
      if (!db.objectStoreNames.contains("assets")) {
        db.createObjectStore("assets") // Create the 'assets' object store
      }
    }

    request.onsuccess = () => {
      resolve(request.result)
    }

    request.onerror = () => {
      reject(request.error)
    }
  })
}

export const saveToIndexedDB = async (data: Uint8Array) => {
  try {
    const db = await openDatabase()
    const transaction = db.transaction("doc", "readwrite")
    const store = transaction.objectStore("doc")
    store.put(data, "snapshot")

    return new Promise<void>((resolve, reject) => {
      transaction.oncomplete = () => {
        resolve()
      }
      transaction.onerror = () => {
        reject(transaction.error)
      }
    })
  } catch (error) {
    console.error("IndexedDB error:", error)
    throw error
  }
}

export const saveAssetToIndexedDB = async (id: number, data: Blob) => {
  try {
    const db = await openDatabase()
    const transaction = db.transaction("assets", "readwrite")
    const store = transaction.objectStore("assets")
    store.put(data, id)

    console.log("Saved asset to IndexedDB:", id)

    return new Promise<void>((resolve, reject) => {
      transaction.oncomplete = () => {
        resolve()
      }
      transaction.onerror = () => {
        reject(transaction.error)
      }
    })
  } catch (error) {
    console.error("IndexedDB error:", error)
    throw error
  }
}

export const loadAssetFromIndexedDB = async (id: string) => {
  try {
    const db = await openDatabase()
    const transaction = db.transaction("assets", "readonly")
    const store = transaction.objectStore("assets")
    const request = store.get(id)

    return new Promise<Blob>((resolve, reject) => {
      request.onsuccess = () => {
        resolve(request.result as Blob)
      }
      request.onerror = () => {
        reject(request.error)
      }
    })
  } catch (error) {
    console.error("IndexedDB error:", error)
    throw error
  }
}

export const deleteAssetFromIndexedDB = async (id: string) => {
  try {
    const db = await openDatabase()
    const transaction = db.transaction("assets", "readwrite")
    const store = transaction.objectStore("assets")
    store.delete(id)

    console.log("Deleted asset from IndexedDB:", id)

    return new Promise<void>((resolve, reject) => {
      transaction.oncomplete = () => {
        resolve()
      }
      transaction.onerror = () => {
        reject(transaction.error)
      }
    })
  } catch (error) {
    console.error("IndexedDB error:", error)
    throw error
  }
}

export const loadFromIndexedDB = async () => {
  if (typeof indexedDB === "undefined") {
    throw new Error("IndexedDB is not available in this environment.")
  }

  const db = await openDatabase()
  const transaction = db.transaction("doc", "readonly")
  const store = transaction.objectStore("doc")
  const request = store.get("snapshot")

  return new Promise<Uint8Array>((resolve, reject) => {
    request.onsuccess = () => {
      resolve(request.result as Uint8Array)
    }
    request.onerror = () => {
      reject(request.error)
    }
  })
}

export async function iterateIndexedDB(
  storeName: string
): Promise<{ key: string; value: unknown }[]> {
  try {
    const db = await openDatabase()

    const transaction = db.transaction(storeName, "readonly")
    const objectStore = transaction.objectStore(storeName)

    const items: { key: string; value: unknown }[] = []
    const cursorRequest = objectStore.openCursor()

    return new Promise((resolve, reject) => {
      cursorRequest.onsuccess = function (event: Event) {
        const cursor = (event.target as IDBRequest).result
        if (cursor) {
          // Access the key and value
          console.log("Key:", cursor.key, "Value:", cursor.value)
          items.push({ key: cursor.key, value: cursor.value })

          // Move to the next entry
          cursor.continue()
        } else {
          // No more entries
          resolve(items)
        }
      }
      cursorRequest.onerror = function (event: Event) {
        reject(event.target)
      }
    })
  } catch (error) {
    console.error("IndexedDB error:", error)
    throw error
  }
}
