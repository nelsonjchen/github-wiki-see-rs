use actix_web::{
    get,
    middleware::Logger,
    post,
    web::{self, scope},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
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

#[get("/{account}/{repository}")] // <- define path parameters
async fn mirror_root(
    web::Path((account, repository)): web::Path<(String, String)>,
) -> impl Responder {
    mirror_content(account, repository, None).await
}

#[get("/{account}/{repository}/{page}")] // <- define path parameters
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
        page.clone().unwrap_or("".to_string())
    );

    let mirrored_html_string =
        scraper::get_element_html(&account, &repository, page.as_deref());

    let mirror_content = MirrorTemplate {
        original_title: "idk",
        original_url: &url,
        mirrored_content: &mirrored_html_string,
    };
    mirror_content
        .render()
        .unwrap()
        .with_header("Content-Type", "text/html; charset=utf-8")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(front_page))
            .service(scope("mirror").service(mirror_root).service(mirror_page))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
