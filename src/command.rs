use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl From<HttpMethod> for actix_web::http::Method {
    fn from(m: HttpMethod) -> Self {
        match m {
            HttpMethod::Get => actix_web::http::Method::GET,
            HttpMethod::Post => actix_web::http::Method::POST,
            HttpMethod::Put => actix_web::http::Method::PUT,
            HttpMethod::Patch => actix_web::http::Method::PATCH,
            HttpMethod::Delete => actix_web::http::Method::DELETE,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage endpoints
    #[command(alias = "ep")]
    Endpoint {
        #[command(subcommand)]
        action: EndpointAction,
    },
}

// TODO: later: add endpoints from json files, handle different methods and formats

#[derive(Subcommand, Debug)]
pub enum EndpointAction {
    /// Add a new endpoint
    #[command(aliases = ["a", "ad", "update", "u", "up"])]
    Add {
        #[arg(ignore_case = true)]
        method: HttpMethod,
        path: String,
        response: String,
    },
    /// Delete endpoint
    #[command(aliases = ["d", "del"])]
    Delete { method: HttpMethod, path: String },
    /// List all endpoints
    #[command(alias = "l")]
    List {
        #[arg(ignore_case = true)]
        method: Option<HttpMethod>,
    },
}
