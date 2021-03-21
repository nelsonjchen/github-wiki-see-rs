use nipper::Document;

async fn download_github_wiki(
    account: &str,
    repository: &str,
    page: Option<&str>,
) -> Result<String, reqwest::Error> {
    let body = reqwest::get(format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        page.unwrap_or("")
    ))
    .await?
    .text()
    .await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn something() {
        let html = r#"<div>
            <a href="/1">One</a>
            <a href="/2">Two</a>
            <a href="/3">Three</a>
        </div>"#;

        let document = Document::from(html);
        let a = document.select("a:nth-child(3)");
        let text: &str = &a.text();
        assert_eq!(text, "Three");
    }

    #[test]
    fn github_html() {
        let html = include_str!("../test-data/wiki-index.html");

        let document = Document::from(html);
        let a = document.select("#wiki-wrapper");
        let text: &str = &a.html();
        assert_eq!(text, "Three");
    }

    #[actix_rt::test]
    async fn download_github_wiki_test() {
        let html = download_github_wiki(
            "nelsonjchen",
             "github-wiki-test",
              None).await.unwrap();


        let document = Document::from(&html);
        let a = document.select("#wiki-wrapper");
        let text: &str = &a.html();
        assert_eq!(text, "Three");
    }
}

