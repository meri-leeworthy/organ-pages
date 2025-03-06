export function updateImageUrls(
  html: string,
  replacer: (id: number) => string
) {
  // the original images are identified in the html title attribute
  // const originalImages = htmlContent?.match(/<img[^>]*title="(\d+)"[^>]*>/g)

  // const idBlobMap = new Map(
  //   originalImages?.map((img: string) => {
  //     const id = img.match(/title="(\d+)"/)?.[1]
  //     const url = img.match(/src="([^"]+)"/)?.[1]
  //     return [id, url]
  //   })
  // )

  const newHtml = html?.replace(
    /<img[^>]*title="(\d+)"[^>]*>/g,
    (match: string, title: string) => {
      // console.log("Match:", match)
      // console.log("title:", title)
      const refreshedUrl = replacer(parseInt(title))
      return `<img src="${refreshedUrl}" title="${title}" />`
    }
  )

  return newHtml
}
