import { handleRequest, originalInfo } from '../src/handler'

describe('handle', () => {
  test('can determine if a URL is indexable', async () => {
    const url = new URL('https://github.com/PixarAnimationStudios/USD/wiki')
    expect((await originalInfo(url)).indexable).toBeTruthy()
  })

  test('redirects an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/PixarAnimationStudios/USD/wiki`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(308)
    expect(result.headers.get('location')).toEqual(
      'https://github.com/PixarAnimationStudios/USD/wiki',
    )
    expect(result.headers.has('Last-Modified')).toBeFalsy()
  })

  test('can determine if a URL is not indexable', async () => {
    const url = new URL('https://github.com/commaai/openpilot/wiki')
    expect(await (await originalInfo(url)).indexable).toBeFalsy()
  })

  test('does not redirect an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/commaai/openpilot/wiki`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
    expect(result.headers.has('Last-Modified')).toBeTruthy()
  })

  test('does not try to redirect wiki_index on an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/PixarAnimationStudios/USD/wiki_index`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
    // The index is synthesized and there is no last modified to claim.
    expect(result.headers.has('Last-Modified')).toBeFalsy()
  })

  test('does not try to redirect wiki_index on an unindexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/commaai/openpilot/wiki_index`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
    // The index is synthesized and there is no last modified to claim.
    expect(result.headers.has('Last-Modified')).toBeFalsy()

    const bodyText = await result.text()

    console.debug(bodyText)
    expect(bodyText.includes("Modified Date")).toBeFalsy()
  })

  test('extracts a date from a non-indexable original page', async () => {
    const url = new URL(
      'https://github.com/nelsonjchen/wiki-public-test/wiki/last-modified-test',
    )
    const info = await originalInfo(url)
    expect(info.indexable).toBeFalsy()
    expect(info.last_modified).toBeInstanceOf(Date)
    if (info.last_modified) {
      expect(info.last_modified).toStrictEqual(
        new Date('2022-04-24T17:07:11.000Z'),
      )
    }
  })

  test('synthesizes a correct last modified', async () => {
    // This page is never edited so this date won't change.
    const request_url =
      'https://github-wiki-see.page/m/nelsonjchen/wiki-public-test/wiki/last-modified-test'
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
    expect(result.headers.get('Last-Modified')).toEqual(
      'Sun, 24 Apr 2022 17:07:11 GMT',
    )

    const bodyText = await result.text()

    expect(bodyText.includes("Last Modified")).toBeTruthy()
  })
})
