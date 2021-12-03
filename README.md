# GitHub Wiki SEE

![GitHub](https://img.shields.io/github/license/nelsonjchen/github-wiki-see-rs) ![GitHub](https://img.shields.io/github/stars/nelsonjchen/github-wiki-see-rs)


*As seen at https://github-wiki-see.page by search engines and archivers!*

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

## Decommissioning

When GitHub removes wikis from their `robots.txt` and `x-robots-tag: none` from the pages on GitHub
or figures out some other way to serve the content in harmony with what they desire for SEO,
this tool will be modified to simply "308" redirect the link to GitHub. May this happen someday and soon.

If some other unforeseen consequence of the tool happens, this may be done as well.

[gcr]: https://cloud.google.com/run
[flyio]: https://fly.io

