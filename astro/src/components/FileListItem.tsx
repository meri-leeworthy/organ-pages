import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu"
import { DotsVerticalIcon } from "@radix-ui/react-icons"
import { File as FileIcon } from "lucide-react"
import { toast } from "sonner"
import { useStore } from "./StoreProvider"
import { useActiveFile } from "@/hooks/useSWRStore"
import type { File } from "@/wasm-worker/types"

// File extension mappings
const extensionMap: Record<string, string> = {
  pages: "md",
  posts: "md",
  templates: "hbs",
  partials: "hbs",
  styles: "css",
  scripts: "js",
}

export function FileListItem({ file }: { file: File }) {
  const { state, updateFile, setSiteActivePage } = useStore()
  const { file: activeFile, setActiveFile } = useActiveFile()

  // Determine if this file is the active file
  const isActive =
    activeFile &&
    activeFile.id === file.id &&
    activeFile.collection === file.collection

  const handleRenameFileClick = async () => {
    if (!state.activeProjectType) return
    const newName = prompt("Enter new name:", file.name)

    if (newName && newName !== file.name) {
      try {
        await updateFile(state.activeProjectType, file.collection, file.id, {
          SetName: newName,
        })
        toast.success("File renamed")
      } catch (error) {
        toast.error("Error renaming file")
        console.error("Error renaming file:", error)
      }
    }
  }

  const handleDeleteFile = async () => {
    // This would need to be implemented with a deleteFile method in the store
    toast.error("Delete functionality not yet implemented")
  }

  const handleSelectFile = () => {
    if (!state.activeProjectType) return

    console.log("[FileListItem] Setting active file: ", file)

    // Set this file as the active file
    setActiveFile({
      collectionName: file.collection_type,
      fileId: file.id,
      projectType: state.activeProjectType,
    })

    // If this is a page type and we're in site context, also set it as the active page
    if (file.collection === "pages" && state.activeProjectType === "site") {
      setSiteActivePage({
        collectionName: file.collection,
        fileId: file.id,
      })
    }
  }

  const handleDeleteFileClick = () => {
    if (confirm(`Are you sure you want to delete ${file.name}?`)) {
      handleDeleteFile()
    }
  }

  // Determine if file should show extension
  const shouldShowExtension = file.collection in extensionMap
  const fileExtension = shouldShowExtension ? extensionMap[file.collection] : ""

  // Format the displayed filename
  const displayName =
    file.name.length > 16 ? file.name.slice(0, 16) + "..." : file.name

  const fullDisplayName = shouldShowExtension
    ? `${displayName}.${fileExtension}`
    : displayName

  return (
    <DropdownMenu>
      <li
        className={`group/file flex cursor-pointer items-center gap-2 rounded-lg p-1 ${
          isActive ? "bg-zinc-900" : "hover:bg-zinc-900/50"
        }`}
        onClick={handleSelectFile}>
        <FileIcon className="w-4 h-4" />
        {fullDisplayName}
        <DropdownMenuTrigger className="ml-auto invisible group-hover/file:visible group-active/file:visible mr-[6px] text-zinc-100 stroke-zinc-100">
          <DotsVerticalIcon />
        </DropdownMenuTrigger>
      </li>

      <DropdownMenuContent>
        <DropdownMenuItem
          className="flex items-center gap-2 px-2 py-1 text-sm rounded cursor-pointer hover:bg-accent hover:outline-none"
          onClick={handleRenameFileClick}>
          Rename
        </DropdownMenuItem>
        <DropdownMenuItem
          className="flex items-center gap-2 px-2 py-1 text-sm rounded cursor-pointer hover:bg-accent hover:outline-none"
          onClick={handleDeleteFileClick}>
          Delete
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
