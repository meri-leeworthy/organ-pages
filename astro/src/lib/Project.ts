import { LoroDoc, LoroMap } from "loro-crdt"
import { randomId } from "./utils"
import { siteTypes, themeTypes, type OrganUserModel } from "./types"
import { Collection, type CollectionMap } from "./Collection"
import type { fileTypeMap, Page, File } from "./File"
import { EventEmitter } from "./EventEmitter"

export class Project extends EventEmitter {
  /* 
  Project is a wrapper around a LoroDoc that encapsulates project-specific functionality 
  It is provided by Store for user convenience
  */

  private type: "site" | "theme"
  private projectId: string
  private created: Date
  private updated: Date
  private projectActiveFile: File | null
  private projectActivePage: Page | null

  // doc contains collections, metadata
  private doc: LoroDoc

  constructor(type: "site" | "theme", themeId?: string) {
    super()
    this.projectId = randomId()
    this.type = type
    this.created = new Date()
    this.updated = new Date()
    this.doc = new LoroDoc()
    this.projectActiveFile = null
    this.projectActivePage = null

    this.doc.getMap("collections")

    if (type === "site") {
      if (!themeId) {
        throw new Error("Theme ID is required for site creation")
      }
      this.initDefaultSite(themeId)
    } else {
      this.initDefaultTheme()
    }
  }

  get id() {
    return this.projectId
  }

  get docJSON() {
    return this.doc.toJSON()
  }

  private get collections() {
    return this.doc.getMap("collections")
  }

  private get meta() {
    return this.doc.getMap("meta")
  }

  get name() {
    return String(this.meta.get("name"))
  }

  get themeId() {
    return String(this.meta.get("themeId"))
  }

  set themeId(themeId: string) {
    this.updated = new Date()
    this.meta.set("themeId", themeId)
  }

  set name(name: string) {
    this.updated = new Date()
    this.meta.set("name", name)
  }

  get activeFile() {
    return this.projectActiveFile
  }

  set activeFile(file: File | null) {
    console.log("setting active file", file)
    this.projectActiveFile = file
    this.emit("update")
  }

  get activePage() {
    return this.projectActivePage
  }

  set activePage(page: Page | null) {
    this.projectActivePage = page
    this.emit("update")
  }

  private addCollection<TCollection extends string>(
    collectionName: TCollection,
    model: OrganUserModel
  ) {
    this.updated = new Date()
    const map = new LoroMap<CollectionMap<TCollection>>()
    this.collections.setContainer(collectionName, map)
    return new Collection<TCollection>(collectionName, map, model)
  }

  getCollection<TCollection extends string>(name: TCollection) {
    return new Collection<TCollection>(
      name,
      this.collections.get(name) as LoroMap<CollectionMap<TCollection>>
    )
  }

  getCollections() {
    return this.collections
      .entries()
      .map(
        ([name, map]) =>
          new Collection(name, map as LoroMap<CollectionMap<string>>)
      )
  }

  // TODO: how to delete nodes from files tree?
  private removeUserModel(name: string) {
    this.updated = new Date()
  }

  private initDefaultTheme() {
    this.updated = new Date()
    this.name = "New Theme"

    this.addCollection("template", {
      content: { type: "text", required: true },
    })

    this.addCollection("partial", {
      content: { type: "text", required: true },
    })

    this.addCollection("text", {
      content: { type: "text", required: true },
    })

    this.addCollection("asset", {
      mime_type: { type: "string", required: true },
      url: { type: "string", required: false },
      alt: { type: "string", required: false },
    })

    const template = this.createFile("index", "template")
    console.log("template", template)
    template.content.insert(0, defaultHbs)

    const style = this.createFile("style", "text")
    style.content.insert(0, defaultCss)
  }

  private initDefaultSite(themeId: string) {
    this.updated = new Date()
    // create a new site
    this.name = "New Site"
    this.themeId = themeId

    this.addCollection("page", {
      template: { type: "string", required: true },
      title: { type: "string", required: true },
      body: { type: "richtext", required: true },
      url: { type: "string", required: false },
    })

    this.addCollection("post", {
      title: { type: "string", required: true },
      body: { type: "richtext", required: true },
      url: { type: "string", required: false },
    })

    this.addCollection("asset", {
      mime_type: { type: "string", required: true },
      url: { type: "string", required: false },
      alt: { type: "string", required: false },
    })

    // add files to the site
    const main = this.createFile("main", "page")
    main.body.insert(0, "Hello World")

    const post = this.createFile("post", "post")
    post.body.insert(0, "Hello World")
  }

  createFile<TFile extends keyof fileTypeMap>(name: string, type: TFile) {
    // check if type is valid
    if (this.name === "theme") {
      if (!(themeTypes as readonly string[]).includes(type)) {
        throw new Error("Invalid file type for theme")
      }
    } else if (this.type === "site") {
      if (!(siteTypes as readonly string[]).includes(type)) {
        // check the models
        if (!this.collections.get(type)) {
          throw new Error("Invalid file type for site")
        }
      }
    }

    return this.getCollection(type).createFile(name).into(type)
  }

  save() {
    this.updated = new Date()
    // this should export all projects
    // and save to IndexedDB
  }
}

const defaultCss = `* {
  font-family: sans-serif;
}

h1 {
  font-size: 2rem;
  font-weight: bold;
}

h2 {
  font-size: 1.5rem;
  font-weight: bold;
}
  
img {
  width: 80%;
}`

const defaultHbs = `<!DOCTYPE html>
<html lang="en">

<head>
<link rel="stylesheet" href="style.css" />
<title>{{title}}</title>
</head>

<body>
<h1>{{title}}</h1>
{{{content}}}
</body>
</html>`
