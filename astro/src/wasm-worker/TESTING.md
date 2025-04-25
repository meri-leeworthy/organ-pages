# Integration Testing for WASM Store Actor

This directory contains integration tests for the WASM Store Actor and IndexedDB persistence functionality.

## Test Files

- **integration-test.ts**: Contains test cases for all supported Actor messages
- **test-runner.html**: Browser-based UI for running integration tests

## Running the Tests

### Browser-based Testing in Astro

1. Start your development server
2. Navigate to `/wasm-tests` in your browser
3. Click the "Run All Tests" button to execute tests
4. Test results will appear in the console and in the results panel

> **Note:** The tests are integrated into the Astro project as a page at `/pages/wasm-tests.astro` because HTML files in arbitrary directories are not served statically in Astro projects.

### Alternative Approach: Using the Public Directory

As an alternative to the Astro page, you can also access the tests via the public directory:

1. Start your development server
2. Navigate to `/wasm-test/index.html` in your browser
3. Click the "Run All Tests" button to execute tests

### Programmatic Testing

You can also import and run the tests programmatically:

```typescript
import { runTests, testInitDefault, testSiteOperations } from './integration-test';

// Run all tests
await runTests();

// Or run specific test functions
await testInitDefault();
await testSiteOperations();
```

## Test Coverage

The integration tests verify the following functionality:

### Core Actor Messages
- `InitDefault`: Setting up initial site and theme
- `CreateSite` / `GetSite`: Creating and retrieving sites
- `CreateTheme` / `GetTheme`: Creating and retrieving themes
- `GetCollection` / `ListCollections`: Retrieving collections
- `CreateFile` / `GetFile` / `ListFiles`: File operations
- `UpdateFile`: File content and metadata updates
- `ExportProject`: Exporting projects to serialized format
- `ImportProject`: Importing projects from serialized format (verified through save/load cycle)

### IndexedDB Persistence
- `SaveState`: Saving projects to IndexedDB
- `LoadState`: Loading projects from IndexedDB

## Adding New Tests

To add new tests:

1. Define a new test function in `integration-test.ts`
2. Add it to the `runTests()` function
3. Export it in the `export` block at the bottom of the file

Example:

```typescript
async function testMyNewFeature() {
  console.log('🧪 Testing my new feature...');
  
  // Test code here
  
  console.log('✅ My new feature works!');
}

// Add to runTests function
async function runTests() {
  // ... existing code
  await testMyNewFeature();
  // ... existing code
}

// Add to exports
export {
  // ... existing exports
  testMyNewFeature,
}
```

## Troubleshooting

### Worker Initialization Errors

If you encounter errors like `Worker error:` or `Failed to initialize WASM client`, this may be due to:

1. **Path Issues**: Make sure the worker.js, client.ts, and integration-test.ts files are correctly referenced. In some Astro configurations, the path to web workers needs to be absolute from the project root.

2. **WASM Loading**: Ensure the WASM file is properly compiled and available at the expected path. Check the browser console for detailed error messages about the WebAssembly module.

3. **CORS Issues**: If testing locally, you might encounter CORS issues. Try using a development server like `npm run dev` instead of opening the HTML file directly.

4. **IndexedDB Access**: Some browsers restrict IndexedDB access in certain contexts. Ensure you're running in a secure context (HTTPS or localhost).

5. **MIME Type Issues**: In Astro projects, TypeScript files from src/ directories are not served directly with the correct MIME type. You may see errors like `NS_ERROR_CORRUPTED_CONTENT` or `was blocked because of a disallowed MIME type`. We've addressed this by creating a JavaScript loader in the public directory.

If issues persist, try:

1. Opening the browser's developer console (F12) for detailed error messages
2. Clearing the browser cache
3. Testing in an incognito/private window
4. Using a different browser (Chrome often has the best compatibility)

### Manual Testing

For troubleshooting WASM initialization issues, we've created a simplified test file that bypasses the worker:

1. Start your development server
2. Navigate to `/wasm-tests/manual-test.html` in your browser
3. Follow the step-by-step process to test the WASM module directly

This manual test helps isolate issues by directly interacting with the WASM module without the worker layer. 