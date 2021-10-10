#[macro_use]
extern crate rocket;

use reqwest::Client;
use retrieval::Content;
use rocket::futures::TryFutureExt;
use rocket::http::{ContentType, Status};
use rocket::response::status;
use rocket::response::{Redirect, Responder};
use rocket::State;

use askama::Template;

use crate::gh_extensions::github_wiki_markdown_to_pure_markdown;
use crate::scraper::process_markdown;

mod gh_extensions;
mod retrieval;
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
    use retrieval::retrieve_source_file;
    use retrieval::ContentError;
    use MirrorError::*;

    // Have original URL to forward to if there is an error.
    let original_url = format!(
        "https://github.com/{}/{}/wiki/{}",
        account, repository, page,
    );
    // Rocket's Redirect doesn't like unencoded URLs.
    let original_url_encoded = format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        percent_encoding::utf8_percent_encode(page, percent_encoding::NON_ALPHANUMERIC),
    );

    let page_title = page.replace("-", " ");

    // Grab main content from GitHub
    // Consider it "fatal" if this doesn't exist/errors and forward to GitHub or return an error.
    let content = retrieve_source_file(account, repository, page, client)
        .map_err(|e| match e {
            ContentError::NotFound => GiveUpSendToGitHub(Redirect::temporary(original_url_encoded)),
            ContentError::OtherError(e) => InternalError(status::Custom(
                Status::InternalServerError,
                MirrorTemplate {
                    original_title: page_title.clone(),
                    original_url: original_url.clone(),
                    mirrored_content: format!("500 Internal Server Error - {}", e),
                },
            )),
        })
        .await?;

    let original_html = content_to_html(content, account, repository, page);

    // The content exists. Now try to get the sidebar.
    let sidebar_content = retrieve_source_file(account, repository, "_Sidebar", client)
        .await
        .ok();

    let sidebar_html =
        sidebar_content.map(|content| content_to_html(content, account, repository, page));

    // Append the sidebar if it exists
    let mirrored_content = if let Some(sidebar_html) = sidebar_html {
        format!(
            "{}\n
            <h1>Sidebar</h1>
            \n{}",
            original_html, sidebar_html,
        )
    } else {
        original_html
    };

    Ok(MirrorTemplate {
        original_title: page_title.clone(),
        original_url: original_url.clone(),
        mirrored_content,
    })
}

fn content_to_html(content: Content, account: &str, repository: &str, page: &str) -> String {
    match content {
        retrieval::Content::Markdown(md) => {
            // Markdown can have mediawiki links in them apparently
            let pure_markdown = github_wiki_markdown_to_pure_markdown(&md, account, repository);
            process_markdown(&pure_markdown, account, repository, page == "Home")
        }
        retrieval::Content::AsciiDoc(ascii_doc) => ascii_doc,
    }
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
