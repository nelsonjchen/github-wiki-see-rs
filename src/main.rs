#[macro_use]
extern crate rocket;

use std::time::Duration;

use reqwest::Client;
use retrieval::{retrieve_wiki_sitemap_index, Content};
use rocket::futures::TryFutureExt;
use rocket::http::{ContentType, Status};
use rocket::response::{content, status};
use rocket::response::{Redirect, Responder};
use rocket::State;

use crate::scraper::process_html;
use askama::Template;

use crate::gh_extensions::github_wiki_markdown_to_pure_markdown;
use crate::scraper::process_markdown;

mod decommission;
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

#[get("/social.png")]
fn social_png() -> (Status, (ContentType, &'static [u8])) {
    (
        Status::Ok,
        (
            ContentType::PNG,
            include_bytes!("../templates/social.png"),
        ),
    )
}

#[get("/callToAction.svg")]
fn call_to_action_svg() -> (Status, (ContentType, &'static [u8])) {
    (
        Status::Ok,
        (
            ContentType::SVG,
            include_bytes!("../templates/callToAction.svg"),
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

#[get("/debug_sitemaps/<account>/<repository>/sitemap.xml")]
async fn wiki_debug_sitemaps(
    account: &str,
    repository: &str,
    client: &State<Client>,
) -> Result<content::RawXml<String>, status::Custom<String>> {
    let content = retrieve_wiki_sitemap_index(account, repository, client)
        .await
        .map_err(|op| status::Custom(Status::InternalServerError, format!("Error: {:?}", op)))?;

    Ok(content::RawXml(content))
}

#[derive(Template)]
#[template(path = "mirror.html")]

struct MirrorTemplate {
    original_title: String,
    original_url: String,
    mirrored_content: String,
    index_url: String,
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

// Copied from percent_encoding crate but modified for what GitHub is OK with.
pub const NON_ALPHANUMERIC_GH: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    // .add(b'\'') // OK to exist in URL
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    // .add(b'-') // OK to exist in URL
    // .add(b'.') // OK to exist in URL
    .add(b'/')
    // .add(b':') // OK to exist in URL
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    // .add(b'_') // OK to exist in URL
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    .add(b'~');

#[get("/<account>/<repository>/wiki/Home", rank = 1)]
async fn mirror_page_redirect_home<'a>(account: &'a str, repository: &'a str) -> Redirect {
    Redirect::permanent(format!("/m/{account}/{repository}/wiki"))
}

#[get("/<account>/<repository>/wiki/<page>", rank = 2)]
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

    // Rocket's Redirect / GitHub itself doesn't like unencoded URLs.
    let original_url_encoded = format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        percent_encoding::utf8_percent_encode(page, NON_ALPHANUMERIC_GH),
    );

    let page_title = format!(
        "{} - {}/{} Wiki",
        page.replace('-', " "),
        account,
        repository
    );

    // Grab main content from GitHub
    // Consider it "fatal" if this doesn't exist/errors and forward to GitHub or return an error.
    let content = retrieve_source_file(account, repository, page, client)
        .map_err(|e| match e {
            ContentError::NotFound => {
                GiveUpSendToGitHub(Redirect::to(original_url_encoded.clone()))
            }
            ContentError::TooMayRequests => {
                GiveUpSendToGitHub(Redirect::temporary(original_url_encoded.clone()))
            }
            ContentError::Decommissioned => {
                GiveUpSendToGitHub(Redirect::permanent(original_url_encoded.clone()))
            }
            ContentError::OtherError(e) => InternalError(status::Custom(
                Status::InternalServerError,
                MirrorTemplate {
                    original_title: page_title.clone(),
                    original_url: original_url.clone(),
                    mirrored_content: format!("500 Internal Server Error - {}", e),
                    index_url: format!("/m/{}/{}/wiki_index", account, repository),
                },
            )),
        })
        .await?;

    let original_html = content_to_html(content, account, repository, page);

    // The content exists. Now try to get the sidebar.
    //
    // Disabled for load reasons
    //
    // let sidebar_content = retrieve_source_file(account, repository, "_Sidebar", client)
    //     .await
    //     .ok();
    let sidebar_content = None;

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
        original_url: original_url_encoded.clone(),
        mirrored_content,
        index_url: format!("/m/{}/{}/wiki_index", account, repository),
    })
}

#[get("/<account>/<repository>/wiki_index")]
async fn mirror_page_index<'a>(
    account: &'a str,
    repository: &'a str,
    client: &State<Client>,
) -> Result<MirrorTemplate, MirrorError> {
    use retrieval::retrieve_wiki_index;
    use retrieval::ContentError;
    use MirrorError::*;

    // Have original URL to forward to if there is an error.
    let original_url = format!("https://github.com/{}/{}/wiki/Home", account, repository);

    let page_title = format!("Page Index - {}/{} Wiki", account, repository);

    // Grab main content from GitHub
    // Consider it "fatal" if this doesn't exist/errors and forward to GitHub or return an error.
    let content = retrieve_wiki_index(account, repository, client)
        .map_err(|e| match e {
            // Retreive wiki index never returns decomissioned
            ContentError::NotFound => GiveUpSendToGitHub(Redirect::to(original_url.clone())),
            ContentError::TooMayRequests => {
                GiveUpSendToGitHub(Redirect::temporary(original_url.clone()))
            }
            // Not used, but could be if index is decommisioned
            ContentError::Decommissioned => {
                GiveUpSendToGitHub(Redirect::permanent(original_url.clone()))
            }
            ContentError::OtherError(e) => InternalError(status::Custom(
                Status::InternalServerError,
                MirrorTemplate {
                    original_title: page_title.clone(),
                    original_url: original_url.clone(),
                    mirrored_content: format!("500 Internal Server Error - {}", e),
                    index_url: format!("/m/{}/{}/wiki_index", account, repository),
                },
            )),
        })
        .await?;

    let original_html = content_to_html(content, account, repository, "Home");

    Ok(MirrorTemplate {
        original_title: page_title.clone(),
        original_url: original_url.clone(),
        mirrored_content: original_html,
        index_url: format!("/m/{}/{}/wiki_index", account, repository),
    })
}

fn content_to_html(content: Content, account: &str, repository: &str, page: &str) -> String {
    match content {
        Content::AsciiDoc(ascii_doc) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render asciidoc. Source for crawling below. Please visit the Original URL!** 🚨\n
```asciidoc\n
{}\n
```\n",
                ascii_doc
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Creole(cr) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Creole. Source for crawling below. Please visit the Original URL!** 🚨\n
```creole\n
{}\n
```\n",
                cr
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Markdown(md) => {
            // Markdown can have mediawiki links in them apparently
            let pure_markdown = github_wiki_markdown_to_pure_markdown(&md, account, repository);
            process_markdown(&pure_markdown, account, repository, page == "Home")
        }
        Content::Mediawiki(mw) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Mediawiki. Source for crawling below. Please visit the Original URL!** 🚨\n
```creole\n
{}\n
```\n",
mw
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Orgmode(og) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Org-Mode. Source for crawling below. Please visit the Original URL!** 🚨\n
```org\n
{}\n
```\n",
og
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Pod(p) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Pod. Source for crawling below. Please visit the Original URL!** 🚨\n
```pod\n
{}\n
```\n",
p
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Rdoc(rd) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Rdoc. Source for crawling below. Please visit the Original URL!** 🚨\n
```rdoc\n
{}\n
```\n",
rd
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::Textile(tt) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render Textile. Source for crawling below. Please visit the Original URL!** 🚨\n
```textile\n
{}\n
```\n",
tt
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::ReStructuredText(rst) => {
            let md = format!(
                "🚨 **github-wiki-see.page does not render ReStructuredText. Source for crawling below. Please visit the Original URL!** 🚨\n
```rst\n
{}\n
```\n",
rst
            );
            process_markdown(&md, account, repository, page == "Home")
        }
        Content::FallbackHtml(html) => {
            let annotated_html = format!("{} <h6>⚠️ **GitHub.com Fallback** ⚠️</h6>", html);
            process_html(&annotated_html, account, repository, page == "Home")
        }
    }
}

#[catch(404)]
fn not_found() -> &'static str {
    "404 Not Found - Links on this service may not work! CONTENT IS FOR CRAWLERS ONLY. Go back and visit the original page on GitHub for working links."
}

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[launch]
fn rocket() -> _ {
    // Mount front Page

    // Mount Mirror
    rocket::build()
        .register("/", catchers![not_found])
        .mount(
            "/m",
            routes![
                mirror_home,
                mirror_page_redirect_home,
                mirror_page,
                mirror_page_index
            ],
        )
        .mount(
            "/",
            routes![
                front,
                favicon,
                call_to_action_svg,
                social_png,
                robots_txt,
                sitemap_xml,
                base_sitemap_xml,
                generated_sitemap_xml,
                seed_sitemaps,
                wiki_debug_sitemaps,
            ],
        )
        .manage(
            Client::builder()
                .user_agent(APP_USER_AGENT)
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Could not build client"),
        )
}
