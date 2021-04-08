use std::{cell::RefCell, rc::Rc};
use web_sys::{Element, Node};

pub fn create_root<'a>(container: Element) -> FiberRootNode<'a> {
    FiberRootNode::new(container)
}

pub trait FunctionComponentTrait: std::fmt::Debug {
    fn render(&self) -> ReactNodeList;
}

#[derive(Debug)]
pub enum ReactNodeList<'a> {
    List(Vec<ReactNodeList<'a>>),
    Host(&'a str, Option<Box<ReactNodeList<'a>>>),
    Text(&'a str),
    FunctionComponent(Box<dyn FunctionComponentTrait>),
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
    container: Element,
    wip: Option<Fiber<'a>>,
}

impl<'a> FiberRootNode<'a> {
    fn new(container: Element) -> Self {
        Self {
            container,
            wip: None,
        }
    }

    pub fn render(mut self, children: &'a ReactNodeList<'a>) {
        let mut root_fiber = FiberNode::new(children);
        root_fiber.dom = Some(self.container.into());
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

fn perform_unit_of_work(fiber: Fiber) -> Option<Fiber> {
    use ReactNodeList::*;
    {
        let mut fiber_mut = fiber.borrow_mut();
        match fiber_mut.element {
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
            Text(ahoj) => {
                super::log(&format!("LOOKMA: {}", ahoj));
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
                if first.is_none() {
                    fiber.borrow_mut().parent = Some(wip_fiber.clone());
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
    let mut dom_parent_fiber = fiber.clone().borrow().parent.clone();
    if let Some(dpf) = dom_parent_fiber.clone() {
        while dpf.borrow().dom.is_none() {
            dom_parent_fiber = dpf.borrow().parent.clone();
        }
    }

    if let Some(dom) = &fiber.borrow().dom {
        dom_parent_fiber
            .unwrap()
            .borrow()
            .dom
            .as_ref()
            .unwrap()
            .append_child(dom)
            .unwrap();
    }

    //   if (fiber.effect_tag === "PLACEMENT" && fiber.dom != null) {
    // domParent.appendChild(fiber.dom);
    //   } else if (fiber.effectTag === "UPDATE" && fiber.dom != null) {
    //     updateDom(fiber.dom, fiber.alternate.props, fiber.props);
    //   } else if (fiber.effectTag === "DELETION") {
    //     commitDeletion(fiber, domParent);
    //   }

    // fiber
    //     .borrow()
    //     .child
    //     .as_ref()
    //     .map(|f| commit_work(f.clone()));
    // fiber
    //     .borrow()
    //     .sibling
    //     .as_ref()
    //     .map(|f| commit_work(f.clone()));
}

#[cfg(test)]
mod tests {
    // use super::{ReactNodeList::*, *};
    // use pretty_assertions::assert_eq;

    // #[test]
    // fn test_create_fiber() {
    //     let ahoj_element = Text("Ahoj");
    //     let span_element = Element("span", Some(ahoj_element));
    //     let empty_span_element = Element("span", None);
    //     let spans = List(vec![span_element, &empty_span_element]);
    //     let div_element = Element("div", Some(&spans));

    //     let fiber = create_fiber(&div_element, None);

    //     let ahoj_fiber = Rc::new(RefCell::new(FiberNode {
    //         element: Some(&ahoj_element),
    //         child: None,
    //         sibling: None,
    //         return_: None,
    //     }));
    //     let empty_span_fiber = Rc::new(RefCell::new(FiberNode {
    //         element: Some(&empty_span_element),
    //         child: None,
    //         sibling: None,
    //         return_: None,
    //     }));
    //     let span_fiber = Rc::new(RefCell::new(FiberNode {
    //         element: Some(&span_element),
    //         child: Some(ahoj_fiber.clone()),
    //         sibling: Some(empty_span_fiber.clone()),
    //         return_: None,
    //     }));
    //     let div_fiber = Rc::new(RefCell::new(FiberNode {
    //         element: Some(&div_element),
    //         child: Some(span_fiber.clone()),
    //         sibling: None,
    //         return_: None,
    //     }));

    //     ahoj_fiber.borrow_mut().set_return(span_fiber.clone());
    //     span_fiber.borrow_mut().set_return(div_fiber.clone());
    //     empty_span_fiber.borrow_mut().set_return(div_fiber.clone());

    //     assert_eq!(format!("{:?}", fiber), format!("{:?}", Some(div_fiber)));
    // }
}

impl std::fmt::Debug for FiberNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // use ReactNodeList::*;
        // let get_element_name = |element: &ReactNodeList| match element {
        //     Host(name, _) => name.to_string(),
        //     Text(content) => format!("Text: {}", content),
        //     FunctionComponent(_) => "FunctionComponent".into(),
        //     _ => "_".into(),
        // };
        f.write_fmt(format_args!(
            "{{\n  child: {:#?}\n  sibling: {:#?}\n  dom: {:#?}\n}}",
            self.child, self.sibling, self.dom
        ))
    }
}
