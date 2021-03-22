# GitHub Wiki SEE

GitHub Wiki Search Engine Enablement is a tool to allow GitHub Wikis to be indexed by search engines.

This is a terribly and hastily built tool. However, it is usable and MVP!

This was made in response to https://github.com/isaacs/github/issues/1683.

## Design

This is designed as a Rust web proxy application. It is not very Rusty and has lots of unwraps, panics, and hackiness.

It is designed to run semi-ondemand as a Docker application on a service such as [Google Cloud Run][gcr].

## Etc.

This is a Rust version of https://github.com/nelsonjchen/github-wiki-see-cfw. Unfortunately, Cloudflare Workers does not have sufficient API or resources to perform what was needed for MVP.

[gcr]: https://cloud.google.com/run