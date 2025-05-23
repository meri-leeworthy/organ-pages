# WASM Worker for Project Persistence

This directory contains the implementation of a Web Worker based system for communicating with the WASM module and providing project persistence.

## Files

- **client.ts**: Client API for the main thread to communicate with the worker
- **worker.js**: Web Worker implementation that handles WASM initialization and persistence
- **types.ts**: TypeScript types for messages and responses

## Key Features

- Message-based communication between main thread and WASM
- IndexedDB-based persistence for projects
- Event system for state change notifications
- Separation of concerns: persistence logic contained in worker

## Usage

```typescript
import wasmClient from './client'

// Initialize
await wasmClient.init()

// Save state
const site = await wasmClient.getSite()
const theme = await wasmClient.getTheme()
await wasmClient.saveState(site.id, "site")
await wasmClient.saveState(theme.id, "theme")

// Load state
await wasmClient.loadState()

// Listen for state changes
const unsubscribe = wasmClient.onStateChange('state_saved', 
  ({ siteId, themeId }) => {
    console.log(`State saved with IDs: ${siteId}, ${themeId}`)
  }
)
```

For a complete example implementation, see `../lib/persistence-example.ts`.

## Architecture

The system uses a layered architecture:

1. **Client API** (client.ts): Provides a promise-based interface for the application
2. **Web Worker** (worker.js): Handles WASM communication and persistence
3. **WASM Module**: Performs project operations and export/import

## Integration Testing

This directory includes comprehensive integration tests for the WASM Worker and Store Actor:

- **integration-test.ts**: Test cases for all supported Actor messages and persistence
- **test-runner.html**: Browser-based UI for executing tests (reference implementation)
- **TESTING.md**: Detailed documentation on testing functionality

To run the tests in your Astro project, you have two options:

1. **Astro Page**: Navigate to `/wasm-tests` in your browser. The tests are available as an Astro page at `/pages/wasm-tests.astro`.

2. **Public Directory**: Navigate to `/wasm-tests/` in your browser. A copy of the test runner is available in the public directory at `public/wasm-tests/index.html`.

Both approaches are provided because HTML files in arbitrary directories are not served statically in Astro projects.

For more details on testing, see [TESTING.md](./TESTING.md).

## Implementation Notes

- IndexedDB operations happen in the worker thread to avoid blocking the UI
- The WASM module doesn't directly interact with IndexedDB - the worker handles storage
- Messages follow a typed format defined in types.ts
- State change events provide hooks for UI updates