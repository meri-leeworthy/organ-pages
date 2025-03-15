import { FileList } from "./FileList"
import { Sidebar, SidebarContent, SidebarTrigger } from "./ui/sidebar"
import { Button } from "./ui/button"
import { NavUser } from "./NavUser"
import { useStore } from "./StoreProvider"
import { useCollections } from "@/hooks/useSWRStore"
import { Spinner } from "./ui/spinner" // You may need to create this component

export function AppSidebar() {
  const { state, setActiveProjectType } = useStore()
  const { collections, isLoading, error } = useCollections()

  // Determine if the site is active
  const siteIsActive = state.activeProjectType === "site"

  // Toggle between site and theme contexts
  const toggleContext = () => {
    setActiveProjectType(siteIsActive ? "theme" : "site")
  }

  return (
    <>
      <Sidebar className="z-10">
        <SidebarContent className="relative z-10 p-1 bg-zinc-800 text-zinc-100">
          <SidebarTrigger className="ml-1 rounded-xl" />

          {isLoading ? (
            <div className="flex justify-center my-4">
              <Spinner />
            </div>
          ) : error ? (
            <div className="p-2 text-red-400">Error loading collections</div>
          ) : collections.length === 0 ? (
            <div className="p-2 text-zinc-400">No collections found</div>
          ) : (
            collections.map(collection => (
              <FileList key={collection.name} type={collection.name} />
            ))
          )}

          <Button
            className="mt-auto rounded-xl text-zinc-100 bg-zinc-700 hover:bg-zinc-600"
            variant="secondary"
            onClick={toggleContext}>
            Edit {siteIsActive ? "Theme" : "Site"}
          </Button>
          <NavUser />
        </SidebarContent>
      </Sidebar>
    </>
  )
}
