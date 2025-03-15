import React, {
  createContext,
  useEffect,
  useState,
  type ReactNode,
} from "react"
import useStore from "@/hooks/useStore"
import { Alert } from "./ui/alert"
import type { StoreAdapter } from "@/lib/StoreAdapter"

interface StoreProviderProps {
  children: ReactNode
}

const StoreContext = createContext<
  | {
      store: StoreAdapter
      loading: boolean
      error: Error | null
    }
  | undefined
>(undefined)

export const StoreProvider: React.FC<StoreProviderProps> = ({ children }) => {
  const { store, loading, error } = useStore()
  const [, forceUpdate] = useState(0)

  // We don't need to manually register for updates as it's handled in the useStore hook now
  // The hook already registers a listener for 'update' events and handles refreshing

  if (loading) {
    return (
      <div className="flex items-center justify-center w-screen h-screen gap-2 bg-zinc-800">
        <Alert className="w-64">Loading Organ...</Alert>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center w-screen h-screen gap-2 bg-zinc-800">
        <Alert variant="destructive" className="w-64">
          Error initializing Organ: {error?.message}
        </Alert>
      </div>
    )
  }

  return (
    <StoreContext.Provider value={{ store, loading, error }}>
      {children}
    </StoreContext.Provider>
  )
}

export const useStoreContext = () => {
  const context = React.useContext(StoreContext)
  if (!context) {
    throw new Error("useStoreContext must be used within a StoreProvider")
  }
  return context
}
