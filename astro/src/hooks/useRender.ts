import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  ReactNode,
} from "react";
import { useBlobStore, type BlobStore } from "@/components/useBlobStore.jsx";
import { type FileData } from "@/lib/types";
import { updateImageUrls } from "@/lib/updateImageUrls.js";
import { useStoreContext } from "@/components/StoreContext.jsx";

export type Context = Record<
  string,
  {
    name: string;
    file_type: string;
    data: string;
    url: string;
  }
>;

interface RenderContextType {
  loading: boolean;
  error: string;
  render: (
    contentFileId: number,
    files: Map<number, FileData<Record<string, any>>>,
    local?: boolean
  ) => string;
  renderLocal: (
    contentFileId: number,
    files: Map<number, FileData<Record<string, any>>>
  ) => string;
  getLiveUrl: (fileId: number) => string;
}

const RenderContext = createContext<RenderContextType | undefined>(undefined);

export function RenderProvider({ children }: { children: ReactNode }) {
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");
  const blobStore = useBlobStore();
  const { store, loading: storeLoading, error: storeError } = useStoreContext();

  const [wasmModule, setWasmModule] = useState<{
    render: (current_file_id: number, context: Context) => string;
  } | null>(null);

  useEffect(() => {
    const loadWasm = async () => {
      try {
        setLoading(true);
        console.log("Loading WASM module...");
        const module = await import("../wasm/minissg/minissg.js");
        setWasmModule(module);
        console.log("WASM module loaded:", module);
        setLoading(false);
      } catch (e) {
        console.error("Failed to load WASM module:", e);
        setError("Failed to load WASM module.");
        setLoading(false);
      }
    };
    loadWasm();
  }, []);

  const render = useCallback(
    (
      contentFileId: number,
      files: Map<number, FileData<Record<string, any>>>,
      local?: boolean
    ) => {
      if (!wasmModule) {
        throw new Error("WASM module not loaded");
      }

      const context: Context = {};
      files.forEach((file) => {
        context[file.id] = {
          name: file.name,
          file_type: file.type,
          data: JSON.stringify(
            local
              ? updateImageUrls(file.data, blobStore.getBlobURL)
              : updateImageUrls(file.data, getLiveUrl)
          ),
          url: file.url || "",
        };
      });

      try {
        const result = wasmModule.render(contentFileId, context);
        setError("");
        return result;
      } catch (e) {
        console.error("Failed to render:", e);
        setError("Failed to render: " + e);
        return "";
      }
    },
    [wasmModule, storeLoading, storeError]
  );

  const renderLocal = useCallback(
    (
      contentFileId: number,
      files: Map<number, FileData<Record<string, any>>>
    ) => {
      const filesWithLocalUrls = injectLocalUrls(files, blobStore);
      return render(contentFileId, filesWithLocalUrls, true);
    },
    [render]
  );

  const getLiveUrl = useCallback(
    (fileId: number) => {
      console.log("getLiveUrl: ", fileId);
      if (storeLoading || storeError) throw new Error("Failed to get live url");
      const query = `SELECT url FROM file WHERE id = ?;`;
      // const result = execute(query, [fileId])
      // return result[0].url as string
    },
    [store, storeLoading, storeError]
  );

  return (
    <RenderContext.Provider value={{ loading, error, render, renderLocal, getLiveUrl }}>
      {children}
    </RenderContext.Provider>
  );
}

export function useRender() {
  const context = useContext(RenderContext);
  if (!context) {
    throw new Error("useRender must be used within a RenderProvider");
  }
  return context;
}

function injectLocalUrls(
  filesMap: Map<number, FileData<Record<string, any>>>,
  blobStore: BlobStore
) {
  const files = [...filesMap.values()];

  const assets = files.filter((file) => file.type === "asset");
  const updatedAssetsArray = assets.map((asset) => ({
    ...asset,
    url: blobStore.getBlobURL(asset.id),
  }));

  updatedAssetsArray.forEach((asset) => {
    filesMap.set(asset.id, asset);
  });

  const templateAssets = files.filter((file) => file.type === "templateAsset");
  const updatedTemplateAssetsArray = templateAssets.map((asset) => {
    if (!asset.data) return asset;
    const blob = new Blob([asset.data.body.content], { type: "text/css" });
    const url = blobStore.addBlob(asset.id, blob);
    return { ...asset, url };
  });

  updatedTemplateAssetsArray.forEach((templateAsset) => {
    filesMap.set(templateAsset.id, templateAsset);
  });

  return filesMap;
}
