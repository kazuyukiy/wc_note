use std::io::Result;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use tracing_subscriber;
// use tracing::{info, info_span}; //  event, instrument, span, Level debug,
// use tracing::warn; //  event, instrument, span, Level debug,
mod js_css;
mod thread_pool;
mod wc_handler;

// #[macro_use]
// extern crate markup5ever;

/// addr: host and port ex: "127.0.0.1:3000"
/// stor_root: root path for storeage of the pages
/// capa: number of thread_pool
/// page_top_path: ex. "/wc_top.html"
// pub fn wc_note(addr: &str, stor_root: &str, capa: usize) -> Result<TcpListener> {
// pub fn wc_note(addr: &str, stor_root: &str, page_top_path: &str, capa: usize) -> Result<()> {
pub fn wc_note(addr: &str, stor_root: &str, capa: usize) -> Result<()> {
    tracing_subscriber::fmt::init();

    // let page_top_path = "/Computing/computing_index.html";
    // let mut page_test = wc_handler::page::Page::new(stor_root, page_top_path);
    // info!("page_test:{}", page_test.file_path());
    // page_test.file_backup_delete();

    // DBG
    // do only backup for test
    // return Ok(());

    // Copy the latest wc.js, wc.css to ./page/.
    // It is done only once when wc_note() is called.
    // If you change contents of wc.js or wc.css, you may recall wc_note() to apply the changes.
    js_css::setup();

    let listener = match TcpListener::bind(addr) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to bind: {:?}", e);
            return Err(e);
        }
    };

    println!("bind {}", addr);

    let pool = thread_pool::ThreadPool::new(capa);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(v) => v,
            Err(_) => continue,
        };

        let stor_root = String::from(stor_root);
        pool.execute(|| {
            handle_connection(stream, stor_root);
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, stor_root: String) {
    // Consider to reject access from wher not local

    let response = wc_handler::response(&mut stream, &stor_root);
    stream.write(&response).unwrap();
    stream.flush().unwrap();
}
