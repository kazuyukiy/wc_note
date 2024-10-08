use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};
use std::collections::HashMap;
// use std::collections::HashSet;
use std::str::FromStr;
use tracing::{error, info};
// use tracing::{event, info, instrument, span, Level, Node};
mod dom_utility;

pub fn file_path(stor_root: &str, page_path: &str) -> String {
    // String + "." + &str
    stor_root.to_string() + page_path
}

pub fn fs_write(file_path: &str, contents: &Vec<u8>) -> Result<(), ()> {
    // DBG
    // info!("DBG save: {} (Not saved indeed, make DBG off)", file_path);
    // return Ok(());

    std::fs::write(&file_path, contents)
        .and_then(|_| {
            info!("save: {}", file_path);
            Ok(())
        })
        .or_else(|e| {
            error!("fn fs_write e: {:?}", e);
            Err(())
        })

    // DBG comment out
    // .or(Err(()))
}

pub fn to_dom(source: &str) -> RcDom {
    dom_utility::to_dom(source)
}

pub fn json_from_dom(dom: &RcDom) -> Option<json::JsonValue> {
    // span node containing json data in text
    let span = span_json_node(dom);
    let children = span.children.borrow();
    if children.len() == 0 {
        eprintln!("Failed, json contents not found in the span element");
        return None;
    }

    let contents = match &children[0].data {
        NodeData::Text { contents } => contents,
        _ => {
            eprintln!("Failed, json contents not found in the span element");
            return None;
        }
    };

    let json_str = contents.borrow().to_string();

    let json_value = match json::parse(&json_str) {
        Ok(page_json_parse) => page_json_parse,
        Err(e) => {
            eprintln!("{:?}", e);
            return None;
        }
    };

    Some(json_value)
}

// Get span node from page_dom
// <span id="page_json_str" style="display: none">{"json":"json_data"}</span>
pub fn span_json_node(page_dom: &RcDom) -> Handle {
    let attrs = &vec![("id", "page_json_str")];
    let ptn_span = dom_utility::node_element("span", &attrs);
    dom_utility::child_match_first(&page_dom, &ptn_span, true).unwrap()
}

fn page_dom_plain() -> RcDom {
    to_dom(page_html_plain())
}

fn page_html_plain() -> &'static str {
    r#"<!DOCTYPE html><html><head><title></title><meta charset="UTF-8"></meta><script src="/wc.js"></script>
    <link rel="stylesheet" href="/wc.css"></link>
    <style type="text/css"></style>
</head><body onload="bodyOnload()"><span id="page_json_str" style="display: none"></span></body></html>
"#
}

/// Create a page source from json value.
fn source_from_json(page_json: &json::JsonValue) -> Vec<u8> {
    let page_dom = page_dom_plain();

    // title
    if let Some(title_str) = page_json["data"]["page"]["title"].as_str() {
        let title_ptn = dom_utility::node_element("title", &vec![]);
        if let Some(title_node) = dom_utility::child_match_first(&page_dom, &title_ptn, true) {
            let title_text = dom_utility::node_text(title_str);
            title_node.children.borrow_mut().push(title_text);
        }
    }

    // put json value into span as str
    let span = span_json_node(&page_dom);
    let _ = &span.children.borrow_mut().clear();
    let json_str = page_json.dump();
    let json_node_text = dom_utility::node_text(&json_str);
    let _ = &span.children.borrow_mut().push(json_node_text);

    //
    let sh = SerializableHandle::from(page_dom.document);
    let mut page_bytes = vec![];
    let _r = serialize(&mut page_bytes, &sh, SerializeOpts::default());

    page_bytes
}

/// Create `supser::Page` from json.
/// // If a file exists, open the file and overwrite page_json.
/// // If no file exists, create a `super::Page` from page_json.
///
pub fn page_from_json(
    stor_root: &str,
    page_path: &str,
    // page_json: json::JsonValue,
    page_json: &json::JsonValue,
    // ) -> Result<super::Page, ()> {
) -> super::Page {
    let source = source_from_json(page_json); // bytes

    let mut page = super::Page::new(stor_root, page_path);
    page.source.replace(Some(source));

    // Ok(page)
    page
}

// pub fn json_rev_match(page: &mut super::Page, json_data2: &json::JsonValue) -> bool {
pub fn json_rev_match(page: &mut super::Page, json_data2: &json::JsonValue) -> Result<(), String> {
    if page.json().is_none() {
        // return false;
        return Err(format!("Failed to get json of {}", page.page_path));
    }

    let rev = match page.json().unwrap().rev() {
        Some(rev) => rev,
        // None => return false,
        None => return Err(format!("Failed to get rev of {}", page.page_path)),
    };

    let rev2 = match json_data2["data"]["page"]["rev"] {
        json::JsonValue::Number(number) => match number.try_into() {
            Ok(rev2) => rev2,
            Err(_) => {
                // eprintln!("Failed to get rev");
                // return false;

                return Err(format!("Failed to get rev from json_data2"));
            }
        },
        // case: rev="12" ( with "" )
        json::JsonValue::Short(short) => {
            let rev2 = short.as_str();
            match u32::from_str(rev2) {
                Ok(rev2) => rev2,
                Err(_) => {
                    return Err(format!("Failed to get rev2"));

                    // eprintln!("Failed to get rev");
                    // return false;
                }
            }
        }
        // _ => return false,
        _ => return Err(format!("Failed to get rev2")),
    };

    // rev == rev2
    if rev == rev2 {
        Ok(())
    } else {
        // DBG
        // Remove a line blow for debug mode
        // error!(
        //     "{}",
        //     format!("DEB SKIP for rev not match {} : {}", rev, rev2)
        // );
        // return Ok(());

        Err(format!("rev not match {} : {}", rev, rev2))
    }
}

/// Create a new page under the parent_page
/// It returns an instance of super::Page
/// but its file is not saved.
/// You need to save the file if needs.
///
/// It return Err if a file alraady exists in the child_href,
///
/// Create new navi data taking over parent_page and adding child_title to it
/// converting href based on child_href
///
/// parent_* : some value of parent_page
/// child_* : some value of new navi data created in this function
///
/// child_href: absolute or related location based on parent_page
///
pub fn page_child_new(
    parent_page: &mut super::Page,
    parent_url: url::Url,
    child_title: &str,
    child_href: &str,
) -> Result<super::Page, ()> {
    // If no parent json, no file or no data, return Err(())
    let _parent_json = parent_page.json().ok_or(())?;

    let (child_title, child_href) = title_href_check(child_title, child_href)?;
    let child_url = child_url(&parent_url, child_href).or(Err(()))?;
    let child_path = child_url.path();

    // child_href might be a relative: ex: ./move2/move2.html, not for Page::new()
    // child_url.path(): /Computing/move2/move2.html
    let mut child_page = super::Page::new(&parent_page.stor_root, child_path);

    // If the file already exists, return Err(())
    if child_page.source().is_some() {
        info!("file {} already exists", child_page.file_path());
        return Err(());
    }

    // json plain
    let mut child_json = super::page_json::page_json_plain();

    // title
    child_json["data"]["page"]["title"] = child_title.into();

    // navi
    // navi of parent_page
    let mut child_navi = child_navi(parent_page, &parent_url, &child_url).or(Err(()))?;

    // add navi of child_href
    let navi_child: Vec<json::JsonValue> = vec![child_title.into(), "".into()];
    if child_navi.push(json::JsonValue::Array(navi_child)).is_err() {
        return Err(());
    }

    child_json["data"]["navi"] = child_navi;

    // return Ok(Page)
    // page_from_json(parent_page.stor_root(), child_path, child_json).or(Err(()))
    // page_from_json(parent_page.stor_root(), child_path, &child_json).or(Err(()))
    let child_page = page_from_json(parent_page.stor_root(), child_path, &child_json);
    Ok(child_page)
}

/// Check title and href
fn title_href_check<'a>(title: &'a str, href: &'a str) -> Result<(&'a str, &'a str), ()> {
    let title = title.trim();
    if title.len() == 0 {
        eprintln!("no child title");
        return Err(());
    }

    let href = href.trim();
    if href.starts_with("#") {
        eprintln!("child href starts with #");
        return Err(());
    }
    if href.len() == 0 {
        eprintln!("no child href");
        return Err(());
    }

    Ok((title, href))
}

fn child_url(parent_url: &url::Url, child_href: &str) -> Result<url::Url, ()> {
    parent_url.join(&child_href).or_else(|_| {
        eprintln!("parent_url.join failed");
        Err(())
    })
}

/// Create a navi data from parent_page except child_url and its title.
/// Convert href based on child_url as relative if possible.
fn child_navi(
    parent_page: &mut super::Page,
    parent_url: &url::Url,
    child_url: &url::Url,
) -> Result<json::JsonValue, ()> {
    let parent_json = parent_page.json().ok_or(())?;
    // let parent_json = parent_json.data().ok_or(())?;
    let parent_json = parent_json.value().ok_or(())?;

    let parent_navi = match &parent_json["data"]["navi"] {
        json::JsonValue::Array(ref v) => v,
        _ => return Err(()),
    };

    let mut child_navi = json::JsonValue::Array(vec![]);

    for navi in parent_navi {
        let title = navi[0].clone();

        // Convert href switching its base on paretn_url to child_url
        let href = navi[1]
            .as_str()
            .and_then(|href| href_url(&parent_url, href, &child_url))
            .or(Some("".to_string())) // only Some
            .unwrap();

        let mut navi2 = json::JsonValue::Array(vec![]);
        navi2.push::<json::JsonValue>(title.into()).or(Err(()))?;
        navi2.push::<json::JsonValue>(href.into()).or(Err(()))?;

        child_navi.push(navi2).or(Err(()))?;
    }

    Ok(child_navi)
}

/// Convert href to href_url based on org_base.
/// And get relative url of href based on new_base if posibble.
fn href_url(org_base: &url::Url, href: &str, new_base: &url::Url) -> Option<String> {
    // Get Url of href based on org_base
    match org_base.join(&href) {
        // Get relative url of href_url based on new_base
        Ok(href_url) => match new_base.make_relative(&href_url) {
            Some(v) => Some(v),
            // No relative exists, so absolute url of href_url(href)
            None => Some(href_url.as_str().to_string()),
        },
        Err(_) => None,
    }
}

// fn page_json(page: &super::Page) -> Option<&json::JsonValue> {
//     page.json_value().or_else(|| {
//         error!(
//             "{}",
//             format!("Failed to get page_json.data of {}", page.path())
//         );
//         None
//     })
// }

/// Move org_page to dest_url as a child of dest_parent_url.
/// dest_parent_url can be None in a case dest_url is the top page.
pub fn page_move(
    stor_root: &str,
    org_url: &url::Url,
    dest_url: url::Url,
    dest_parent_url: Option<&url::Url>,
) -> Result<(), String> {
    let mut org_page = super::Page::new(stor_root, org_url.path());

    let mut dest_parent_page = dest_parent_url.and_then(|url| {
        let dest_parent_page = super::Page::new(stor_root, url.path());
        Some(dest_parent_page)
    });

    let dest_parent_page_json = match dest_parent_page.as_mut() {
        Some(page) => page.json(),
        None => None,
    };

    let dest_parent_json = dest_parent_page_json.and_then(|page_json| page_json.value());

    let mut page_moving = PageMoving::new();

    let org_json = org_page
        .json()
        .and_then(|page_json| page_json.value())
        .ok_or("Failed to get page_Json.")?;

    page_move_json(
        &mut page_moving,
        stor_root,
        org_json,
        org_url,
        &dest_url,
        dest_parent_url,
        dest_parent_json,
    )?;

    dest_page_save(stor_root, &page_moving);
    org_page_save(stor_root, &page_moving);

    Ok(())
}

struct PageMoving {
    org_path_list: Vec<String>,
    // // <org_url, (org_url, dest_url, dest_json)>
    // <org_path, (org_url, dest_url, dest_json)>
    data: HashMap<String, (url::Url, url::Url, json::JsonValue)>,
}

impl PageMoving {
    fn new() -> PageMoving {
        PageMoving {
            org_path_list: vec![],
            data: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        org_url: url::Url,
        dest_url: url::Url,
        json: json::JsonValue,
    ) -> Result<(), String> {
        // Use key of url.path().
        // url.as_str() starts http or https those become different keys.
        // if self.data.contains_key(org_url.as_str()) {
        if self.data.contains_key(org_url.path()) {
            // return Err(format!("org_url recurred: {}", org_url.path()).to_string());
            return Err(format!("org_url recurred: {}", org_url.path()));
        }

        // self.org_path_list.push(org_url.to_string());
        self.org_path_list.push(org_url.path().to_string());
        self.data
            .insert(org_url.path().to_string(), (org_url, dest_url, json));

        Ok(())
    }

    fn contains_org_url(&self, org_url: &url::Url) -> bool {
        // self.data.contains_key(org_url.as_str())
        self.data.contains_key(org_url.path())
    }

    fn get(&self, org_path: &str) -> Option<(url::Url, url::Url, &json::JsonValue)> {
        if let Some((org_url, dest_url, dest_json)) = self.data.get(org_path) {
            return Some((org_url.clone(), dest_url.clone(), dest_json));
        };

        None
    }

    fn org_path_list(&self) -> Vec<&str> {
        // self.org_path_list.iter().map(|url| url.as_str()).collect()
        // self.org_path_list.iter().map(|path| path).collect()
        self.org_path_list
            .iter()
            .map(|path| path.as_str())
            .collect()
    }
}

fn page_move_json(
    page_moving: &mut PageMoving,
    stor_root: &str,
    org_json: &json::JsonValue,
    org_url: &url::Url,
    dest_url: &url::Url,
    dest_parent_url: Option<&url::Url>,
    dest_parent_json: Option<&json::JsonValue>,
) -> Result<(), String> {
    // DBG
    info!("\n org_url: {} to\ndest_url: {}", org_url, dest_url);

    // org_url duplication avoiding endlessloop
    if page_moving.contains_org_url(org_url) {
        return Err(format!("Duplicated org_url: {}", org_url.as_str()));
    }

    // Already moved page
    if !org_json["data"]["page"]["moved_to"].is_empty() {
        return Err(format!("Already moved : {}", org_url));
    }

    page_move_dest_already_data(stor_root, dest_url)?;

    let mut dest_json = super::page_json::page_json_plain();

    page_move_system_and(&mut dest_json, &org_json);

    page_move_navi(&mut dest_json, dest_parent_url, dest_parent_json);

    let org_children_href = page_move_subsections(dest_url, &mut dest_json, org_json, &org_url)?;

    page_moving.insert(org_url.clone(), dest_url.clone(), dest_json.clone())?;

    page_move_children(
        page_moving,
        stor_root,
        org_children_href,
        org_url,
        dest_url,
        &dest_json,
    )?;

    // temp
    Ok(())
}

/// Return Err if page in dest_url has subsection data.
fn page_move_dest_already_data(
    // a_page: &mut super::Page,
    stor_root: &str,
    dest_url: &url::Url,
) -> Result<(), String> {
    // Return Err if dest_page already exists, except no data in the page.
    let mut dest_page = super::Page::new(stor_root, dest_url.path());
    // Case the page already has subsection data, abort moving.
    if dest_page.json_subsections_data_exists() {
        return Err(format!("The file data already exists: {}", dest_url.path()));
        // let err_msg = format!("The file data already exists: {}", dest_url.path());
        // return Err(err_msg);
    }
    return Ok(());
}

fn page_move_system_and(dest_json: &mut json::JsonValue, org_json: &json::JsonValue) {
    dest_json["system"] = org_json["system"].clone();
    dest_json["data"]["page"] = org_json["data"]["page"].clone();
}

/// Set page title in dest_json["data"]["page"]["title"] before call this.
fn page_move_navi(
    dest_json: &mut json::JsonValue,
    dest_parent_url: Option<&url::Url>,
    dest_parent_json: Option<&json::JsonValue>,
) {
    // Get title at here before calling page_move_navi_parent
    // to avoid a borrow err.
    let title = dest_json["data"]["page"]["title"]
        .as_str()
        .or(Some("no title"))
        .unwrap()
        .to_string();

    let dest_navi = &mut dest_json["data"]["navi"];

    // navi to the parent
    page_move_navi_parent(dest_parent_url, dest_parent_json, dest_navi);

    // navi to this page
    let title = json::JsonValue::from(title);
    let href = json::JsonValue::from("");
    let navi = json::array![title, href];
    let _ = dest_navi.push(navi);
}

fn page_move_navi_parent(
    dest_parent_url: Option<&url::Url>,
    dest_parent_json: Option<&json::JsonValue>,
    dest_navi: &mut json::JsonValue,
) -> Option<()> {
    let dest_parent_url = dest_parent_url?;
    let dest_parent_navi = match dest_parent_json?["data"]["navi"] {
        json::JsonValue::Array(ref vec) => Some(vec),
        _ => None,
    }?;

    for p_navi in dest_parent_navi.iter() {
        let title = p_navi[0].as_str().or(Some("no title")).unwrap();
        let title = json::JsonValue::from(title);

        let href = p_navi[1].as_str().or(Some(""))?;
        // href based on org_parent_url
        // href: String of url.path()
        let href = dest_parent_url
            .join(href)
            .and_then(|url| Ok(url.path().to_string()))
            // "" if failed.
            .or::<Result<&str, ()>>(Ok("".to_string()))
            .unwrap();
        let href = json::JsonValue::from(href);
        let navi = json::array![title, href];
        let _ = dest_navi.push(navi);
    }

    Some(())
}

fn page_move_subsections(
    dest_url: &url::Url,
    dest_json: &mut json::JsonValue,
    org_json: &json::JsonValue,
    org_url: &url::Url,
) -> Result<Vec<String>, String> {
    let subsections = match &org_json["data"]["subsection"]["data"] {
        json::JsonValue::Object(ref object) => Some(object),
        _ => None,
    }
    .ok_or("Failed to get subsection data".to_string())?;

    let mut dest_subsections = json::object! {};
    let mut children_href: Vec<String> = vec![];

    for (id, org_subsection) in subsections.iter() {
        dest_subsections[id] =
            page_move_subsection(dest_url, org_url, org_subsection, &mut children_href)?;
    }

    dest_json["data"]["subsection"]["data"] = dest_subsections;

    // temp
    Ok(children_href)
}

fn page_move_subsection(
    dest_url: &url::Url,
    org_url: &url::Url,
    org_subsection: &json::JsonValue,
    children_href: &mut Vec<String>,
) -> Result<json::JsonValue, String> {
    let mut dest_subsection = json::object! {};
    page_move_subsection_title_and(org_subsection, &mut dest_subsection);
    let org_href = org_subsection["href"].as_str().or(Some("")).unwrap();
    if let Some((dest_href, is_child)) = href_move(org_url, org_href, dest_url) {
        dest_subsection["href"] = dest_href.as_str().into();
        if is_child {
            // In case a child dest_href is relative and
            // can be used for org_url also
            children_href.push(dest_href);
        }
    };
    dest_subsection["content"] =
        page_move_subsection_content(&org_subsection["content"], org_url, dest_url)?;

    Ok(dest_subsection)
}

fn page_move_subsection_title_and(
    subsection: &json::JsonValue,
    dest_subsection: &mut json::JsonValue,
) {
    dest_subsection["parent"] = subsection["parent"].clone();
    // dest_subsection["id"] = subsection["id"].clone();
    dest_subsection["title"] = subsection["title"].clone();
    dest_subsection["child"] = subsection["child"].clone();

    // Set id as str, converting number to str.
    let id_str = match subsection["id"] {
        json::JsonValue::Number(number) => {
            let id: f64 = number.clone().into();
            id.to_string()
        }
        _ => subsection["id"].as_str().or(Some("")).unwrap().to_string(),
    };
    dest_subsection["id"] = id_str.into();
}

/// Premise: all urls of org_children_url are children of parent_org_url.
fn page_move_children(
    page_moving: &mut PageMoving,
    stor_root: &str,
    org_children_href: Vec<String>,
    parent_org_url: &url::Url,
    parent_dest_url: &url::Url,
    parent_dest_json: &json::JsonValue,
) -> Result<(), String> {
    for child_org_href in org_children_href {
        let (child_org_json, child_org_url, child_dest_url) = match page_move_children_prepare(
            stor_root,
            &child_org_href,
            parent_org_url,
            parent_dest_url,
        ) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };
        page_move_json(
            page_moving,
            stor_root,
            &child_org_json,
            &child_org_url,
            &child_dest_url,
            Some(parent_dest_url),
            Some(parent_dest_json),
        )?;
    }

    Ok(())
}

fn page_move_children_prepare(
    stor_root: &str,
    child_org_href: &str,
    parent_org_url: &url::Url,
    parent_dest_url: &url::Url,
) -> Result<(json::JsonValue, url::Url, url::Url), String> {
    let child_org_url = match parent_org_url.join(child_org_href) {
        Ok(v) => v,
        Err(_) => return Err(format!("Failed to get url for : {}", child_org_href)),
    };

    let mut child_org_page = super::Page::new(stor_root, child_org_url.path());

    let child_org_json = match child_org_page
        .json()
        .and_then(|page_json| page_json.value())
    {
        Some(v) => v,
        None => {
            return Err(format!(
                "Failed to get page_json of {}",
                child_org_page.file_path()
            ))
        }
    };

    let child_dest_url = match parent_dest_url.join(child_org_href) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Failed to get child_dest_url of {}",
                child_org_href
            ));
        }
    };

    Ok((child_org_json.clone(), child_org_url, child_dest_url))
}

fn page_move_subsection_content(
    org_contents: &json::JsonValue,
    org_url: &url::Url,
    dest_url: &url::Url,
    //
    // dest_subsection: &mut json::JsonValue,
    // org_children_url: &mut HashSet<url::Url>,
) -> Result<json::JsonValue, String> {
    let org_contents = match org_contents {
        json::JsonValue::Array(ref v) => v,
        _ => {
            let msg = format!("Failed to get content of {} as Arrray", org_url.path());
            return Err(msg);
        }
    };

    let mut dest_contents = json::array![];
    for org_content in org_contents {
        // "content" : [ {"type" : "text", "value" : "sample"} ],
        let mut dest_content = json::object! {};
        dest_content["type"] = org_content["type"].clone();

        let org_content_value = org_content["value"].as_str().or(Some("")).unwrap();
        let dest_content_value =
            page_move_content_href_convert(org_content_value, org_url, dest_url);
        dest_content["value"] = dest_content_value.into();

        dest_contents.push(dest_content).or_else(|e| {
            let msg = format!(
                "Failed to push content {}\n with {:?}",
                org_content_value, e
            );
            Err(msg)
        })?;
    }

    Ok(dest_contents)
}

fn dest_page_save(stor_root: &str, page_moving: &PageMoving) {
    for org_path in page_moving.org_path_list() {
        let (_org_url, dest_url, dest_json) = match page_moving.get(org_path) {
            Some(v) => v,
            None => {
                error!("{}", format!("No page2Moving for {}", org_path));
                continue;
            }
        };
        let mut dest_page = page_from_json(stor_root, dest_url.path(), dest_json);
        // dest_page.dir_build();
        if dest_page.dir_build().is_err() {
            continue;
        }

        let _r = dest_page.file_save_and_rev();
    }
}

fn org_page_save(stor_root: &str, page_moving: &PageMoving) {
    for org_path in page_moving.org_path_list() {
        let (mut org_page, org_page_json) =
            match page_org_page_moved(stor_root, org_path, &page_moving) {
                Ok(v) => v,
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };

        if let Err(e) = org_page.json_replace_save(org_page_json) {
            error!("{}", e);
        }
    }
}

// fn page_moved_to(page: &mut super::Page) -> Option<&str> {
//     let json_value = page.json_value()?;
//     json_value["data"]["page"]["moved_to"].as_str()

//     // let mut org_page = super::Page::new(stor_root, org_url.path());
//     // let org_json = org_page
//     //     .json_value()
//     //     .ok_or(format!("Failed to get page_json.data of {}", org_url))?;

//     //    org_json_uped["data"]["page"]["moved_to"] = dest_url.as_str().into();
// }

/// Set the page of org_url as moved.
fn page_org_page_moved(
    stor_root: &str,
    org_path: &str,
    page_moving: &PageMoving,
) -> Result<(super::Page, json::JsonValue), String> {
    let (org_url, dest_url, _dest_json) = match page_moving.get(org_path) {
        Some(v) => v,
        None => return Err(format!("No page2Moving for {}", org_path)),
    };

    let mut org_page = super::Page::new(stor_root, org_url.path());
    let org_json = org_page
        .json_value()
        .ok_or(format!("Failed to get page_json.data of {}", org_url))?;

    // let org_json = org_page
    //     .json()
    //     .and_then(|page_json| page_json.value())
    //     .ok_or(format!("Failed to get page_json.data of {}", org_url))?;

    // let org_json = match org_page.json().and_then(|page_json| page_json.value()) {
    //     Some(v) => v,
    //     None => return Err(format!("Failed to get page_json.data of {}", org_url)),
    // };

    let mut org_json_uped = org_json.clone();

    // moved_to
    org_json_uped["data"]["page"]["moved_to"] = dest_url.as_str().into();

    // title
    let title = org_json["data"]["page"]["title"]
        .as_str()
        .or(Some(""))
        .unwrap();
    let title = format!("Moved({}) to {}", title, dest_url);

    // navi
    let navi = match &mut org_json_uped["data"]["navi"] {
        json::JsonValue::Array(ref mut v) => v,
        _ => return Err(format!("Failed to vet navi data of : {}", org_url)),
    };

    if navi.len() == 0 {
        return Err(format!("Failed to vet navi data of : {}", org_url));
    }

    let pos_last = navi.len() - 1;
    navi[pos_last][0] = title.into();

    Ok((org_page, org_json_uped))
}

/// Returns Option<(String, bool)
/// Convert org_href to href based on dest_url.
/// String: href value in String that can be used in the page at dest_url.
/// bool: true if org_href is child of org_url, false if org_href is link to the page of org_url or not child of org_url.
/// Return None if failed to convert href.
///
/// org_url: original url base
/// org_href: original href defined in the page of org_url
/// dest_url: url where new href is used at.
///
/// href to child page of org_url can be relative,
/// but otherwise should be absolute.
/// "/abc" : absolute, start with /
/// "abc" : relative, start without /
fn href_move(org_url: &url::Url, org_href: &str, _dest_url: &url::Url) -> Option<(String, bool)> {
    let org_href_url = org_url.join(org_href).ok()?;

    let is_not_child = false;

    // Case the host is not of this page, return the original value as it is.
    if org_href_url.host() != org_url.host() {
        // full url
        let href = org_href_url.as_str().to_string();
        return Some((href, is_not_child));
    }

    // Case org_href path is as same as org_url path, means same page.
    // if org_href is empty, no need to make a new link.
    if org_url.path() == org_href_url.path() {
        // org_href may be as same as href we get here,
        // but org_href might have some more infomation than the reference.
        // fragment: (#)subsection1 (exclude #)
        let fragment = org_href_url.fragment()?;
        let href = "#".to_string() + fragment;

        return Some((href, is_not_child));
    }

    // Case org_href is child of org_url,
    // In case org_href path is as same as org_url path, it was handled previously and it does not come here.
    // relative href can be used, so you can forget about dest_url
    //
    // remove file name from org_url.path()
    let filename = org_url.path_segments().and_then(|split| split.last())?;
    let org_dir = org_url.path().strip_suffix(filename)?;
    if org_href_url.path().starts_with(org_dir) {
        //      org_dir: org/url/  (org_url without filename)
        // org_href_url: org/url/href/page.html#fragment
        // remove prefix(: org/url/ ), remains: href/page.html
        // href: href/page.html
        let mut href = org_href_url.path().strip_prefix(org_dir)?.to_string();
        if let Some(fragment) = org_href_url.fragment() {
            href = href + "#" + fragment;
        }
        // dest base
        let is_child = true;
        return Some((href, is_child));
    }

    // Case not child of the orig_url
    let org_href_url = org_url.join(org_href).unwrap();

    // println!("org_href_url: mk1 {}", org_href_url.as_str());

    let dest_href_url = org_href_url;
    let mut href = dest_href_url.path().to_string();
    if let Some(fragment) = dest_href_url.fragment() {
        href = href + "#" + fragment;
    }

    Some((href, is_not_child))
}

fn page_move_content_href_convert(
    org_content: &str,
    org_url: &url::Url,
    dest_url: &url::Url,
) -> String {
    let mut index: usize = 0;
    let mut content = String::from(org_content);

    loop {
        if content.len() <= index {
            break;
        }

        // Search where href="xxx".
        let href_pos = href_pos(&content, index);
        // href not found.
        if href_pos.is_none() {
            break;
        }
        let (href_start, href_end, href_value_start, href_value_end) = href_pos.unwrap();

        // Convert href value for moving.
        let org_href = &content[href_value_start..href_value_end];
        let op_href_move = href_move(org_url, org_href, dest_url);

        // Failed to convert href valuye.
        // Leave the href="xxx" as it is.
        // Keep the loop from href_value_end.
        if op_href_move.is_none() {
            index = href_value_end;
            continue;
        }

        let (dest_href, _is_child) = op_href_move.unwrap();

        // make href="converted_href_value"
        // put a space before "href=".
        let dest_href_equation = " href=\"".to_string() + &dest_href + "\"";
        content.replace_range(href_start..href_end, &dest_href_equation);

        index = match href_start.checked_add(dest_href_equation.len()) {
            Some(v) => v,
            None => break,
        }
    }
    content
}

/// Ssearch an element <a href="href_value"> on argument str and return href and its value positions as Some((href_start, href_end, href_value_start, href_value_end))
/// Return None if not found or any error.
/// The search starts on &str[find_start..], so &str[..find_start] is ignored.
/// Start end end position are counted as &str[0] is 0.
///
/// Serch pattern takes speces before href="xxx" if there is, but not at the and.
/// So to remake href, you may put space before href="yyy"
fn href_pos(str: &str, search_start: usize) -> Option<(usize, usize, usize, usize)> {
    // Search <a, but not escaped \<a:
    let (_a_start, a_end) = pos_not_escaped(str, search_start, "<a")?;

    // Search href="value"
    // href=" or href='
    // (?i) : not case-sensitive
    let reg_href = regex::Regex::new(r#"(?i)\s*href\s*=\s*["']"#).unwrap();
    let href_mat = reg_href.find(&str[a_end..])?;
    // begining point of href=""
    // starts a_end so add it to the result
    let href_start = a_end.checked_add(href_mat.start())?;
    // position of the first quote charactor
    // starts a_end so add it to the result
    let q1_end = a_end.checked_add(href_mat.end())?;
    // one charactor " before
    let q1_start = q1_end.checked_add_signed(-1)?;
    // Get the first quote.
    let quote = &str[q1_start..q1_end];
    // Search second quote in after href=", q1_end position.
    let (q2_start, q2_end) = pos_not_escaped(str, q1_end, quote)?;
    let href_value_start = q1_end;
    let href_value_end = q2_start;
    let href_end = q2_end;

    // Return positions of value part: abc of href="abc"
    // Some((q1_end, q2_start))
    Some((href_start, href_end, href_value_start, href_value_end))
}

/// Search ptn on str and return the first position of it as Some(start, end),
/// or None if not found or any error.
/// It does not match if `/` is found before ptn as an escape key.
/// It start serching from search_start position of str
fn pos_not_escaped(str: &str, search_start: usize, ptn: &str) -> Option<(usize, usize)> {
    // Regular expression of ptn
    let re_ptn = regex::Regex::new(&ptn).ok()?;

    // Regular expression of backslash `\` continuing more than two.
    let re_esc = regex::Regex::new(r"(\\+)$").ok()?;

    let mut index_start = search_start;

    loop {
        // Serching reaced at the end and ptn was not found.
        if str.len() <= index_start {
            return None;
        }

        // Search ptn
        let ptn_match = re_ptn.find(&str[index_start..])?;

        // index position of ptn starts.
        let ptn_start = index_start.checked_add(ptn_match.start())?;

        // Check if the ptn is escaped.
        // To do that, count number of \ before ptn.
        //
        // \\\\ptn (\ is more than one)
        // if \ exists just befor ptn
        // it might be an escape code (\ptn)
        // or just `\` charactor (\\ptn: `\` + ptn)
        //
        // In case of single `\` charactor,
        // it should be escape code \ before `\` charactor
        // so \\ is a caractor `\` with escaped code.
        //
        // If number of continuous \ is odd, the last \ is escape code for ptn.
        // eg "\\ \\ \\ \\ \ptn" (The parrern is escaped by the last \.)
        // (consider as spaces in above are not exists, those spaces are only for easy to see.)
        //
        // If number of continuous \ is even, those are some couple of
        // escape code and it means `\` charactors.
        // eg "\\ \\ \\ \\ ptn" (The parrern is not escaped by \.)
        //
        // If make some couple of \ (\\) and still remains one \
        // it means ptn is escaped.
        // In case of html, it is not an element since < is escaped with \.
        // eg: \<a\>
        //
        // Find `\` just befor ptn position.
        // &str[index_start..ptn_start]: str just before ptn, just beforeptn_start
        let escape_cap = re_esc.captures(&str[index_start..ptn_start]);

        // Set index_start position to end of ptn.
        index_start = index_start.checked_add(ptn_match.end())?;

        if let Some(cap) = escape_cap {
            // If number of `\` is odd, ptn is escaped.
            if &cap[1].len() % 2 == 1 {
                // Search ptn again after new index_start position
                continue;
            }
        }

        let ptn_end = index_start;
        return Some((ptn_start, ptn_end));
    }
}
