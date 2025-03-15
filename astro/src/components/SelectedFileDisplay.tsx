import { useStore } from "./StoreProvider"
// import { type OrganUserModelField } from "../lib/types"
import { AutoResizeTextarea } from "./ui/textarea"
// import { useBlobStore } from "./useBlobStore"
import { Label } from "./ui/label"
import {
  Select,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectItem,
} from "./ui/select"
import { Input } from "./ui/input"
import { Button } from "./ui/button"
import { Card } from "./ui/card"
import { X } from "lucide-react"
// import { useClient } from "@/hooks/useClient"
// import useRender from "@/hooks/useRender"
import { EditorComponent as Editor } from "./Editor"
import { useCallback, useEffect, useState } from "react"
import { toast } from "sonner"
// import { File } from "@/lib/File"
import { useActiveFile, useCollection } from "@/hooks/useSWRStore"
import type { FieldDefinition } from "@/wasm-worker/types"
import { useDebounce } from "@/hooks/useDebounce"

export const SelectedFileDisplay = ({ onClose }: { onClose: () => void }) => {
  // const blobStore = useBlobStore()
  // const client = useClient()
  // const { render } = useRender()
  const { state, saveState } = useStore()
  const [publishLoading, setPublishLoading] = useState(false)
  const { file, updateActiveFile } = useActiveFile()
  const { collection, isLoading, error } = useCollection(
    state.activeProjectType,
    file?.collection_type
  )
  const debouncedFile = useDebounce(file, 1000)

  console.log("collection", collection)
  console.log("file", file)

  // const handlePublishFile = async () => {
  //   if (!store || loading || error) return
  //   console.log("uploading file", file)
  //   setPublishLoading(true)
  //   if (file.type === "asset") {
  //     const blob = await blobStore.getBlob(file.id)

  //     try {
  //       const mimeType = file.tryGetField("mimeType")
  //       if (!mimeType) {
  //         throw new Error("No mime type found")
  //       }
  //       const url = await client.uploadFile(blob, file.name, String(mimeType))
  //       if (!url) {
  //         throw new Error("No url found")
  //       }
  //       file.setField("url", url)
  //       console.log("success", file)
  //       // store.setActiveFile(file)
  //       setPublishLoading(false)
  //       toast.success("File published")
  //     } catch (error) {
  //       setPublishLoading(false)
  //       toast.error((error as Error).message)
  //     }
  //     return
  //   }

  //   if (file.type === "text") {
  //     try {
  //       let mimeType = "text/css"
  //       if (file.name.endsWith(".js")) {
  //         mimeType = "text/javascript"
  //       }
  //       const body = file.tryGetField("content")
  //       if (!body) throw new Error("No content found")
  //       const blob = new Blob([String(body)], { type: mimeType })
  //       const url = await client.uploadFile(blob, file.name, mimeType)
  //       if (!url) throw new Error("No url found")
  //       file.setField("url", url)
  //       // store.setActiveFile(file)
  //       setPublishLoading(false)
  //       toast.success("File published")
  //     } catch (error) {
  //       setPublishLoading(false)
  //       toast.error((error as Error).message)
  //     }
  //     return
  //   }

  //   // render file if it is not an asset
  //   try {
  //     const query =
  //       "SELECT file.id, file.name, file.data, file.url, model.name as type FROM file JOIN model ON file.model_id = model.id;"
  //     // const result = execute(query)
  //     // const files = result.map((file: ParamsObject): [number, FileData] => [
  //     //   file.id as number,
  //     //   {
  //     //     id: file.id as number,
  //     //     name: file.name?.toString() || "",
  //     //     type: file.type?.toString() as FileData["type"],
  //     //     data: JSON.parse(file.data?.toString() || "{}"),
  //     //     url: file.url?.toString() || "",
  //     //   },
  //     // ])
  //     // const renderedFile = render(file.id, new Map(files))

  //     // console.log("renderedFile", renderedFile)

  //     // now upload the rendered file
  //     // const blob = new Blob([renderedFile], { type: "text/html" })
  //     // const url = await client.uploadFile(blob, file.name, "text/html")
  //     // if (!url) {
  //     //   throw new Error("No url found")
  //     // }
  //     // const newFile = {
  //     //   ...file,
  //     //   url,
  //     // }
  //     // setFile(newFile)
  //     // execute("UPDATE file SET url = ? WHERE id = ?;", [url, file.id])
  //     setPublishLoading(false)
  //     toast.success("File published")
  //   } catch (error) {
  //     setPublishLoading(false)
  //     toast.error((error as Error).message)
  //   }
  // }

  const handleDataChange = async (fieldName: string, value: any) => {
    if (!file) return

    try {
      await updateActiveFile(fieldName, value)
    } catch (error) {
      toast.error(
        `Failed to update ${fieldName}: ${error instanceof Error ? error.message : "Unknown error"}`
      )
    }

    console.log("file", file)
  }

  useEffect(() => {
    if (debouncedFile) {
      console.log("saving state", state.activeProjectType)
      if (!state.site?.id || !state.theme?.id) return
      saveState(state.site?.id, state.theme?.id, state.activeProjectType)
    }
  }, [debouncedFile])

  const renderFieldWithLabel = (name: string, field: FieldDefinition) => {
    if (!file) return null
    if (field.type === "richtext")
      return (
        <div className="flex flex-col gap-2" key={`${file?.id}-${name}`}>
          <Label htmlFor="body" className="capitalize">
            Body
          </Label>
          <Editor
            key={file.id}
            file={file}
            onChange={html => void handleDataChange("body", html)}
          />
        </div>
      )

    return (
      <div className="flex flex-col gap-2" key={`${file?.id}-${name}`}>
        <Label htmlFor={name} className="capitalize">
          {name}
        </Label>
        {renderField(name, field)}
      </div>
    )
  }

  const renderField = useCallback(
    (name: string, field: FieldDefinition) => {
      if (!file) return null
      const rawValue = file?.[field.name]
      console.log("rawValue", rawValue)
      let value: string | number | string[] | undefined

      if (
        name === "body" &&
        typeof rawValue === "object" &&
        rawValue !== null
      ) {
        value = (rawValue as { content: string })?.content
      } else if (field.type === "array") {
        value = Array.isArray(rawValue) ? rawValue : []
      } else {
        value = rawValue ? String(rawValue) : ""
      }

      if (name === "template") {
        return (
          <Select
            value={String(value || "")}
            onValueChange={value => {
              void handleDataChange(name, value)
            }}>
            <SelectTrigger className="">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {/* {[...templates.entries()].map(template => (
              <SelectItem
                key={template[0]}
                value={template[0].toString()}
                textValue={template[1]}>
                {template[1]}
              </SelectItem>
            ))} */}
            </SelectContent>
          </Select>
        )
      }

      switch (field.type) {
        case "string":
          return (
            <Input
              value={String(value || "")}
              onChange={e => {
                void handleDataChange(name, e.target.value)
              }}
            />
          )
        case "number":
          return (
            <Input
              type="number"
              value={String(value || "")}
              onChange={e => {
                void handleDataChange(name, e.target.value)
              }}
            />
          )
        case "datetime":
          return (
            <Input
              type="date"
              value={String(value || "")}
              onChange={e => {
                void handleDataChange(name, e.target.value)
              }}
            />
          )
        case "array": {
          const arrayValue = Array.isArray(value) ? value : []
          return (
            <div className="space-y-2">
              {arrayValue.map((item: string, index: number) => (
                <div key={index} className="flex items-center space-x-2">
                  <Input
                    value={item}
                    onChange={e => {
                      const newValue = [...arrayValue]
                      newValue[index] = e.target.value
                      void handleDataChange(name, newValue)
                    }}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      const newValue = arrayValue.filter(
                        (_: string, i: number) => i !== index
                      )
                      void handleDataChange(name, newValue)
                    }}>
                    Remove
                  </Button>
                </div>
              ))}
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() =>
                  void handleDataChange(name, [...arrayValue, ""])
                }>
                Add Item
              </Button>
            </div>
          )
        }
        case "text":
          return (
            <AutoResizeTextarea
              key={file?.id}
              className="flex-grow h-20 font-mono resize-none"
              placeholder="Enter your content here..."
              value={String(value || "")}
              onInput={e => {
                void handleDataChange(name, e.currentTarget.value)
              }}
            />
          )
        default:
          console.log("field not found", field)
          return null
      }
    },
    [file]
  )

  if (!collection || !file) return null

  return (
    <div className="relative z-0 flex flex-col items-center justify-center flex-1 h-screen pt-12 min-w-96 bg-zinc-700">
      <header className="absolute top-0 left-0 right-0 flex items-center w-full h-8 max-w-full px-4 mb-auto overflow-hidden font-mono text-sm font-medium border-b border-black shadow-xl text-zinc-300 bg-zinc-900 text-nowrap text-ellipsis">
        {file.name}

        <a
          href={file.url?.toString()}
          target="_blank"
          className="flex-grow ml-2 mr-auto overflow-hidden border-pink-400 text-zinc-400 text-nowrap text-ellipsis"
          rel="noopener noreferrer">
          {file.url?.toString()}
        </a>
        {/* {file.type !== "template" && (
          <Button
            className={`font-sans h-6 px-2 mr-2 ml-2 border rounded-xl border-green-400 hover:text-white text-zinc-900 bg-green-400 ${
              publishLoading ? "bg-green-500 text-zinc-900 animate-pulse" : ""
            }`}
            onClick={handlePublishFile}>
            Post
          </Button>
        )} */}
        <Button
          variant="ghost"
          size="icon"
          onClick={onClose}
          className="w-6 h-6 rounded-full">
          <X />
        </Button>
      </header>

      <Card className="w-5/6 p-4 my-10 overflow-y-scroll">
        {file.type === "asset" ? (
          <div className="flex flex-col gap-2">
            <img src={file.url?.toString()} alt="Selected Asset" />
          </div>
        ) : (
          <div className="flex flex-col h-full space-y-2">
            {collection.fields.map(field =>
              renderFieldWithLabel(field.name, field)
            )}
          </div>
        )}
      </Card>
    </div>
  )
}
