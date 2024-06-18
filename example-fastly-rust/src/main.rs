//! Default Compute template program.

use fastly::http::StatusCode;
use fastly::{mime, Error, Request, Response};

fn err(s: &str) -> Result<(), Error>  {
    Err(Error::msg(s.to_string()))
}

const SECRET : &str = "04b3623a9f7c553c272e3d3def949e3ac781ff8145ee87f22defc7616dae3f86a165547706f5e381a4d70070b234109fdd8daf80167e673ceda05503eb0d3123";

fn lagom_verify(req: &Request, page: &str, amount: i64) -> Result<(), Error> {
    //  extract callback params from URL, decode and parse
    let lguid = req.get_query_parameter("lguid").unwrap_or_default();
    let lgts = req.get_query_parameter("lgts").unwrap_or_default();
    let lgsig = req.get_query_parameter("lgsig").unwrap_or_default();
    let lgid = req.get_query_parameter("lgid").unwrap_or_default();
    let lgamt = req.get_query_parameter("lgamt").unwrap_or_default();

    // verify timestamp freshness
    if chrono::Utc::now().timestamp() > lgts.parse::<i64>()? + 10 {
        return err("This link has expired");
    }

    // check amount and page
    if lgamt.parse::<i64>()? != amount || req.get_path() != page {
        return err("This link is not valid");
    }

    // verify signature with pre shared secret
    let verif = lguid.to_string() + lgid + lgts + page + lgamt;
    let good = hmac_sha256::HMAC::mac(&verif.as_bytes(), SECRET);
    if lgsig != hex::encode(good) {
        return err("This link is not valid");
    }

    Ok(())
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let article = include_str!("../../public/article.html").to_string();
    let full_article = include_str!("../../public/full/article.html").to_string();
    let favicon = include_bytes!("../../public/favicon.ico");

    let body = if req.get_path() == "/favicon.ico" {
        favicon
    } else if lagom_verify(&req, "/", 100).is_ok()  {
        full_article.as_bytes()
    } else {
        article.as_bytes()
    };

    Ok(Response::from_status(StatusCode::OK)
        .with_content_type(mime::TEXT_HTML_UTF_8)
        .with_body(body))
}
