use quick_xml::events::BytesText;
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use std::future::Future;
use std::sync::LazyLock;
use thiserror::Error;

use crate::decommission::DECOMMISSION_LIST;
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentError {
    #[error("not found")]
    NotFound,
    #[error("too many requests")]
    TooMayRequests,
    #[error("wiki has been decommissioned")]
    Decommissioned,
    #[error("{0}")]
    OtherError(String),
}

const FALLBACK_HOST: &str = "https://gh-mirror-gucl6ahvva-uc.a.run.app";
static HTML_IN_MARKDOWN_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("<.{3,10}>").expect("html detection regex should compile"));
static WIKI_BODY_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("#wiki-body").expect("wiki body selector should compile"));

fn repo_slug(account: &str, repository: &str) -> String {
    format!("{account}/{repository}")
}

fn raw_wiki_source_url(account: &str, repository: &str, page: &str, extension: &str) -> String {
    let page_encoded =
        percent_encoding::utf8_percent_encode(page, percent_encoding::NON_ALPHANUMERIC);
    format!(
        "https://raw.githubusercontent.com/wiki/{account}/{repository}/{page_encoded}.{extension}"
    )
}

fn wiki_html_url(domain: &str, account: &str, repository: &str, page: &str) -> String {
    if page.is_empty() || page == "Home" {
        format!("{domain}/{account}/{repository}/wiki")
    } else {
        format!("{domain}/{account}/{repository}/wiki/{page}")
    }
}

fn response_to_content_error(status: StatusCode) -> Result<(), ContentError> {
    match status {
        StatusCode::NOT_FOUND | StatusCode::FOUND | StatusCode::MOVED_PERMANENTLY => {
            Err(ContentError::NotFound)
        }
        StatusCode::TOO_MANY_REQUESTS => Err(ContentError::TooMayRequests),
        status if status.is_success() => Ok(()),
        status => Err(ContentError::OtherError(format!("Remote: {status}"))),
    }
}

fn markdown_contains_html(content: &Content) -> bool {
    matches!(content, Content::Markdown(md) if HTML_IN_MARKDOWN_RE.is_match(md))
}

async fn with_rate_limit_fallback<T, Fut, F>(fetch: F) -> Result<T, ContentError>
where
    F: Fn(&'static str) -> Fut,
    Fut: Future<Output = Result<T, ContentError>>,
{
    match fetch("https://github.com").await {
        Err(ContentError::TooMayRequests) => fetch(FALLBACK_HOST).await,
        result => result,
    }
}

pub async fn retrieve_source_file(
    account: &str,
    repository: &str,
    page: &str,
    client: &Client,
) -> Result<Content, ContentError> {
    // Skip decommissioned wikis
    if DECOMMISSION_LIST.contains(repo_slug(account, repository).as_str()) {
        return Err(ContentError::Decommissioned);
    }

    match retrieve_source_file_extension(account, repository, page, client, Content::Markdown, "md")
        .await
    {
        Ok(content) if !markdown_contains_html(&content) => Ok(content),
        Ok(_) | Err(_) => {
            with_rate_limit_fallback(|domain| async move {
                retrieve_fallback_html(account, repository, page, client, domain).await
            })
            .await
        }
    }
}

async fn retrieve_github_com_html(
    account: &str,
    repository: &str,
    page: &str,
    client: &Client,
    domain: &str,
) -> Result<String, ContentError> {
    let response = client
        .get(wiki_html_url(domain, account, repository, page))
        .send()
        .await
        .map_err(|error| ContentError::OtherError(error.to_string()))?;

    response_to_content_error(response.status())?;

    response
        .text()
        .await
        .map_err(|error| ContentError::OtherError(error.to_string()))
}

async fn retrieve_fallback_html(
    account: &str,
    repository: &str,
    page: &str,
    client: &Client,
    domain: &str,
) -> Result<Content, ContentError> {
    let html = retrieve_github_com_html(account, repository, page, client, domain).await?;

    let document = Html::parse_document(&html);
    document
        .select(&WIKI_BODY_SELECTOR)
        .next()
        .map(|e| e.inner_html())
        .map(Content::FallbackHtml)
        .ok_or(ContentError::NotFound)
}

// https://github-wiki-see.page/m/nelsonjchen/github-wiki-test/wiki/Fallback
async fn retrieve_source_file_extension<T>(
    account: &str,
    repository: &str,
    page: &str,
    client: &Client,
    enum_constructor: T,
    extension: &str,
) -> Result<Content, ContentError>
where
    T: Fn(String) -> Content,
{
    let response = client
        .get(raw_wiki_source_url(account, repository, page, extension))
        .send()
        .await
        .map_err(|error| ContentError::OtherError(error.to_string()))?;

    response_to_content_error(response.status())?;

    let body = response
        .text()
        .await
        .map_err(|error| ContentError::OtherError(error.to_string()))?;

    Ok(enum_constructor(body))
}

pub async fn retrieve_wiki_index(
    account: &str,
    repository: &str,
    client: &Client,
) -> Result<Content, ContentError> {
    let html = with_rate_limit_fallback(|domain| async move {
        retrieve_github_com_html(account, repository, "", client, domain).await
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
            .map(|(url, text)| format!("* [{text}]({url})"))
            .collect::<Vec<String>>()
            .join("\n"),
    ));
    Ok(content)
}

pub async fn retrieve_wiki_sitemap_index(
    account: &str,
    repository: &str,
    client: &Client,
) -> Result<String, ContentError> {
    let html = with_rate_limit_fallback(|domain| async move {
        retrieve_github_com_html(account, repository, "", client, domain).await
    })
    .await?;
    let mut wiki_page_urls = process_html_index(&html);

    // Add the synthetic index page
    wiki_page_urls.push((
        format!("/{account}/{repository}/wiki_index"),
        "Wiki Index".to_string(),
    ));

    use quick_xml::events::{BytesEnd, BytesStart, Event};

    use quick_xml::Writer;
    use std::io::Cursor;

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut urlset_el = BytesStart::new("urlset");
    urlset_el.push_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"));
    urlset_el.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    urlset_el.push_attribute(("xsi:schemaLocation", "http://www.sitemaps.org/schemas/sitemap/0.9 http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd"));

    writer
        .write_event(Event::Start(urlset_el))
        .map_err(|o| ContentError::OtherError(o.to_string()))?;

    for (url, _) in wiki_page_urls {
        let url_el = BytesStart::new("url");
        writer
            .write_event(Event::Start(url_el))
            .map_err(|o| ContentError::OtherError(o.to_string()))?;

        let loc_el = BytesStart::new("loc");
        writer
            .write_event(Event::Start(loc_el))
            .map_err(|o| ContentError::OtherError(o.to_string()))?;

        writer
            .write_event(Event::Text(BytesText::new(&format!(
                "https://github-wiki-see.page/m{url}"
            ))))
            .map_err(|o| ContentError::OtherError(o.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new("loc")))
            .map_err(|o| ContentError::OtherError(o.to_string()))?;

        writer
            .write_event(Event::End(BytesEnd::new("url")))
            .map_err(|o| ContentError::OtherError(o.to_string()))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("urlset")))
        .map_err(|op| ContentError::OtherError(op.to_string()))?;

    use std::str;
    let written = &writer.into_inner().into_inner();
    let xml_str = str::from_utf8(written).map_err(|op| ContentError::OtherError(op.to_string()))?;
    Ok(xml_str.to_string())
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

        println!("{content:?}");
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

        println!("{content:?}");
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn html_in_markdown() {
        let client = Client::new();

        let future = retrieve_source_file("wlsdn2316", "1-tetris-", "Functions", &client);
        let content = future.await;

        assert!(content.is_ok());
        // Fallback must be used for HTML in Markdown documents
        assert!(matches!(content, Ok(Content::FallbackHtml(_))));
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

        println!("{content:?}");
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

        println!("{content:?}");
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn page_list() {
        let client = Client::new();
        let future = retrieve_wiki_index("nelsonjchen", "github-wiki-test", &client);
        let content = future.await;

        println!("{content:?}");
        assert!(content.is_ok());
    }

    #[tokio::test]
    async fn wiki_sitemap_index() {
        let client = Client::new();
        let future = retrieve_wiki_sitemap_index("nelsonjchen", "github-wiki-test", &client);
        let content = future.await;

        println!("{content:?}");
        assert!(content.is_ok());
    }
}
