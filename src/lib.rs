//
// shortcode module
//

use tera::{Result, Function};
use std::{collections::HashMap, path::PathBuf};

pub struct Shortcodes {
    pub list: HashMap<String, fn(&HashMap<String, tera::Value>) -> String>,
    pub cache_dir: PathBuf,
}

impl Shortcodes {

    fn cache_hash(&self,
        args: &HashMap<String, tera::Value>,
    ) -> String {
        let mut words_list = vec![];
        for (key, value) in args {
            if key == "rotate" {
                continue;
            }
            words_list.push(key.clone());
            let value = value.as_str()
                .unwrap()
                .trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                words_list.push(value.to_string());
            }
        }
        words_list.sort();

        // println!("words: {:?}", words_list);

        // Generate a unique hash from the words list
        let digest = md5::compute(&words_list.join("-"));
        // Convert the digest to a hexadecimal string
        format!("{:x}", digest)
    }

    fn rotate_cached_file(&self,
        file_ref: &str,
        rotate: i32,
    ) -> bool {

        let file_path = self.cache_dir.join(format!("{}.html", file_ref));
        if !file_path.exists() {
            return true;
        }

        if rotate <= 0 {
            return false;
        }

        // Get file metadata
        let metadata = std::fs::metadata(&file_path)
            .expect("Failed to get file metadata");

        // Get file creation time
        let creation_time = metadata.created()
            .expect("Failed to get file creation time");

        // Get the current time
        let now = std::time::SystemTime::now();

        // Convert SystemTime to DateTime<Utc>
        let creation_time: chrono::DateTime<chrono::Utc> = chrono::DateTime::from(creation_time);
        let now: chrono::DateTime<chrono::Utc> = chrono::DateTime::from(now);

        // Calculate the difference
        let duration = now.signed_duration_since(creation_time);
        if duration.num_seconds() > rotate as i64 {
            eprintln!("Remove cache file: {}", file_path.to_string_lossy());
            return match std::fs::remove_file(file_path.as_path()) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("Cannot remove old cache file: {}", e);
                    false
                },
            };
        }
        false
    }

    fn fragment_from_cache(&self,
        file_ref: &str,
    ) -> std::result::Result<String, std::io::Error> {

        let file_path = self.cache_dir.join(format!("{}.html", file_ref));
        if file_path.exists() {
            std::fs::read_to_string(file_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cache file not found: {}", file_ref),
            ))
        }
    }

    fn fragment_to_cache(&self,
        file_ref: &str,
        rendered: &str,
    ) -> std::result::Result<(), std::io::Error> {
    
        let file_path = self.cache_dir.join(format!("{}.html", file_ref));
        // Check if the file already exists and give an error if it does
        if file_path.exists() {
            // eprintln!("Cache file already exists: {:?}", file_path);
            return Ok(());
        }
        std::fs::write(file_path, &rendered)
    }

    pub fn new(cache_dir: PathBuf, purge_cache: bool) -> Self {

        // Check if the directory exists, and create it if it doesn't
        if cache_dir.exists() {
            if purge_cache {
                for entry in std::fs::read_dir(&cache_dir).unwrap() {
                    let entry = entry.unwrap();
                    let file_type = entry.file_type().unwrap();
        
                    // If the entry is a file, remove it
                    if file_type.is_file() {
                        std::fs::remove_file(entry.path())
                            .expect("Removing file failed");
                    }
                }
            }
        } else {
            std::fs::create_dir_all(&cache_dir)
                .expect("Cannot cache create directory");
        }

        Shortcodes {
            list: HashMap::new(),
            cache_dir,
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

        let cache_file_hash = &self.cache_hash(args);

        // println!("Reference Hash: {}", cache_file_hash);

        let rotate = match args.get("rotate") {
            Some(value) => value.as_str()
                .unwrap()
                .trim_matches(|c| c == '"' || c == '\'')
                .parse()
                .unwrap_or_else(|_| 0),
            None => 0,
        };

        if !self.rotate_cached_file(cache_file_hash, rotate) {
            // eprintln!("Fetch cache file: {}", cache_file_ref);
            match self.fragment_from_cache(cache_file_hash) {
                Ok(content) => return Ok(tera::Value::String(content)),
                Err(err) => eprintln!("Fetching cache: {}", err),
            }
        }

        let fragment = match self.list.get(display) {
            Some(shortcode_fn) => shortcode_fn(args),
            None => {
                return Ok(tera::Value::String(format!("Unknown shortcode display name: {}", display)))
            },
        };

        self.fragment_to_cache(cache_file_hash, &fragment)
            .expect("Error caching rendered content");

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

    format!(r#"<script>
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
    fetch_js)
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