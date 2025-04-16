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

import type { Cursor } from "loro-crdt"
import { Loro, UndoManager } from "loro-crdt"
import {
  type Command,
  EditorState,
  Plugin,
  PluginKey,
  type StateField,
  TextSelection,
} from "prosemirror-state"
import { EditorView } from "prosemirror-view"
import {
  convertPmSelectionToCursors,
  cursorToAbsolutePosition,
} from "./cursor-plugin"
import { loroSyncPluginKey } from "./sync-plugin"
import { configLoroTextStyle } from "./text-style"

export interface LoroUndoPluginProps {
  doc: Loro
  undoManager?: UndoManager
}

export const loroUndoPluginKey = new PluginKey<LoroUndoPluginState>("loro-undo")

interface LoroUndoPluginState {
  undoManager: UndoManager
  canUndo: boolean
  canRedo: boolean
}

type Cursors = { anchor: Cursor | null; focus: Cursor | null }
export const LoroUndoPlugin = (props: LoroUndoPluginProps): Plugin => {
  const undoManager = props.undoManager || new UndoManager(props.doc, {})
  let lastSelection: Cursors = {
    anchor: null,
    focus: null,
  }
  return new Plugin({
    key: loroUndoPluginKey,
    state: {
      init: (config, editorState): LoroUndoPluginState => {
        configLoroTextStyle(props.doc, editorState.schema)

        undoManager.addExcludeOriginPrefix("sys:init")
        return {
          undoManager,
          canUndo: undoManager.canUndo(),
          canRedo: undoManager.canRedo(),
        }
      },
      apply: (tr, state, oldEditorState, newEditorState) => {
        const undoState = loroUndoPluginKey.getState(oldEditorState)
        const loroState = loroSyncPluginKey.getState(oldEditorState)
        if (!undoState || !loroState) {
          return state
        }

        const canUndo = undoState.undoManager.canUndo()
        const canRedo = undoState.undoManager.canRedo()
        {
          const { anchor, focus } = convertPmSelectionToCursors(
            oldEditorState.doc,
            oldEditorState.selection,
            loroState
          )
          lastSelection = {
            anchor: anchor ?? null,
            focus: focus ?? null,
          }
        }
        return {
          ...state,
          canUndo,
          canRedo,
        }
      },
    } as StateField<LoroUndoPluginState>,

    view: (view: EditorView) => {
      undoManager.setOnPush((isUndo, _counterRange) => {
        const loroState = loroSyncPluginKey.getState(view.state)
        if (loroState?.doc == null) {
          return {
            value: null,
            cursors: [],
          }
        }

        const cursors: Cursor[] = []
        let selection = lastSelection
        if (!isUndo) {
          const loroState = loroSyncPluginKey.getState(view.state)
          if (loroState) {
            const { anchor, focus } = convertPmSelectionToCursors(
              view.state.doc,
              view.state.selection,
              loroState
            )
            selection.anchor = anchor || null
            selection.focus = focus || null
          }
        }

        if (selection.anchor) {
          cursors.push(selection.anchor)
        }
        if (selection.focus) {
          cursors.push(selection.focus)
        }

        return {
          value: null,
          // The undo manager will internally transform the cursors.
          // Undo/redo operations may recreate deleted content, so we need to remap
          // the cursors to their new positions. Additionally, if containers are deleted
          // and recreated, they also need remapping. Remote changes to the document
          // should be considered in these transformations.
          cursors,
        }
      })
      undoManager.setOnPop((_isUndo, meta, _counterRange) => {
        // After this call, the `onPush` will be called immediately.
        // The immediate `onPush` will contain the inverse operations that undone the effect caused by the current `onPop`
        const loroState = loroSyncPluginKey.getState(view.state)
        if (loroState?.doc == null) {
          return
        }

        const anchor = meta.cursors[0] ?? null
        const focus = meta.cursors[1] ?? null
        if (anchor == null) {
          return
        }

        setTimeout(() => {
          try {
            const anchorPos = cursorToAbsolutePosition(
              anchor,
              loroState.doc,
              loroState.mapping
            )[0]
            const focusPos =
              focus &&
              cursorToAbsolutePosition(
                focus,
                loroState.doc,
                loroState.mapping
              )[0]
            const selection = TextSelection.create(
              view.state.doc,
              anchorPos,
              focusPos ?? undefined
            )
            view.dispatch(view.state.tr.setSelection(selection))
          } catch (e) {
            console.error(e)
          }
        }, 0)
      })
      return {
        destroy: () => {
          undoManager.setOnPop()
          undoManager.setOnPush()
        },
      }
    },
  })
}

export function canUndo(state: EditorState): boolean {
  const undoState = loroUndoPluginKey.getState(state)
  return undoState?.undoManager.canUndo() || false
}

export function canRedo(state: EditorState): boolean {
  const undoState = loroUndoPluginKey.getState(state)
  return undoState?.undoManager.canRedo() || false
}

export const undo: Command = (state, dispatch): boolean => {
  const undoState = loroUndoPluginKey.getState(state)
  if (!undoState) {
    return false
  }
  if (dispatch) {
    return undoState.undoManager.undo()
  } else {
    return undoState.undoManager.canUndo()
  }
}

export const redo: Command = (state, dispatch): boolean => {
  const undoState = loroUndoPluginKey.getState(state)
  if (!undoState) {
    return false
  }
  if (dispatch) {
    return undoState.undoManager.redo()
  } else {
    return undoState.undoManager.canRedo()
  }
}
