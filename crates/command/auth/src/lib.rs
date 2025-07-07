mod error;
pub use error::*;
use lib_auth::{delete_token, set_token};
use log::{error, warn};
use tiny_http::{Header, Request, Response, Server, StatusCode};

const SELF_ADDR: &str = "0.0.0.0:8182";
const SELF_URL: &str = "http://0.0.0.0:8182";

pub fn auth(delete: bool) -> Result<()> {
    if delete {
        delete_token()?;
        return Ok(());
    }

    let server = Server::http(SELF_ADDR).map_err(Error::server_creation)?;

    eprintln!("Open {SELF_URL} in your browser and follow the instructions");
    // non-fatal error
    if let Err(_) = open::that_detached(SELF_URL) {
        warn!("Unable to automatically open browser, follow the link yourself")
    }

    for request in server.incoming_requests() {
        match request.url() {
            "/" => handle_main_page(request)?,
            "/save_token" => match handle_token(request) {
                Err(e) => error!("unable to save token: {e}"),
                Ok(_) => break,
            },
            _ => handle_unknown_res(request)?,
        };
    }
    eprintln!("Token successfully saved!");
    Ok(())
}

fn handle_main_page(request: Request) -> Result<()> {
    let main_page_html = include_str!("../res/index.html");
    let content_type_header =
        Header::from_bytes(b"Content-Type", b"text/html; charset=utf-8").expect("correct header");
    request.respond(
        Response::from_string(main_page_html)
            .with_status_code(200)
            .with_header(content_type_header),
    )?;
    Ok(())
}

fn handle_unknown_res(request: Request) -> Result<()> {
    request.respond(Response::new_empty(StatusCode(404)))?;
    Ok(())
}

fn handle_token(request: Request) -> Result<()> {
    // Extract token first, so we don't hold a borrow on request
    let token = {
        let headers = request.headers();
        headers
            .iter()
            .find(|it| it.field.equiv("X-Figma-Token"))
            .ok_or_else(|| Error::Custom("X-Figma-Token header is absent".to_string()))?
            .value
            .to_string()
    };

    match set_token(&token) {
        Ok(_) => request.respond(Response::new_empty(StatusCode(200)))?,
        Err(e) => {
            request.respond(Response::new_empty(StatusCode(503)))?;
            return Err(e.into());
        }
    }
    Ok(())
}
