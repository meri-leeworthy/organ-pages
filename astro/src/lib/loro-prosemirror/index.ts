/*
Copyright 2024 LORO

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

export {
  LoroSyncPlugin,
  loroSyncPluginKey,
  type LoroSyncPluginProps,
  type LoroSyncPluginState,
} from "./sync-plugin"
export type { LoroDocType } from "./lib"
export {
  createNodeFromLoroObj,
  updateLoroToPmState,
  ROOT_DOC_KEY,
  NODE_NAME_KEY,
  CHILDREN_KEY,
  ATTRIBUTES_KEY,
  type LoroNodeMapping,
} from "./lib"
export { LoroCursorPlugin } from "./cursor-plugin.ts"
export { CursorAwareness } from "./awareness.ts"
export {
  LoroUndoPlugin,
  loroUndoPluginKey,
  type LoroUndoPluginProps,
  undo,
  redo,
  canUndo,
  canRedo,
} from "./undo-plugin.ts"
