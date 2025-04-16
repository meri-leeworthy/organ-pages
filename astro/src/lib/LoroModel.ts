// This file provides TypeScript interfaces for the Rust/WASM Loro implementation

export enum ProjectType {
  Site = 0,
  Theme = 1,
}

export enum CollectionType {
  Page = 0,
  Post = 1,
  Template = 2,
  Partial = 3,
  Asset = 4,
  Text = 5,
}

export enum FieldType {
  RichText = 0,
  Text = 1,
  List = 2,
  Map = 3,
  DateTime = 4,
  String = 5,
  Number = 6,
  Object = 7,
  Array = 8,
  Blob = 9,
}

export interface EventEmitter {
  on(eventName: string, callback: (data: any) => void): Function
  off(eventName: string, callback: Function): void
  emit(eventName: string, data: any): void
}

export interface Store extends EventEmitter {
  init_default(): void
  get_active_theme(): Project | null
  get_active_site(): Project | null
  get_active_project(): Project | null
  set_active_theme(theme: Project): void
  set_active_site(site: Project): void
  set_active_project(project: Project): void
  export_to_idb(): any
  import_from_idb(): void
}

export interface Project {
  id(): string
  name(): string
  set_name(name: string): void
  theme_id(): string | null
  set_theme_id(theme_id: string): void
  get_active_file(): File | null
  set_active_file(file: File): void
  on(eventName: string, callback: (data: any) => void): Function
  off(eventName: string, callback: Function): void
  get_collection(name: string): Collection
  get_collections(): Collection[]
  create_file(name: string, collection_type: string): File
  save(): void
  to_json(): any
}

export interface Collection {
  name(): string
  add_field(name: string, field_type: FieldType, required: boolean): void
  get_field(name: string): any
  get_fields(): any[]
  create_file(name: string): File
  get_files(): File[]
}

export interface File {
  id(): string
  name(): string
  set_name(name: string): void
  collection_type(): string

  // Methods for page and post types
  init_body_with_content(content: string): void
  init_body(): void
  get_body(): string
  set_title(title: string): void
  get_title(): string

  // Methods for template, partial, and text types
  set_content(content: string): void
  get_content(): string

  // General methods
  set_field(field: string, value: string): void
  get_field(field: string): any

  // URL handling for various types
  set_url(url: string): void
  get_url(): string

  // Methods for asset type
  set_mime_type(mime_type: string): void
  get_mime_type(): string
  set_alt(alt: string): void
  get_alt(): string

  to_json(): any
}
