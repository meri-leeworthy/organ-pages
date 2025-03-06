import { EventEmitter } from "./EventEmitter"
import { Project } from "./Project"

export class Store extends EventEmitter {
  /*
  The state management singleton which holds the active site, the active theme, 
  a list of references to other projects, and maybe the user?
  */

  private projects: string[]
  private storeActiveTheme: Project | null
  private storeActiveSite: Project | null
  private storeActiveProject: Project | null

  constructor() {
    super()
    this.projects = []
    this.storeActiveTheme = null
    this.storeActiveSite = null
    this.storeActiveProject = null
  }

  initDefault() {
    const defaultTheme = new Project("theme")
    defaultTheme.on("update", () => {
      this.emit("update")
    })
    this.projects.push(defaultTheme.id)
    const defaultSite = new Project("site", defaultTheme.id)
    defaultSite.on("update", () => {
      this.emit("update")
    })
    this.projects.push(defaultSite.id)
    this.storeActiveTheme = defaultTheme
    this.storeActiveSite = defaultSite
    this.storeActiveProject = defaultSite
    this.emit("update")
  }

  get activeSite() {
    return this.storeActiveSite
  }

  set activeSite(site: Project | null) {
    this.storeActiveSite = site
    this.storeActiveSite?.on("update", () => {
      this.emit("update")
    })
    this.emit("update")
  }

  get activeTheme() {
    return this.storeActiveTheme
  }

  set activeTheme(theme: Project | null) {
    this.storeActiveTheme = theme
    this.storeActiveTheme?.on("update", () => {
      this.emit("update")
    })
    this.emit("update")
  }

  get activeProject() {
    return this.storeActiveProject
  }

  set activeProject(project: Project | null) {
    this.storeActiveProject = project
    this.storeActiveProject?.on("update", () => {
      this.emit("update")
    })
    this.emit("update")
  }

  loadSite() {
    // TODO: this should load the site from IndexedDB
  }

  loadTheme() {
    // TODO: this should load the theme from IndexedDB
  }

  export() {
    // TODO: this should export all projects and save to IndexedDB
  }

  import(data: any) {
    // TODO: this should import all projects and load from IndexedDB
  }
}
