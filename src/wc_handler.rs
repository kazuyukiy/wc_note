// use std::io::Read;
use std::net::TcpStream;
// mod href_connector;
mod http_request; // does not work
mod page;

// Initialization
// fn system_ini() is to be called at beginning of lib.rs wc_node()
// so it will be done once at the biginning
// not every Tcp connection
//
// Copy wc.js, wc.css to
pub fn system_ini() {

    // under construction
}

pub fn response(stream: &mut TcpStream, page_root_path: &str) -> Vec<u8> {
    let http_request = http_request::HttpRequest::new(stream);

    // println!("wc_handler.rs fn response cp0");

    let mut page = match page(page_root_path, &http_request) {
        Ok(v) => v,
        Err(e) => return e,
    };

    // println!("wc_handler.rs fn response cp1");

    let method = match http_request.method() {
        Some(v) => v,
        None => return http_404(),
    };

    if method == "GET" {
        return handle_get(&mut page);
    }

    if method == "POST" {
        return handle_post(page, &http_request);
    }

    // temp
    http_hello()
}

/// Return page::Page instance that is of http_request::HttpRequest.
fn page(
    page_root_path: &str,
    http_request: &http_request::HttpRequest,
) -> Result<page::Page, Vec<u8>> {
    let path = match http_request.path() {
        Some(v) => v,
        None => return Err(http_404()),
    };

    // println!("wc_handler.rs fn page cp0");

    let mut page = match page::Page::open(page_root_path, path) {
        Ok(v) => v,
        Err(_) => return Err(http_404()),
    };

    // println!("wc_handler.rs fn page cp1");

    // Prepare some data of page in case those data are required.
    if let Err(v) = page_prepare(&mut page, http_request) {
        return Err(v);
    }

    // println!("wc_handler.rs fn page cp2");

    Ok(page)
}

// page::Page may have some value inside of it.
// But not all values are set when its instance are created.
// fn page_prepare sets some values of page::Page depending on its requirement.
// In case of method is GET, neither page_json nor page.url are required.
//
fn page_prepare(
    page: &mut page::Page,
    http_request: &http_request::HttpRequest,
) -> Result<(), Vec<u8>> {
    let method = match http_request.method() {
        Some(v) => v,
        None => return Err(http_404()),
    };

    if method == "GET" {
        return Ok(());
    }

    // println!("wc_handler.rs fn page_prepare cp0");

    // case backup file
    // Not sure it is goot to check whether it is backup file.
    if page.name_end_num() {
        return Err(http_400());
    }

    // page.json_set();
    page.contents_set();

    // println!("wc_handler.rs fn page_prepare cp1");

    // println!(
    //     "wc_handler.rs fn page_prepare page.contents().rev():{:?}",
    //     page.contents().unwrap().rev()
    // );
    // page.contents().unwrap().rev();

    // println!("wc_handler.rs fn page_prepare cp2");

    // If rev() contains some value,
    // it means the file contains json data properly.
    // If not, no further processes
    if page
        .contents()
        .map(|contents| contents.rev()) // rev(): Option<u32>
        // map: wap return of rev() ... Option<Option<u32>>
        .flatten() // Option<u32>
        .is_none()
    {
        return Err(http_400());
    }

    // println!("wc_handler.rs fn page_prepare cp3");

    match http_request.url() {
        Some(v) => page.url_set(v),
        None => return Err(http_400()),
    }

    Ok(())
}

fn handle_get(page: &mut page::Page) -> Vec<u8> {
    match page.source() {
        Some(v) => http_ok(v),
        None => http_404(),
    }
}

fn handle_post(mut page: page::Page, http_request: &http_request::HttpRequest) -> Vec<u8> {
    // println!("wc_handler.rs fn handle_post");

    let wc_request = match http_request.wc_request() {
        Some(wc_request) => wc_request,
        None => return http_400(),
    };

    if wc_request == "json_save" {
        return handle_json_save(&mut page, http_request);
    }

    if wc_request == "page_new" {
        return handle_page_new(&mut page, http_request);
    }

    if wc_request == "href" {
        // memo
        // wc/js
        // hrefEventHandle(event) {
        //     let data = {"href" : href};
        //     let res = postData("href", data);

        return handle_href(&page, http_request);
    }

    if wc_request == "page_move" {
        return handle_page_move(&mut page, http_request);
    }
    // temp
    http_404()
}

// overwite changed on `Page`
// Page::new return err if already exists
// replace current `Page` to updated `Page`, maybe easier than changing content and dom and json, so consume `Page` and replace to new `Page`.
//
fn handle_json_save(page: &mut page::Page, http_request: &http_request::HttpRequest) -> Vec<u8> {
    // println!("wc_handler fn handle_json_save");

    let json_post = match http_request.body_json() {
        Some(v) => v,
        None => return http_400(),
    };

    let _ = page.file_save_rev();

    let res = match page.json_post_save(json_post) {
        Ok(_) => r#"{"res":"post_handle page_json_save"}"#,
        Err(_) => r#"{"res":"failed to save page_json"}"#,
    };

    http_ok(&res.as_bytes().to_vec())
}

fn handle_page_new(page: &mut page::Page, http_request: &http_request::HttpRequest) -> Vec<u8> {
    //
    let json_post = match http_request.body_json() {
        Some(v) => v,
        None => return http_400(),
    };

    let _res = match page.page_sub_new_save(json_post) {
        Ok(v) => v,
        Err(_) => {
            let res = r#"{"res":"failed to create new page"}"#;
            return http_ok(&res.as_bytes().to_vec());
        }
    };

    // dbg comment out
    // http_ok(&res.as_bytes().to_vec())

    // temp
    http_404()
}

/// json: {"href" : href}
/// return {"dest":"href"}
fn handle_href(page: &page::Page, http_request: &http_request::HttpRequest) -> Vec<u8> {
    // href_connector::href_destination(page);

    handle_href_temp(&page, http_request)

    // caller_url: the page's url
    // let caller_url;
    // http
    // host: header - "Host"
    // let url = format!("https://{}{}", &host, &self.path);
    // path : request.path

    // url_req: destination url
    // req: json_post
    // let url_req = req["href"].as_str().unwrap();

    // let mut href_inspec = match href_inspec::HrefInspec::from(&caller_url, &url_req) {

    // match href_inspec.href_req_handle() {

    // temp
    // http_404()
}

// Return href posted
// This is temporary function just return href that was posted.
fn handle_href_temp(_page: &page::Page, http_request: &http_request::HttpRequest) -> Vec<u8> {
    let json_post = match http_request.body_json() {
        Some(v) => v,
        None => return http_400(),
    };

    let href_req = match json_post["href"].as_str() {
        Some(v) => v,
        None => return http_400(),
    };

    // {"dest":"href"}
    let res = format!(r#"{{"dest":"{}"}}"#, href_req);
    // Ok(_) => r#"{"dest":"href"}"#,

    http_ok(&res.as_bytes().to_vec())
}

fn handle_page_move(
    post_page: &mut page::Page,
    http_request: &http_request::HttpRequest,
) -> Vec<u8> {
    // json_post["parent_url"]
    // json_post["dest_url"]
    let json_post = match http_request.body_json() {
        Some(v) => v,
        None => return http_400(),
    };

    let parent_url = match json_post["parent_url"].as_str() {
        Some(v) => v,
        None => return http_404(),
    };

    let dest_url = match json_post["dest_url"].as_str() {
        Some(v) => v,
        None => return http_404(),
    };

    match post_page.page_move(dest_url, parent_url) {
        Ok(_) => {
            let res = format!(r#"{{"Ok":"ok"}}"#);
            http_ok(&res.as_bytes().to_vec())
        }
        Err(_) => http_400(),
    }
}

fn http_hello() -> Vec<u8> {
    let contents = b"Hello".to_vec();
    http_ok(&contents)
}

fn http_form(status: &str, contents: &Vec<u8>) -> Vec<u8> {
    let header = format!(
        // "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
        status,
        contents.len()
    );

    [header.into_bytes(), contents.clone()].concat()
}

fn http_ok(contents: &Vec<u8>) -> Vec<u8> {
    http_form("200 OK", contents)
}

fn http_err(status: &str) -> Vec<u8> {
    http_form(status, &status.as_bytes().to_vec())
}

fn http_400() -> Vec<u8> {
    http_err("400 Bad Request.")
}

fn http_404() -> Vec<u8> {
    http_err("404 Not Found.")
}
