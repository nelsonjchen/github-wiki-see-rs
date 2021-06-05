# GitHub Wiki SEE

*As seen at https://github-wiki-see.page by search engines!*

GitHub Wiki Search Engine Enablement is a service to allow GitHub Wikis to be indexed by search engines.

This is a terribly and hastily built service. However, it is usable and MVP!

This was made in response to https://github.com/isaacs/github/issues/1683.

## Usage

1. Get a link like https://github.com/nelsonjchen/github-wiki-test/wiki .

2. Make a link like https://github-wiki-see.page/m/nelsonjchen/github-wiki-test/wiki and post it somewhere like a blog, Twitter, GitHub issue, bottom of your README or something. Somewhere where a search engine like Google can see it and crawl it!
  * You can also add it to the sitemap at: https://github.com/nelsonjchen/github-wiki-see-rs-sitemaps .

## Design

This is designed as a Rust web proxy application. It is not very Rusty and has lots of unwraps, panics, and hackiness. Cleanup, major overhauls, and straightening are much appreciated!

It is designed to run semi-ondemand as a Docker application on a service such as [Google Cloud Run][gcr]. Uptime and latency are important so that the service appears on search engines and gets a high ranking.

[gcr]: https://cloud.google.com/run
