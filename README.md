# GitHub Wiki SEE

*As seen at https://github-wiki-see.page !*

GitHub Wiki Search Engine Enablement is a tool to allow GitHub Wikis to be indexed by search engines.

This is a terribly and hastily built tool. However, it is usable and MVP!

This was made in response to https://github.com/isaacs/github/issues/1683.

## Usage

1. Get a link like https://github.com/nelsonjchen/github-wiki-test/wiki .

2. Make a link like https://github-wiki-see.page/mirror/nelsonjchen/github-wiki-test/wiki and post it somewhere like a blog, twitter, GitHub issue, or something. Somewhere where a search engine like Google can see it!

## Design

This is designed as a Rust web proxy application. It is not very Rusty and has lots of unwraps, panics, and hackiness. Cleanup, major overhauls, and straightening much appreciated!

It is designed to run semi-ondemand as a Docker application on a service such as [Google Cloud Run][gcr]. Uptime and latency is important so that it appears on search engines and gets a high ranking.

## Etc.

This is a Rust version of https://github.com/nelsonjchen/github-wiki-see-cfw. Unfortunately, Cloudflare Workers does not have sufficient API or resources to perform what was needed for MVP at free level.

[gcr]: https://cloud.google.com/run
