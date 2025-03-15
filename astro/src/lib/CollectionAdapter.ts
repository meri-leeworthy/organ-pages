// CollectionAdapter.ts
// This file provides adapters for the WASM-based Collection model to work with existing UI components

import type { Collection, File, FieldType } from "./LoroModel";
import { FileAdapter, createFileAdapter } from "./FileAdapter";

// Adapter class that makes the WASM-based Collection model compatible with the existing UI
export class CollectionAdapter {
  private collection: Collection;

  constructor(collection: Collection) {
    this.collection = collection;
  }

  // Core properties
  get name(): string {
    return this.collection.name();
  }

  // Field management
  addField(name: string, fieldType: FieldType, required: boolean): void {
    try {
      this.collection.add_field(name, fieldType, required);
    } catch (e) {
      console.error(`Failed to add field ${name}:`, e);
    }
  }

  getField(name: string): any {
    try {
      return this.collection.get_field(name);
    } catch (e) {
      console.warn(`Failed to get field ${name}:`, e);
      return null;
    }
  }

  getFields(): any[] {
    try {
      return this.collection.get_fields();
    } catch (e) {
      console.error("Failed to get fields:", e);
      return [];
    }
  }

  // File operations
  createFile(name: string): FileAdapter | null {
    try {
      const file = this.collection.create_file(name);
      return createFileAdapter(file);
    } catch (e) {
      console.error(`Failed to create file ${name}:`, e);
      return null;
    }
  }

  getFiles(): FileAdapter[] {
    try {
      const files = this.collection.get_files();
      return Array.from(files).map(file => createFileAdapter(file));
    } catch (e) {
      console.error("Failed to get files:", e);
      return [];
    }
  }

  // Get the raw WASM Collection object
  getRawCollection(): Collection {
    return this.collection;
  }
}

// Helper function to create a CollectionAdapter from a WASM Collection
export function createCollectionAdapter(collection: Collection): CollectionAdapter {
  return new CollectionAdapter(collection);
}