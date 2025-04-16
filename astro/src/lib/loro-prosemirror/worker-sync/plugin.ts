import { Extension } from "@tiptap/core"
import { Plugin, PluginKey } from "prosemirror-state"
import { wasmClient } from "@/wasm-worker/client"
import { Step } from "prosemirror-transform"
import { EditorView } from "prosemirror-view"
import type { ProjectType } from "@/wasm-worker/types"
import type { StoreContextValue } from "@/wasm-worker/types"
/**
 * Plugin key for identifying the worker sync plugin
 */
export const wasmWorkerSyncPluginKey = new PluginKey("wasmWorkerSync")

/**
 * Interface for plugin options
 */
export interface WasmWorkerSyncOptions {
  store: StoreContextValue | null
  clientId: string
  debug?: boolean
}

/**
 * Interface for plugin state
 */
export interface WasmWorkerSyncState {
  version: number
  initialized: boolean
  projectType: ProjectType
  collectionName: string
  fileId: string
  clientId: string
  debug: boolean
}

/**
 * A ProseMirror plugin for syncing document changes with a WASM worker
 */
export const WasmWorkerSyncPlugin = Extension.create<WasmWorkerSyncOptions>({
  name: "wasmWorkerSync",

  // default options
  addOptions() {
    return {
      store: null,
      clientId: "client-" + Math.random().toString(36).substring(2, 9),
      debug: true, // whether to log debug messages
    }
  },

  addProseMirrorPlugins() {
    const options = this.options

    return [
      new Plugin({
        key: wasmWorkerSyncPluginKey,

        state: {
          init: () => {
            const activeProjectType = options.store?.state.activeProjectType
            const activeFile =
              activeProjectType === "site"
                ? options.store?.state.siteActiveFile
                : options.store?.state.themeActiveFile

            if (options.debug) {
              console.log(
                `[WasmWorkerSync] Initializing plugin for document ${activeFile?.fileId}`
              )
            }

            return {
              version: 0,
              initialized: false,
              projectType: activeProjectType,
              collectionName: activeFile?.collectionName,
              fileId: activeFile?.fileId,
              clientId: options.clientId,
              debug: options.debug,
            } as WasmWorkerSyncState
          },

          apply: (tr, pluginState: WasmWorkerSyncState) => {
            // Check for explicit state updates
            const meta = tr.getMeta(wasmWorkerSyncPluginKey)

            if (meta?.type === "update-state") {
              if (pluginState.debug) {
                console.log(
                  `[WasmWorkerSync] Updating plugin state (${pluginState.fileId}):`,
                  meta.state
                )
              }
              return {
                ...pluginState,
                ...meta.state,
              }
            }

            // Handle document changes
            if (tr.docChanged) {
              if (pluginState.debug) {
                console.log(
                  `[WasmWorkerSync] Document changed (${pluginState.fileId}):`,
                  {
                    steps: tr.steps,
                    version: pluginState.version,
                    meta,
                  }
                )
              }

              // Phase 5: Send local changes to worker
              if (meta?.type !== "remote-update" && pluginState.initialized) {
                // Send the steps to the worker only if this is a local change
                try {
                  // Validate that we have steps to send
                  if (!tr.steps || !tr.steps.length) {
                    if (pluginState.debug) {
                      console.log(
                        `[WasmWorkerSync] No steps to send to worker (${pluginState.fileId})`
                      )
                    }
                    return pluginState
                  }

                  // Extract the steps from the transaction
                  const stepsData = tr.steps
                    .map(step => {
                      try {
                        const stepJSON = step.toJSON()
                        const from = stepJSON["from"]
                        const to = stepJSON["to"]
                        const resolvedFrom = tr.before.resolve(from)
                        const resolvedTo = tr.before.resolve(to)
                        console.log("from", from, "resolvedFrom", resolvedFrom)
                        console.log("to", to, "resolvedTo", resolvedTo)
                        const fromParent = resolvedFrom.parent
                        const toParent = resolvedTo.parent
                        console.log("fromParent", fromParent)
                        console.log("toParent", toParent)

                        if (!resolvedFrom.sameParent(resolvedTo)) {
                          throw new Error(
                            "from and to are not in the same parent"
                          )
                        }

                        if (resolvedFrom.depth !== 1) {
                          throw new Error(
                            "fromParent is not a text node at depth 1"
                          )
                        }

                        if (resolvedTo.depth !== 1) {
                          throw new Error(
                            "toParent is not a text node at depth 1"
                          )
                        }

                        if (fromParent.type.name !== "paragraph") {
                          throw new Error("fromParent is not a paragraph node")
                        }

                        if (toParent.type.name !== "paragraph") {
                          throw new Error("toParent is not a paragraph node")
                        }

                        console.log("from index", resolvedFrom.index())
                        console.log("to index", resolvedTo.index())

                        return stepJSON
                      } catch (e) {
                        console.error(
                          `[WasmWorkerSync] Error serializing step (${pluginState.fileId}):`,
                          e
                        )
                        return null
                      }
                    })
                    .filter(step => step !== null)

                  if (stepsData.length === 0) {
                    console.warn(
                      `[WasmWorkerSync] All steps failed to serialize (${pluginState.fileId})`
                    )
                    return pluginState
                  }

                  if (pluginState.debug) {
                    console.log(
                      `[WasmWorkerSync] Sending ${stepsData.length} steps to worker (${pluginState.fileId}):`,
                      {
                        steps: stepsData,
                        version: pluginState.version,
                      }
                    )
                  }

                  // Send steps to worker asynchronously
                  wasmClient
                    .applySteps(
                      pluginState.fileId,
                      stepsData,
                      pluginState.version
                    )
                    .then(response => {
                      if (pluginState.debug) {
                        console.log(
                          `[WasmWorkerSync] Worker response for steps (${pluginState.fileId}):`,
                          response
                        )
                      }

                      // Check for errors in response
                      if ("Error" in response) {
                        console.error(
                          `[WasmWorkerSync] Worker reported error (${pluginState.fileId}):`,
                          response.Error
                        )
                      }
                    })
                    .catch(error => {
                      console.error(
                        `[WasmWorkerSync] Error sending steps to worker (${pluginState.fileId}):`,
                        error
                      )
                    })
                } catch (error) {
                  console.error(
                    `[WasmWorkerSync] Error processing steps (${pluginState.fileId}):`,
                    error
                  )
                  return pluginState
                }

                // Return state with incremented version for local changes
                return {
                  ...pluginState,
                  version: pluginState.version + 1,
                }
              }
            }

            return pluginState
          },
        },

        view: (view: EditorView) => {
          if (!options.store) {
            throw new Error("Store is not initialized")
          }

          const activeProjectType = options.store.state.activeProjectType
          const activeFile =
            activeProjectType === "site"
              ? options.store.state.siteActiveFile
              : options.store.state.themeActiveFile

          if (options.debug) {
            console.log(
              `[WasmWorkerSync] View initialized (${activeFile?.fileId})`
            )
          }

          if (!activeFile?.fileId) {
            throw new Error("Active file is not initialized")
          }

          // Phase 7: Set up subscription for document changes from worker
          const unsubscribe = wasmClient.onDocumentChange(
            activeFile?.fileId,
            data => {
              if (options.debug) {
                console.log(
                  `[WasmWorkerSync] Received remote changes (${activeFile?.fileId}):`,
                  data
                )
              }

              // Skip our own changes
              if (data.source === options.clientId) {
                if (options.debug) {
                  console.log(
                    `[WasmWorkerSync] Ignoring own changes (${activeFile?.fileId})`
                  )
                }
                return
              }

              // Apply remote steps to local document
              try {
                const { steps, version } = data

                if (steps && Array.isArray(steps) && steps.length > 0) {
                  // Create a transaction with the current editor state
                  const tr = view.state.tr

                  // Track applied steps
                  let appliedSteps = 0
                  const totalSteps = steps.length

                  // Apply each step to the document
                  for (const stepJSON of steps) {
                    try {
                      // Skip null or invalid steps
                      if (!stepJSON || typeof stepJSON !== "object") {
                        console.warn(
                          `[WasmWorkerSync] Skipping invalid step (${activeFile?.fileId})`,
                          stepJSON
                        )
                        continue
                      }

                      // Convert the step JSON back to a Step object
                      const step = Step.fromJSON(view.state.schema, stepJSON)

                      // Validate if step can be applied to current document
                      if (step.apply(tr.doc).failed) {
                        console.warn(
                          `[WasmWorkerSync] Step cannot be applied to current document (${activeFile?.fileId})`,
                          stepJSON
                        )
                        continue
                      }

                      // Apply step to transaction
                      tr.step(step)
                      appliedSteps++
                    } catch (e) {
                      console.error(
                        `[WasmWorkerSync] Error applying step (${activeFile?.fileId}):`,
                        e,
                        stepJSON
                      )
                    }
                  }

                  if (appliedSteps > 0) {
                    // Mark this as a remote update to avoid re-sending
                    tr.setMeta(wasmWorkerSyncPluginKey, {
                      type: "remote-update",
                      source: data.source,
                      version: version,
                    })

                    // Dispatch the transaction to update the editor
                    view.dispatch(tr)

                    if (options.debug) {
                      console.log(
                        `[WasmWorkerSync] Applied ${appliedSteps}/${totalSteps} remote steps (${activeFile?.fileId})`
                      )
                    }
                  } else {
                    console.warn(
                      `[WasmWorkerSync] No steps could be applied out of ${totalSteps} received (${activeFile?.fileId})`
                    )
                  }
                } else {
                  console.warn(
                    `[WasmWorkerSync] Received invalid steps array (${activeFile?.fileId}):`,
                    steps
                  )
                }
              } catch (error) {
                console.error(
                  `[WasmWorkerSync] Error applying remote changes (${activeFile?.fileId}):`,
                  error
                )
              }
            }
          )

          // Initialize document with worker
          setTimeout(async () => {
            try {
              if (options.debug) {
                console.log(
                  `[WasmWorkerSync] Requesting document from worker (${activeFile?.fileId})`
                )
              }

              // Make sure the worker client is initialized
              await wasmClient.init()

              // First, try to initialize the document (idempotent operation)
              try {
                if (options.debug) {
                  console.log(
                    `[WasmWorkerSync] Initializing document (${activeFile?.fileId})`
                  )
                }

                // Initialize document with schema
                await wasmClient.sendMessage({
                  InitializeDocument: {
                    document_id: activeFile?.fileId,
                    schema: JSON.stringify(view.state.schema.spec),
                  },
                })
              } catch (initError) {
                console.warn(
                  `[WasmWorkerSync] Non-critical error during document init (${activeFile?.fileId}):`,
                  initError
                )
                // Continue despite init error - we'll try to get the document anyway
              }

              // Request document from worker
              const response = await wasmClient.sendMessage({
                GetFile: {
                  file_id: activeFile?.fileId,
                  project_type: activeProjectType,
                  collection_name: activeFile?.collectionName,
                },
              })

              // More forgiving check for success response
              if (
                "Success" in response &&
                response.Success &&
                typeof response.Success === "object"
              ) {
                if (options.debug) {
                  console.log(
                    `[WasmWorkerSync] Retrieved document (${activeFile?.fileId}):`,
                    response.Success
                  )
                }

                // Phase 4: Load document content into editor
                try {
                  // Create a transaction to replace the current document
                  const tr = view.state.tr

                  // Get the document content from the response, with fallbacks
                  const responseObj = response.Success as any
                  const docContent = responseObj.content || {
                    type: "doc",
                    content: [
                      {
                        type: "paragraph",
                        content: [{ type: "text", text: "" }],
                      },
                    ],
                  }

                  // Create a document node from the JSON
                  // Make sure we have a valid document structure
                  const schema = view.state.schema

                  // Instead of direct nodeFromJSON which can cause errors,
                  // create a properly structured document with proper content
                  let newDoc
                  try {
                    // Try to parse the document directly
                    if (typeof docContent === "object" && docContent !== null) {
                      newDoc = schema.nodeFromJSON(docContent)
                    } else {
                      throw new Error("Invalid document content format")
                    }

                    // Validate document - check if there's at least one text node with content
                    let hasNonEmptyText = false
                    const walkNodes = (node: any) => {
                      if (
                        node.type === "text" &&
                        node.text &&
                        node.text.trim().length > 0
                      ) {
                        hasNonEmptyText = true
                      } else if (node.content && Array.isArray(node.content)) {
                        node.content.forEach(walkNodes)
                      }
                    }
                    walkNodes(docContent)

                    if (!hasNonEmptyText) {
                      console.log(
                        "[WasmWorkerSync] Document has no non-empty text nodes, creating default"
                      )
                      throw new Error("Document has no non-empty text nodes")
                    }
                  } catch (error) {
                    console.log(
                      "[WasmWorkerSync] Error parsing document, creating default:",
                      error
                    )

                    // Create a properly structured document with at least one paragraph (with no text)
                    newDoc = schema.node("doc", {}, [
                      schema.node("paragraph", {}, []),
                    ])
                  }

                  // Replace the current document content with the loaded content
                  tr.replaceWith(0, view.state.doc.content.size, newDoc.content)

                  // Set meta information to identify this as a remote update
                  tr.setMeta(wasmWorkerSyncPluginKey, {
                    type: "remote-update",
                    source: "initial-load",
                    version: responseObj.version || 0,
                  })

                  // Dispatch the transaction to update the editor
                  view.dispatch(tr)

                  if (options.debug) {
                    console.log(
                      `[WasmWorkerSync] Document loaded into editor (${activeFile?.fileId})`
                    )
                  }

                  // Update plugin state with version
                  const pluginState = wasmWorkerSyncPluginKey.getState(
                    view.state
                  )
                  if (pluginState) {
                    const updateTr = view.state.tr.setMeta(
                      wasmWorkerSyncPluginKey,
                      {
                        type: "update-state",
                        state: {
                          ...pluginState,
                          initialized: true,
                          version: responseObj.version || 0,
                        },
                      }
                    )
                    view.dispatch(updateTr)
                  }
                } catch (err) {
                  console.error(
                    `[WasmWorkerSync] Error loading document content:`,
                    err
                  )
                }
              } else {
                if (options.debug) {
                  console.log(
                    `[WasmWorkerSync] No existing document, initializing new one (${activeFile?.fileId})`
                  )
                }

                // Initialize a new document in the worker
                await wasmClient.initializeDocument(
                  activeFile?.fileId,
                  JSON.stringify(view.state.schema.spec)
                )

                // Update plugin state to mark as initialized
                const pluginState = wasmWorkerSyncPluginKey.getState(view.state)
                if (pluginState) {
                  const updateTr = view.state.tr.setMeta(
                    wasmWorkerSyncPluginKey,
                    {
                      type: "update-state",
                      state: {
                        ...pluginState,
                        initialized: true,
                        version: 0,
                      },
                    }
                  )
                  view.dispatch(updateTr)
                }
              }
            } catch (error) {
              console.error(
                `[WasmWorkerSync] Error initializing document (${activeFile?.fileId}):`,
                error
              )
            }
          }, 0)

          return {
            update: (view: EditorView) => {
              // We'll implement update handling in Phase 7
            },
            destroy: () => {
              if (options.debug) {
                console.log(
                  `[WasmWorkerSync] View destroyed (${activeFile?.fileId})`
                )
              }

              // Clean up document change subscription
              if (unsubscribe) {
                unsubscribe()

                if (options.debug) {
                  console.log(
                    `[WasmWorkerSync] Unsubscribed from document changes (${activeFile?.fileId})`
                  )
                }
              }
            },
          }
        },
      }),
    ]
  },
})

export default WasmWorkerSyncPlugin
