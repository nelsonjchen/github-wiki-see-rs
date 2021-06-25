# GitHub Wiki SEE

*As seen at https://github-wiki-see.page by search engines!*

GitHub Wiki Search Engine Enablement is a service to allow GitHub Wikis to be indexed by search engines.

This is a terribly and hastily built service. However, it is usable and MVP!

This was made in response to https://github.com/isaacs/github/issues/1683.

## Design

This is designed as a Rust web proxy application. It is not very Rusty and has lots of unwraps, panics, and hackiness. Cleanup, major overhauls, and straightening are much appreciated!

It is designed to run ondemand as a Docker application on a service such as [Google Cloud Run][gcr]. Uptime and latency are important so that the service appears on search engines and gets a high ranking.

301/302s are intentially **not** used as to not give search engines the impression that the page is a redirect and to
ignore the content. Humans should see it as a redirect; the robots should not.

All links rendered in the tool going outside of GitHub are tagged with `rel="nofollow ugc"` to prevent ranking
 manipulation which was probably the reason wiki content was excluded from indexing.

## Decomissioning

When GitHub removes wikis from their `robots.txt`, this tool will be modified to simply "301" redirect the link to
GitHub. May this happen someday and soon.

If some other unforeseen consequence of the tool happens, this may be done as well.

[gcr]: https://cloud.google.com/run
