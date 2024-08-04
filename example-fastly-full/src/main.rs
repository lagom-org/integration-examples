use fastly::http::header;
use fastly::http::StatusCode;
use fastly::{mime, Error, Request, Response};
use simple_cookie::{decode_cookie, encode_cookie, parse_cookie_header_value};
use serde_json;

macro_rules! bail {
    ($msg:expr) => {
        eprintln!("Error: {}", $msg);
        return Err(Error::msg($msg.to_string()))
    };
}

const COOKIE_KEY: [u8; 32] = [
    0x1a, 0xd1, 0x97, 0xfa, 0xbe, 0x78, 0xce, 0x3e, 0x71, 0xc2, 0x72, 0xc6, 0xd7, 0x89, 0x83, 0x6d,
    0x0e, 0xa4, 0xef, 0xfa, 0x92, 0x9d, 0x24, 0x98, 0x5f, 0xe9, 0x05, 0x8a, 0x61, 0x60, 0x34, 0xc0,
];

const SECRET :&str = "04b3623a9f7c553c272e3d3def949e3ac781ff8145ee87f22defc7616dae3f86a165547706f5e381a4d70070b234109fdd8daf80167e673ceda05503eb0d3123";
const PROVIDER_ID: &str = "test-provider";

#[allow(non_snake_case)]
#[derive(serde::Deserialize, serde::Serialize)]
struct Article {
    Timestamp: String,
    Callback: String,
    Amount: i64,
}

fn lagom_verify_pay(req: &Request, page: &str, amount: i64) -> Result<(), Error> {
    //  extract callback params from URL, decode and parse
    let lguid = req.get_query_parameter("lguid").unwrap_or_default();
    let lgts = req.get_query_parameter("lgts").unwrap_or_default();
    let lgsig = req.get_query_parameter("lgsig").unwrap_or_default();
    let lgid = req.get_query_parameter("lgid").unwrap_or_default();
    let lgamt = req.get_query_parameter("lgamt").unwrap_or_default();

    // verify timestamp freshness
    if chrono::Utc::now().timestamp() > lgts.parse::<i64>()? + 10 {
        bail!("This link has expired");
    }

    // check amount and page
    if lgamt.parse::<i64>()? != amount || req.get_path() != page {
        bail!("This link is not valid");
    }

    // verify signature with pre shared secret
    let verif = lguid.to_string() + lgid + lgts + page + lgamt;
    let good = hmac_sha256::HMAC::mac(&verif.as_bytes(), SECRET);
    if lgsig != hex::encode(good) {
        bail!("This link is not valid");
    }

    Ok(())
}

fn lagom_verify_login(req: &Request) -> Result<Response, Error> {
    //  extract callback params from URL, decode and parse
    let lguid = req.get_query_parameter("lguid").unwrap_or_default();
    let lgts = req.get_query_parameter("lgts").unwrap_or_default();
    let lgsig = req.get_query_parameter("lgsig").unwrap_or_default();
    let lgin = req.get_query_parameter("lgin").unwrap_or_default();

    // verify timestamp freshness
    if chrono::Utc::now().timestamp() > lgts.parse::<i64>()? + 10 {
        bail!("This link has expired");
    }

    // verify signature with pre shared secret
    let verif = lgin.to_string() + lguid + lgts + req.get_path();
    let good = hmac_sha256::HMAC::mac(&verif.as_bytes(), SECRET);
    if lgsig != hex::encode(good) {
        bail!("This link is not valid");
    }

    // set cookie with uid and initials, and redirect
    let mut resp = Response::from_status(StatusCode::FOUND);
    resp.set_header(header::LOCATION, "/account");
    let encoded_uid = encode_cookie(
        &COOKIE_KEY,
        "account_id",
        lguid.as_bytes(),
    );
    let cookie_hdr = format!(
        "session={}; Max-Age=604800; Secure; SameSite=Strict",
        encoded_uid
    );
    let initials_cookie = format!(
        "initials={}; Max-Age=604800; Secure; SameSite=Strict",
        lgin
    );

    resp.set_header(header::SET_COOKIE, cookie_hdr);
    resp.append_header(header::SET_COOKIE, initials_cookie);
    Ok(resp)
}

fn verify_cookie(req: &Request) -> Result<String, Error> {
    let header = req
        .get_header(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    if header.is_empty() {
        bail!("no cookie");
    }
    let (_, encoded_val) = parse_cookie_header_value(header.as_bytes())
        .find(|(name, _value)| *name == "session")
        .unwrap_or_default();
    let uid = decode_cookie(&COOKIE_KEY, "account_id", encoded_val).unwrap_or_default();
    let uid = std::str::from_utf8(&uid)?.to_string();
    return Ok(uid)
}

fn list_paid(uid: &str) -> Result<Vec<Article>, Error> {
    let sig = hmac_sha256::HMAC::mac(uid, SECRET);
    let url = format!(
        "http://127.0.0.1:3000/provider/listPaid/{}/{}/{}",
        PROVIDER_ID,
        uid,
        hex::encode(sig)
    );
    let req = Request::post(url);
    let mut ret = req.send("apilocal")?;
    let body = ret.get_body_prefix_str_mut(300);
    let retstr = body.as_str();
    let articles: Vec<Article> = serde_json::from_str(retstr)?;
    Ok(articles)
}

fn is_paid(article_path: &str, paid: &Vec<Article>) -> bool {
    paid.iter().find(|&x| x.Callback == article_path).is_some()
}

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let favicon = include_bytes!("../../public/favicon.ico");
    let index = include_str!("../../public/index.html").to_string();
    let account = include_str!("../../public/account.html").to_string();
    let article0 = include_str!("../../public/article0.html").to_string();
    let article1 = include_str!("../../public/article1.html").to_string();
    let fullarticle0 = include_str!("../../public/full/article0.html").to_string();
    let fullarticle1 = include_str!("../../public/full/article1.html").to_string();

    let path = req.get_path();

    // shortcuts
    if path == "/favicon.ico" {
        return Ok(Response::from_status(StatusCode::OK).with_content_type(mime::TEXT_HTML_UTF_8).with_body(favicon.to_vec()));
    }

    if path == "/logout" {
        let mut resp = Response::from_status(StatusCode::FOUND);
        resp.set_header(header::LOCATION, "/");
        resp.set_header(header::SET_COOKIE, "session=; Max-Age=0; Secure; SameSite=Strict");
        resp.append_header(header::SET_COOKIE, "initials=; Max-Age=0; Secure; SameSite=Strict");
        return Ok(resp);
    }

    // login callback response cookie if login cb is valid
    if path == "/account_cb" {
        let resp = lagom_verify_login(&req)?;
        return Ok(resp);
    }

    // check if we're logged in
    let uid = verify_cookie(&req).ok();

    // check paid pages here
    let mut paid = Vec::new();
    if let Some(uid) = uid.as_ref() {
        paid = list_paid(uid)?;
    }

    if path == "/listPaid" {
        if uid.is_none() {
            bail!("Not logged in");
        }
        let paid_json = serde_json::to_string(&paid)?;
        return Ok(Response::from_status(StatusCode::OK).with_content_type(mime::TEXT_HTML_UTF_8).with_body(paid_json));
    }

    // account page
    if path == "/account" && uid.is_some() {
        return Ok(Response::from_status(StatusCode::OK).with_content_type(mime::TEXT_HTML_UTF_8).with_body(account.as_bytes()));
    }

    // article pages
    let mut body = index.as_bytes();

    if path == "/article0.html" {
        if lagom_verify_pay(&req, "/article0.html", 100).is_ok() || is_paid("/article0.html", &paid) {
            body = fullarticle0.as_bytes()
        } else {
            body = article0.as_bytes()
        }
    }

    if path == "/article1.html" {
        if lagom_verify_pay(&req, "/article1.html", 100).is_ok() || is_paid("/article1.html", &paid) {
            body = fullarticle1.as_bytes()
        } else {
            body = article1.as_bytes()
        }
    }

    Ok(Response::from_status(StatusCode::OK).with_content_type(mime::TEXT_HTML_UTF_8).with_body(body))
}
