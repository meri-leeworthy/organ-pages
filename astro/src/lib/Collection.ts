import { LoroMap, LoroTree, LoroTreeNode } from "loro-crdt"
import { validateModel, type OrganUserModel } from "./types"
import { Asset, File, Page, Partial, Post, Template, Text } from "./File"

export interface CollectionMap<T extends string = string>
  extends Record<string, unknown> {
  name: T
  files: LoroTree
  fields: LoroMap<OrganUserModel>
}

export class Collection<T extends string> {
  /*
  Collection is a wrapper around a LoroMap that encapsulates collection-specific functionality
  It takes a LoroMap as an argument and validates it

  If this takes a LoroMap as the parent in the constructor, the constructor will create a new child LoroMap as the model
  using the provided fields
  */

  private collectionName: T
  private map: LoroMap<CollectionMap<T>>

  constructor(name: T, map: LoroMap<CollectionMap<T>>, maybeModel?: unknown) {
    this.collectionName = name
    this.map = map

    if (!this.fileTree) {
      this.map.setContainer("files", new LoroTree())
    }

    const existingFields = this.map.get("fields")
    console.log("existing fields before setting new LoroMap:", existingFields)
    if (!existingFields) {
      console.log("setting fields with new LoroMap")
      this.map.setContainer("fields", new LoroMap<OrganUserModel>())
    }

    console.log("name", this.name)

    if (maybeModel) {
      const validatedModel = validateModel(maybeModel)
      for (const [fieldName, fieldDescription] of Object.entries(
        validatedModel
      )) {
        console.log("setting fieldName", fieldName)
        console.log("fieldDescription", fieldDescription)
        this.fields.set(fieldName, fieldDescription)
      }
    }

    console.log("fields", Array.from(this.fields.entries()))
  }

  get name() {
    return this.collectionName
  }

  get dirId(): `${number}@${number}` {
    return this.map.get("dirId") as `${number}@${number}`
  }

  private get fileTree(): LoroTree | undefined {
    return this.map.get("files") as LoroTree | undefined
  }

  get files(): File[] {
    return this.fileTree?.getNodes().map(node => new File(node)) || []
  }

  get fields(): LoroMap<OrganUserModel> {
    return this.map.get("fields") as LoroMap<OrganUserModel>
  }

  private set dirId(dirId: `${number}@${number}`) {
    this.map.set("dirId", dirId)
  }

  createFile<TFile = T>(name: string): CreateFileResult<TFile> {
    const newFile = this.fileTree?.createNode()
    if (!newFile)
      throw new Error("Failed to create file, files not initialized")
    switch (this.name) {
      case "page":
        return new Page(newFile, name) as CreateFileResult<TFile>
      case "post":
        return new Post(newFile, name) as CreateFileResult<TFile>
      case "template":
        return new Template(newFile, name) as CreateFileResult<TFile>
      case "partial":
        return new Partial(newFile, name) as CreateFileResult<TFile>
      case "asset":
        return new Asset(newFile, name) as CreateFileResult<TFile>
      case "text":
        return new Text(newFile, name) as CreateFileResult<TFile>
      default:
        return new File(newFile, name, this.name) as CreateFileResult<TFile>
    }
  }

  getFields() {
    return Array.from(this.fields.entries())
  }

  getField(name: string) {
    return this.fields.get(name)
  }
}

type CreateFileResult<T> = T extends "page"
  ? Page
  : T extends "post"
    ? Post
    : T extends "template"
      ? Template
      : T extends "partial"
        ? Partial
        : T extends "text"
          ? Text
          : T extends "asset"
            ? Asset
            : File
