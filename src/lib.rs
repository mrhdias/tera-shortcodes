//
// shortcode module
//

use tera::{Result, Function};
use std::collections::HashMap;
use once_cell::sync::Lazy;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| reqwest::Client::new());

// const ROBOTS_TXT: &'static str = "Link for Robots (No JavaScript)";

/// A struct that manages shortcode functions for use in Tera templates.
/// 
/// # Fields
/// 
/// - `functions`: A `HashMap` where the key is the shortcode display name (a `String`), and the value is a function pointer that takes a reference to a `HashMap` of arguments and returns a `String` representing the generated content.
pub struct Shortcodes {
    pub functions: HashMap<String, fn(&HashMap<String, tera::Value>) -> String>,
}

impl Shortcodes {

    /// Creates a new `Shortcodes` instance with an empty set of registered functions.
    /// 
    /// # Returns
    /// 
    /// A `Shortcodes` struct with an empty `functions` map.
    pub fn new() -> Self {
        Shortcodes {
            functions: HashMap::new(),
        }
    }

    /// Registers a new shortcode function in the `Shortcodes` struct.
    /// 
    /// # Parameters
    /// 
    /// - `display`: The shortcode display name as a `&str`, which will be used as the key in the `functions` map.
    /// - `shortcode_fn`: A function pointer that takes a `HashMap` of arguments and returns a `String`.
    /// 
    /// # Returns
    /// 
    /// An updated instance of `Shortcodes` with the newly registered shortcode function.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use tera_shortcodes::Shortcodes;
    /// 
    /// let shortcodes = Shortcodes::new().register("example", |args| {
    ///     "Shortcode output".to_string()
    /// });
    /// ```
    pub fn register(mut self,
        display: &str,
        shortcode_fn: fn(&HashMap<String, tera::Value>) -> String,
    ) -> Self {
        self.functions.insert(display.to_owned(), shortcode_fn);
        self
    }

}

impl Function for Shortcodes {

    /// Invokes a registered shortcode function by its display name.
    /// 
    /// # Parameters
    /// 
    /// - `args`: A reference to a `HashMap` containing the arguments passed to the shortcode function.
    /// 
    /// # Returns
    /// 
    /// A `Result<tera::Value>` that contains the generated content as a `String` or an error message if the display name is missing or unknown.
    /// 
    /// # Error Handling
    /// 
    /// - If the `display` attribute is missing, it returns an error message `"Missing display attribute"`.
    /// - If no function is registered for the given display name, it returns an error message `"Unknown shortcode display name: <display>"`.
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

/// Generates a JavaScript snippet that asynchronously fetches data from a URL using either the GET
/// or POST HTTP method and injects the response into the DOM. The function also provides fallback
/// content for crawlers/robots that do not support JavaScript. If the response has JavaScript code
/// like <script>console.log('test');</script>, it will be executable.
///
/// # Parameters
///
/// - `url`: A string slice containing the URL to which the HTTP request will be made.
/// - `method`: An optional HTTP method, either `GET` or `POST`. Defaults to `GET` if `None` is provided.
/// - `json_body`: An optional JSON string for the request body when using the `POST` method. Defaults to
///   an empty JSON object (`{}`) if `None` is provided. Ignored if the method is `GET`.
/// - `alt`: An optional alternative content to display in a `<noscript>` block for crawlers/robots without JavaScript. 
///   This is only used if the method is `GET`. Defaults to `None`.
///
/// # Returns
///
/// A `String` containing the generated JavaScript code that can be inserted into an HTML page. The script:
/// - Sends an asynchronous `fetch` request to the specified URL.
/// - If the response is successful, it injects the response content into the DOM.
/// - If the request fails, it logs an error message to the browser's console.
/// - If an invalid HTTP method is passed (anything other than `GET` or `POST`), an HTML `<output>` element
///   with an error message is returned instead of the JavaScript code.
///
/// If the `GET` method is used and `alt` is provided, the function also includes a `<noscript>` fallback
/// to display a link in case JavaScript is disabled or not supported.
///
/// # Example
///
/// ```rust
/// use tera_shortcodes::fetch_shortcode_js;
/// 
/// let js_code = fetch_shortcode_js(
///     "https://example.com/data", 
///     Some("POST"), 
///     Some("{\"key\": \"value\"}"), 
///     Some("No JavaScript fallback")
/// );
///
/// println!("{}", js_code);
/// ```
///
/// This will generate JavaScript code to make a `POST` request to `https://example.com/data` with the
/// provided JSON body, and include a fallback for users without JavaScript.
///
/// # Error Handling
///
/// - If an unsupported HTTP method is provided (anything other than `GET` or `POST`), the function will
///   return an HTML `<output>` element with an error message specifying the invalid method.
pub fn fetch_shortcode_js(
    url: &str,
    method: Option<&str>,
    json_body: Option<&str>,
    alt: Option<&str>,
) -> String {

    let method = method.unwrap_or("GET");
    let json_body = json_body.unwrap_or("{}");

    let fetch_js = match method.to_lowercase().as_str() {
        "get" => format!(r#"const response = await fetch("{}");"#, url),
        "post" => format!(r#"
const request = new Request("{}", {{
    headers: (() => {{
        const headers = new Headers();
        headers.append("Content-Type", "application/json");
        return headers;
    }})(),
    method: "POST",
    body: JSON.stringify({}),
}});
const response = await fetch(request);"#, url, json_body),
        _ => return format!(r#"<output style="background-color:#f44336;color:#fff;padding:6px;">
Invalid method {} for url {} (only GET and POST methods available)
</output>"#, method, url),
    };

    // reScript function ia a trick to make the Javascript code work when inserted.
    // Replace it with another clone element script.
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
    function reScript(helper) {{
        for (const node of helper.childNodes) {{
            if (node.hasChildNodes()) {{
                reScript(node);
            }}
            if (node.nodeName === 'SCRIPT') {{
                const script = document.createElement('script');
                script.type = "text/javascript";
                script.textContent = node.textContent;
                node.replaceWith(script);
            }}
        }}
    }}
    (async () => {{
        const currentScript = document.currentScript;
        const content = await fetchShortcodeData();
        // console.log(content);
        const helper = document.createElement('div');
        helper.id = 'helper';
        helper.innerHTML = content;
        reScript(helper);
        currentScript.after(...helper.childNodes);
        currentScript.remove();
    }})();
}})();
</script>"#,
    fetch_js);

    if method.to_lowercase().as_str() == "get" && alt.is_some() {
        let alt = alt.unwrap();
        js_code.to_string() + &format!(r#"<noscript><a href="{}">{}</a></noscript>"#, url, alt)
    } else {
        js_code
    }
}

/// Sends an HTTP request to the provided URL using either the `GET` or `POST` method and returns the response as a String.
/// This function handles asynchronous requests but executes them in a synchronous context using Tokio function `block_in_place`.
/// Note: This function is slow. For better performance, consider using the fetch_shortcode_js function instead.
///
/// # Parameters
///
/// - `url`: A string slice that holds the URL to which the HTTP request will be sent.
/// - `method`: An optional HTTP method, either `GET` or `POST`. Defaults to `GET` if `None` is provided.
/// - `json_body`: An optional JSON string to be used as the request body for `POST` requests. 
///   Defaults to an empty JSON object (`{}`) if `None` is provided. This parameter is ignored for `GET` requests.
///
/// # Returns
///
/// A `String` containing either the response body from the server or an error message in case
/// of failure. 
/// - If the HTTP request succeeds and returns a valid response, the body of the response is returned as a `String`.
/// - If the HTTP request fails (due to network errors, invalid URLs, or server errors), a descriptive error message is returned.
///
/// # Error Handling
///
/// - If an invalid HTTP method is provided, the function returns `"Invalid method: <method>"`.
/// - If the request fails, either due to network issues or an unsuccessful HTTP status, the function returns 
///   an error message like `"Request failed with status: <status>"` or `"Request error: <error>"`.
///
/// # Blocking and Asynchronous Execution
///
/// This function uses `tokio::task::block_in_place` to run the asynchronous request synchronously.
/// This allows the function to be used in synchronous contexts while still performing asynchronous
/// operations under the hood.
///
/// # Example
///
/// ```rust
/// use tera_shortcodes::fetch_shortcode;
/// 
/// #[tokio::main]
/// async fn main() {
///     let response = fetch_shortcode(
///         "https://example.com/api", 
///         Some("POST"), 
///         Some(r#"{"key": "value"}"#)
///     );
///
///     println!("Response: {}", response);
/// }
/// ```
///
/// This will perform a `POST` request to `https://example.com/api` with the given JSON body and print the response.
pub fn fetch_shortcode(
    url: &str,
    method: Option<&str>,
    json_body: Option<&str>,
) -> String {

    let method = method.unwrap_or("GET");
    let json_body = json_body.unwrap_or("{}");

    let data_to_route = async {
        let response = match method.to_lowercase().as_str() {
            "get" => CLIENT.get(url)
                .send()
                .await,
            "post" => CLIENT.post(url)
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