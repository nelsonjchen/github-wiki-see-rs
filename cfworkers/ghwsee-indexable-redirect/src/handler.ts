interface OriginalInfo {
  indexable: boolean
  last_modified?: Date
  moved_to?: string
}

interface HandleRequestOptions {
  originalInfoFetcher?: (url: URL) => Promise<OriginalInfo>
}

const WORKER_CACHE_CONTROL =
  'public, max-age=2592000, stale-while-revalidate=604800'

const LEGAL_BLOCK_REPOS = new Set(['mms75/sfz'])

function withWorkerCacheHeaders(response: Response): Response {
  const headers = new Headers(response.headers)
  headers.set('cache-control', WORKER_CACHE_CONTROL)

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  })
}

function repoSlugFromPathComponents(
  pathComponents: string[],
): string | undefined {
  if (pathComponents.length < 3) {
    return undefined
  }

  return `${pathComponents[1]}/${pathComponents[2]}`.toLowerCase()
}

function legalBlockResponse(repoSlug: string): Response {
  return new Response(
    `451 Unavailable For Legal Reasons - ${repoSlug} is unavailable on this service.`,
    {
      status: 451,
      statusText: 'Unavailable For Legal Reasons',
      headers: {
        'cache-control': WORKER_CACHE_CONTROL,
        'content-type': 'text/plain; charset=utf-8',
      },
    },
  )
}

class ModifiedDateAppender implements HTMLRewriterElementContentHandlers {
  date: Date

  constructor(date: Date) {
    this.date = date
  }

  element(element: Element) {
    element.append(`<p>📅 Last Modified: ${this.date.toUTCString()}</p>`, {
      html: true,
    })
  }
}

export async function handleRequest(
  request: Request,
  options: HandleRequestOptions = {},
): Promise<Response> {
  const githubUrl = new URL(
    request.url.replace('github-wiki-see.page/m', 'github.com'),
  )

  const pathComponents = githubUrl.pathname.split('/')
  const repoSlug = repoSlugFromPathComponents(pathComponents)
  if (repoSlug && LEGAL_BLOCK_REPOS.has(repoSlug)) {
    return legalBlockResponse(repoSlug)
  }

  const ghwseeResponse = fetch(request, {
    cf: {
      cacheEverything: true,
      cacheTtl: 7200,
    },
  })

  // Don't redirect wiki_index path. Index that, even for indexable wikis.
  if (pathComponents.length > 3 && pathComponents[2] === 'wiki_index') {
    const response = withWorkerCacheHeaders(await ghwseeResponse)
    // The index is synthesized and there is no last modified to claim.
    response.headers.delete('last-modified')
    return response
  }

  console.log(request.headers.get('user-agent'))

  let lastModifiedDate: Date | undefined = undefined

  try {
    const info = await (options.originalInfoFetcher ?? originalInfo)(githubUrl)
    if (info.moved_to) {
      console.log('Repo Moved Redirect: ' + githubUrl.href)

      const redirectUrl =
        'https://github-wiki-see.page/m' + new URL(info.moved_to).pathname

      return new Response('', {
        status: 308,
        headers: {
          'cache-control': WORKER_CACHE_CONTROL,
          location: redirectUrl,
        },
      })
    }
    if (info.indexable) {
      console.log('Indexable Redirect: ' + githubUrl.href)
      return new Response(null, {
        status: 308,
        statusText: 'Permanent Redirect',
        headers: {
          'cache-control': WORKER_CACHE_CONTROL,
          Location: githubUrl.toString(),
        },
      })
    }
    lastModifiedDate = info.last_modified
  } catch (e) {
    console.error(e)
  }

  console.log('No Redirect: ' + githubUrl.href)

  const response = await ghwseeResponse
  if (response.status === 308 && !request.url.endsWith('/wiki/Home')) {
    console.warn('Redirected Unindexable: ' + response.headers.get('Location'))
  }

  let maybeDatedResponse = withWorkerCacheHeaders(response)

  if (lastModifiedDate) {
    maybeDatedResponse.headers.set(
      'last-modified',
      lastModifiedDate.toUTCString(),
    )
    maybeDatedResponse = new HTMLRewriter()
      .on('#header_info', new ModifiedDateAppender(lastModifiedDate))
      .transform(maybeDatedResponse)
  } else {
    // Don't claim a last modified date if it wasn't found on the original page.
    maybeDatedResponse.headers.delete('last-modified')
  }

  return maybeDatedResponse
}

export async function originalInfo(url: URL): Promise<OriginalInfo> {
  const response = await fetch(url.toString(), {
    redirect: 'follow',
    cf: {
      cacheEverything: true,
      cacheTtl: 86400,
    },
  })

  if (response.status != 200 || response.headers.has('x-robots-tag')) {
    // Check if Moved Repo
    if (response.redirected) {
      // If the account and repository parts of the url are different from the original url, then it's a moved repo.
      const originalAccountRepo = /\.com(\/.*\/.*\/)/.exec(url.toString())
      const redirectedAccountRepo = /\.com(\/.*\/.*\/)/.exec(response.url)
      if (
        originalAccountRepo &&
        redirectedAccountRepo &&
        originalAccountRepo[1] !== redirectedAccountRepo[1]
      ) {
        return {
          indexable: false,
          moved_to: response.url,
        }
      }
    }

    // Scan the response body for a date of last change
    const body = await response.text()

    // <relative-time datetime="2022-04-24T17:07:11Z"
    const dateRegex = /<relative-time datetime="([^"]+)"/g
    const match = dateRegex.exec(body)

    if (match) {
      const dateString = match[1]
      const date = new Date(dateString)
      return {
        indexable: false,
        last_modified: date,
      }
    }

    return {
      indexable: false,
    }
  }

  return {
    indexable: true,
  }
}
