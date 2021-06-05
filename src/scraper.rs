use nipper::Document;

pub struct HtmlWithInfo {
    pub original_title: String,
    pub html: String,
}

fn download_github_wiki(
    account: &str,
    repository: &str,
    page: Option<&str>,
) -> Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(format!(
        "https://github.com/{}/{}/wiki/{}",
        account,
        repository,
        page.unwrap_or("")
    ))?
    .text()?;
    Ok(body)
}

pub fn get_element_html(account: &str, repository: &str, page: Option<&str>) -> HtmlWithInfo {
    let html = download_github_wiki(account, repository, page);

    let processed_html = process_html(html.unwrap());

    let document = Document::from(&processed_html);
    let a = document.select("#wiki-wrapper");
    let title = String::from(document.select("title").text());
    HtmlWithInfo {
        original_title: title,
        html: a.html().to_string(),
    }

}

pub fn process_html(original_html: String) -> String {
    let document = Document::from(&original_html);
    document.select("a").iter().for_each(|mut thing| {
        if let Some(href) = thing.attr("href") {
            let string_href = String::from(href);
            if string_href.starts_with('/') {
                let new_string_href = "/m".to_owned() + &string_href;
                thing.set_attr("href", &new_string_href);
            }
        }
    });
    String::from(document.html())
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
        assert_ne!(text.len(), 0);
    }

    #[test]
    fn download_github_wiki_test() {
        let html = download_github_wiki("nelsonjchen", "github-wiki-test", None).unwrap();

        let document = Document::from(&html);
        let a = document.select("#wiki-wrapper");
        let text: &str = &a.html();
        assert_ne!(text.len(), 0);
    }

    #[test]
    fn transform_urls_to_new_root() {
        let html = "<a href=\"/\"></a>";

        let document = Document::from(html);
        document.select("a").iter().for_each(|mut thing| {
            if let Some(href) = thing.attr("href") {
                let string_href = String::from(href);
                if string_href.starts_with("/") {
                    let new_string_href = "/m".to_owned() + &string_href;
                    thing.set_attr("href", &new_string_href);
                }
            }
        });

        assert_eq!(
            String::from(document.select("a").html()),
            "<a href=\"/m/\"></a>"
        );
    }
}
