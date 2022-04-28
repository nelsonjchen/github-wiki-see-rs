# GitHub Wiki SEE

![GitHub](https://img.shields.io/github/license/nelsonjchen/github-wiki-see-rs) ![GitHub](https://img.shields.io/github/stars/nelsonjchen/github-wiki-see-rs)

_As seen at https://github-wiki-see.page by search engines and archivers!_

GitHub Wiki Search Engine Enablement is a service to allow GitHub Wikis to be indexed by search engines.

This is a terribly and hastily built service. However, it is usable and MVP!

This was made in response to https://github.com/github/feedback/discussions/4992.

## Design

This is designed as a Rust web proxy application. It is not very Rusty and has lots of hackiness. Cleanup, major overhauls, and straightening are much appreciated!

It is designed to run as a simple Docker application on a service such as [fly.io][flyio] and/or [Google Cloud Run][gcr]. Uptime and latency are important so that the service appears on search engines and gets a high ranking.

301/302/307/308s are intentionally **not** used as to not give search engines the impression that the page is a redirect and to
ignore the content.
Humans should see the "content" as a redirect; the robots should not.

All links rendered in the tool going outside of GitHub are tagged with `rel="nofollow ugc"` to prevent ranking
manipulation which is probably one the reason wiki content was excluded from indexing.

A Cloudflare Worker is placed in front to additionally protect against the service accidentally mirroring indexable content
on GitHub. The worker also enriches a "last modified" header date on the proxied content if possible from the original content if the original content isn't indexable to better hint to search engines the freshness of content and better utilize their crawler budget.

## Decommissioning

Please see:

https://github-wiki-see.page/#decommissioning

But basically if GitHub lets it be indexed, this service will 308 redirect it.

[gcr]: https://cloud.google.com/run
[flyio]: https://fly.io
