// use std::io::BufRead;
use std::io::Read;
use std::net::TcpStream;
// use tracing::info; //  event, instrument, span, Level
use tracing::{error, info}; //  event, info, instrument, span, Level

pub struct HttpRequest {
    method: String,
    path: String,
    wc_request: Option<String>,
    host: Option<String>,
    body: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn from(stream: &mut TcpStream) -> Result<HttpRequest, ()> {
        let stream_data = stream_read(stream);
        // let stream_data = stream_read_timeout(stream);
        let mut headers = [httparse::EMPTY_HEADER; 64];
        // let mut headers = [httparse::EMPTY_HEADER; 128];
        let mut request = httparse::Request::new(&mut headers);

        let body_offset = match request.parse(&stream_data) {
            Ok(s) => match s {
                httparse::Status::Complete(l) => Some(l),
                httparse::Status::Partial => {
                    // DBG
                    info!("httparse::Status::Partial");
                    None
                }
            },
            Err(_) => {
                error!("request.parse Failed to parse request.");
                return Err(());
            }
        };

        // request.path
        let path = match request.path {
            // Some(path) => path.to_string(),
            // DBG
            Some(path) => path.to_string(),
            // {
            //     println!("path:{}", path);
            //     path.to_string()
            // }
            None => return Err(()),
        };

        // request.method
        let method = match request.method {
            Some(method) => method.to_string(),
            None => return Err(()),
        };

        let mut http_request = HttpRequest {
            method,
            path,
            wc_request: None,
            host: None,
            body: None,
        };

        // GET
        if http_request.method() == "GET" {
            return Ok(http_request);
        }

        // request.headers
        // wc-request
        if let Some(v) = head_value(&request, "wc-request") {
            let vu8 = v.to_vec();
            if let Ok(v) = String::from_utf8(vu8) {
                http_request.wc_request.replace(v);
            };
        }

        // Host // ex.: 127.0.0.1:3000
        if let Some(v) = head_value(&request, "Host") {
            let v = v.to_vec();
            if let Ok(v) = String::from_utf8(v) {
                http_request.host.replace(v);
            }
        }

        // body
        if body_offset.is_some() {
            let body = stream_data[body_offset.unwrap()..].to_vec();
            http_request.body.replace(body);
        };
        Ok(http_request)
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn wc_request(&self) -> Option<&str> {
        self.wc_request.as_ref().map(|v| v.as_str())
    }

    // fn _body(&self) -> Option<&Vec<u8>> {
    //     // pub fn dbg_body(&self) -> Option<&Vec<u8>> {
    //     self.body.as_ref()
    // }

    fn body_string(&self) -> Option<String> {
        self.body
            .as_ref()
            .and_then(|v| Some(v.to_vec()))
            .and_then(|v| String::from_utf8(v).ok())
    }

    pub fn body_json(&self) -> Option<json::JsonValue> {
        let json_post = self.body_string()?;
        json::parse(&json_post).ok()
    }

    pub fn url(&self) -> Option<url::Url> {
        let host = self.host.as_ref()?;
        let path = &self.path;

        // let url = format!("https://{}{}", host, path);
        let url = format!("http://{}{}", host, path);
        url::Url::parse(&url).ok()
    }
}

// const MESSAGE_SIZE: usize = 512;
// const MESSAGE_SIZE: usize = 5;
// const MESSAGE_SIZE: usize = 10;
const MESSAGE_SIZE: usize = 128;
// const MESSAGE_SIZE: usize = 1024;

// 1024 can not get body contents in most times, seldom success.
// 64 can get bodys almost all time.
// I think reading loop should be slow enough to recieve much data for MESSAGE_SIZE full fill.
// 64, some page gets error to get wc.js, 512 does not.
// const MESSAGE_SIZE: usize = 64;

// 64, some page gets error to get wc.js, 512 does not.
// const MESSAGE_SIZE: usize = 512;

// const MESSAGE_SIZE: usize = 1024;

fn stream_read(stream: &mut TcpStream) -> Vec<u8> {
    // let _r = stream.set_read_timeout(None);

    let mut rx_bytes = [0u8; MESSAGE_SIZE];
    let mut stream_data: Vec<u8> = vec![];

    // DBG
    // print!("\nstream_read start ");
    // info!("stream_read");

    loop {
        // It reads some data some times.
        // Some times reads its length is zero.
        // match stream.read_to_end(&mut stream_data) {}
        match stream.read(&mut rx_bytes) {
            Ok(bytes_read) => {
                stream_data.extend_from_slice(&rx_bytes[..bytes_read]);

                // DBG
                // print!(",{}", bytes_read);

                if bytes_read < MESSAGE_SIZE {
                    break;
                }
            }
            Err(e) => {
                error!("stream_read: {:?}", e);
                break;
            }
        }
    }
    // DBG
    // println!(" stream_read end");

    stream_data
}

// copied to stream_read02 for debug, return it stream_read02to stream_read to rewind.
fn _stream_read02(stream: &mut TcpStream) -> Vec<u8> {
    // const MESSAGE_SIZE: usize = 5;
    // const MESSAGE_SIZE: usize = 1024;

    // 1024 can not get body contents in most times, seldom success.
    // 64 can get bodys almost all time.
    // I think reading loop should be slow enough to recieve much data for MESSAGE_SIZE full fill.
    // 64, some page gets error to get wc.js, 512 does not.
    // const MESSAGE_SIZE: usize = 64;

    // 64, some page gets error to get wc.js, 512 does not.
    const MESSAGE_SIZE: usize = 512;

    // const MESSAGE_SIZE: usize = 1024;
    let mut rx_bytes = [0u8; MESSAGE_SIZE];
    let mut stream_data: Vec<u8> = vec![];

    // if let Some(v) = content_length(&request) {}

    loop {
        // It reads some data some times.
        // Some times reads its length is zero.
        // match stream.read_to_end(&mut stream_data) {}
        match stream.read(&mut rx_bytes) {
            Ok(bytes_read) => {
                stream_data.extend_from_slice(&rx_bytes[..bytes_read]);
                if bytes_read < MESSAGE_SIZE {
                    break;
                }
            }
            Err(e) => {
                error!("stream_read: {:?}", e);
                break;
            }
        }
    }
    stream_data
}

fn _stream_read_timeout(stream: &mut TcpStream) -> Vec<u8> {
    // const MESSAGE_SIZE: usize = 5;
    const MESSAGE_SIZE: usize = 1024;
    let mut rx_bytes = [0u8; MESSAGE_SIZE];
    let mut stream_data: Vec<u8> = vec![];

    // DBG
    info!("fn stream_read_timeout");

    let _r = stream.set_read_timeout(Some(std::time::Duration::new(0, 5000)));

    // DBG
    // info!("read_timeout: {:?}", stream.read_timeout());

    // DBG
    info!("loop in");

    loop {
        match stream.read(&mut rx_bytes) {
            Ok(bytes_read) => {
                info!("stream.read: {} bytes", bytes_read);

                stream_data.extend_from_slice(&rx_bytes[..bytes_read]);

                if bytes_read == 0 {
                    break;
                }

                // if bytes_read < MESSAGE_SIZE {
                //     break;
                // }
            }

            Err(e) => {
                error!("stream_read: {:?}", e);
                break;
            }
        }
    }

    // DBG
    info!("loop out");

    stream_data
}

fn head_value<'a>(request: &'a httparse::Request, name: &str) -> Option<&'a [u8]> {
    match request.headers.iter().find(|&&h| h.name == name) {
        Some(header) => Some(header.value),
        None => None,
    }
}

fn _content_length(request: &httparse::Request) -> Option<usize> {
    head_value(request, "Content-Length")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| usize::from_str_radix(v, 10).ok())
}
