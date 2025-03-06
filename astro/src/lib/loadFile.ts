import type { Collection } from "./types"
import type { BlobStore } from "@/components/useBlobStore"
import { fileTypeFromBuffer } from "file-type"
import type { Project } from "./Project"
import type { File as OrganFile, Page, Partial, Template, Text } from "./File"

export async function loadTextFile(
  file: File,
  type: Collection,
  project: Project
): Promise<OrganFile> {
  const content = await file.text()
  const newFile = project.createFile(file.name, type).into(type)
  if (type === "page") (newFile as Page).body.insert(0, content)
  if (type === "text") (newFile as Text).content.insert(0, content)
  if (type === "template") (newFile as Template).content.insert(0, content)
  if (type === "partial") (newFile as Partial).content.insert(0, content)

  return newFile
}

export async function loadAssetFile(
  file: File,
  project: Project,
  blobStore: BlobStore
): Promise<OrganFile> {
  const fileBuffer = await file.arrayBuffer()
  const fileType = await fileTypeFromBuffer(fileBuffer)

  const newFile = project.createFile(file.name, "asset").into("asset")
  if (fileType && fileType.mime) newFile.mimeType = fileType.mime

  const blob = new Blob([fileBuffer])
  blobStore.addBlob(newFile.id, blob)

  return newFile
}
