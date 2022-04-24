import { handleRequest, originalInfo } from '../src/handler'

describe('handle', () => {
  test('can determine if a URL is indexable', async () => {
    const url = new URL('https://github.com/PixarAnimationStudios/USD/wiki')
    expect(((await originalInfo(url)).indexable)).toBeTruthy()
  })

  test('redirects an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/PixarAnimationStudios/USD/wiki`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(308)
    expect(result.headers.get('location')).toEqual(
      'https://github.com/PixarAnimationStudios/USD/wiki'
    )
  })

  test('can determine if a URL is not indexable', async () => {
    const url = new URL('https://github.com/commaai/openpilot/wiki')
    expect(await (await originalInfo(url)).indexable).toBeFalsy()
  })

  test('does not redirects an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/commaai/openpilot/wiki`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
  })

  test('does not try to redirect wiki_index on an indexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/PixarAnimationStudios/USD/wiki_index`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
  })

  test('does not try to redirect wiki_index on an unindexable wiki', async () => {
    const request_url = `https://github-wiki-see.page/m/commaai/openpilot/wiki_index`
    console.debug(request_url)
    const result = await handleRequest(
      new Request(request_url, { method: 'GET' }),
    )

    expect(result.status).toEqual(200)
  })
})
