# tera-shortcodes
Tera Shortcodes in Rust: A WordPress-Like Implementation

The goal of this library is to bring the functionality of WordPress shortcodes to Tera template engine, enabling them to be inserted into templates to display content or functionality provided by the shortcode.

```html
{{ shortcode(display="myshortcode", foo="bar", bar="bing") | safe }}
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
{{ shortcode(display="my_shortcode", foo="bar", bar="bing") | safe }}
```

```rust
use tera_shortcodes;

let shortcodes = tera_shortcodes::Shortcodes::new()
  .register("my_shortcode", |args| -> String {

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
        
    tera_shortcodes::fetch_shortcode_js(
      &url,
      Some("post"),
      Some(&json_body)
    )
  })
  .register("another_shortcode", another_shortcode_fn);

let mut tera = Tera::new("templates/**/*").unwrap();

// Register the custom function
tera.register_function("shortcode", shortcodes);
```

## Benchemarks

Fetch shortcode in Rust with `block_in_place` to run the async function.
```sh
wrk -t12 -c400 -d30s http://127.0.0.1:8080/test
Running 30s test @ http://127.0.0.1:8080/test
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.52s   339.42ms   2.00s    66.52%
    Req/Sec    18.99     13.10    80.00     77.55%
  5824 requests in 30.09s, 1.79MB read
  Socket errors: connect 0, read 0, write 0, timeout 2479
Requests/sec:    193.55
Transfer/sec:     61.05KB
```
Fetch a shortcode using an `auxiliary function` in JavaScript.
```sh
wrk -t12 -c400 -d30s http://127.0.0.1:8080/test
Running 30s test @ http://127.0.0.1:8080/test
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    47.65ms   19.24ms 135.45ms   68.79%
    Req/Sec   696.16     57.63     2.03k    78.89%
  249698 requests in 30.07s, 230.27MB read
Requests/sec:   8303.94
Transfer/sec:      7.66MB
```