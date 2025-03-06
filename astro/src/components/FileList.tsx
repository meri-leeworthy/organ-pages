import { useEffect, useState } from "react"
import { headingMap, type Collection, type SelectedFiles } from "../lib/types"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "./ui/collapsible"
import { Plus } from "lucide-react"
import { useStoreContext } from "./StoreContext"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu"
import {
  SidebarGroup,
  SidebarGroupAction,
  SidebarGroupLabel,
} from "./ui/sidebar"

import { useBlobStore } from "./useBlobStore"
// import { loadAssetFile, loadTextFile } from "@/lib/loadFile"
import { FileListItem } from "./FileListItem"
import type { File } from "@/lib/File"

export function FileList({ type }: { type: string }) {
  const [isOpen, setIsOpen] = useState<boolean>(true)

  const { store, loading, error } = useStoreContext()
  // const blobStore = useBlobStore()

  const files = store.activeProject?.getCollection(type).files

  if (loading) return <div>Loading...</div>
  if (error) return <div>Error: {error.message}</div>

  const handleCreateFile = (type: string) => {
    if (loading || error) return

    // Function to generate unique filename
    const generateUniqueFileName = (
      baseName: string,
      files: Map<string, File>
    ) => {
      let fileName = `${baseName}`
      let counter = 1

      // Check if file with this name already exists
      while ([...files.values()].some(file => file.name === fileName)) {
        fileName = `${baseName}${counter}`
        counter++
      }

      return fileName
    }

    // const newFileName = generateUniqueFileName("untitled", files)

    // const newFile = store.activeProject?.createFile(newFileName, type)
  }

  const handleLoadFile = async (type: string) => {
    if (loading || error) return
    const input = document.createElement("input")
    input.type = "file"

    const loadExtensionMap = {
      template: ".html,.htm,.hbs",
      partial: ".html,.htm,.hbsp",
      page: ".md",
      post: ".md",
      text: "*",
      asset: "*",
    } as const
    input.accept =
      type in loadExtensionMap
        ? loadExtensionMap[type as keyof typeof loadExtensionMap]
        : "*"

    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (file) {
        if (type === "asset") {
          // parse file and add to database
          // const newFile = await loadAssetFile(file, execute, blobStore)
          // setFiles(files => files.set(newFile.id, newFile))
          // setSelectedFiles(selectedFiles => ({
          //   activeFileId: newFile.id,
          //   contentFileId: selectedFiles.contentFileId,
          // }))
        } else {
          // parse file and add to database
          // const newFile = await loadTextFile(file, type, execute)
          // setFiles(files => files.set(newFile.id, newFile))
          // setSelectedFiles(selectedFiles => ({
          //   activeFileId: newFile.id,
          //   contentFileId:
          //     type === "page" ? newFile.id : selectedFiles.contentFileId,
          // }))
        }
      }
    }
    input.click()
  }

  const heading =
    type in headingMap ? headingMap[type as keyof typeof headingMap] : type

  return (
    <Collapsible
      open={isOpen}
      onOpenChange={setIsOpen}
      className="group/collapsible">
      <SidebarGroup className="p-1">
        <SidebarGroupLabel asChild>
          <CollapsibleTrigger className="text-zinc-100">
            {heading}
          </CollapsibleTrigger>
        </SidebarGroupLabel>
        <SidebarGroupAction title={"Add " + type} className="rounded-lg">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Plus className="w-4 h-4" />
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {type === "asset" ? null : (
                <DropdownMenuItem onClick={() => handleCreateFile(type)}>
                  New File
                </DropdownMenuItem>
              )}
              <DropdownMenuItem onClick={() => handleLoadFile(type)}>
                Load File
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarGroupAction>

        <CollapsibleContent>
          <ul className="">
            {files?.map(file => <FileListItem key={file.id} file={file} />)}
          </ul>
        </CollapsibleContent>
      </SidebarGroup>
    </Collapsible>
  )
}
