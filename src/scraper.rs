use comrak::{markdown_to_html, ComrakOptions};
use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use nipper::Document; // <-- Add nipper import back

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
    options.extension.header_ids = Some("".to_string());
    options.render.github_pre_lang = true;

    let original_html = markdown_to_html(original_markdown, &options);
    process_html(&original_html, account, repository, homepage_prepend)
}

// New lol_html version of process_html
pub fn process_html(
    original_html: &str,
    account: &str,
    repository: &str,
    homepage_prepend: bool,
) -> String {
    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a[href]", |el: &mut Element| {
                    if let Some(href) = el.get_attribute("href") {
                        if !href.starts_with("http://")
                            && !href.starts_with("https://")
                            && !href.starts_with("//")
                        {
                            if href.starts_with('/') {
                                let new_href = format!("/m{}", href);
                                el.set_attribute("href", &new_href).unwrap();
                            } else {
                                if homepage_prepend && !href.starts_with("wiki/") {
                                    let new_href = format!("wiki/{}", href);
                                    el.set_attribute("href", &new_href).unwrap();
                                }
                            }
                        } else {
                            el.set_attribute("rel", "nofollow ugc").unwrap();
                        }
                    }
                    Ok(())
                }),
                element!("img[src]", |el: &mut Element| {
                    if let Some(src) = el.get_attribute("src") {
                        if !src.starts_with("http://")
                            && !src.starts_with("https://")
                            && !src.starts_with("//")
                        {
                            if src.starts_with('/') {
                                let new_src = format!("https://github.com{}", src);
                                el.set_attribute("src", &new_src).unwrap();
                            } else if !src.starts_with("wiki") {
                                let new_src =
                                    format!("https://github.com/{}/{}/wiki/{}", account, repository, src);
                                el.set_attribute("src", &new_src).unwrap();
                            } else {
                                let new_src =
                                    format!("https://github.com/{}/{}/{}", account, repository, src);
                                el.set_attribute("src", &new_src).unwrap();
                            }
                        }
                    }
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(original_html.as_bytes()).unwrap();
    rewriter.end().unwrap();

    String::from_utf8(output).unwrap()
}

// Original nipper version of process_html_index
pub fn process_html_index(original_html: &str) -> Vec<(String, String)> {
    let document = Document::from(original_html);
    document
        .select("#wiki-pages-box a, .flex-auto.min-width-0.col-12.col-md-8 a")
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
    fn get_page_list() {
        let html = include_str!("../test-data/wiki-index.html");

        let pages = process_html_index(html);
        assert!(pages.len() > 3);
        let page_1 = pages.first().unwrap();
        assert!(page_1.0.contains("nelsonjchen"));
        assert!(page_1.0.contains("wiki"));
        assert_eq!(page_1.1.trim(), "Home");
    }

    #[test]
    fn get_page_list_homeless() {
        let html = include_str!("../test-data/wiki-homeless-index.html");

        let pages = process_html_index(html);
        use more_asserts::assert_ge;
        assert_ge!(pages.len(), 3);
        assert!(pages.first().unwrap().0.contains("Homeless"));
        assert_eq!(pages.first().unwrap().1.trim(), "Homeless");
        assert!(pages.get(1).unwrap().0.contains("Ooze"));
        assert_eq!(pages.get(1).unwrap().1.trim(), "Ooze");
        assert!(pages.get(2).unwrap().0.contains("Porkchops"));
        assert_eq!(pages.get(2).unwrap().1.trim(), "Porkchops");
    }
}
