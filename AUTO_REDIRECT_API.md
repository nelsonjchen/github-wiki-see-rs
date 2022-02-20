# Auto Redirect API

This is the Javascript/CSS/HTML API to use to automatically redirect when one land on GHWSEE that scripters, extension authors, and power users may want to use.

Run this on the domain https://github-wiki-see.page and it will automatically redirect to GitHub.com's wiki page when landing from a search page onto an indexable GHWSEE page and won't fire on other pages such as the front page.

```javascript
if (document.getElementById('header_button'))
    document.querySelector(".visit_url_button").click();
```

This API will be maintained as the mirror page changes or gets updated. The ID names and class names used in the example above will stay this and will not change for the foreseeable future.

## Examples

Here are some examples of this "API" in use.

Please contribute if you have other examples of using this API with other setups and ecosystems.

### Page Extender.app

PageExtender is a Safari Extension that injects CSS and JS files into websites, allowing you to customize your favorite websites to your needs.

See [@gingerbeardman](https://github.com/gingerbeardman)'s post:

https://github.com/nelsonjchen/github-wiki-see-rs/issues/136#issuecomment-1040821971

