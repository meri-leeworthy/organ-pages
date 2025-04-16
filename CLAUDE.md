# CLAUDE.md - Agent Reference Guide

## Build & Development Commands
- Install dependencies: `npm install` (astro dir)
- Dev server: `npm run dev` (localhost:4321)
- Build: `npm run build` (outputs to ./dist/)
- Preview build: `npm run preview`
- Type check: `npm run astro check`
- Build WASM: `cd ../wasm && wasm-pack build --target bundler --out-dir ../astro/src/wasm/minissg`

## Code Style Guidelines

### TypeScript/React
- **Imports**: Group by source (React, components, utils); use path aliases (@/components)
- **Components**: Functional components with typed props (React.FC<Props>)
- **Hooks**: Use 'use' prefix; handle errors with state
- **Types**: Strong typing throughout; interfaces/types at file top
- **Error handling**: Try/catch for async; conditional rendering for errors

### Rust/WASM
- **Structure**: Public functions with #[wasm_bindgen]
- **Naming**: Snake_case functions, CamelCase for structs/enums
- **Error handling**: Result<T, E> return types; ? operator for propagation
- **Types**: Derive appropriate traits (Debug, Clone, PartialEq)

### General
- Tailwind CSS for styling
- Context-based state management
- Type safety across both TypeScript and Rust

## Project Persistence
The application uses a Web Worker to communicate with the WASM module and handle storage:

```typescript
// Initialize and save state
await wasmClient.init()
const site = await wasmClient.getSite()
const theme = await wasmClient.getTheme()
await wasmClient.saveState(site.id, theme.id)

// Listen for state change events
const unsubscribe = wasmClient.onStateChange('state_saved', ({ siteId, themeId }) => {
  console.log(`State saved: siteId=${siteId}, themeId=${themeId}`)
})

// Load state on application startup
await wasmClient.loadState() // Will load last active projects
// Or load specific projects
await wasmClient.loadState(specificSiteId, specificThemeId)
```

The persistence implementation uses IndexedDB to store project data, with a separate metadata store for active project information.