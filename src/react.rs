use std::{cell::RefCell, rc::Rc};
use web_sys::{Element, Node};

pub fn create_root<'a>(container: Element) -> FiberRootNode<'a> {
    FiberRootNode::new(container)
}

pub trait FunctionComponentTrait: std::fmt::Debug {
    // fn get_children(&self) -> &ReactNodeList;
    // fn set_children<'a>(&mut self, children: ReactNodeList<'a>);
    // fn init(&mut self);
    fn render(&mut self) -> &ReactNodeList;
}

#[derive(Debug)]
pub enum ReactNodeList<'a> {
    Root(Box<ReactNodeList<'a>>),
    List(Vec<ReactNodeList<'a>>),
    Host(&'a str, Option<Box<ReactNodeList<'a>>>),
    Text(&'a str),
    FunctionComponent(Box<dyn FunctionComponentTrait>),
}

impl ReactNodeList<'_> {
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

type Fiber<'a> = Rc<RefCell<FiberNode<'a>>>;

pub struct FiberNode<'a> {
    element: &'a ReactNodeList<'a>,
    dom: Option<Node>,
    child: Option<Fiber<'a>>,
    sibling: Option<Fiber<'a>>,
    parent: Option<Fiber<'a>>,
}

impl<'a> FiberNode<'a> {
    fn new(element: &'a ReactNodeList<'a>) -> Self {
        Self {
            element,
            dom: None,
            child: None,
            sibling: None,
            parent: None,
        }
    }

    fn set_child(&mut self, child: Fiber<'a>) {
        self.child = Some(child);
    }

    fn set_sibling(&mut self, sibling: Fiber<'a>) {
        self.sibling = Some(sibling);
    }

    fn set_return(&mut self, parent: Fiber<'a>) {
        self.parent = Some(parent);
    }
}
pub struct FiberRootNode<'a> {
    element: Option<ReactNodeList<'a>>,
    dom: Option<Element>,
    wip: Option<Fiber<'a>>,
}

impl<'a> FiberRootNode<'a> {
    fn new(dom: Element) -> Self {
        Self {
            dom: Some(dom),
            element: None,
            wip: None,
        }
    }

    pub fn render(&'a mut self, children: ReactNodeList<'a>) {
        self.element = Some(ReactNodeList::Root(Box::new(children)));

        let mut root_fiber = FiberNode::new(self.element.as_ref().unwrap());
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

fn perform_unit_of_work<'a>(fiber: Fiber<'a>) -> Option<Fiber<'a>> {
    use ReactNodeList::*;
    {
        let mut fiber_mut = fiber.borrow_mut();

        super::log(&format!("PERFORM: {}", fiber_mut.element.get_name()));
        match fiber_mut.element {
            Root(children) => {
                drop(fiber_mut);
                reconcile_children(fiber.clone(), children);
            }
            Host(tag, children) => {
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
                    .for_each(|child| reconcile_children(fiber.clone(), child));
            }
            Text(txt) => {
                fiber_mut.dom = Some(create_text(txt));
            }
            FunctionComponent(component) => {
                // fiber_mut.element = component.render();
                // reconcile_children(fiber.clone(), component.render());
            }
            _ => (),
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

fn reconcile_children<'a>(wip_fiber: Fiber<'a>, children: &'a ReactNodeList<'a>) {
    use ReactNodeList::*;
    match children {
        Host(_, _child) => {
            let mut fiber = FiberNode::new(children);
            fiber.parent = Some(wip_fiber.clone());
            wip_fiber.borrow_mut().child = Some(Rc::new(RefCell::new(fiber)));
        }
        List(children) => {
            let mut first = None;
            let mut previous_sibling = None;
            for child in children.into_iter() {
                let fiber = Rc::new(RefCell::new(FiberNode::new(child)));
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
        _ => (),
    }
}

fn commit_work(fiber: Fiber) {
    let mut dom_parent_fiber = fiber.borrow().parent.clone();
    if let Some(dpf) = dom_parent_fiber.clone() {
        while dpf.borrow().dom.is_none() {
            dom_parent_fiber = dpf.borrow().parent.clone();
        }
    }

    super::log(&format!("COMMITING {}", fiber.borrow().element.get_name()));

    if let Some(dom) = &fiber.borrow().dom {
        dom_parent_fiber
            .unwrap()
            .borrow()
            .dom
            .as_ref()
            .unwrap()
            .append_child(dom)
            .unwrap();
    } else {
        panic!("FUCK");
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

impl std::fmt::Debug for FiberNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{{\n  child: {:#?}\n  sibling: {:#?}\n  dom: {:#?}\n}}",
            self.child,
            self.sibling,
            self.element.get_name()
        ))
    }
}
