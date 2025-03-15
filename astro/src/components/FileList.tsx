import { useState } from "react"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "./ui/collapsible"
import { Plus } from "lucide-react"
import { useStore } from "./StoreProvider"
import { useFiles } from "@/hooks/useSWRStore"
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
import { FileListItem } from "./FileListItem"
import { Spinner } from "./ui/spinner"

// Mapping for collection type to display name
const headingMap = {
  pages: "Pages",
  posts: "Posts",
  templates: "Templates",
  partials: "Partials",
  assets: "Assets",
  styles: "Styles",
  scripts: "Scripts",
}

export function FileList({ type }: { type: string }) {
  const [isOpen, setIsOpen] = useState<boolean>(true)
  const { state, createFile } = useStore()
  const { files, isLoading, error, refreshFiles } = useFiles(
    state.activeProjectType,
    type
  )

  const handleCreateFile = async () => {
    if (!state.activeProjectType) return

    try {
      // Generate a unique filename
      const baseName = "untitled"
      let fileName = baseName
      let counter = 1

      // Check if file with this name already exists
      while (files.some(file => file.name === fileName)) {
        fileName = `${baseName}${counter}`
        counter++
      }

      // Create the new file
      await createFile(state.activeProjectType, type, fileName)

      // Refresh the file list
      refreshFiles()
    } catch (err) {
      console.error("Failed to create file:", err)
    }
  }

  const handleLoadFile = async () => {
    if (!state.activeProjectType) return

    const input = document.createElement("input")
    input.type = "file"

    const loadExtensionMap = {
      templates: ".html,.htm,.hbs",
      partials: ".html,.htm,.hbsp",
      pages: ".md",
      posts: ".md",
      styles: ".css",
      scripts: ".js",
      assets: "*",
    }

    input.accept =
      type in loadExtensionMap
        ? loadExtensionMap[type as keyof typeof loadExtensionMap]
        : "*"

    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (file) {
        // Here you would implement file loading logic
        // This is a placeholder for future implementation
        console.log(`Loading file ${file.name} for collection ${type}`)

        // After loading the file, refresh the list
        refreshFiles()
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
              {type === "assets" ? null : (
                <DropdownMenuItem onClick={handleCreateFile}>
                  New File
                </DropdownMenuItem>
              )}
              <DropdownMenuItem onClick={handleLoadFile}>
                Load File
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarGroupAction>

        <CollapsibleContent>
          {isLoading ? (
            <div className="flex justify-center py-2">
              <Spinner size="sm" />
            </div>
          ) : error ? (
            <div className="px-2 py-1 text-sm text-red-400">
              Failed to load files
            </div>
          ) : files.length === 0 ? (
            <div className="px-2 py-1 text-sm text-zinc-500">No files</div>
          ) : (
            <ul>
              {files.map(file => (
                <FileListItem key={file.id} file={file} />
              ))}
            </ul>
          )}
        </CollapsibleContent>
      </SidebarGroup>
    </Collapsible>
  )
}
