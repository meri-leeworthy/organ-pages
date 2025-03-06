import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { loadAssetFromIndexedDB } from "./idbHelper"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// this is faulty and should be removed - fetch always throws Error for blob urls due to cors
export async function isBlobUrlValid(blobUrl: string): Promise<boolean> {
  try {
    const response = await fetch(blobUrl)
    return response.ok
  } catch (error) {
    console.error("Blob URL not valid: ", blobUrl, error)
    return false
  }
}

export const getValidBlobUrl = async (id: number, url: string) => {
  console.log("getValidBlobUrl", id, url)
  const isValid = await isBlobUrlValid(url)
  if (!isValid) {
    const asset = await loadAssetFromIndexedDB(id)
    const blob = new Blob([asset], { type: "image/jpeg" })
    const url = URL.createObjectURL(blob)
    // const query = `
    //   UPDATE files
    //   SET content = ?
    //   WHERE name = ?
    //   AND model_id = 6;
    //   `
    // execute(query, [url, id])
    return url
  }
  return url
}

export const randomId = () =>
  Math.floor(Math.random() * 0x10000)
    .toString(16)
    .padStart(4, "0")
