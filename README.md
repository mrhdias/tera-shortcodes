# tera-shortcodes
Tera Shortcodes in Rust: A WordPress-Like Implementation

The goal of this library is to bring WordPress-style shortcodes to the Tera template engine, enabling their use in templates to display dynamic content or functionality. Two methods are available for rendering shortcode content: a slower one that utilizes Tokio's `block_in_place` function, and a faster one that leverages an auxiliary JavaScript function.

[![Rust](https://github.com/mrhdias/tera-shortcodes/actions/workflows/rust.yml/badge.svg)](https://github.com/mrhdias/tera-shortcodes/actions/workflows/rust.yml)

```html
{{ shortcode(display="myshortcode", foo="bar", bar="bing", jscaller="true") | safe }}
```

## How to perform a test?

An example is provided to demonstrate how a shortcode is implemented.
Clone the repository and edit the test template if you wish. In the products shortcode, you can change the limit and order.

```sh
git clone https://github.com/mrhdias/tera-shortcodes
cd tera-shortcodes
cargo build
nano -w examples/templates/test_shortcode.html
cargo run --example app
curl http://127.0.0.1:8080/test
```

## Usage

An example of shortcode:
```html
{{ shortcode(display="my_shortcode", foo="bar", bar="bing", jscaller="true") | safe }}
```

```rust
use tera_shortcodes;

let shortcodes = tera_shortcodes::Shortcodes::new()
  .register("my_shortcode", |args| -> String {

    let js_caller = match args.get("jscaller") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\'')
            .parse()
            .unwrap_or(false),
        None => false,
    };

    let foo = match args.get("foo") {
      Some(value) => value.as_str().unwrap()
        .trim_matches(|c| c == '"' || c == '\''),
      None => "no foo",
    };

    let bar = match args.get("bar") {
      Some(value) => value.as_str().unwrap()
        .trim_matches(|c| c == '"' || c == '\''),
      None => "no bar",
    };

    let json_body = serde_json::to_string(&DataTest {
      foo: foo.to_string(),
      bar: bar.to_string(),
    }).unwrap();
  
    // axum route that receives the data from the shortcode
    let url = format!("http://{}/data", ADDRESS);

    if js_caller {
      tera_shortcodes::fetch_shortcode_js(
        &url,
        Some("post"),
        Some(&json_body),
        None,
      )
    } else {
      tera_shortcodes::fetch_shortcode(
        &url,
        Some("post"),
        Some(&json_body),
      )
    }
  })
  .register("another_shortcode", another_shortcode_fn);

let mut tera = Tera::new("templates/**/*").unwrap();

// Register the custom function
tera.register_function("shortcode", shortcodes);
```

## Benchemarks

Fetch shortcode in Rust with `block_in_place` to run the async function.
```sh
Running 30s test @ http://127.0.0.1:8080/test
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   250.98ms   60.48ms 528.39ms   68.72%
    Req/Sec   131.94     31.32   260.00     68.57%
  47193 requests in 30.10s, 84.30MB read
Requests/sec:   1567.81
Transfer/sec:      2.80MB
```
Fetch a shortcode using an `auxiliary function` in JavaScript.
```sh
Running 30s test @ http://127.0.0.1:8080/test
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    57.46ms   22.86ms 162.02ms   69.30%
    Req/Sec   577.12     48.80     1.43k    73.41%
  207147 requests in 30.10s, 437.77MB read
Requests/sec:   6882.17
Transfer/sec:     14.54MB
```
