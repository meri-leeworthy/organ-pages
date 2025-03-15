// ProjectAdapter.ts
// This file provides adapters for the WASM-based Project model to work with existing UI components

import type { Project, Collection, File } from "./LoroModel";
import { FileAdapter, createFileAdapter } from "./FileAdapter";
import { CollectionAdapter, createCollectionAdapter } from "./CollectionAdapter";

// Adapter class that makes the WASM-based Project model compatible with the existing UI
export class ProjectAdapter {
  private project: Project;
  private activeFile: FileAdapter | null = null;

  constructor(project: Project) {
    this.project = project;
  }

  // Core properties
  get id(): string {
    return this.project.id();
  }

  get name(): string {
    return this.project.name();
  }

  set name(value: string) {
    this.project.set_name(value);
  }

  get themeId(): string | null {
    return this.project.theme_id() || null;
  }

  set themeId(value: string) {
    if (value) {
      this.project.set_theme_id(value);
    }
  }

  // Collection management
  getCollection(name: string): CollectionAdapter | null {
    try {
      const collection = this.project.get_collection(name);
      return createCollectionAdapter(collection);
    } catch (e) {
      console.error(`Failed to get collection ${name}:`, e);
      return null;
    }
  }

  getCollections(): CollectionAdapter[] {
    try {
      const collections = this.project.get_collections();
      return Array.from(collections).map(collection => 
        createCollectionAdapter(collection)
      );
    } catch (e) {
      console.error("Failed to get collections:", e);
      return [];
    }
  }

  // File management
  createFile(name: string, type: string): FileAdapter | null {
    try {
      const file = this.project.create_file(name, type);
      return createFileAdapter(file);
    } catch (e) {
      console.error(`Failed to create file ${name} of type ${type}:`, e);
      return null;
    }
  }

  // Active file handling
  get activeFile(): FileAdapter | null {
    if (this.activeFile) return this.activeFile;
    
    try {
      const file = this.project.get_active_file();
      if (file) {
        this.activeFile = createFileAdapter(file);
        return this.activeFile;
      }
      return null;
    } catch (e) {
      console.warn("Failed to get active file:", e);
      return null;
    }
  }

  set activeFile(file: FileAdapter | null) {
    this.activeFile = file;
    if (file) {
      this.project.set_active_file(file.getRawFile());
    }
  }

  // Event handling
  on(eventName: string, callback: (data: any) => void): Function {
    return this.project.on(eventName, callback);
  }

  off(eventName: string, callback: Function): void {
    this.project.off(eventName, callback);
  }

  // Persistence
  save(): void {
    this.project.save();
  }

  // Export to JSON
  toJSON(): any {
    return this.project.to_json();
  }

  // Get the raw WASM Project object
  getRawProject(): Project {
    return this.project;
  }
}

// Helper function to create a ProjectAdapter from a WASM Project
export function createProjectAdapter(project: Project): ProjectAdapter {
  return new ProjectAdapter(project);
}