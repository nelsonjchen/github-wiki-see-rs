export async function handleRequest(request: Request): Promise<Response> {
  // Check if the original GitHub URL is indexable. If it is, redirect.
  const url = new URL(request.url)
  const path = url.pathname

  return await fetch(request)
}

export function indexable(url: URL): boolean {

  return url.hostname.endsWith('3vngqvvpoq-uc.a.run.app')
}
