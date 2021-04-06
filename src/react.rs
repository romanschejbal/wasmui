use std::{cell::RefCell, rc::Rc};
use web_sys::Element;

pub trait FunctionComponentT: std::fmt::Debug {
    fn render(&self) -> ReactNodeList;
}

#[derive(Debug)]
pub enum ReactNodeList<'a> {
    List(Vec<&'a ReactNodeList<'a>>),
    Element(&'a str, Option<&'a ReactNodeList<'a>>),
    Text(&'a str),
    FunctionComponent(Box<dyn FunctionComponentT>),
}

pub struct FiberNode<'a> {
    element: Option<&'a ReactNodeList<'a>>,
    child: Option<Rc<RefCell<FiberNode<'a>>>>,
    sibling: Option<Rc<RefCell<FiberNode<'a>>>>,
    return_: Option<Rc<RefCell<FiberNode<'a>>>>,
}

impl std::fmt::Debug for FiberNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#?}", self.element))
    }
}

impl<'a> FiberNode<'a> {
    fn new() -> Self {
        Self {
            element: None,
            child: None,
            sibling: None,
            return_: None,
        }
    }

    fn new_with_element(element: &'a ReactNodeList<'a>) -> Self {
        Self {
            element: Some(element),
            child: None,
            sibling: None,
            return_: None,
        }
    }

    fn set_child(&mut self, child: Rc<RefCell<FiberNode<'a>>>) {
        self.child = Some(child);
    }

    fn set_sibling(&mut self, sibling: Rc<RefCell<FiberNode<'a>>>) {
        self.sibling = Some(sibling);
    }

    fn set_return(&mut self, return_: Rc<RefCell<FiberNode<'a>>>) {
        self.return_ = Some(return_);
    }
}

pub struct FiberRootNode<'a> {
    container: Element,
    current: FiberNode<'a>,
}

impl<'a> FiberRootNode<'a> {
    fn new(container: Element) -> Self {
        Self {
            container,
            current: FiberNode::new(),
        }
    }

    pub fn render(&mut self, children: ReactNodeList) {
        let wip = create_fiber(&children, None);
        super::log(&format!("{:?}", wip));
    }
}

pub fn create_root<'a>(container: Element) -> FiberRootNode<'a> {
    FiberRootNode::new(container)
}

fn create_fiber<'a>(
    children: &'a ReactNodeList,
    return_: Option<Rc<RefCell<FiberNode<'a>>>>,
) -> Option<Rc<RefCell<FiberNode<'a>>>> {
    println!("Create fiber for {:?}", children);
    use ReactNodeList::*;
    match children {
        Element(_, child) => {
            let fiber = Rc::new(RefCell::new(FiberNode::new_with_element(children)));
            if let Some(return_) = return_ {
                fiber.borrow_mut().set_return(return_);
            }
            child
                .as_ref()
                .map(|child| create_fiber(child, Some(fiber.clone())))
                .flatten()
                .map(|child| fiber.borrow_mut().set_child(child));
            Some(fiber)
        }
        List(children) => {
            let mut first = None;
            let mut previous_sibling = None;
            for child in children.into_iter() {
                let next = create_fiber(*child, return_.clone());
                if first.is_none() {
                    first = next.clone();
                }
                previous_sibling.map(|prev: Rc<RefCell<FiberNode>>| {
                    if next.is_some() {
                        prev.borrow_mut().set_sibling(next.clone().unwrap())
                    }
                });
                previous_sibling = next;
            }
            first
        }
        Text(_) => {
            let fiber = Rc::new(RefCell::new(FiberNode::new_with_element(children)));
            if let Some(return_) = return_ {
                fiber.borrow_mut().set_return(return_);
            }
            Some(fiber)
        }
        _ => None,
    }
}

fn update_container(element: ReactNodeList, root: FiberRootNode) {}

#[cfg(test)]
mod tests {
    use super::{ReactNodeList::*, *};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_create_fiber() {
        let ahoj_element = Text("Ahoj");
        let span_element = Element("span", Some(&ahoj_element));
        let empty_span_element = Element("span", None);
        let spans = List(vec![&span_element, &empty_span_element]);
        let div_element = Element("div", Some(&spans));

        let fiber = create_fiber(&div_element, None);

        let ahoj_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&ahoj_element),
            child: None,
            sibling: None,
            return_: None,
        }));
        let empty_span_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&empty_span_element),
            child: None,
            sibling: None,
            return_: None,
        }));
        let span_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&span_element),
            child: Some(ahoj_fiber.clone()),
            sibling: Some(empty_span_fiber.clone()),
            return_: None,
        }));
        let div_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&div_element),
            child: Some(span_fiber.clone()),
            sibling: None,
            return_: None,
        }));

        ahoj_fiber.borrow_mut().set_return(span_fiber.clone());
        span_fiber.borrow_mut().set_return(div_fiber.clone());
        empty_span_fiber.borrow_mut().set_return(div_fiber.clone());

        assert_eq!(format!("{:?}", fiber), format!("{:?}", Some(div_fiber)));
    }
}
