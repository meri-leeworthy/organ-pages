# Organ Pages

Organ Pages provides a simple way to build a static web pages completely locally in the browser, aiming to provide an alternative to server-heavy website builders. The frontend is a file browser and editor built with [Astro](https://astro.build) and React. You can edit content in a WYSIWYG rich text editor, and you can edit templates and linked styles in a code editor. When you edit files, they are instantly converted into a live preview. Files are autosaved and persisted locally in the browser's IndexedDB.

## Getting Started

To get started, clone this repository and run `npm install` to install the dependencies.

A distribution of the WASM module is included in the `astro` directory. You need `rust` and `wasm-pack` installed to build the WASM module from source. You can install `wasm-pack` with `cargo install wasm-pack`.

All commands are run from inside the `astro` directory, from a terminal:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run wasm`            | Build the WASM module                            |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help` | Get help using the Astro CLI                     |

## Architecture

The frontend is built with [Astro](https://astro.build) and React. The WASM module is built with Rust and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

The WASM module is built with the `wasm` script in the `astro` directory. This builds the WASM module and copies it into the `astro` directory to be bundled with the Astro build. The purpose of the WASM module is essentially to convert handlebars templates and content data into HTML in a fast and efficient manner. The module renders one HTML page at a time, and the Astro frontend is responsible for rendering the page and all the other pages in the site. The rendering with the WASM module is abstracted via the `useRender` hook.

The live preview is rendered in an iframe in the `Preview.tsx` React component. To minimise flashing and unnecessary re-renders, the iframe loads `morphdom` as a dependency, and the parent page passes HTML to the iframe using messages, which are efficiently processed by `morphdom` to update the iframe's DOM.

Source files are stored in an in-memory WASM SQLite database `sql.js`, which is essentially used as a state management system for the frontend. The database is automatically saved and loaded from IndexedDB when the page is loaded. The database is made available to the app through the `useSql` hook which accesses the `SqlProvider` Context.

# Licence

This project is currently licensed under the AGPL. The copyright holder reserves the right to offer alternative licensing terms in the future.