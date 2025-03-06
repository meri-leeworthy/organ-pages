import React, {
  createContext,
  useEffect,
  useState,
  type ReactNode,
} from "react"
import useStore from "@/hooks/useStore"
import { Alert } from "./ui/alert"
import type { Store } from "@/lib/Store"

interface StoreProviderProps {
  children: ReactNode
}

const StoreContext = createContext<
  | {
      store: Store
      loading: boolean
      error: Error | null
    }
  | undefined
>(undefined)

export const StoreProvider: React.FC<StoreProviderProps> = ({ children }) => {
  const { store, loading, error } = useStore()
  const [, forceUpdate] = useState(0)

  useEffect(() => {
    store.on("update", () => {
      forceUpdate(prev => prev + 1)
    })
  }, [store])

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
