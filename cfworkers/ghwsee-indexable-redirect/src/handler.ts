export async function handleRequest(request: Request): Promise<Response> {
  const githubUrl = new URL(request.url.replace("github-wiki-see.page/m", "github.com"))
  const isIndexable = indexable(githubUrl)
  const ghwseeResponse = await fetch(request)
  if (await isIndexable) {
    console.log("Redirecting to: " + githubUrl.href)
    return new Response(null, {
      status: 308,
      statusText: "Permanent Redirect",
      headers: {
        "Location": githubUrl.toString(),
      }
    })
  }
  return ghwseeResponse
}

export async function indexable(url: URL): Promise<boolean> {
  const response = await fetch(url.toString(), {
    redirect: 'manual',
  })
  if (response.status != 200) {
    return false
  }
  if (response.headers.has('x-robots-tag')) {
    return false
  }
  return true
}
