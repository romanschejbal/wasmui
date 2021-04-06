use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};
use web_sys::Element;

#[derive(Debug, PartialEq)]
pub enum ReactNodeList<'a> {
    List(Vec<&'a ReactNodeList<'a>>),
    Element(&'a str, Option<&'a ReactNodeList<'a>>),
    Text(&'a str), // FunctionComponent(Box<dyn Fn()>),
}

#[derive(Debug, PartialEq)]
pub struct FiberNode<'a> {
    element: Option<&'a ReactNodeList<'a>>,
    child: Option<Rc<RefCell<FiberNode<'a>>>>,
}

impl<'a> FiberNode<'a> {
    fn new() -> Self {
        Self {
            element: None,
            child: None,
        }
    }

    fn new_with_element(element: &'a ReactNodeList<'a>) -> Self {
        Self {
            element: Some(element),
            child: None,
        }
    }

    fn set_child(&mut self, child: Rc<RefCell<FiberNode<'a>>>) {
        self.child = Some(child);
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
        let _wip = create_fiber(&children, None);
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
            child
                .as_ref()
                .map(|child| create_fiber(child, Some(fiber.clone())))
                .flatten()
                .map(|child| fiber.borrow_mut().set_child(child));
            Some(fiber)
        }
        List(children) => {
            let mut first = None;
            let mut previous_sibling = return_;
            for child in children.into_iter() {
                let next = create_fiber(*child, previous_sibling);
                previous_sibling = next.clone();
                if first.is_none() {
                    first = next;
                }
            }
            first
        }
        Text(_) => Some(Rc::new(RefCell::new(FiberNode::new_with_element(children)))),
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
        }));
        let span_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&span_element),
            child: Some(ahoj_fiber.clone()),
        }));
        // let empty_span_fiber = Rc::new(RefCell::new(FiberNode {
        //     element: Some(&span_element),
        //     child: None,
        // }));
        let div_fiber = Rc::new(RefCell::new(FiberNode {
            element: Some(&div_element),
            child: Some(span_fiber.clone()),
        }));

        assert_eq!(fiber, Some(div_fiber));
    }
}
