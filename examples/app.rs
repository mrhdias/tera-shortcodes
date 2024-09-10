//
// Axum/Tera & Real Shortcodes in Rust: A WordPress-Like Implementation
//

use tera_shortcodes;

use axum::{
    extract::{Extension, Request, Json, Query},
    response::Html,
    routing::{get, post},
    Router,
    ServiceExt,
};

use serde::{Serialize, Deserialize};
use tera::{Tera, Context};
use std::collections::HashMap;

const ADDRESS: &str = "127.0.0.1:8080";

#[derive(Clone, Serialize)]
struct Product {
    id: u32,
    name: String,
    image_url: String,
    price: f64,
}

#[derive(Serialize, Deserialize)]
struct ProductsShortcode {
    limit: Option<i32>,
    orderby: Option<String>,
}

// {{ shortcode(display="products", limit=4) | safe }}
fn products_shortcode_fn(
    args: &HashMap<String, tera::Value>,
) -> String {

    let mut parameters = vec![];

    if let Some(limit) = args.get("limit") {
        parameters.push(format!("limit={}", limit.as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\'')));
    }

    if let Some(orderby) = args.get("orderby") {
        parameters.push(format!("orderby={}", orderby.as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\'')));
    }

    let url = format!("http://{}/products?{}", ADDRESS, parameters.join("&"));

    tera_shortcodes::fetch_shortcode_js(
        &url,
        Some("get"), 
        None,
    )

    // shortcodes::fetch_shortcode(
    //    url, 
    //    Some("get"), 
    //    None,
    // )
}

async fn products(
    Query(parameters): Query<ProductsShortcode>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {

    let limit = match parameters.limit {
        Some(limit) => limit,
        None => 4,
    };

    let orderby = match parameters.orderby {
        Some(ref orderby) => orderby.as_str(),
        None => "id",
    };

    let mut products = vec![
        Product {
            id: 1,
            name: "Lorem ipsum dolor".to_string(),
            image_url: "https://picsum.photos/210/300".to_string(),
            price: 39.99,
        },
        Product {
            id: 2,
            name: "Donec rutrum dui".to_string(),
            image_url: "https://picsum.photos/220/300".to_string(),
            price: 59.99,
        },
        Product {
            id: 3,
            name: "Mauris imperdiet massa".to_string(),
            image_url: "https://picsum.photos/230/300".to_string(),
            price: 29.99,
        },
        Product {
            id: 4,
            name: "Sed tristique tellus".to_string(),
            image_url: "https://picsum.photos/240/300".to_string(),
            price: 9.99,
        },
        Product {
            id: 5,
            name: "Vivamus tempus".to_string(),
            image_url: "https://picsum.photos/250/300".to_string(),
            price: 49.99,
        },
        Product {
            id: 6,
            name: "Aliquam rutrum viverra".to_string(),
            image_url: "https://picsum.photos/260/300".to_string(),
            price: 19.99,
        },
    ];

    match orderby {
        "name" => products.sort_by(|a, b| a.name.cmp(&b.name)),
        "price" => products.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
        _ => products.sort_by(|a, b| a.id.cmp(&b.id)),
    };

    // Convert the limit from i32 to usize
    let limit = std::cmp::min(limit as usize, products.len());
    let products_by_limit = products[0..limit].to_vec();

    let mut data = Context::new();
    data.insert("products", &products_by_limit);
    let rendered = tera.render("shortcodes/products.html", &data).unwrap();

    Html(rendered)
}

fn another_shortcode_fn(
    args: &HashMap<String, tera::Value>,
) -> String {
    let width = match args.get("width") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\''),
        None => "200",
    };
    let height = match args.get("height") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\''),
        None => "200",
    };
    let image_src = match args.get("image_src") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\''),
        None => "No image attribute specified",
    };

    format!(r#"<img src="{}" width="{}" height="{}">"#, image_src, width, height)
}

fn my_shortcode_fn(
    args: &HashMap<String, tera::Value>,
) -> String {

    let foo = match args.get("foo") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\''),
        None => "no foo",
    };
    let bar = match args.get("bar") {
        Some(value) => value
            .as_str()
            .unwrap()
            .trim_matches(|c| c == '"' || c == '\''),
        None => "no bar",
    };

    let json_body = serde_json::to_string(&DataTest {
        foo: foo.to_string(),
        bar: bar.to_string(),
    }).unwrap();

    let url = format!("http://{}/data", ADDRESS);

    tera_shortcodes::fetch_shortcode_js(
        &url,
        Some("post"),
        Some(&json_body)
    )
}

#[derive(Serialize, Deserialize)]
struct DataTest {
    foo: String,
    bar: String,
}

// Handler function that returns JSON content
async fn data(
    Json(payload): Json<DataTest>,
) -> Json<DataTest> {

    let data = DataTest {
        foo: format!("ok {}", payload.foo),
        bar: format!("ok {}", payload.bar),
    };

    // Return the JSON response
    Json(data)
}

async fn test(
    Extension(tera): Extension<Tera>,
) -> Html<String> {

    let context = Context::new();
    // Render the template with the context
    let rendered = tera
        .render("test_shortcode.html", &context)
        .unwrap();

    Html(rendered)
}

#[tokio::main]
async fn main() {

    let shortcodes = tera_shortcodes::Shortcodes::new()
        .register("my_shortcode", my_shortcode_fn)
        .register("another_shortcode", another_shortcode_fn)
        .register("products", products_shortcode_fn);

    let mut tera = Tera::new("examples/templates/**/*").unwrap();

    // Register the custom function
    tera.register_function("shortcode", shortcodes);

    // Build our application with a route
    let app = Router::new()
        .route("/", get(|| async {
            "Hello world!"
        }))
        .route("/test", get(test))
        .route("/data", post(data))
        .route("/products", get(products))
        .layer(Extension(tera));

    // Run the server
    let listener = tokio::net::TcpListener::bind(ADDRESS)
        .await
        .unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();
}