import { StoreProvider } from "./StoreContext.jsx"
import React from "react"
import { SidebarProvider } from "./ui/sidebar.jsx"
import { Toaster } from "@/components/ui/sonner.tsx"
import { ClientProvider } from "./ClientContext.jsx"
import { BlobStoreProvider } from "./useBlobStore.jsx"
import { AppPage } from "./AppPage.jsx"

const App: React.FC = () => {
  return (
    <>
      <StoreProvider>
        <BlobStoreProvider>
          <ClientProvider>
            <SidebarProvider>
              <AppPage />
            </SidebarProvider>
          </ClientProvider>
        </BlobStoreProvider>
      </StoreProvider>
      <Toaster />
    </>
  )
}

export default App
