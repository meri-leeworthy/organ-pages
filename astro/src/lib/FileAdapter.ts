// FileAdapter.ts
// This file provides adapters for the WASM-based File model to work with existing UI components

import type { File } from "./LoroModel";

// Adapter class that makes the WASM-based File model compatible with the existing UI
export class FileAdapter {
  private file: File;

  constructor(file: File) {
    this.file = file;
  }

  // Proxy for id
  get id(): string {
    return this.file.id();
  }

  // Proxy for name
  get name(): string {
    try {
      return this.file.name();
    } catch (e) {
      return "";
    }
  }

  set name(value: string) {
    try {
      this.file.set_name(value);
    } catch (e) {
      console.error("Failed to set name:", e);
    }
  }

  // Proxy for type
  get type(): string {
    return this.file.collection_type();
  }

  // Proxy for getting fields
  tryGetField(fieldName: string): any {
    try {
      return this.file.get_field(fieldName);
    } catch (e) {
      console.warn(`Field ${fieldName} not found:`, e);
      return null;
    }
  }

  // Proxy for setting fields
  setField(fieldName: string, value: string): void {
    try {
      this.file.set_field(fieldName, value);
    } catch (e) {
      console.error(`Failed to set field ${fieldName}:`, e);
    }
  }

  // Content handling for different file types
  get content(): string {
    try {
      if (this.type === "template" || this.type === "partial" || this.type === "text") {
        return this.file.get_content();
      } else {
        return "";
      }
    } catch (e) {
      console.warn("Failed to get content:", e);
      return "";
    }
  }

  set content(value: string) {
    try {
      if (this.type === "template" || this.type === "partial" || this.type === "text") {
        this.file.set_content(value);
      }
    } catch (e) {
      console.error("Failed to set content:", e);
    }
  }

  // Body handling for page and post types
  get body(): string {
    try {
      if (this.type === "page" || this.type === "post") {
        return this.file.get_body();
      } else {
        return "";
      }
    } catch (e) {
      console.warn("Failed to get body:", e);
      return "";
    }
  }

  set body(value: string) {
    try {
      if (this.type === "page" || this.type === "post") {
        this.file.set_body(value);
      }
    } catch (e) {
      console.error("Failed to set body:", e);
    }
  }

  // Title handling for page and post types
  get title(): string {
    try {
      if (this.type === "page" || this.type === "post") {
        return this.file.get_title();
      } else {
        return "";
      }
    } catch (e) {
      console.warn("Failed to get title:", e);
      return "";
    }
  }

  set title(value: string) {
    try {
      if (this.type === "page" || this.type === "post") {
        this.file.set_title(value);
      }
    } catch (e) {
      console.error("Failed to set title:", e);
    }
  }

  // URL handling
  get url(): string {
    try {
      return this.file.get_url();
    } catch (e) {
      return "";
    }
  }

  set url(value: string) {
    try {
      this.file.set_url(value);
    } catch (e) {
      console.error("Failed to set URL:", e);
    }
  }

  // Asset-specific methods
  get mimeType(): string {
    try {
      if (this.type === "asset") {
        return this.file.get_mime_type();
      } else {
        return "";
      }
    } catch (e) {
      return "";
    }
  }

  set mimeType(value: string) {
    try {
      if (this.type === "asset") {
        this.file.set_mime_type(value);
      }
    } catch (e) {
      console.error("Failed to set mime type:", e);
    }
  }

  get alt(): string {
    try {
      if (this.type === "asset") {
        return this.file.get_alt();
      } else {
        return "";
      }
    } catch (e) {
      return "";
    }
  }

  set alt(value: string) {
    try {
      if (this.type === "asset") {
        this.file.set_alt(value);
      }
    } catch (e) {
      console.error("Failed to set alt text:", e);
    }
  }

  // Get the raw WASM File object
  getRawFile(): File {
    return this.file;
  }

  // Convert to JSON representation
  toJSON(): any {
    try {
      return this.file.to_json();
    } catch (e) {
      console.error("Failed to convert file to JSON:", e);
      return {};
    }
  }
}

// Helper function to create a FileAdapter from a WASM File
export function createFileAdapter(file: File): FileAdapter {
  return new FileAdapter(file);
}