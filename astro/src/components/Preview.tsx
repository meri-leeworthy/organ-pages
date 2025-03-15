import { useEffect, useRef, useState } from "react"
import { useStore } from "./StoreProvider.jsx"
import { Alert, AlertDescription, AlertTitle } from "./ui/alert.jsx"
import { useRender } from "@/hooks/useRender.js"

export const Preview = () => {
  const iframeRef = useRef<HTMLIFrameElement | null>(null)
  const [iframeLoaded, setIframeLoaded] = useState<boolean>(false)

  const { state, setActiveContext } = useStore()
  const { loading: wasmLoading, error: wasmError, renderLocal } = useRender()

  function getBodyContent(htmlString: string) {
    // Create a temporary DOM container
    const temp = document.createElement("html")
    temp.innerHTML = htmlString

    const linkTags = Array.from(temp.querySelectorAll("link"))
      .map(link => link.outerHTML)
      .join("")
    console.log("linkTags", linkTags)
    // Return only what's inside <body>, ignoring the root <html> and <head>
    return linkTags + temp.querySelector("body")?.innerHTML || htmlString
  }

  useEffect(() => {
    if (!state || wasmLoading || !iframeLoaded || !state.activeContext) return

    const handleRender = () => {
      try {
        const query =
          "SELECT file.id, file.name, file.data, file.url, model.name as type FROM file JOIN model ON file.model_id = model.id;"
        // const result = execute(query)

        // const files = result.map((file: ParamsObject): [number, FileData] => [
        //   file.id as number,
        //   {
        //     id: file.id as number,
        //     name: file.name?.toString() || "",
        //     type: file.type?.toString() as FileData["type"],
        //     data: JSON.parse(file.data?.toString() || "{}"),
        //     url: file.url?.toString() || "",
        //   },
        // ])

        // const filesMap = new Map(files)

        // const combinedContent = renderLocal(
        //   selectedFiles.contentFileId,
        //   filesMap
        // )

        // console.log("full content", combinedContent)

        // const bodyContent = getBodyContent(combinedContent)
        // console.log("bodyContent", bodyContent)

        // if (iframeRef.current?.contentWindow) {
        //   iframeRef.current.contentWindow.postMessage(
        //     { type: "update", html: bodyContent },
        //     "*"
        //   )
        // }
      } catch (e) {
        console.error("Error during conversion:", e)
      }
    }
    const delayedHandleRender = () => setTimeout(handleRender, 50)

    window.addEventListener("keydown", delayedHandleRender)
    window.addEventListener("click", delayedHandleRender)

    delayedHandleRender()

    // Clean up the event listener on component unmount
    return () => {
      window.removeEventListener("keydown", delayedHandleRender)
      window.removeEventListener("click", delayedHandleRender)
    }
  }, [store.activeProject?.activeFile, wasmLoading, storeLoading, iframeLoaded])

  // const onLinkClick = (href: string) => {
  //   const newFileId = [...files.values()].find(
  //     file => file.name === href.slice(1)
  //   )?.id
  //   if (!newFileId) {
  //     console.error("File not found:", href)
  //     return
  //   }
  //   setSelectedFiles({
  //     activeFileId: newFileId,
  //     contentFileId: newFileId,
  //   })
  // }

  // // handle link clicks inside the iframe
  // const handleIframeClick = (event: MouseEvent) => {
  //   const target = event.target as HTMLElement

  //   if (target.tagName.toLowerCase() === "a") {
  //     const href = target.getAttribute("href")
  //     if (href) {
  //       const isInternal = href.startsWith("/")
  //       if (isInternal) {
  //         event.preventDefault()
  //         onLinkClick(href)
  //       } else {
  //         // Optionally handle external links, e.g., open in new tab
  //         // window.open(href, '_blank')
  //       }
  //     }
  //   }
  // }
  // // useEffect(() => {
  // //   const iframe = iframeRef.current
  // //   if (iframe && iframe.contentDocument) {
  // //     iframe.contentDocument.removeEventListener("click", handleIframeClick)
  // //   }
  // // }, [previewContent])

  const handleIframeLoad = () => {
    setIframeLoaded(true)
    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    iframe.contentWindow.postMessage({ type: "initialize" }, "*")
  }

  return (
    <>
      {wasmError ? (
        <div className="flex items-center justify-center h-screen">
          <Alert variant="destructive">
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{wasmError}</AlertDescription>
          </Alert>
        </div>
      ) : (
        <iframe
          ref={iframeRef}
          className="items-center w-full h-screen overflow-y-scroll border-l-1"
          id="preview-pane"
          onLoad={handleIframeLoad}
          srcDoc={`<!DOCTYPE html>
            <html>
            <head>
              <style>
                body { margin: 0; padding: 1rem; font-family: sans-serif; }
                img { max-width: 100%; height: auto; display: block; }
              </style>
              <script>
                document.addEventListener("DOMContentLoaded", () => {
                  console.log("dom content loaded")
                  window.morphdomReady = false;
                  function loadScript(url, callback) {
                    const script = document.createElement("script");
                    script.src = url;
                    script.onload = callback;
                    document.head.appendChild(script);
                  }
                  
                  loadScript("/morphdom-umd.min.js", () => {
                    window.morphdomReady = true;
                  });
                });
                
                window.addEventListener("message", (event) => {
                  console.log("message received", event)
                  try {
                    if (event.data.type === "update") {
                      if (document.readyState !== "complete") {
                        console.warn("Skipping update, document not fully loaded yet.");
                      return;
                    }
                    if (!document.body) {
                      console.log("document:", document)
                      console.error("Morphdom update skipped: document.body is null");
                      return;
                    }
                    if (!event.data.html) {
                      console.error("Morphdom update skipped: event.data.html is null");
                      return;
                    }
                    if (!window.morphdomReady) {
                      console.warn("Morphdom not ready yet, update skipped");
                      return;
                    }
                    console.log("morphdom")
                    morphdom(document.body, event.data.html, {
                      childrenOnly: true
                    });
                    } else if (event.data.type === "initialize") {
                      window.morphdomReady = true;
                      console.log("morphdom ready")
                    }
                  } catch (e) {
                    console.error("Error during message", e)
                  }
                })
              </script>
            </head>
            <body>
              <p>Loading preview...</p>
            </body>
            </html>`}
        />
      )}
    </>
  )
}
