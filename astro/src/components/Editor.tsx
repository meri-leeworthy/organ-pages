// import type { FileData } from "@/lib/types"
import { EditorContent, useEditor } from "@tiptap/react"
import type { Editor } from "@tiptap/core"
import Document from "@tiptap/extension-document"
import Paragraph from "@tiptap/extension-paragraph"
import Text from "@tiptap/extension-text"
import Bold from "@tiptap/extension-bold"
import Italic from "@tiptap/extension-italic"
import Underline from "@tiptap/extension-underline"
import Image from "@tiptap/extension-image"
import Link from "@tiptap/extension-link"
import Dropcursor from "@tiptap/extension-dropcursor"
// import Heading from "@tiptap/extension-heading"
import {
  Bold as BoldIcon,
  ImageIcon,
  Italic as ItalicIcon,
  Underline as UnderlineIcon,
  // Heading as HeadingIcon,
} from "lucide-react"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import { Button } from "./ui/button"
import { DropdownMenu, DropdownMenuTrigger } from "./ui/dropdown-menu"
import type { File } from "@/wasm-worker/types"
// import { ImageSelector } from "./ImageSelector"

import { Extension } from "@tiptap/core"
import { Plugin, PluginKey } from "@tiptap/pm/state"
// import {
//   LoroSyncPlugin,
//   LoroUndoPlugin,
//   LoroCursorPlugin,
//   redo,
//   undo,
// } from "loro-prosemirror"
// import { keymap } from "@tiptap/pm/keymap"

// Import our new worker sync plugin
import { WasmWorkerSyncPlugin } from "@/lib/loro-prosemirror/worker-sync"
import { useStore } from "./StoreProvider"

// Legacy plugin for reference - will be replaced by WasmWorkerSyncPlugin
const LoroSyncPlugin = Extension.create({
  name: "loroSync",
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey("loroSync"),
        // view: () => ({
        //   update: view => {
        //     console.log("View updated:", view)
        //   },
        // }),
        state: {
          init: () => {
            console.log("State initialized")
          },
          apply: (tr, value) => {
            console.log("State transaction:", tr)
            return value
          },
        },
      }),
    ]
  },
})

const extensions = [
  Document,
  Paragraph,
  Text,
  Bold,
  Italic,
  Underline,
  Dropcursor,
  Image,
  Link,
  // We'll configure the WasmWorkerSyncPlugin per-file in the component
  // to ensure each file gets its own document ID
]

export function EditorComponent({
  file,
  onChange,
}: {
  file: File
  onChange: (html: string) => void
}) {
  const store = useStore()
  const editor = useEditor({
    extensions: [
      ...extensions,
      // Configure the WasmWorkerSyncPlugin with the file ID
      // This ensures each file has its own document in the WASM store
      WasmWorkerSyncPlugin.configure({
        store: store,
        clientId: "editor-" + Math.random().toString(36).substring(2, 9),
      }),
    ],
    content: file.body?.content || "",
    editorProps: {
      attributes: {
        class: "prose prose-sm sm:prose-base focus:outline-none",
      },
    },
    // onUpdate({ editor }) {
    //   onChange(editor.getHTML())
    // },
  })

  if (!editor) return null

  return (
    <div className="flex flex-col items-start">
      <Toolbar editor={editor} />
      <EditorContent
        name="body"
        editor={editor}
        className="w-full border rounded-md border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring *:disabled:cursor-not-allowed *:disabled:opacity-50"
      />
    </div>
  )
}

export function Toolbar({ editor }: { editor: Editor }) {
  return (
    <div className="flex flex-row gap-2 mb-2">
      <ToggleGroup type="multiple" className="">
        <ToggleGroupItem
          value="bold"
          aria-label="Toggle bold"
          onClick={() => editor.chain().focus().toggleBold().run()}>
          <BoldIcon className="w-4 h-4" />
        </ToggleGroupItem>
        <ToggleGroupItem
          value="italic"
          aria-label="Toggle italic"
          onClick={() => editor.chain().focus().toggleItalic().run()}>
          <ItalicIcon className="w-4 h-4" />
        </ToggleGroupItem>
        <ToggleGroupItem
          value="strikethrough"
          aria-label="Toggle strikethrough"
          onClick={() => editor.chain().focus().toggleUnderline().run()}>
          <UnderlineIcon className="w-4 h-4" />
        </ToggleGroupItem>
      </ToggleGroup>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon">
            <ImageIcon className="w-4 h-4" />
          </Button>
        </DropdownMenuTrigger>
        {/* <ImageSelector editor={editor} /> */}
      </DropdownMenu>
    </div>
  )
}
