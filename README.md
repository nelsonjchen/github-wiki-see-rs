# GitHub Wiki SEE

*As previously seen at https://github-wiki-see.page by search engines!*

GitHub Wiki Search Engine Enablement was a service to allow GitHub Wikis to be indexed by search engines.

This was a terribly and hastily built service. However, it was usable and MVP!

This was made in response to https://github.com/github/feedback/discussions/4992 which eventually concluded on October 6, 2021 with the removal of the entry from `robots.txt`. If GitHub's response was correct about the timing, this was a reversal of a nearly decade long policy.

## Design

This was designed as a Rust web proxy application. It was not very Rusty and has lots of unwraps, panics, and hackiness. Cleanup, major overhauls, and straightening were much appreciated!

It was designed to run ondemand as a Docker application on a service such as [Google Cloud Run][gcr]. Uptime and latency were important so that the service appears on search engines and got a high ranking.

301/302s were intentionally **not** used as to not give search engines the impression that the page was a redirect and to
ignore the content. Humans should have seen it as a redirect; the robots should have not.

All links rendered in the tool going outside of GitHub were tagged with `rel="nofollow ugc"` to prevent ranking
 manipulation which was probably the reason wiki content was excluded from indexing.

## Decomissioning

GitHub had removed wikis from their `robots.txt` and this tool be modified to simply "308" permanently redirect the link to
GitHub.

https://github.com/github/feedback/discussions/4992#discussioncomment-1439485

[gcr]: https://cloud.google.com/run
