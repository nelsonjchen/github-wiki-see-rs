use core::str;

use comrak::{markdown_to_html, ComrakOptions};
use nipper::Document;

pub fn process_markdown(
    original_markdown: &str,
    account: &str,
    repository: &str,
    homepage_prepend: bool,
) -> String {
    let mut options = ComrakOptions::default();
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.render.github_pre_lang = true;

    let original_html = markdown_to_html(original_markdown, &options);
    process_html(&original_html, account, repository, homepage_prepend)
}

pub fn process_html(
    original_html: &str,
    account: &str,
    repository: &str,
    homepage_prepend: bool,
) -> String {
    let document = Document::from(original_html);
    document.select("a").iter().for_each(|mut thing| {
        if let Some(href) = thing.attr("href") {
            let string_href = String::from(href);
            if !string_href.starts_with("http://")
                && !string_href.starts_with("https://")
                && !string_href.starts_with("//")
            {
                if string_href.starts_with('/') {
                    let new_string_href = "/m".to_owned() + &string_href;
                    thing.set_attr("href", &new_string_href);
                } else {
                    // Prepend wiki if homepage
                    if homepage_prepend {
                        let new_string_href = "wiki/".to_owned() + &string_href;
                        thing.set_attr("href", &new_string_href);
                    }
                }
            } else {
                thing.set_attr("rel", "nofollow ugc");
            }
        }
    });
    document.select("img").iter().for_each(|mut thing| {
        if let Some(href) = thing.attr("src") {
            let string_href = String::from(href);
            if !string_href.starts_with("http://")
                && !string_href.starts_with("https://")
                && !string_href.starts_with("//")
            {
                if !string_href.starts_with("wiki") {
                    let new_string_href =
                        format!("https://github.com/{}/{}/wiki/", account, repository)
                            + &string_href;
                    thing.set_attr("src", &new_string_href);
                } else {
                    let new_string_href =
                        format!("https://github.com/{}/{}/", account, repository) + &string_href;
                    thing.set_attr("src", &new_string_href);
                }
            }
            if string_href.starts_with('/') {
                let new_string_href = "https://github.com".to_owned() + &string_href;
                thing.set_attr("src", &new_string_href);
            }
        }
    });
    // Unlink revisions _history link
    document
        .select("a.Link--muted")
        .iter()
        .for_each(|mut thing| {
            thing.replace_with_html(thing.text());
        });
    String::from(document.html())
}

pub fn process_html_index(original_html: &str) -> Vec<(String, String)> {
    let document = Document::from(original_html);
    document
        .select(".flex-1.py-1.text-bold, #wiki-content > div.Box a")
        .iter()
        .filter_map(|element| {
            element
                .attr("href")
                .map(|attr_value| (String::from(attr_value), String::from(element.text())))
        })
        .collect()
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
    fn transform_non_relative_urls_to_nofollow_ugc_https() {
        let html = "<html><head></head><body><a href=\"https://example.com\"></a></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><a href=\"https://example.com\" rel=\"nofollow ugc\"></a></body></html>"
        );
    }

    #[test]
    fn transform_non_relative_urls_to_nofollow_ugc_agnostic() {
        let html = "<html><head></head><body><a href=\"//example.com\"></a></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><a href=\"//example.com\" rel=\"nofollow ugc\"></a></body></html>"
        );
    }

    #[test]
    fn transform_non_relative_urls_to_nofollow_ugc_http() {
        let html = "<html><head></head><body><a href=\"http://example.com\"></a></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><a href=\"http://example.com\" rel=\"nofollow ugc\"></a></body></html>"
        );
    }

    #[test]
    fn transform_img_src_to_github_root() {
        let html = "<html><head></head><body><img src=\"/Erithano/Timon-Your-FAQ-bot-for-Microsoft-Teams/wiki/images/Guide1.1.jpg\"></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><img src=\"https://github.com/Erithano/Timon-Your-FAQ-bot-for-Microsoft-Teams/wiki/images/Guide1.1.jpg\"></body></html>"
        );
    }

    #[test]
    fn transform_img_src_to_github_root_relative() {
        // https://github.com/ant-media/Ant-Media-Server/wiki
        let html =
            "<html><head></head><body><img src=\"wiki/images/false-icon.png\"></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><img src=\"https://github.com/some_account/some_repo/wiki/images/false-icon.png\"></body></html>"
        );
    }

    #[test]
    fn transform_img_src_to_github_root_non_relative() {
        let html = "<html><head></head><body><img src=\"https://camo.githubusercontent.com/\"></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><img src=\"https://camo.githubusercontent.com/\"></body></html>"
        );
    }

    #[test]
    fn transform_img_src_to_github_root_non_relative_2() {
        // https://github.com/zanonmark/Google-4-TbSync/wiki/How-to-generate-your-own-Google-API-Console-project-credentials
        let html = "<html><head></head><body><img src=\"images/something.png\"></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><img src=\"https://github.com/some_account/some_repo/wiki/images/something.png\"></body></html>"
        );
    }

    #[test]
    fn transform_img_src_blob_to_github_raw() {
        // https://github.com/Navid200/xDrip/wiki/Updates
        let html = "<html><head></head><body><img src=\"https://github.com/Navid200/xDrip/blob/master/Documentation/images/Releases.png\"></body></html>";

        assert_eq!(
            process_html(html, "some_account", "some_repo", false),
            "<html><head></head><body><img src=\"https://github.com/Navid200/xDrip/raw/master/Documentation/images/Releases.png\"></body></html>"
        );
    }

    #[test]
    fn get_page_list() {
        let html = include_str!("../test-data/wiki-index.html");

        let pages = process_html_index(html);
        assert!(pages.len() > 3);
        let page_1 = pages.get(0).unwrap();
        assert!(page_1.0.contains("nelsonjchen"));
        assert!(page_1.0.contains("wiki"));
    }

    #[test]
    fn get_page_list_homeless() {
        let html = include_str!("../test-data/wiki-homeless-index.html");

        let pages = process_html_index(html);
        assert!(pages.len() > 3);
        let page_1 = pages.get(0).unwrap();
        assert!(page_1.0.contains("yuchberry"));
        assert!(page_1.0.contains("DDNS"));
    }
}
