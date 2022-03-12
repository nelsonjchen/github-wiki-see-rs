export async function handleRequest(request: Request): Promise<Response> {
  const githubUrl = new URL(request.url.replace("github-wiki-see.page/m", "github.com"))

  const ghwseeResponse = fetch(request)

  const pathComponents = githubUrl.pathname.split("/")
  if (pathComponents.length > 3 && pathComponents[2] === "wiki_index") {
    return await ghwseeResponse
  }

  try {
    if (await indexable(githubUrl)) {
      console.log("Indexable Redirect: " + githubUrl.href)
      return new Response(null, {
        status: 308,
        statusText: "Permanent Redirect",
        headers: {
          "Location": githubUrl.toString(),
        }
      })
    }
  } catch (e) {
    console.error(e)
  }

  console.log("No Redirect: " + githubUrl.href)

  const response = await ghwseeResponse
  if (response.status === 308) {
    console.warn("Redirected Unindexable: " + response.headers.get("Location"))
  }

  return await ghwseeResponse
}

export async function indexable(url: URL): Promise<boolean> {
  const response = await fetch(url.toString(), {
    redirect: 'follow',
  })
  if (response.status != 200) {
    return false
  }
  if (response.headers.has('x-robots-tag')) {
    return false
  }
  return true
}
