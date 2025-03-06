// export interface FileData<
//   TData extends CollectionSchema = Record<string, any>,
// > {
//   id: number
//   name: string
//   type: Collection
//   data?: TData
//   blob_url?: string
//   url?: string
// }

// export const fileDataDefault: FileData = {
//   id: 0,
//   name: "",
//   type: "template",
//   data: {
//     body: {
//       type: "html",
//       content: "<p>Hello World</p>",
//     },
//   },
// }

export interface SelectedFiles {
  activeFileId: `${number}@${number}` | null
  contentFileId: `${number}@${number}`
}

export const themeTypes = ["template", "partial", "text", "asset"] as const
export const siteTypes = ["page", "asset"] as const

export type ThemeType = (typeof themeTypes)[number]
export type SiteType = (typeof siteTypes)[number]

export type Collection = ThemeType | SiteType

export type OrganUserModelField = {
  type: OrganUserModelFieldType
  required: boolean
}

export type OrganUserModel = {
  [fieldName: string]: OrganUserModelField
}

// list of valid types for user models
export const organUserModelFieldTypes = [
  "richtext",
  "text",
  "list",
  "map",
  "datetime",
  "string",
  "number",
  "object",
  "array",
  "blob",
] as const

export type OrganUserModelFieldType = (typeof organUserModelFieldTypes)[number]

export const extensionMap = {
  template: "hbs",
  partial: "hbsp",
} as const

export const headingMap = {
  template: "Templates",
  partial: "Partials",
  text: "Text",
  blob: "Template Assets",
  page: "Pages",
  asset: "Assets",
  post: "Posts",
} as const

export type BodySchema<T extends "plaintext" | "html" = "plaintext"> = {
  type: T
  content: string
}

export type PageSchema = {
  url?: string
  template: number
  title: string
  body: BodySchema<"html">
}

export type PostSchema = {
  url?: string
  template: number
  body: BodySchema<"html">
  title: string
  date: string
  tags: string[]
}

export type TemplateAssetSchema = {
  url?: string
  body: BodySchema<"plaintext">
}

export type TemplateSchema = {
  body: BodySchema<"plaintext">
}

export type PartialSchema = {
  body: BodySchema<"html">
}

export type AssetSchema = {
  url?: string
  mime_type: string
}

export type CollectionSchema =
  | PageSchema
  | PostSchema
  | TemplateAssetSchema
  | AssetSchema
  | TemplateSchema
  | PartialSchema
  | Record<string, string>

export function validateModel(maybeModel: unknown): OrganUserModel {
  if (typeof maybeModel === "object" && maybeModel !== null) {
    for (const [fieldName, unknownFieldDescription] of Object.entries(
      maybeModel
    )) {
      const maybeFieldDescription = unknownFieldDescription as unknown
      if (
        typeof maybeFieldDescription !== "object" ||
        maybeFieldDescription === null
      ) {
        throw new Error(`Field description for ${fieldName} is not an object`)
      }
      if (
        "type" in maybeFieldDescription &&
        typeof maybeFieldDescription["type"] === "string"
      ) {
        if (
          !(organUserModelFieldTypes as readonly string[]).includes(
            maybeFieldDescription["type"]
          )
        ) {
          throw new Error(
            `Field type for ${fieldName} is not a valid field type`
          )
        }
      }
      if ("required" in maybeFieldDescription) {
        if (typeof maybeFieldDescription["required"] !== "boolean") {
          throw new Error(`Field required for ${fieldName} is not a boolean`)
        }
      }
    }
  }
  return maybeModel as OrganUserModel
}
