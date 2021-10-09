use reqwest::{Client, StatusCode};
use rocket::{
    futures::{FutureExt, TryFutureExt},
    response::status::NotFound,
};

enum Content {
    Markdown(String),
    Unhandled(String),
}

enum ContentError {
    NotFound,
    OtherError(String),
}

async fn retrieve_source<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &'a Client,
) -> Result<Content, ContentError> {
    let raw_github_assets_url = format!(
        "https://raw.githubusercontent.com/wiki/{}/{}/{}.md",
        account, repository, page,
    );

    let resp = client
        .get(&raw_github_assets_url)
        .send()
        .map_err(|e| ContentError::OtherError(e.to_string()))
        .and_then(|r| async {
            if r.status() == StatusCode::NOT_FOUND {
                return Err(ContentError::NotFound);
            }
            Ok(r)
        })
        .map_ok(|r| r.text()
            .map_err(|e| ContentError::OtherError(e.to_string()))
        )
        .and_then(|t| t)
        .await?;

    Ok(Content::Markdown(resp))
}
