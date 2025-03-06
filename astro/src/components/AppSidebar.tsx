import { FileList } from "./FileList"
import { Sidebar, SidebarContent, SidebarTrigger } from "./ui/sidebar"
import { Button } from "./ui/button"
import { NavUser } from "./NavUser"
import { useStoreContext } from "./StoreContext"

export function AppSidebar() {
  const { store } = useStoreContext()
  const collections = store.activeProject?.getCollections()
  const siteIsActive = store.activeProject === store.activeSite

  return (
    <>
      <Sidebar className="z-10">
        <SidebarContent className="relative z-10 p-1 bg-zinc-800 text-zinc-100">
          <SidebarTrigger className="ml-1 rounded-xl" />

          {collections?.map(collection => (
            <FileList key={collection.name} type={collection.name} />
          ))}

          <Button
            className="mt-auto rounded-xl text-zinc-100 bg-zinc-700 hover:bg-zinc-600"
            variant="secondary"
            onClick={() =>
              siteIsActive
                ? (store.activeProject = store.activeTheme)
                : (store.activeProject = store.activeSite)
            }>
            Edit {siteIsActive ? "Theme" : "Site"}
          </Button>
          <NavUser />
        </SidebarContent>
      </Sidebar>
    </>
  )
}
