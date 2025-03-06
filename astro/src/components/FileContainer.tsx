import { SelectedFileDisplay } from "./SelectedFileDisplay"
import { useStoreContext } from "@/components/StoreContext"
// import type { Field } from "./SelectedFileDisplay"

// export interface Schema {
//   fields: Field[]
// }

export function FileContainer({ onClose }: { onClose: () => void }) {
  const { store } = useStoreContext()

  console.log("FileContainer: active file", store.activeProject?.activeFile)

  if (!store.activeProject?.activeFile) return null

  return <SelectedFileDisplay onClose={onClose} />
}
