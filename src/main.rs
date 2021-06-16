use std::{process, time::Duration};

use actix_web::{
    get, http,
    middleware::Logger,
    rt::{spawn, time},
    web::{self, scope},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;

mod scraper;

#[derive(Template)]
#[template(path = "front_page.html")]

struct FrontPageTemplate {}

async fn front_page(_req: HttpRequest) -> impl Responder {
    let hello = FrontPageTemplate {}; // instantiate your struct
    hello
        .render()
        .unwrap()
        .with_header("Content-Type", "text/html; charset=utf-8")
}

#[derive(Template)]
#[template(path = "mirror.html")]

struct MirrorTemplate<'a> {
    original_title: &'a str,
    original_url: &'a str,
    mirrored_content: &'a str,
}

#[get("/{account}/{repository}/wiki")] // <- define path parameters
async fn mirror_root(
    web::Path((account, repository)): web::Path<(String, String)>,
) -> impl Responder {
    mirror_content(account, repository, None).await
}

#[get("/{account}/{repository}/wiki/{page}")] // <- define path parameters
async fn mirror_page(
    web::Path((account, repository, page)): web::Path<(String, String, String)>,
) -> impl Responder {
    mirror_content(account, repository, Some(page)).await
}

async fn mirror_content(
    account: String,
    repository: String,
    page: Option<String>,
) -> impl Responder {
    let url = format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        page.clone().unwrap_or_else(|| "".to_string())
    );

    let html_info = scraper::get_element_html(&account, &repository, page.as_deref())
        .await
        .unwrap();

    let mirror_content = MirrorTemplate {
        original_title: &html_info.original_title,
        original_url: &url,
        mirrored_content: &(html_info.html),
    };

    if mirror_content.original_title.contains("Page not found") {
        mirror_content
            .render()
            .unwrap()
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_status(http::StatusCode::NOT_FOUND)
    } else if mirror_content.original_title.eq("Rate limit · GitHub") {
        // Quit in some seconds if rate limit is hit
        spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            interval.tick().await;
            process::exit(0);
        });
        mirror_content
            .render()
            .unwrap()
            .with_header("Content-Type", "text/html; charset=utf-8")
            .with_status(http::StatusCode::TOO_MANY_REQUESTS)
    } else {
        mirror_content
            .render()
            .unwrap()
            .with_header("Content-Type", "text/html; charset=utf-8")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(front_page))
            .route(
                "favicon.ico",
                web::get().to(|| {
                    HttpResponse::Ok()
                        .body(include_bytes!("../templates/favicon.ico") as &'static [u8])
                }),
            )
            .route(
                "robots.txt",
                web::get().to(|| {
                    HttpResponse::Ok()
                        .body(include_bytes!("../templates/robots.txt") as &'static [u8])
                }),
            )
            .route(
                "sitemap.xml",
                web::get().to(|| {
                    HttpResponse::MovedPermanently().header(
                     http::header::LOCATION,
                      "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/sitemap_index.xml"
                    ).finish()
                }),
            )
            .route(
                "base_sitemap.xml",
                web::get().to(|| {
                    HttpResponse::MovedPermanently().header(
                     http::header::LOCATION,
                      "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/base_sitemap.xml"
                    ).finish()
                }),
            )
            .route(
                "generated_sitemap.xml",
                web::get().to(|| {
                    HttpResponse::MovedPermanently().header(
                     http::header::LOCATION,
                      "https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/generated_sitemap.xml"
                    ).finish()
                }),
            )
            .route(
                "seed_sitemaps/{id}",
                web::get().to(|web::Path(id): web::Path<String>| {
                    HttpResponse::MovedPermanently().header(
                     http::header::LOCATION,
                      format!("https://nelsonjchen.github.io/github-wiki-see-rs-sitemaps/seed_sitemaps/{}", id)
                    ).finish()
                }),
            )
            .service(scope("m").service(mirror_root).service(mirror_page))
            .wrap(Logger::default())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
