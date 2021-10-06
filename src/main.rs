#[macro_use]
extern crate rocket;

use reqwest::{Client, StatusCode};
use rocket::futures::{FutureExt, TryFutureExt};
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::response::{Redirect, Responder};
use rocket::State;

use askama::Template;

use crate::gh_extensions::github_wiki_markdown_to_pure_markdown;
use crate::scraper::process_markdown;

mod gh_extensions;
mod scraper;

#[derive(Template)]
#[template(path = "front_page.html")]
struct FrontPageTemplate {}

#[get("/")]
fn front() -> FrontPageTemplate {
    FrontPageTemplate {}
}

#[get("/favicon.ico")]
fn favicon() -> (Status, (ContentType, &'static [u8])) {
    (
        Status::Ok,
        (
            ContentType::Icon,
            include_bytes!("../templates/favicon.ico"),
        ),
    )
}

#[get("/robots.txt")]
fn robots_txt() -> (Status, (ContentType, &'static [u8])) {
    (
        Status::Ok,
        (
            ContentType::Plain,
            include_bytes!("../templates/robots.txt"),
        ),
    )
}

#[get("/sitemap.xml")]
fn sitemap_xml() -> Redirect {
    Redirect::permanent(uri!(
        "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/sitemap_index.xml"
    ))
}

#[get("/base_sitemap.xml")]
fn base_sitemap_xml() -> Redirect {
    Redirect::permanent(uri!(
        "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/base_sitemap.xml"
    ))
}

#[get("/generated_sitemap.xml")]
fn generated_sitemap_xml() -> Redirect {
    Redirect::permanent(uri!(
        "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/generated_sitemap.xml"
    ))
}

#[get("/seed_sitemaps/<id>")]
fn seed_sitemaps(id: &str) -> Redirect {
    Redirect::permanent(format!(
        "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/seed_sitemaps/{}",
        id
    ))
}

#[derive(Template)]
#[template(path = "mirror.html")]

struct MirrorTemplate {
    original_title: String,
    original_url: String,
    mirrored_content: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Responder)]
enum MirrorError {
    // DocumentNotFound(NotFound<MirrorTemplate>),
    InternalError(status::Custom<MirrorTemplate>),
    GiveUpSendToGitHub(Redirect),
}

#[get("/<account>/<repository>/wiki")]
async fn mirror_home<'a>(
    account: &'a str,
    repository: &'a str,
    client: &State<Client>,
) -> Result<MirrorTemplate, MirrorError> {
    mirror_page(account, repository, "Home", client).await
}

#[get("/<account>/<repository>/wiki/<page>")]
async fn mirror_page<'a>(
    account: &'a str,
    repository: &'a str,
    page: &'a str,
    client: &State<Client>,
) -> Result<MirrorTemplate, MirrorError> {
    use MirrorError::*;

    // Check github_assets
    let raw_github_assets_url = format!(
        "https://raw.githubusercontent.com/wiki/{}/{}/{}.md",
        account, repository, page,
    );

    // Have original URL to forward to if there is an error.
    let original_url = format!(
        "https://github.com/{}/{}/wiki/{}",
        account, repository, page,
    );
    let original_url_encoded = format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        percent_encoding::utf8_percent_encode(page, percent_encoding::NON_ALPHANUMERIC),
    );

    let page_title = page.replace("-", " ");

    // Try to grab Stuff

    // Download raw_github_assets_url
    let resp = client
        .get(&raw_github_assets_url)
        .send()
        .await
        .map_err(|e| {
            InternalError(status::Custom(
                Status::InternalServerError,
                MirrorTemplate {
                    original_title: page_title.clone(),
                    original_url: original_url.clone(),
                    mirrored_content: format!("500 Internal Server Error - {}", e),
                },
            ))
        })?;

    if resp.status() == StatusCode::NOT_FOUND {
        // return Err(DocumentNotFound(NotFound(MirrorTemplate {
        //     original_title: page_title.clone(),
        //     original_url: original_url.clone(),
        //     mirrored_content: format!("{}", resp.status()),
        // })));

        // Just send them onto GitHub
        return Err(GiveUpSendToGitHub(Redirect::temporary(
            original_url_encoded,
        )));
    }

    if !resp.status().is_success() {
        return Err(InternalError(status::Custom(
            Status::InternalServerError,
            MirrorTemplate {
                original_title: page_title.clone(),
                original_url: original_url.clone(),
                mirrored_content: format!("Remote: {}", resp.status()),
            },
        )));
    }

    let original_markdown = resp.text().await.map_err(|e| {
        InternalError(status::Custom(
            Status::InternalServerError,
            MirrorTemplate {
                original_title: page_title.clone(),
                original_url: original_url.clone(),
                mirrored_content: format!("Internal Server Error - {}", e.to_string()),
            },
        ))
    })?;

    let sidebar_markdown_future = client
        .get(format!(
            "https://github.com/{}/{}/wiki/_Sidebar.md",
            account, repository
        ))
        .send()
        .map(|r| r.map(|r| r.text()))
        .and_then(|f| f);

    let sidebar_markdown_option: Option<String> = sidebar_markdown_future
        .await
        .ok()
        .map(|s| format!("\n\n# Sidebar\n\n{}", s));

    let content_markdown = {
        if let Some(sidebar_markdown) = sidebar_markdown_option {
            format!("{}{}", original_markdown, sidebar_markdown)
        } else {
            original_markdown
        }
    };

    let pure_markdown =
        github_wiki_markdown_to_pure_markdown(&content_markdown, account, repository);

    let mirrored_content = if page == "Home" {
        process_markdown(&pure_markdown, account, repository, true)
    } else {
        process_markdown(&pure_markdown, account, repository, false)
    };

    Ok(MirrorTemplate {
        original_title: page_title.clone(),
        original_url: original_url.clone(),
        mirrored_content,
    })
}

#[launch]
fn rocket() -> _ {
    // Mount front Page

    // Mount Mirror
    rocket::build()
        .mount("/m", routes![mirror_home, mirror_page,])
        .mount(
            "/",
            routes![
                front,
                favicon,
                robots_txt,
                sitemap_xml,
                base_sitemap_xml,
                generated_sitemap_xml,
                seed_sitemaps
            ],
        )
        .manage(Client::new())
}
