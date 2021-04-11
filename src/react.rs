use std::{cell::RefCell, rc::Rc};
use web_sys::{Element, Node};

pub fn create_root(container: Element) -> FiberRootNode {
    FiberRootNode::new(container)
}

pub trait FunctionComponentTrait: std::fmt::Debug {
    // fn get_children(&self) -> &ReactNodeList;
    // fn set_children<'static>(&mut self, children: ReactNodeList<'static>);
    // fn init(&mut self);
    fn render(&self) -> ReactNodeList;
}

#[derive(Debug)]
pub enum ReactNodeList {
    Root(Rc<ReactNodeList>),
    List(Vec<Rc<ReactNodeList>>),
    Host(&'static str, Option<Rc<ReactNodeList>>),
    Text(String),
    FunctionComponent(Box<dyn FunctionComponentTrait + 'static>),
}

impl ReactNodeList {
    fn get_name(&self) -> String {
        use ReactNodeList::*;
        match self {
            Root(_) => "ROOT".into(),
            Host(name, _) => format!("{} (host)", name.to_string()),
            Text(content) => format!("{} (text)", content),
            FunctionComponent(_) => "FunctionComponent".into(),
            _ => "_".into(),
        }
    }
}

type Fiber = Rc<RefCell<FiberNode>>;

pub struct FiberNode {
    element: Rc<ReactNodeList>,
    dom: Option<Node>,
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

    fn set_child(&mut self, child: Fiber) {
        self.child = Some(child);
    }

    fn set_sibling(&mut self, sibling: Fiber) {
        self.sibling = Some(sibling);
    }

    fn set_return(&mut self, parent: Fiber) {
        self.parent = Some(parent);
    }
}
pub struct FiberRootNode {
    element: Option<Rc<ReactNodeList>>,
    dom: Option<Element>,
    wip: Option<Fiber>,
}

impl FiberRootNode {
    fn new(dom: Element) -> Self {
        Self {
            dom: Some(dom),
            element: None,
            wip: None,
        }
    }

    pub fn render(&mut self, children: ReactNodeList) {
        self.element = Some(Rc::new(ReactNodeList::Root(Rc::new(children))));

        let mut root_fiber = FiberNode::new(self.element.clone().unwrap());
        root_fiber.dom = Some(self.dom.take().unwrap().into());

        let boxed_root_fiber = Rc::new(RefCell::new(root_fiber));
        self.wip = Some(boxed_root_fiber.clone());

        let mut next_unit_of_work = Some(boxed_root_fiber.clone());
        while let Some(next) = next_unit_of_work {
            next_unit_of_work = perform_unit_of_work(next);
        }

        commit_work(boxed_root_fiber.borrow().child.clone().unwrap());

        super::log(&format!("{:?}", self.wip));
    }
}

fn create_dom(tag: &str) -> Node {
    web_sys::window()
        .expect("window not available")
        .document()
        .expect("document not available")
        .create_element(tag)
        .expect("can't create element")
        .into()
}

fn create_text(text: &str) -> Node {
    web_sys::window()
        .expect("window not available")
        .document()
        .expect("document not available")
        .create_text_node(text)
        .into()
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
            Host(tag, children) => {
                let children = children.clone();
                if fiber_mut.dom.is_none() {
                    fiber_mut.dom = Some(create_dom(tag));
                    // @todo update_dom with props
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
        Host(_, _child) => {
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
                    prev.borrow_mut().set_sibling(fiber.clone())
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
        FunctionComponent(component) => {
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
        // panic!("FUCK {:#?}", fiber);
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
            "{{\n  child: {:#?}\n  sibling: {:#?}\n  dom: {:#?}\n}}",
            self.child,
            self.sibling,
            self.element.get_name()
        ))
    }
}
