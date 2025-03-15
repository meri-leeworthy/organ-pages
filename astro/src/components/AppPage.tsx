import { Suspense, useEffect, useState } from "react"
import { AppSidebar } from "./AppSidebar"
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./ui/resizable"
// import { FileContainer } from "./FileContainer"
// import { Preview } from "./Preview"
import { SelectedFileDisplay } from "./SelectedFileDisplay"
import { useActiveFile } from "@/hooks/useSWRStore"

export function AppPage() {
  const [isVertical, setIsVertical] = useState(false)
  const { setActiveFile, activeFileId } = useActiveFile()

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
      {activeFileId ? (
        <ResizablePanelGroup direction={isVertical ? "vertical" : "horizontal"}>
          <ResizablePanel>
            <div className="flex-grow h-full">
              <Suspense fallback={<div>Loading...</div>}>
                <SelectedFileDisplay
                  onClose={() => {
                    setActiveFile(undefined)
                  }}
                />
              </Suspense>
            </div>
          </ResizablePanel>
          <ResizableHandle className="bg-zinc-700" />
          {/* <ResizablePanel maxSize={70}>
            <Preview />
          </ResizablePanel> */}
        </ResizablePanelGroup>
      ) : (
        // <Preview />
        <div>No file selected</div>
      )}
    </>
  )
}
