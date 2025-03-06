import { extensionMap, type SelectedFiles } from "@/lib/types"
import { useBlobStore } from "./useBlobStore"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu"
import { DotsVerticalIcon } from "@radix-ui/react-icons"
import { useClient } from "@/hooks/useClient"
import { useStoreContext } from "./StoreContext"
import { File as FileIcon } from "lucide-react"
import { toast } from "sonner"
import type { File } from "@/lib/File"

export function FileListItem({ file }: { file: File }) {
  const { store, loading, error } = useStoreContext()
  const blobStore = useBlobStore()
  const client = useClient()

  const handleRenameFileClick = async () => {
    if (!store || loading || error) return
    const newName = prompt("Enter new name:", file.name)
    if (newName) {
      try {
        // If the file has a URL (is in R2), rename it there first
        let url: string = String(file.tryGetField("url"))
        if (url) {
          url = String(await client.renameFile(url, newName))
        }

        // Update the file in the local database
        // execute("UPDATE file SET name = ?, url = ? WHERE id = ?;", [
        //   newName,
        //   newUrl || null, // Use null if newUrl is undefined
        //   file.id,
        // ])
        file.name = newName
        file.setField("url", url || "")
        toast.success("File renamed")
      } catch (error) {
        toast.error("Error renaming file")
        console.error("Error renaming file:", error)
      }
    }
  }

  const handleDeleteFile = async () => {
    if (!store || loading || error) return

    console.log("deleting file", file.id, file.type)

    // Delete from R2 if the file has a URL
    let url = file.tryGetField("url")
    if (url) {
      try {
        await client.deleteFile(String(url))
      } catch (error) {
        console.error("Error deleting file from R2:", error)
        toast.error("Error deleting file from R2")
      }
    }

    // execute(
    //   `
    //   DELETE FROM file
    //   WHERE id = ?
    //   `,
    //   [fileId]
    // )

    // Delete the file from the blob store
    blobStore.deleteBlob(file.id)

    // const newFiles = new Map(files)
    // newFiles.delete(file.id)
    // if (selectedFiles.activeFileId === file.id) {
    //   setSelectedFiles({
    //     activeFileId: "1@1",
    //     contentFileId: "1@1",
    //   })
    // }
    if (file.type === "asset") {
      blobStore.deleteBlob(file.id)
    }
    toast.success("File deleted")
  }

  const handlePublishFile = async (id: string) => {
    if (!store || loading || error) return

    // const file = files.get(id)
    if (!file) return
    console.log("uploading file", file)

    const blob = await blobStore.getBlob(file.id)

    try {
      const mimeType = file.tryGetField("mime_type")
      if (!mimeType) {
        throw new Error("No mime type found")
      }
      const url = await client.uploadFile(blob, file.name, String(mimeType))
      if (!url) {
        throw new Error("No url found")
      }
      file.setField("url", url)
      // setFiles(files => files.set(file.id, file))
      toast.success("File published")
    } catch (error) {
      console.error("Error fetching blob:", error)
      toast.error("Error publishing file")
    }
  }

  const handleDeleteFileClick = () => {
    if (confirm(`Are you sure you want to delete ${file.name}?`)) {
      handleDeleteFile()
    }
  }

  return (
    <DropdownMenu key={file.id}>
      <li
        className={`group/file flex cursor-pointer items-center gap-2 rounded-lg p-1 ${
          store.activeProject?.activeFile?.id === file.id
            ? "bg-zinc-900"
            : "hover:bg-zinc-900/50"
        }`}
        onClick={() => {
          store.activeProject ? (store.activeProject.activeFile = file) : null
        }}>
        <FileIcon className="w-4 h-4" />
        {(file.name.length > 16 ? file.name.slice(0, 16) + "..." : file.name) +
          (file.type === "template" ? `.${extensionMap[file.type]}` : "")}
        <DropdownMenuTrigger className="ml-auto invisible group-hover/file:visible group-active:visible mr-[6px] text-zinc-100 stroke-zinc-100">
          <DotsVerticalIcon />
        </DropdownMenuTrigger>
      </li>

      <DropdownMenuContent>
        <DropdownMenuItem
          className="flex items-center gap-2 px-2 py-1 text-sm rounded cursor-pointer hover:bg-accent hover:outline-none"
          onClick={() => handleRenameFileClick()}>
          Rename
        </DropdownMenuItem>
        <DropdownMenuItem
          className="flex items-center gap-2 px-2 py-1 text-sm rounded cursor-pointer hover:bg-accent hover:outline-none"
          onClick={() => handleDeleteFileClick()}>
          Delete
        </DropdownMenuItem>
        {file.type === "asset" &&
          (!file.tryGetField("url") || file.tryGetField("url") === "") && (
            <DropdownMenuItem
              className="flex items-center gap-2 px-2 py-1 text-sm rounded cursor-pointer hover:bg-accent hover:outline-none"
              onClick={() => handlePublishFile(file.id)}>
              Publish
            </DropdownMenuItem>
          )}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
