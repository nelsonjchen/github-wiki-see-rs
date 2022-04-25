interface OriginalInfo {
  indexable: boolean
  last_modified?: Date
}

export async function handleRequest(request: Request): Promise<Response> {
  const githubUrl = new URL(
    request.url.replace('github-wiki-see.page/m', 'github.com'),
  )

  const ghwseeResponse = fetch(request, {
    cf: {
      cacheEverything: true,
      cacheTtl: 7200,
    },
  })

  const pathComponents = githubUrl.pathname.split('/')

  // Don't redirect wiki_index path. Index that, even for indexable wikis.
  if (pathComponents.length > 3 && pathComponents[2] === 'wiki_index') {
    return await ghwseeResponse
  }

  console.log(request.headers.get('user-agent'))

  let lastModifiedDate: Date | undefined = undefined

  try {
    const info = await originalInfo(githubUrl)
    if (info.indexable) {
      console.log('Indexable Redirect: ' + githubUrl.href)
      return new Response(null, {
        status: 308,
        statusText: 'Permanent Redirect',
        headers: {
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

  const newResponse = new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: response.headers,
  })

  if (lastModifiedDate) {
    newResponse.headers.set('last-modified', lastModifiedDate.toUTCString())
  } else {
    // Don't claim a last modified date if it wasn't found on the original page.
    newResponse.headers.delete('last-modified')
  }

  return newResponse
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
