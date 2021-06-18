use std::{net::IpAddr, sync::Mutex};

use actix_web::{
    get, http,
    middleware::Logger,
    web::{self, scope},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use log::info;

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
    data: web::Data<Mutex<AppData>>,
) -> impl Responder {
    mirror_content(account, repository, None, data).await
}

#[get("/{account}/{repository}/wiki/{page}")] // <- define path parameters
async fn mirror_page(
    web::Path((account, repository, page)): web::Path<(String, String, String)>,
    data: web::Data<Mutex<AppData>>,
) -> impl Responder {
    mirror_content(account, repository, Some(page), data).await
}

async fn mirror_content(
    account: String,
    repository: String,
    page: Option<String>,
    data: web::Data<Mutex<AppData>>,
) -> impl Responder {
    // Shutdown after 30 connections.
    // {
    //     let mut app_data = data.lock().unwrap();
    //     app_data.request_odometer += 1;
    //     info!(
    //         "({}) GitHub request odometer calls so far: {}",
    //         app_data.public_ip_addr, app_data.request_odometer
    //     );
    //     let limit = 30;
    //     if app_data.request_odometer > limit {
    //         info!(
    //             "Requesting shutdown as odometer ({}) past limit ({}) for ip address {:?}",
    //             app_data.request_odometer, limit, app_data.public_ip_addr
    //         );
    //         app_data.shutdown_sender.send(()).unwrap();
    //     }
    // }

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
        let app_data = data.lock().unwrap();
        info!(
            "Requesting shutdown as rate limit hit for IP address {:?} with odometer at {}",
            app_data.public_ip_addr, app_data.request_odometer,
        );
        app_data.shutdown_sender.send(()).unwrap();

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

struct AppData {
    request_odometer: usize,
    shutdown_sender: crossbeam_channel::Sender<()>,
    public_ip_addr: IpAddr,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // IP Address
    let public_ip_addr: IpAddr = external_ip::get_ip().await.unwrap();
    info!("Discovered Public IP address: {:?}", public_ip_addr);

    // Shutdown Channel
    let (s, r) = crossbeam_channel::unbounded::<()>();

    let data = web::Data::new(Mutex::new(AppData {
        request_odometer: 0,
        shutdown_sender: s,
        public_ip_addr,
    }));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
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
    .run();
    // Forever-ish wait for shutdown notice
    r.recv().unwrap();
    server.stop(true).await;
    Ok(())
}
