use std::future::Future;

use reqwest::{Client, StatusCode};
use rocket::futures::TryFutureExt;
use scraper::{Html, Selector};

use crate::scraper::process_html_index;

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

#[derive(Debug, PartialEq, Eq)]
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
            retrieve_fallback_html(account, repository, page, client, "https://github.com").await
        })
        .or_else(|err| async {
            if err == ContentError::TooMayRequests {
                retrieve_fallback_html(
                    account,
                    repository,
                    page,
                    client,
                    "https://gh-mirror-gucl6ahvva-uc.a.run.app",
                )
                .await
            } else {
                Err(err)
            }
        })
        .await
}

async fn retrieve_github_com_html<'a>(
    account: &str,
    repository: &str,
    page: &str,
    client: &'a Client,
    domain: &'a str,
) -> Result<String, ContentError> {
    // Home is special
    let raw_github_url = if page == "Home" {
        format!("{}/{}/{}/wiki", domain, account, repository)
    } else {
        format!("{}/{}/{}/wiki/{}", domain, account, repository, page)
    };

    let resp_attempt = client.get(raw_github_url).send().await;

    let resp = resp_attempt.map_err(|e| ContentError::OtherError(e.to_string()))?;

    if resp.status() == StatusCode::NOT_FOUND {
        return Err(ContentError::NotFound);
    }

    // GitHub does this for unlogged in pages.
    if resp.status() == StatusCode::FOUND {
        return Err(ContentError::NotFound);
    }
    if resp.status() == StatusCode::MOVED_PERMANENTLY {
        return Err(ContentError::NotFound);
    }

    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(ContentError::TooMayRequests);
    }
    if !resp.status().is_success() {
        return Err(ContentError::OtherError(format!(
            "Remote: {}",
            resp.status()
        )));
    }

    resp.text()
        .await
        .map_err(|e| ContentError::OtherError(e.to_string()))
}

async fn retrieve_fallback_html<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
    domain: &'a str,
) -> Result<Content, ContentError> {
    let html = retrieve_github_com_html(account, repository, page, client, domain).await?;

    let document = Html::parse_document(&html);
    println!("{:?}", document);
    document
        .select(&Selector::parse("#wiki-body").unwrap())
        .next()
        .map(|e| e.inner_html())
        .map(Content::FallbackHtml)
        .ok_or(ContentError::NotFound)
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
    let page_encoded =
        percent_encoding::utf8_percent_encode(page, percent_encoding::NON_ALPHANUMERIC);
    let raw_github_assets_url = format!(
        "https://raw.githubusercontent.com/wiki/{}/{}/{}.{}",
        account, repository, page_encoded, extension
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
        .map_ok(enum_constructor)
}

pub async fn retrieve_wiki_index<'a>(
    account: &'a str,
    repository: &'a str,
    client: &'a Client,
) -> Result<Content, ContentError> {
    let html = retrieve_github_com_html(account, repository, "", client, "https://github.com")
        .or_else(|err| async {
            if err == ContentError::TooMayRequests {
                retrieve_github_com_html(
                    account,
                    repository,
                    "",
                    client,
                    "https://gh-mirror-gucl6ahvva-uc.a.run.app",
                )
                .await
            } else {
                Err(err)
            }
        })
        .await?;
    let wiki_page_urls = process_html_index(&html);
    let content = Content::Markdown(format!(
        "{} page(s) in this GitHub Wiki:

{}
",
        wiki_page_urls.len(),
        wiki_page_urls
            .into_iter()
            .map(|(url, text)| format!("* [{}]({})", text, url))
            .collect::<Vec<String>>()
            .join("\n"),
    ));
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() {
        let client = Client::new();

        let future = retrieve_source_file_extension(
            "nelsonjchen",
            "github-wiki-test",
            "Home",
            &client,
            &Content::Markdown,
            "md",
        );
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn encoded() {
        let client = Client::new();

        let future = retrieve_source_file_extension(
            "naver",
            "billboard.js",
            "How-to-bundle-for-legacy-browsers?",
            &client,
            &Content::Markdown,
            "md",
        );
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn fallback_encoded() {
        let client = Client::new();

        let future = retrieve_github_com_html(
            "naver",
            "billboard.js",
            "How-to-bundle-for-legacy-browsers?",
            &client,
            "https://github.com",
        );
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn fallback_soapy() {
        let client = Client::new();

        let future = retrieve_github_com_html(
            "pothosware",
            "SoapySDR",
            "Home",
            &client,
            "https://github.com",
        );
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn page_list() {
        let client = Client::new();
        let future = retrieve_wiki_index("nelsonjchen", "github-wiki-test", &client);
        let content = future.await;

        println!("{:?}", content);
        assert!(content.is_ok());
    }
}
