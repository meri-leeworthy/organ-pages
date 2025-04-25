// indexeddb.js
const IDB_VERSION = 2

export function saveToIndexedDB(dbName, storeName, key, value) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, IDB_VERSION)

    request.onupgradeneeded = function (event) {
      const db = event.target.result
      if (!db.objectStoreNames.contains(storeName)) {
        // Specify keyPath as null to allow external keys
        db.createObjectStore(storeName, { keyPath: null })
      }
    }

    request.onsuccess = function (event) {
      const db = event.target.result
      const tx = db.transaction(storeName, "readwrite")
      const store = tx.objectStore(storeName)

      // Log the key and value for debugging
      console.log("Saving with key:", key)
      console.log(
        "Value type:",
        typeof value,
        value instanceof Uint8Array ? "Uint8Array" : ""
      )

      const putRequest = store.put(value, key)

      putRequest.onsuccess = function () {
        console.log("Data successfully written:", key)
      }

      putRequest.onerror = function (event) {
        console.error("Error writing data:", event.target.error)
        reject(event.target.error)
      }

      tx.oncomplete = function () {
        console.log("Transaction completed")
        resolve(true)
      }

      tx.onerror = function (event) {
        console.error("Transaction error:", event.target.error)
        reject(tx.error)
      }
    }

    request.onerror = function (event) {
      console.error("Database error:", event.target.error)
      reject(request.error)
    }
  })
}

export function loadFromIndexedDB(dbName, storeName, key) {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(dbName, IDB_VERSION)

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
