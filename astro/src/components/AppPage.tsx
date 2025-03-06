import { Suspense, useEffect, useState } from "react"
import { AppSidebar } from "./AppSidebar"
import { useStoreContext } from "./StoreContext"
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./ui/resizable"
import { FileContainer } from "./FileContainer"
import { Preview } from "./Preview"
import { SelectedFileDisplay } from "./SelectedFileDisplay"

export function AppPage() {
  const [isVertical, setIsVertical] = useState(false)
  const { store } = useStoreContext()

  useEffect(() => {
    const handleResize = () => {
      // Adjust the threshold width to your preference
      setIsVertical(window.innerWidth <= 768)
    }

    // Set the initial layout direction
    handleResize()

    window.addEventListener("resize", handleResize)
    return () => {
      window.removeEventListener("resize", handleResize)
    }
  }, [])

  return (
    <>
      <AppSidebar />
      {store.activeProject?.activeFile ? (
        <ResizablePanelGroup direction={isVertical ? "vertical" : "horizontal"}>
          <ResizablePanel>
            <div className="flex-grow h-full">
              <Suspense fallback={<div>Loading...</div>}>
                <SelectedFileDisplay
                  onClose={() => {
                    store.activeProject!.activeFile = null
                  }}
                />
              </Suspense>
            </div>
          </ResizablePanel>
          <ResizableHandle className="bg-zinc-700" />
          <ResizablePanel maxSize={70}>
            <Preview />
          </ResizablePanel>
        </ResizablePanelGroup>
      ) : (
        <Preview />
      )}
    </>
  )
}
