// indexeddb.js
export function saveToIndexedDB(dbName, storeName, key, value) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, 1)

    request.onupgradeneeded = function (event) {
      const db = event.target.result
      if (!db.objectStoreNames.contains(storeName)) {
        db.createObjectStore(storeName)
      }
    }

    request.onsuccess = function (event) {
      const db = event.target.result
      const tx = db.transaction(storeName, "readwrite")
      const store = tx.objectStore(storeName)
      store.put(value, key)
      tx.oncomplete = () => resolve(true)
      tx.onerror = () => reject(tx.error)
    }

    request.onerror = () => reject(request.error)
  })
}

export function loadFromIndexedDB(dbName, storeName, key) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, 1)

    request.onsuccess = function (event) {
      const db = event.target.result
      const tx = db.transaction(storeName, "readonly")
      const store = tx.objectStore(storeName)
      const getRequest = store.get(key)

      getRequest.onsuccess = () => resolve(getRequest.result)
      getRequest.onerror = () => reject(getRequest.error)
    }

    request.onerror = () => reject(request.error)
  })
}
