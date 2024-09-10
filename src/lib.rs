//
// shortcode module
//

use tera::{Result, Function};
use std::collections::HashMap;

pub struct Shortcodes {
    pub functions: HashMap<String, fn(&HashMap<String, tera::Value>) -> String>,
}

impl Shortcodes {

    pub fn register(mut self,
        display: &str,
        shortcode_fn: fn(&HashMap<String, tera::Value>) -> String,
    ) -> Self {
        self.functions.insert(display.to_owned(), shortcode_fn);
        self
    }

    pub fn new() -> Self {
        Shortcodes {
            functions: HashMap::new(),
        }
    }
}

impl Function for Shortcodes {

    fn call(&self,
        args: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value> {

        let display = match args.get("display") {
            Some(value) => value.as_str()
                .unwrap()
                .trim_matches(|c| c == '"' || c == '\''),
            None => return Ok(tera::Value::String("Missing display attribute".to_owned())),
        };

        let fragment = match self.functions.get(display) {
            Some(shortcode_fn) => shortcode_fn(args),
            None => {
                return Ok(tera::Value::String(format!("Unknown shortcode display name: {}", display)))
            },
        };

        Ok(tera::Value::String(fragment))

    }
}

pub fn fetch_shortcode_js(
    url: &str,
    method: Option<&str>,
    json_body: Option<&str>,
) -> String {

    let method = method.unwrap_or("GET");
    let json_body = json_body.unwrap_or("{}");

    let fetch_js = match method.to_lowercase().as_str() {
        "get" => format!(r#"const response = await fetch("{}");"#, url),
        "post" => format!(r#"
const request = new Request("{}", {{
    headers: (() => {{
        const myHeaders = new Headers();
        myHeaders.append("Content-Type", "application/json");
        return myHeaders;
    }})(),
    method: "POST",
    body: JSON.stringify({}),
}});
const response = await fetch(request);"#, url, json_body),
        _ => panic!("Invalid method: {}", method),
    };

    let js_code = format!(r#"<script>
(function () {{
    async function fetchShortcodeData() {{
        try {{
            {}
            if (!response.ok) {{
                throw new Error(`HTTP error! Status: ${{response.status}}`);
            }}
            return await response.text();
        }} catch (error) {{
            console.error("Fetch failed:", error);
            return "";
        }}
    }}
    (async () => {{
        const currentScript = document.currentScript;
        const content = await fetchShortcodeData();
        // console.log(content);
        currentScript.insertAdjacentHTML('beforebegin', content);
        currentScript.remove();
    }})();
}})();
</script>"#,
    fetch_js);

    if method.to_lowercase().as_str() == "get" {
        js_code.to_string() + &format!(r#"<noscript><a href="{}">Link for Robots (No JavaScript)</a></noscript>"#, url)
    } else {
        js_code
    }
}

// Auxiliary function to transfer data to route
pub fn fetch_shortcode(
    url: &str,
    method: Option<&str>,
    json_body: Option<&str>,
) -> String {

    let method = method.unwrap_or("GET");
    let json_body = json_body.unwrap_or("{}");

    let client = reqwest::Client::new();

    let data_to_route = async {
        let response = match method.to_lowercase().as_str() {
            "get" => client.get(url)
                .send()
                .await,
            "post" => client.post(url)
                .header("Content-Type", "application/json")
                .body(json_body.to_owned())
                .send()
                .await,
                _ => return format!("Invalid method: {}", method),
            };

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    res.text().await.unwrap_or_else(|_| "Failed to read response body".into())
                } else {
                    format!("Request failed with status: {}", res.status())
                }
            }
            Err(e) => format!("Request error: {}", e),
        }
    };

    // Use `block_in_place` to run the async function
    // within the blocking context
    tokio::task::block_in_place(||
        // We need to access the current runtime to
        // run the async function
        tokio::runtime::Handle::current()
            .block_on(data_to_route)
    )
}