// StoreAdapter.ts
// This file provides adapters for the WASM-based Store model to work with existing UI components

import type { Store, Project } from "./LoroModel";
import { ProjectAdapter, createProjectAdapter } from "./ProjectAdapter";

// Adapter class that makes the WASM-based Store model compatible with the existing UI
export class StoreAdapter {
  private store: Store;
  private activeTheme: ProjectAdapter | null = null;
  private activeSite: ProjectAdapter | null = null;
  private activeProject: ProjectAdapter | null = null;

  constructor(store: Store) {
    this.store = store;
  }

  // Initialization
  initDefault(): void {
    this.store.init_default();
    this.updateActiveProjects();
  }

  // Active project handling
  private updateActiveProjects(): void {
    const theme = this.store.get_active_theme();
    if (theme) {
      this.activeTheme = createProjectAdapter(theme);
    }

    const site = this.store.get_active_site();
    if (site) {
      this.activeSite = createProjectAdapter(site);
    }

    const project = this.store.get_active_project();
    if (project) {
      this.activeProject = createProjectAdapter(project);
    }
  }

  get activeTheme(): ProjectAdapter | null {
    if (!this.activeTheme) {
      const theme = this.store.get_active_theme();
      if (theme) {
        this.activeTheme = createProjectAdapter(theme);
      }
    }
    return this.activeTheme;
  }

  set activeTheme(theme: ProjectAdapter | null) {
    this.activeTheme = theme;
    if (theme) {
      this.store.set_active_theme(theme.getRawProject());
    }
  }

  get activeSite(): ProjectAdapter | null {
    if (!this.activeSite) {
      const site = this.store.get_active_site();
      if (site) {
        this.activeSite = createProjectAdapter(site);
      }
    }
    return this.activeSite;
  }

  set activeSite(site: ProjectAdapter | null) {
    this.activeSite = site;
    if (site) {
      this.store.set_active_site(site.getRawProject());
    }
  }

  get activeProject(): ProjectAdapter | null {
    if (!this.activeProject) {
      const project = this.store.get_active_project();
      if (project) {
        this.activeProject = createProjectAdapter(project);
      }
    }
    return this.activeProject;
  }

  set activeProject(project: ProjectAdapter | null) {
    this.activeProject = project;
    if (project) {
      this.store.set_active_project(project.getRawProject());
    }
  }

  // Event handling
  on(eventName: string, callback: (data: any) => void): Function {
    const listener = this.store.on(eventName, (data) => {
      // Update active projects when store updates
      if (eventName === "update") {
        this.updateActiveProjects();
      }
      callback(data);
    });
    return listener;
  }

  off(eventName: string, callback: Function): void {
    this.store.off(eventName, callback);
  }

  // Persistence
  export(): void {
    this.store.export_to_idb();
  }

  import(data: any): void {
    this.store.import_from_idb();
    this.updateActiveProjects();
  }

  // Access the raw WASM store
  getRawStore(): Store {
    return this.store;
  }
}

// Helper function to create a StoreAdapter from a WASM Store
export function createStoreAdapter(store: Store): StoreAdapter {
  return new StoreAdapter(store);
}