use std::future::Future;

use reqwest::{Client, StatusCode};
use rocket::futures::TryFutureExt;

#[derive(Debug)]
pub enum Content {
    Markdown(String),
    // Unhandled(String),
}

#[derive(Debug)]
pub enum ContentError {
    NotFound,
    OtherError(String),
}

pub fn retrieve_source_file<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
) -> impl Future<Output = Result<Content, ContentError>> {
    retrieve_source_file_extension(account, repository, page, client, &Content::Markdown, "md")
}

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
