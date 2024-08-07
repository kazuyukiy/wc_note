use html5ever::parse_document; // , serialize
use html5ever::tendril::TendrilSink;
use markup5ever::interface::Attribute;
use markup5ever::{namespace_url, ns};
use markup5ever::{tendril::Tendril, LocalName, QualName};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom}; // , SerializableHandle
use std::cell::RefCell;
use std::rc::Rc;
// use tendril::Tendril;

// convert source in str to node
pub fn to_dom(source: &str) -> RcDom {
    parse_document(markup5ever_rcdom::RcDom::default(), Default::default()).one(source)
}

pub fn qual_name(name: &str) -> QualName {
    QualName::new(
        None,
        // ns!(html), // <script unknown_namespace:type="text/javascript">
        ns!(), // <script type="text/javascript">
        // Namespace::from("http://abc.rs"),
        LocalName::from(name),
        // local_name!(name),
    )
} // end of fn qual_name

pub fn attrs(attrs_vec: &Vec<(&str, &str)>) -> RefCell<Vec<Attribute>> {
    let mut attr_list: Vec<Attribute> = vec![];
    for (name, value) in attrs_vec {
        attr_list.push(attr(&name, &value));
    }
    RefCell::new(attr_list)
} // end of fn attrs

pub fn attr(name: &str, value: &str) -> Attribute {
    Attribute {
        name: qual_name(&name),
        value: Tendril::from(value.to_string()),
    }
} // end of fn attr

pub fn node_element(ele_name: &str, attrs_vec: &Vec<(&str, &str)>) -> Rc<Node> {
    Node::new(NodeData::Element {
        name: qual_name(ele_name),
        attrs: attrs(&attrs_vec),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
} // end of fn node_element

pub fn node_text(contents: &str) -> Rc<Node> {
    // Tendril::from(value.to_string())

    // let contents = Tendril::from(contents.to_string());
    let contents = Tendril::from(contents);
    let node_data = NodeData::Text {
        contents: RefCell::new(contents),
    };

    Node::new(node_data)
}

fn node_child_match(
    node_obj: &Handle,
    node_ptn: &Node,
    node_list: &mut Vec<Handle>,
    recursive: bool,
) {
    for child in node_obj.children.borrow().iter() {
        if element_match(child, node_ptn) {
            // dbg
            // println!("dom_urility node_child_match matched");

            node_list.push(Rc::clone(child));
        }

        if recursive {
            node_child_match(child, node_ptn, node_list, recursive);
        }
    }
}

fn element_match(node: &Handle, node_ptn: &Node) -> bool {
    match node.data {
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let name_obj = name;
            let attrs_obj = attrs;
            match node_ptn.data {
                NodeData::Element {
                    ref name,
                    ref attrs,
                    ..
                } => {
                    // both node and node_ptn are Element
                    if name_obj.local == name.local {
                        // dbg
                        // println!(
                        //     "dom_urility fn element_match matched name: {:?}",
                        //     name.local
                        // );

                        // see if all attrs of node_ptn match attrs of node

                        // dbg
                        // if attrs_match(attrs_obj.clone(), attrs.clone()) {
                        //     println!("dom_urility fn element_match attrs matched");
                        // }

                        return attrs_match(attrs_obj.clone(), attrs.clone());
                    }
                    // names do not match
                    false
                }
                // node_ptn is not Element
                _ => false,
            }
        }
        // node is not Element
        _ => return false,
    }
}

fn attrs_match(attrs: RefCell<Vec<Attribute>>, attrs_ptn: RefCell<Vec<Attribute>>) -> bool {
    if 0 == attrs_ptn.borrow().len() {
        // no attrs condition
        // match always
        return true;
    }

    for att_for in attrs_ptn.borrow().iter() {
        'attrs: loop {
            for att in attrs.borrow().iter() {
                // dbg
                // println!("dom_urility fn attrs_match name:{:?}", att_for.name);

                if att_for.name.local == att.name.local {
                    if att_for.value == att.value {
                        // att_for match att
                        // see next att_for
                        break 'attrs;
                    } else {
                        // value does not match
                        return false;
                    }
                }
            }
            // att_for.name does not exists in att
            return false;
        } // end of 'attrs: loop
    }

    // all of attrs_ptn match attrs
    true
}

pub fn child_match_list(node_obj: &Handle, node_ptn: &Node, recursive: bool) -> Vec<Handle> {
    let mut node_list: Vec<Handle> = vec![];
    node_child_match(node_obj, node_ptn, &mut node_list, recursive);
    node_list
}

// pub fn child_match_first(node_obj: &Handle, node_ptn: &Node, recursive: bool) -> Option<Handle> {
pub fn child_match_first(dom: &RcDom, node_ptn: &Node, recursive: bool) -> Option<Handle> {
    // let list = child_match_list(node_obj, node_ptn, recursive);
    let list = child_match_list(&dom.document, node_ptn, recursive);

    // dbg
    // println!("dom_urility fn child_match_first length: {}", list.len());

    if list.len() < 1 {
        return None;
    }

    let child = Rc::clone(&list[0]);
    Some(child)
}
