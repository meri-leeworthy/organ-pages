# Organ WASM Module

This is the WebAssembly (WASM) component of Organ Static Site Generator (SSG). It provides the core functionality for managing content using CRDT data structures via Loro.

## Architecture

The WASM module uses an actor-based approach with message passing to interact with the JavaScript/TypeScript frontend:

``` text
React/Astro UI <-> JS Client API <-> Web Worker <-> WASM Module
                                       |
                                IndexedDB Storage
```

### Core Data Model

```rust
pub struct Store {
    active_theme: RwLock<Option<String>>,
    active_site: RwLock<Option<String>>,
}

pub struct Project {
    id: String,
    project_type: ProjectType,
    created: f64, // Store as timestamp
    updated: f64, // Store as timestamp
    doc: LoroDoc,
}

pub struct Collection {
    name: String,
    map: LoroMap,
}

pub struct File {
    pub files: LoroTree,
    pub id: TreeID,
    pub collection_type: String,
}
```

### Components

1. **Actor (Rust)**: 
   - Manages internal state through the Store
   - Processes messages in a type-safe way
   - Maintains CRDT data structures via Loro

2. **Web Worker (JS)**:
   - Loads and initializes the WASM module
   - Handles message passing with the main thread
   - Isolates computational work from UI thread
   - Manages project persistence in IndexedDB

3. **Client API (TS)**:
   - Provides promise-based interface to the application
   - Serializes/deserializes messages
   - Manages worker communication
   - Provides event subscription for state changes

4. **React Context Provider**:
   - Maintains application state in sync with WASM state
   - Provides hooks for components to access state
   - Handles loading and error states

## Build and Development

### Building the WASM Module

```sh
wasm-pack build --target bundler --out-dir ../astro/src/wasm/minissg
```

### Using in the Astro Project

Import and use the React context provider:

```tsx
import { WasmStoreProvider, useWasmStore } from '@/wasm-worker/StoreProvider';

// Wrap your app
<WasmStoreProvider>
  <YourApp />
</WasmStoreProvider>

// Use in components
function YourComponent() {
  const { state, createProject, updateFile } = useWasmStore();
  // ...
}
```

## Implementation Details

### Message Types

The Actor processes messages defined in `messages.rs`, which include:

- Project operations (create, list, get, etc.)
- Collection operations
- File operations
- Storage operations
- Rendering operations

### Serialization

All messages and responses use JSON serialization for interop between Rust and JavaScript:

1. JS Client serializes a message
2. Web Worker passes it to the WASM Actor
3. Actor deserializes, processes, and serializes a response
4. Web Worker returns response to main thread

### Project Persistence

The system uses IndexedDB for persistent storage of projects:

#### Data Storage Design

- **Database**: `organ-static-projects` (version 1)
- **Object Stores**:
  - `projects`: Stores project binary data with metadata
  - `metadata`: Stores active project IDs and project list

#### Project Storage Format
```js
{
  id: string,               // Project ID
  binary: string,           // Serialized project data from WASM export
  metadata: {
    id: string,             // Project ID
    type: "site" | "theme", // Project type
    createdAt: number,      // Creation timestamp
    updatedAt: number       // Update timestamp
  },
  updatedAt: number         // Last update timestamp
}
```

#### Persistence Flow

1. **Saving State**:
   - Client calls `saveState(siteId, themeId)`
   - Worker intercepts the message
   - Worker exports projects from WASM using Actor API
   - Worker stores data in IndexedDB
   - Worker updates metadata and sends notification

2. **Loading State**:
   - Client calls `loadState(siteId?, themeId?)`
   - Worker loads project data from IndexedDB
   - Worker imports projects into WASM using Actor API
   - Worker updates metadata and sends notification

#### State Change Events

The client API provides an event system for state changes:

```typescript
// Subscribe to state change events
const unsubscribe = wasmClient.onStateChange('state_saved', ({ siteId, themeId }) => {
  console.log(`State saved: siteId=${siteId}, themeId=${themeId}`)
})

// Later, when no longer needed
unsubscribe()
```

## Migration Strategy

The migration from direct WASM binding to actor-based approach is being done gradually:

1. Implement the actor system in parallel with existing code
2. Create web worker and client infrastructure
3. Implement React context provider that mirrors the existing API
4. Gradually transition components to use the new provider
5. Remove old direct bindings when migration is complete