use lol_html::html_content::ContentType;
use lol_html::{element, HtmlRewriter, Settings};
use worker::wasm_bindgen::JsValue;
use worker::*;

#[event(fetch)]
async fn main(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let my_domain = env.var("MY_DOMAIN")?.to_string();
    let notion_domain = env.var("NOTION_DOMAIN")?.to_string();
    let index = env.var("INDEX_PAGE_ID")?.to_string();
    let links = env.var("LINK_PAGE_ID")?.to_string();
    let donate = env.var("DONATE_PAGE_ID")?.to_string();
    // let title = env.var("DONATE_PAGE_ID")?.to_string();
    // let des = env.var("PAGE_DESCRIPTION")?.to_string();
    match req.path().as_str() {
        "/" => {
            return Response::redirect(format!("https://{}/{}", &my_domain, &index).parse()?);
        }
        "/links" => {
            return Response::redirect(format!("https://{}/{}", &my_domain, &links).parse()?);
        }
        "/donate" => {
            return Response::redirect(format!("https://{}/{}", &my_domain, &donate).parse()?);
        }
        _ => {}
    }
    if matches!(req.method(), Method::Options) {
        let response = Response::empty()?;
        let mut header = Headers::new();
        header.set("Access-Control-Allow-Origin", "*")?;
        header.set(
            "Access-Control-Allow-Methods",
            "GET,POST,PUT,PATCH,TRACE,DELETE,HEAD,OPTIONS",
        )?;
        header.set("Access-Control-Allow-Headers", "Content-Type")?;
        header.set("Access-Control-Allow-Credentials", "True")?;
        header.set("Access-Control-Max-Age", "1728000")?;
        return Ok(response.with_headers(header));
    }
    if req.path() == "/api/v3/teV1" {
        return Response::ok("success");
    }
    let mut full_url = req.url()?;
    full_url.set_host(Some(&notion_domain))?;
    if (req.path().starts_with("/app") || req.path().starts_with("/mermaid"))
        && req.path().ends_with("js")
    {
        let request = Request::new_with_init(
            full_url.as_str(),
            RequestInit::new().with_method(req.method()),
        )?;
        if let Ok(mut o) = Fetch::Request(request).send().await {
            let body = o.bytes().await.unwrap_or_default();
            let body = String::from_utf8_lossy(&body).to_string();
            let new_body = body
                .replace(&my_domain, &notion_domain)
                .replace(&my_domain, &notion_domain);
            let response = Response::from_bytes(new_body.as_bytes().to_vec())?;
            let mut response_headers = Headers::new();
            response_headers.set("Content-Type", "application/x-javascript")?;
            return Ok(response.with_headers(response_headers));
        }
        Response::redirect(full_url)
    } else if req.path().starts_with("/api") {
        let mut headers = req.headers().clone();
        headers.set("Content-Type", "application/json;charset=UTF-8")?;
        headers.set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.163 Safari/537.36")?;
        headers.set("Access-Control-Allow-Origin", "*")?;
        let body = if req.path() != "/api/v3/getPublicPageData" {
            Some(JsValue::from_str(
                String::from_utf8_lossy(&req.bytes().await.unwrap_or_default()).as_ref(),
            ))
        } else {
            None
        };
        let request = Request::new_with_init(
            full_url.as_str(),
            RequestInit::new()
                .with_headers(headers)
                .with_body(body)
                .with_method(Method::Post),
        )?;
        if let Ok(response) = Fetch::Request(request).send().await {
            let mut response_headers = Headers::new();
            response_headers.delete("Content-Security-Policy")?;
            response_headers.delete("X-content-Security-Policy")?;
            response_headers.delete("Content-Security-Policy")?;
            response_headers.delete("X-Content-Security-Policy")?;
            response_headers.delete("Referrer-Policy")?;
            response_headers.delete("X-Permitted-Cross-Domain-Policies")?;
            response_headers.set("Access-Control-Allow-Origin", "*")?;
            response_headers.set("Timing-Allow-Origin", "*")?;
            return Ok(response.with_headers(response_headers));
        }
        return Response::redirect(full_url);
    } else {
        let headers = req.headers().clone();
        let request = Request::new_with_init(
            full_url.as_str(),
            RequestInit::new()
                .with_headers(headers)
                .with_method(req.method()),
        )?;
        return append_javascript(Fetch::Request(request).send().await?, my_domain).await;
    }
}

async fn append_javascript(mut response: Response, my_domain: String) -> Result<Response> {
    let body = response.bytes().await.unwrap_or_default();
    let new_response = Response::from_bytes(rewriter(body, my_domain))
        .unwrap()
        .with_headers(response.headers().clone())
        .with_status(response.status_code());
    Ok(new_response)
}

fn rewriter(html: Vec<u8>, my_domain: String) -> Vec<u8> {
    let mut output = vec![];
    let h = r#"
    <div>Powered by <a href="https://blog.kali-team.cn">Kali-Team</a></div>
      <script>
      localStorage.__console = true;
      window.CONFIG.domainBaseUrl = 'https://MY_DOMAIN';
      let redirected = false;
      const el = document.createElement('div');
      const waitFor = (...selectors) => new Promise(resolve => {
        const delay = 500;
        const f = () => {
            const elements = selectors.map(selector => document.querySelector(selector));
            if (elements.every(element => element != null)) {
                resolve(elements);
            } else {
                setTimeout(f, delay);
            }
        }
        f();
      });
      function remove_notion_page_content(){
        let scroll_bar = document.getElementsByClassName("notion-page-content");
        if (scroll_bar.length > 0){
          scroll_bar[0].style.paddingBottom = "0vh";
        }
        let iterable = [
          "html.notion-html body.notion-body div#notion-app div.notion-app-inner.notion-light-theme div div.notion-cursor-listener div main.notion-frame div.notion-scroller.vertical div.pseudoSelection div",
          "html.notion-html body.notion-body div#notion-app div.notion-app-inner.notion-light-theme div div.notion-cursor-listener div main.notion-frame div.notion-scroller.vertical div.pseudoSelection div div div",
          "html.notion-html body.notion-body div#notion-app div.notion-app-inner.notion-light-theme div div.notion-cursor-listener div main.notion-frame div.notion-scroller.vertical div.pseudoSelection div div div div img",
          "html.notion-html body.notion-body div#notion-app div.notion-app-inner.notion-light-theme div div.notion-cursor-listener div main.notion-frame div.notion-scroller.vertical div.pseudoSelection div div div div img"];
        for (const entry of iterable) {
          let pseudo_selection = document.querySelector(entry);
          if (pseudo_selection !== null){
            pseudo_selection.style.height = "8vh";
          }
        }
        let notion_page_controls = document.querySelector("html.notion-html body.notion-body div#notion-app div.notion-app-inner.notion-light-theme div.notion-cursor-listener div div.notion-frame div.notion-scroller.vertical div div div div div.pseudoSelection div.notion-page-controls");
        if (notion_page_controls !== null){
          notion_page_controls.remove()
        }
      }
      remove_notion_page_content();
            function onDark() {
        el.innerHTML = '<div title="Change to Light Mode" style="margin-top: 8px; padding-left: 8px; padding-right: 8px; margin-left: 8px; margin-right: 8px; min-width: 0px;"><svg id="moon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"></path></svg></div></div>';
        document.body.classList.add('dark');
        __console.environment.ThemeStore.setState({ mode: 'dark' });
      };
      function onLight() {
        el.innerHTML = '<div title="Change to Dark Mode" style="margin-top: 8px; padding-left: 8px; padding-right: 8px; margin-left: 8px; margin-right: 8px; min-width: 0px;"><svg id="sun" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg></div></div>';
        document.body.classList.remove('dark');
        __console.environment.ThemeStore.setState({ mode: 'light' });
      }
      function toggle() {
        if (document.body.classList.contains('dark')) {
          onLight();
        } else {
          onDark();
        }
      }
      function addDarkModeButton(device) {
        const nav = device === 'web' ? document.querySelector('.notion-topbar').firstChild : document.querySelector('.notion-topbar-mobile');
        el.className = 'toggle-mode';
        el.addEventListener('click', toggle);
        nav.appendChild(el);
        onLight();
      }
      // Notion 浮动 TOC
      function TOC() {
        waitFor('.notion-table_of_contents-block').then(([el]) => {
          const toc = document.querySelector('.notion-table_of_contents-block');
          if (toc) {
              const toc_p = toc.parentElement;
              if (!toc_p.classList.contains('notion-column-block')) {
                  return;
              }
              toc_p.style.position = 'sticky';
              toc_p.style.top = '0';
              toc_p.style.overflowY = 'scroll';
              toc_p.style.maxHeight = '50vh';
          }
      });
      }
      const observer = new MutationObserver(function() {
        remove_notion_page_content();
        TOC();
        if (redirected) return;
        const nav = document.querySelector('.notion-topbar');
        const mobileNav = document.querySelector('.notion-topbar-mobile');
        if (nav && nav.firstChild && nav.firstChild.firstChild
          || mobileNav && mobileNav.firstChild) {
          redirected = true;
          addDarkModeButton(nav ? 'web' : 'mobile');
        }
      });
      observer.observe(document.querySelector('#notion-app'), {
        childList: true,
        subtree: true,
      });
      remove_notion_page_content();
    </script>"#.replace("MY_DOMAIN", &my_domain);
    let head = r#"
      <style>
      div.notion-topbar > div > div:nth-child(3) { display: none !important; }
      // div.notion-topbar > div > div:nth-child(4) { display: none !important; }
      div.notion-topbar > div > div:nth-child(5) { display: none !important; }
      div.notion-topbar > div > div:nth-child(6) { display: none !important; }
      div.notion-topbar > div > div:nth-child(7) { display: none !important; }
      div.notion-topbar > div > div:nth-child(8) { display: none !important; }
      div.notion-topbar-mobile > div:nth-child(3) { display: none !important; }
      div.notion-topbar-mobile > div:nth-child(4) { display: none !important; }
      div.notion-topbar > div > div:nth-child(1n).toggle-mode { display: block !important; }
      div.notion-topbar-mobile > div:nth-child(1n).toggle-mode { display: block !important; }
      </style>
    "#;
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("body", |el| {
                    el.append(&h, ContentType::Html);
                    Ok(())
                }),
                element!("head", |el| {
                    el.append(head, ContentType::Html);
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );
    rewriter.write(&html).unwrap();
    rewriter.end().unwrap();
    output
}

#[cfg(test)]
mod tests {
    use crate::rewriter;

    #[test]
    fn it_works() {
        let h = r#"<body class="notion-body"><a>ss</a></body>"#.to_string();
        rewriter(h.as_bytes().to_vec(), "blog.kali-team.cn".to_string());
    }
}
