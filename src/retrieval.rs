use std::future::Future;

use reqwest::{Client, StatusCode};
use rocket::futures::{future::select_ok, FutureExt, TryFutureExt};
use scraper::{Html, Selector};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Content {
    AsciiDoc(String),
    Creole(String),
    Markdown(String),
    Mediawiki(String),
    Orgmode(String),
    Pod(String),
    Rdoc(String),
    Textile(String),
    ReStructuredText(String),
    FallbackHtml(String),
}

#[derive(Debug)]
pub enum ContentError {
    NotFound,
    TooMayRequests,
    OtherError(String),
}

pub async fn retrieve_source_file<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
) -> Result<Content, ContentError> {
    // Pull extensions from
    // https://github.com/gollum/gollum-lib/blob/b074c6314dc47571cae91dd333bd1b1f2a816c48/lib/gollum-lib/markups.rb#L70

    // Try markdown first

    retrieve_source_file_extension(account, repository, page, client, &Content::Markdown, "md")
        .or_else(|_| async {
            select_ok([
                // AsciiDoc
                retrieve_source_file_extension(
                    account,
                    repository,
                    page,
                    client,
                    &Content::AsciiDoc,
                    "asciidoc",
                )
                .boxed(),
                // Creole
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Creole,
                //     "creole",
                // )
                // .boxed(),
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Markdown,
                //     "mkd",
                // )
                // .boxed(),
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Markdown,
                //     "mkdn",
                // )
                // .boxed(),
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Markdown,
                //     "mdown",
                // )
                // .boxed(),
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Markdown,
                //     "markdown",
                // )
                // .boxed(),
                // Mediawiki
                retrieve_source_file_extension(
                    account,
                    repository,
                    page,
                    client,
                    &Content::Mediawiki,
                    "mediawiki",
                )
                .boxed(),
                // Mediawiki
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::Mediawiki,
                //     "wiki",
                // )
                // .boxed(),
                // Org-Mode
                retrieve_source_file_extension(
                    account,
                    repository,
                    page,
                    client,
                    &Content::Orgmode,
                    "org",
                )
                .boxed(),
                // Pod
                // retrieve_source_file_extension(account, repository, page, client, &Content::Pod, "pod")
                //     .boxed(),
                // Rdoc
                // retrieve_source_file_extension(account, repository, page, client, &Content::Rdoc, "rdoc")
                //     .boxed(),
                // Textile
                retrieve_source_file_extension(
                    account,
                    repository,
                    page,
                    client,
                    &Content::Textile,
                    "textile",
                )
                .boxed(),
                // ReStructuredText
                retrieve_source_file_extension(
                    account,
                    repository,
                    page,
                    client,
                    &Content::ReStructuredText,
                    "rest",
                )
                .boxed(),
                // retrieve_source_file_extension(
                //     account,
                //     repository,
                //     page,
                //     client,
                //     &Content::ReStructuredText,
                //     "rst",
                // )
                // .boxed(),
            ])
            .await
            .map(|o| o.0)
        })
        .or_else(|_| async {
            retrieve_fallback_html(account, repository, page, client, "https://github.com")
                .await
        })
        .or_else(|_| async {
            retrieve_fallback_html(
                account,
                repository,
                page,
                client,
                "https://gh-mirror-gucl6ahvva-uc.a.run.app",
            )
            .await
        })
        .await
}

fn retrieve_fallback_html<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
    domain: &'a str,
) -> impl Future<Output = Result<Content, ContentError>> {
    let raw_github_url = format!("{}/{}/{}/wiki/{}", domain, account, repository, page);

    client
        .get(&raw_github_url)
        .send()
        .map_err(|e| ContentError::OtherError(e.to_string()))
        .and_then(|r| async {
            if r.status() == StatusCode::NOT_FOUND {
                return Err(ContentError::NotFound);
            }
            if r.status() == StatusCode::TOO_MANY_REQUESTS {
                return Err(ContentError::TooMayRequests);
            }
            if !r.status().is_success() {
                return Err(ContentError::OtherError(format!("Remote: {}", r.status())));
            }
            Ok(r)
        })
        .map_ok(|r| {
            r.text()
                .map_err(|e| ContentError::OtherError(e.to_string()))
        })
        .and_then(|t| t)
        .and_then(|html| async move {
            // Find #wiki-body and save it.
            let document = Html::parse_document(&html);
            document
                .select(&Selector::parse("#wiki-body").unwrap())
                .next()
                .map(|e| e.inner_html())
                .map(Content::FallbackHtml)
                .ok_or(ContentError::NotFound)
        })
}

// https://github-wiki-see.page/m/nelsonjchen/github-wiki-test/wiki/Fallback
fn retrieve_source_file_extension<'a, T: Fn(String) -> Content>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
    enum_constructor: T,
    extension: &'a str,
) -> impl Future<Output = Result<Content, ContentError>> {
    let raw_github_assets_url = format!(
        "https://raw.githubusercontent.com/wiki/{}/{}/{}.{}",
        account, repository, page, extension
    );

    client
        .get(&raw_github_assets_url)
        .send()
        .map_err(|e| ContentError::OtherError(e.to_string()))
        .and_then(|r| async {
            if r.status() == StatusCode::NOT_FOUND {
                return Err(ContentError::NotFound);
            }
            if !r.status().is_success() {
                return Err(ContentError::OtherError(format!("Remote: {}", r.status())));
            }
            Ok(r)
        })
        .map_ok(|r| {
            r.text()
                .map_err(|e| ContentError::OtherError(e.to_string()))
        })
        .and_then(|t| t)
        .map_ok(move |t| enum_constructor(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() {
        let future = retrieve_source_file_extension(
            "nelsonjchen",
            "github-wiki-test",
            "Home",
            &Client::new(),
            &Content::Markdown,
            "md",
        );
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }
}
