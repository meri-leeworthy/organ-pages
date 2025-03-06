import { LoroList, LoroText, LoroTreeNode, type Container } from "loro-crdt"

export class File {
  /* 
  File is a wrapper around a LoroTreeNode that encapsulates file-specific functionality 
  */

  node: LoroTreeNode<Record<string, unknown>>

  constructor(node: LoroTreeNode, name?: string, type?: string) {
    this.node = node
    if (name) this.node.data.set("name", name)
    if (type) this.node.data.set("type", type)
  }

  get id(): `${number}@${number}` {
    return this.node.id
  }

  get name() {
    return String(this.node.data.get("name"))
  }

  set name(name: string) {
    this.node.data.set("name", name)
  }

  get type() {
    return String(this.node.data.get("type"))
  }

  tryGetField<T extends string>(field: T): unknown {
    return this.node.data.get(field) as unknown
  }

  setField<T extends string>(field: T, value: unknown) {
    this.node.data.set(field, value)
  }

  into<T extends keyof fileTypeMap>(type: T): fileTypeMap[T] {
    switch (type) {
      case "page":
        return new Page(this.node, this.name) as fileTypeMap[T]
      case "post":
        return new Post(this.node, this.name) as fileTypeMap[T]
      case "asset":
        return new Asset(this.node, this.name) as fileTypeMap[T]
      case "template":
        return new Template(this.node, this.name) as fileTypeMap[T]
      case "partial":
        return new Partial(this.node, this.name) as fileTypeMap[T]
      case "text":
        return new Text(this.node, this.name) as fileTypeMap[T]
      default:
        throw new Error(`Unknown file type: ${type}`)
    }
  }
}

export class Page extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "page")
    this.node.data.setContainer("body", new LoroText())
    this.node.data.set("title", name)
  }

  get body(): LoroText {
    return this.node.data.get("body") as LoroText
  }

  get title(): string {
    return this.node.data.get("title") as string
  }

  set url(url: string) {
    this.node.data.set("url", url)
  }

  get url() {
    return this.node.data.get("url") as string
  }
}

export class Post extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "post")
    this.node.data.setContainer("body", new LoroText())
    this.node.data.setContainer("tags", new LoroList())
    this.node.data.set("date", new Date())
    this.node.data.set("title", name)
  }

  get body(): LoroText {
    return this.node.data.get("body") as LoroText
  }

  get date(): Date {
    return this.node.data.get("date") as Date
  }

  get tags(): LoroList {
    return this.node.data.get("tags") as LoroList
  }

  get title(): string {
    return this.node.data.get("title") as string
  }

  set url(url: string) {
    this.node.data.set("url", url)
  }

  get url() {
    return this.node.data.get("url") as string
  }
}

export class Asset extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "asset")
    this.node.data.set("mimeType", "image/png")
    this.node.data.set("url", "")
    this.node.data.set("alt", "")
  }

  get mimeType(): string {
    return this.node.data.get("mimeType") as string
  }

  set mimeType(mimeType: string) {
    this.node.data.set("mimeType", mimeType)
  }

  set url(url: string) {
    this.node.data.set("url", url)
  }

  get url() {
    return this.node.data.get("url") as string
  }

  get alt(): string {
    return this.node.data.get("alt") as string
  }

  set alt(alt: string) {
    this.node.data.set("alt", alt)
  }
}

export class Template extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "template")
    this.node.data.setContainer("content", new LoroText())
  }

  get content(): LoroText {
    return this.node.data.get("content") as LoroText
  }
}

export class Partial extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "partial")
    this.node.data.setContainer("content", new LoroText())
  }

  get content(): LoroText {
    return this.node.data.get("content") as LoroText
  }
}

export class Text extends File {
  constructor(node: LoroTreeNode, name?: string) {
    super(node, name, "text")
    this.node.data.setContainer("content", new LoroText())
  }

  get content(): LoroText {
    return this.node.data.get("content") as LoroText
  }
}

export type fileTypeMap = {
  page: Page
  post: Post
  asset: Asset
  template: Template
  partial: Partial
  text: Text
}
