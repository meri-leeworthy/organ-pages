/**
 * Message types that can be sent to the WASM Actor
 * These types should match the Rust message enum
 */

// Field definition for collections
export interface FieldDefinition {
  name: string
  type: string
  required: boolean
}

// Site and Theme operations
interface CreateSiteMessage {
  CreateSite: {
    name: string
    theme_id: string
  }
}

interface GetSiteMessage {
  GetSite: null
}

interface CreateThemeMessage {
  CreateTheme: {
    name: string
  }
}

interface GetThemeMessage {
  GetTheme: null
}

export type ProjectType = "site" | "theme"

// Collection operations
interface AddCollectionMessage {
  AddCollection: {
    project_type: ProjectType
    name: string
    fields: FieldDefinition[]
  }
}

interface GetCollectionMessage {
  GetCollection: {
    project_type: ProjectType
    name: string
  }
}

interface ListCollectionsMessage {
  ListCollections: {
    project_type: ProjectType
  }
}

// File operations
interface CreateFileMessage {
  CreateFile: {
    project_type: ProjectType
    collection_name: string
    name: string
  }
}

interface UpdateFileMessage {
  UpdateFile: {
    project_type: ProjectType
    collection_name: string
    file_id: string
    updates: FileUpdate
  }
}

interface GetFileMessage {
  GetFile: {
    project_type: ProjectType
    collection_name: string
    file_id: string
  }
}

interface ListFilesMessage {
  ListFiles: {
    project_type: ProjectType
    collection_name: string
  }
}

// Storage operations
interface SaveStateMessage {
  SaveState: {
    site_id: string | undefined
    theme_id: string | undefined
    active_project_type: ProjectType | undefined
  }
}

interface LoadStateMessage {
  LoadState: {
    site_id: string | undefined
    theme_id: string | undefined
  }
}

interface ExportProjectMessage {
  ExportProject: {
    project_type: string
  }
}

interface ImportProjectMessage {
  ImportProject: {
    data: number[] | Uint8Array
    id: string
    project_type: ProjectType
    created: number
    updated: number
  }
}

// Rendering operations
interface RenderFileMessage {
  RenderFile: {
    file_id: string
    context: any
  }
}

// Initialization
interface InitDefaultMessage {
  InitDefault: null
}

// File update types
export type FileUpdate =
  | { SetName: string }
  | { SetContent: string }
  | { SetBody: string }
  | { SetTitle: string }
  | { SetField: { name: string; value: string } }
  | { SetUrl: string }
  | { SetMimeType: string }
  | { SetAlt: string }

// Document operations for ProseMirror integration
interface InitializeDocumentMessage {
  InitializeDocument: {
    document_id: string
    schema: string // JSON stringified ProseMirror schema
  }
}

interface GetDocumentMessage {
  GetDocument: {
    document_id: string
  }
}

interface ApplyStepsMessage {
  ApplySteps: {
    document_id: string
    steps: any[] // Serialized ProseMirror steps
    version: number
  }
}

// Union of all message types
export type Message =
  | CreateSiteMessage
  | GetSiteMessage
  | CreateThemeMessage
  | GetThemeMessage
  | AddCollectionMessage
  | GetCollectionMessage
  | ListCollectionsMessage
  | CreateFileMessage
  | UpdateFileMessage
  | GetFileMessage
  | ListFilesMessage
  | SaveStateMessage
  | LoadStateMessage
  | ExportProjectMessage
  | ImportProjectMessage
  | RenderFileMessage
  | InitDefaultMessage
  | InitializeDocumentMessage
  | GetDocumentMessage
  | ApplyStepsMessage

// Response from the Actor
export type Response<T> =
  | { Success: T } // Contains serialized JSON value
  | { Error: string } // Contains error message

// Types for React Context
export interface Site {
  id: string
  name: string
  themeId: string
}

export interface Theme {
  id: string
  name: string
}

export interface Collection {
  id: string
  name: string
  fields: FieldDefinition[]
}

export interface File {
  id: string
  name: string
  collection: string
  projectType: ProjectType
  // Additional properties based on collection type
  [key: string]: any
}

// UI active file reference
export interface ActiveFile {
  projectType: ProjectType
  collectionName: string
  fileId: string
}

export interface StoreState {
  initialized: boolean
  loading: boolean
  site?: Site
  theme?: Theme
  collections: Collection[]
  files: { [collectionName: string]: File[] }
  error?: string

  // UI state (not persisted to WASM)
  activeProjectType: ProjectType
  siteActiveFile?: ActiveFile
  siteActivePage?: ActiveFile
  themeActiveFile?: ActiveFile
}

// Document types for ProseMirror integration
export interface DocumentData {
  version: number
  content: any // JSON representation of document
}

export interface StoreContextValue {
  state: StoreState
  // Core operations
  initialize: () => Promise<void>
  createSite: (name: string, themeId: string) => Promise<Site>
  getSite: () => Promise<Site>
  createTheme: (name: string) => Promise<Theme>
  getTheme: () => Promise<Theme>
  addCollection: (
    projectType: ProjectType,
    name: string,
    fields: FieldDefinition[]
  ) => Promise<Collection>
  getCollection: (projectType: ProjectType, name: string) => Promise<Collection>
  listCollections: (projectType?: ProjectType) => Promise<Collection[]>
  createFile: (
    projectType: ProjectType,
    collectionName: string,
    name: string
  ) => Promise<File>
  updateFile: (
    projectType: ProjectType,
    collectionName: string,
    fileId: string,
    update: FileUpdate
  ) => Promise<void>
  getFile: (
    projectType: ProjectType,
    collectionName: string,
    fileId: string
  ) => Promise<File>
  listFiles: (
    projectType: ProjectType,
    collectionName: string
  ) => Promise<File[]>
  renderFile: (fileId: string, context: any) => Promise<string>
  initializeDocument: (
    documentId: string,
    schema: string
  ) => Promise<Response<void>>
  getDocument: (documentId: string) => Promise<Response<DocumentData>>
  applySteps: (
    documentId: string,
    steps: any[],
    version: number
  ) => Promise<Response<void>>
  saveState: (
    siteId?: string,
    themeId?: string,
    activeProjectType?: ProjectType
  ) => Promise<Response<void>>
  loadState: (siteId?: string, themeId?: string) => Promise<boolean>

  // UI state setters (persisted to localStorage)
  setActiveProjectType: (projectType: ProjectType) => void
  setSiteActiveFile: (activeFile: ActiveFile | undefined) => void
  setSiteActivePage: (
    activePage: Omit<ActiveFile, "projectType"> | undefined
  ) => void
  setThemeActiveFile: (activeFile: ActiveFile | undefined) => void
}
