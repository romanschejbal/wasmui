use core::panic;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Node};

pub fn create_root(container: Element) -> FiberRootNode {
    FiberRootNode::new(container)
}

pub trait FunctionComponent: std::fmt::Debug {
    fn render(&self) -> ReactNodeList;
}

pub trait HostAttribute {
    type Type;
    fn set(&self, key: &str, component: &mut Self::Type);
    fn remove(&self, key: &str, component: &mut Self::Type);
}

impl std::fmt::Debug for dyn HostAttribute<Type = Element> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostAttribute")
    }
}

type HostProps = HashMap<&'static str, Box<dyn HostAttribute<Type = Element>>>;

pub struct EventListener(pub Closure<dyn FnMut()>);
impl HostAttribute for EventListener {
    type Type = Element;

    fn set(&self, key: &str, component: &mut Self::Type) {
        component
            .add_event_listener_with_callback(key, self.0.as_ref().unchecked_ref())
            .unwrap();
    }

    fn remove(&self, key: &str, component: &mut Self::Type) {
        component
            .remove_event_listener_with_callback(key, self.0.as_ref().unchecked_ref())
            .unwrap();
    }
}

pub struct StringAttr(pub String);
impl HostAttribute for StringAttr {
    type Type = Element;

    fn set(&self, key: &str, component: &mut Self::Type) {
        component.set_attribute(key, &self.0).unwrap();
    }

    fn remove(&self, key: &str, component: &mut Self::Type) {
        component.remove_attribute(key).unwrap();
    }
}

#[derive(Debug)]
pub enum ReactNodeList {
    Root(Rc<ReactNodeList>),
    List(Vec<Rc<ReactNodeList>>),
    Host(&'static str, HostProps, Option<Rc<ReactNodeList>>),
    Text(String),
    FunctionComponent(Box<dyn FunctionComponent + 'static>),
}

impl ReactNodeList {
    fn get_name(&self) -> String {
        use ReactNodeList::*;
        match self {
            Root(_) => "ROOT".into(),
            Host(name, _, _) => format!("{} (host)", name.to_string()),
            Text(content) => format!("{} (text)", content),
            FunctionComponent(_) => "FunctionComponent".into(),
            _ => "_".into(),
        }
    }
}

type Fiber = Rc<RefCell<FiberNode>>;

pub struct FiberNode {
    element: Rc<ReactNodeList>,
    dom: Option<Element>,
    child: Option<Fiber>,
    sibling: Option<Fiber>,
    parent: Option<Fiber>,
}

impl FiberNode {
    fn new(element: Rc<ReactNodeList>) -> Self {
        Self {
            element,
            dom: None,
            child: None,
            sibling: None,
            parent: None,
        }
    }
}
pub struct FiberRootNode {
    element: Option<Rc<ReactNodeList>>,
    dom: Option<Element>,
    wip_root: Option<Fiber>,
    ric: Option<super::utils::RequestIdleCallback>,
}

impl FiberRootNode {
    fn new(dom: Element) -> Self {
        Self {
            dom: Some(dom),
            element: None,
            wip_root: None,
            ric: None,
        }
    }

    pub fn render(&mut self, children: ReactNodeList) {
        self.element = Some(Rc::new(ReactNodeList::Root(Rc::new(children))));

        let mut root_fiber = FiberNode::new(self.element.clone().unwrap());
        root_fiber.dom = Some(self.dom.take().unwrap().into());

        let boxed_root_fiber = Rc::new(RefCell::new(root_fiber));
        self.wip_root = Some(boxed_root_fiber.clone());

        let mut next_unit_of_work = Some(boxed_root_fiber.clone());
        let mut wip_root_fiber = next_unit_of_work.clone();
        self.ric = Some(super::utils::RequestIdleCallback::new(Box::new(
            move || {
                if let Some(next) = next_unit_of_work.take() {
                    next_unit_of_work = perform_unit_of_work(next);
                    commit_work(
                        wip_root_fiber
                            .clone()
                            .unwrap()
                            .borrow()
                            .child
                            .clone()
                            .unwrap(),
                    );
                } else if let Some(wip_root) = wip_root_fiber.take() {
                    super::log("COMMITING...");
                    commit_work(wip_root.borrow().child.clone().unwrap());
                }
            },
        )));
        self.ric.as_ref().unwrap().start();

        super::log(&format!("{:?}", self.wip_root));
    }
}

fn create_dom(fiber: &mut RefMut<FiberNode>) -> Element {
    use ReactNodeList::*;
    let (mut node, props) = match fiber.element.as_ref() {
        Host(tag, props, _children) => (
            web_sys::window()
                .expect("window not available")
                .document()
                .expect("document not available")
                .create_element(tag)
                .expect("can't create element"),
            props,
        ),
        _ => panic!("Unsupported element type passed to create_dom function"),
    };
    update_dom(&mut node, &Box::new(HashMap::new()), props);
    node
}

fn update_dom(node: &mut Element, prev_props: &HostProps, props: &HostProps) {
    // Remove old or changed
    prev_props
        .iter()
        .filter(|(k, _)| is_gone(k, props) || is_new(k, prev_props, props))
        .for_each(|(key, attr)| {
            attr.remove(key, node);
        });

    // Set new or changed
    props
        .iter()
        .filter(|(k, _)| is_new(k, prev_props, props))
        .for_each(|(key, attr)| attr.set(key, node));
}

fn is_gone(key: &str, props: &HostProps) -> bool {
    !props.contains_key(key)
}

fn is_new(_key: &str, _prev_props: &HostProps, _props: &HostProps) -> bool {
    // prevProps.get(key) != props.get(key)
    true
}

fn create_text(text: &str) -> Element {
    let node: Node = web_sys::window()
        .expect("window not available")
        .document()
        .expect("document not available")
        .create_text_node(text)
        .into();
    let el = web_sys::window()
        .expect("window not available")
        .document()
        .expect("document not available")
        .create_element("span")
        .expect("can't create element");
    el.append_child(&node).unwrap();
    el
}

fn perform_unit_of_work(fiber: Fiber) -> Option<Fiber> {
    use ReactNodeList::*;
    {
        let mut fiber_mut = fiber.borrow_mut();

        super::log(&format!("PERFORM: {}", fiber_mut.element.get_name()));
        match fiber_mut.element.as_ref() {
            Root(children) => {
                let children = children.clone();
                drop(fiber_mut);
                reconcile_children(fiber.clone(), children);
            }
            Host(_tag, _props, children) => {
                let children = children.clone();
                if fiber_mut.dom.is_none() {
                    fiber_mut.dom = Some(create_dom(&mut fiber_mut));
                }
                drop(fiber_mut);
                if let Some(children) = children {
                    reconcile_children(fiber.clone(), children);
                }
            }
            List(children) => {
                children
                    .iter()
                    .for_each(|child| reconcile_children(fiber.clone(), child.clone()));
            }
            Text(txt) => {
                fiber_mut.dom = Some(create_text(txt));
            }
            FunctionComponent(component) => {
                let children = Rc::new(component.as_ref().render());
                drop(fiber_mut);
                reconcile_children(fiber.clone(), children);
            }
        }
    }

    if let Some(child) = &RefCell::borrow(&fiber).child {
        return Some(child.clone());
    }

    let mut next_fiber_opt = Some(fiber);
    while let Some(next_fiber) = next_fiber_opt {
        if let Some(sibling) = &RefCell::borrow(&next_fiber).sibling {
            return Some(sibling.clone());
        }
        next_fiber_opt = RefCell::borrow(&next_fiber).parent.clone();
    }

    return None;
}

fn reconcile_children(wip_fiber: Fiber, children: Rc<ReactNodeList>) {
    use ReactNodeList::*;
    match children.as_ref() {
        Host(_, _props, _child) => {
            let mut fiber = FiberNode::new(children);
            fiber.parent = Some(wip_fiber.clone());
            wip_fiber.borrow_mut().child = Some(Rc::new(RefCell::new(fiber)));
        }
        List(children) => {
            let mut first = None;
            let mut previous_sibling = None;
            for child in children.into_iter() {
                let fiber = Rc::new(RefCell::new(FiberNode::new(child.clone())));
                fiber.borrow_mut().parent = Some(wip_fiber.clone());
                if first.is_none() {
                    first = Some(fiber.clone());
                }
                previous_sibling.map(|prev: Rc<RefCell<FiberNode>>| {
                    prev.borrow_mut().sibling = Some(fiber.clone());
                });
                previous_sibling = Some(fiber);
            }
            wip_fiber.borrow_mut().child = first;
        }
        Text(_txt) => {
            let mut fiber = FiberNode::new(children);
            fiber.parent = Some(wip_fiber.clone());
            wip_fiber.borrow_mut().child = Some(Rc::new(RefCell::new(fiber)));
        }
        FunctionComponent(_component) => {
            let mut fiber = FiberNode::new(children);
            fiber.parent = Some(wip_fiber.clone());
            wip_fiber.borrow_mut().child = Some(Rc::new(RefCell::new(fiber)));
        }
        Root(_) => {}
    }
}

fn commit_work(fiber: Fiber) {
    super::log(&format!("COMMITING {}", fiber.borrow().element.get_name()));

    let mut dom_parent_fiber_opt = fiber.borrow().parent.clone();
    while let Some(dom_parent_fiber) = dom_parent_fiber_opt.clone() {
        if dom_parent_fiber.borrow().dom.is_none() {
            dom_parent_fiber_opt = dom_parent_fiber.borrow().parent.clone();
        } else {
            break;
        }
    }

    if let Some(dom) = &fiber.borrow().dom {
        dom_parent_fiber_opt
            .unwrap()
            .borrow()
            .dom
            .as_ref()
            .unwrap()
            .append_child(dom)
            .unwrap();
    } else {
        // fn component
    }

    //   if (fiber.effect_tag === "PLACEMENT" && fiber.dom != null) {
    // domParent.appendChild(fiber.dom);
    //   } else if (fiber.effectTag === "UPDATE" && fiber.dom != null) {
    //     updateDom(fiber.dom, fiber.alternate.props, fiber.props);
    //   } else if (fiber.effectTag === "DELETION") {
    //     commitDeletion(fiber, domParent);
    //   }

    fiber
        .borrow()
        .child
        .as_ref()
        .map(|f| commit_work(f.clone()));
    fiber
        .borrow()
        .sibling
        .as_ref()
        .map(|f| commit_work(f.clone()));
}

impl std::fmt::Debug for FiberNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{{\n child: {:#?}\n sibling: {:#?}\n dom: {:#?}\n}}",
            self.child,
            self.sibling,
            self.element.get_name()
        ))
    }
}
