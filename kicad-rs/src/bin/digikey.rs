use clap::{App, Arg};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Method, Request, Response, Server};
use kicad_rs::error::DynamicResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::mpsc::{self, Receiver, Sender};

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
            Arg::with_name("CLIENT_SECRET")
                .help("Digi-Key App Client Secret")
                .required(true),
        )
        .arg(
            Arg::with_name("REDIRECT_URI")
                .help("OAuth 2.0 Callback URL")
                .default_value("http://localhost:8080"),
        )
        .get_matches();

    // Prompt for user to login to Digi-Key and authorize this application
    let auth_code = get_auth_code(
        matches.value_of("CLIENT_ID").unwrap(),
        matches.value_of("REDIRECT_URI").unwrap(),
    )
    .await
    .unwrap();

    // Use received authorization code to request OAuth tokens
    let tokens = get_tokens(
        matches.value_of("CLIENT_ID").unwrap(),
        matches.value_of("CLIENT_SECRET").unwrap(),
        matches.value_of("REDIRECT_URI").unwrap(),
        &auth_code,
    )
    .await
    .unwrap();

    println!("Received tokens: {:#?}", tokens);

    Ok(())
}

/// Start a local http server for receiving auth code over a redirect
async fn code_rx_task(tx: Sender<Result<String, ()>>) {
    let addr: SocketAddr = "127.0.0.1:8080"
        .parse()
        .expect("Could not parse socket address");
    let make_service = make_service_fn(move |_| {
        let tx = tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let tx = tx.clone();
                async move {
                    if req.method() == &Method::GET && req.uri().path() == "/" {
                        let params = get_query_params(&req);
                        if let Some(code) = params.get("code") {
                            tx.send(Ok(code.clone())).await.unwrap();
                        };
                    }

                    Ok::<_, Error>(Response::new(Body::from("")))
                }
            }))
        }
    });

    println!("Listening on: {:?}", addr);
    let server = Server::bind(&addr).serve(make_service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn get_auth_code(client_id: &str, redirect_uri: &str) -> DynamicResult<String> {
    // start local http server for receiving auth code over redirect
    let (tx, mut rx) = mpsc::channel::<Result<String, ()>>(1);
    let code_rx_task_handle = tokio::spawn(code_rx_task(tx));

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

    let code = rx.recv().await.unwrap().unwrap();

    Ok(code)
}

/// Response from digikey Digi-Key API endpoint: https://sandbox-api.digikey.com/v1/oauth2/token
#[derive(Debug, Serialize, Deserialize)]
struct OAuthTokens {
    access_token: String,
    refresh_token: String,
    expires_in: usize,
    refresh_token_expires_in: usize,
    token_type: String,
}

/// Get access and refresh tokens from the token API endpoint
async fn get_tokens(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    auth_code: &str,
) -> DynamicResult<OAuthTokens> {
    let token_endpoint = "https://sandbox-api.digikey.com/v1/oauth2/token";
    let request = Client::new().post(token_endpoint).form(&[
        ("code", auth_code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ]);

    let response = request.send().await?;
    let response_text = response.text().await?;
    let tokens = serde_json::from_str(&response_text)?;

    Ok(tokens)
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
