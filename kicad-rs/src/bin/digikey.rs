use clap::{App, Arg};
use kicad_rs::error::DynamicResult;
use reqwest::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::{Method, StatusCode};
use std::collections::HashMap;
use std::{convert::Infallible, net::SocketAddr};
use url::form_urlencoded;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> DynamicResult<()> {

    // Parse command line arguments
    let matches = App::new("DigiKey API Client Helper")
        .about("Helper for fetching OAuth 2.0 tokens from DigiKey authorization server")
        .author("Verneri Hirvonen (@chiplet), The Racklet Project")
        .version(VERSION.unwrap_or("unknown"))
        .version_short("v")
        .arg(
            Arg::with_name("CLIENT_ID")
                .help("Digi-Key App Client ID")
                .required(true),
        )
        .arg(
            Arg::with_name("REDIRECT_URI")
                .help("OAuth 2.0 Callback URL")
                .default_value("http://localhost"),
        )
        .get_matches();

    let code = get_auth_code(
        matches.value_of("CLIENT_ID").unwrap(),
        matches.value_of("REDIRECT_URI").unwrap(),
    )
    .await.unwrap();

    Ok(())
}

async fn get_auth_code(client_id: &str, redirect_uri: &str) -> DynamicResult<String> {
    // Start local http server for receiving auth code over a redirect
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_auth_code)) });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    // build authentication request url
    let auth_endpoint = "https://sandbox-api.digikey.com/v1/oauth2/authorize";
    let request = Client::new()
        .get(auth_endpoint)
        .query(&[
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
        ])
        .build()?;
    let auth_url = request.url();
    println!("Request url: {}", auth_url);

    webbrowser::open(auth_url.as_str())?;

    Ok(String::new())
}

/// Get access and refresh tokens from the token API endpoint
async fn get_tokens(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    auth_code: &str,
) -> DynamicResult<(String, String)> {
    let token_endpoint = "https://sandbox-api.digikey.com/v1/oauth2/token";
    let request = Client::new().post(token_endpoint).form(&[
        ("code", auth_code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ]);

    let resp = request.send().await?;
    println!("{:#?}", resp);
    println!("{:#?}", resp.text().await?);

    Ok((String::from("a"), String::from("b")))
}

fn get_query_params(request: &Request<Body>) -> HashMap<String, String> {
    request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new)
}

async fn handle_auth_code(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    
    let mut response = Response::new(Body::empty());
    match (request.method(), request.uri().path()) {
        (&Method::GET, "/") => {
            let params = get_query_params(&request);

            println!("Got a request!! {:#?}", request);
            println!("Query params: {:#?}", params);
        },
        _ => {
            *response.body_mut() = Body::from("Could not authenticate application");
        }
    }

    Ok(response)
}