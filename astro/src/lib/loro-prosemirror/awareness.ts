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

import { Awareness, Cursor, type PeerID } from "loro-crdt"

export class CursorAwareness extends Awareness<{
  anchor: Uint8Array | null
  focus: Uint8Array | null
  user: { name: string; color: string } | null
}> {
  constructor(peer: PeerID, timeout: number = 30_000) {
    super(peer, timeout)
  }

  getAll(): { [peer in PeerID]: { anchor?: Cursor; focus?: Cursor } } {
    const ans: {
      [peer in PeerID]: {
        anchor?: Cursor
        focus?: Cursor
        user?: { name: string; color: string }
      }
    } = {}
    for (const [peer, state] of Object.entries(this.getAllStates())) {
      ans[peer as PeerID] = {
        anchor: state.anchor ? Cursor.decode(state.anchor) : undefined,
        focus: state.focus ? Cursor.decode(state.focus) : undefined,
        user: state.user ? state.user : undefined,
      }
    }
    return ans
  }

  setLocal(state: {
    anchor?: Cursor
    focus?: Cursor
    user?: { name: string; color: string }
  }) {
    this.setLocalState({
      anchor: state.anchor?.encode() || null,
      focus: state.focus?.encode() || null,
      user: state.user || null,
    })
  }

  getLocal() {
    const state = this.getLocalState()
    if (!state) {
      return undefined
    }

    return {
      anchor: state.anchor && Cursor.decode(state.anchor),
      focus: state.focus && Cursor.decode(state.focus),
      user: state.user,
    }
  }
}

export function cursorEq(a?: Cursor | null, b?: Cursor | null) {
  if (!a && !b) {
    return true
  }
  if (!a || !b) {
    return false
  }

  const aPos = a.pos()
  const bPos = b.pos()
  return (
    aPos?.peer === bPos?.peer &&
    aPos?.counter === bPos?.counter &&
    a.containerId() === b.containerId()
  )
}
