struct List<'a> {
    front: Option<&'a Node<'a> + 'a>,
    back: Option<&'a Node<'a> + 'a>,
    n: uint,
}

enum NodeLocation {
    Front,
    Back,
    Middle,
    NotInList,
}

trait Node<'a> {
    fn prev(&self) -> Option<&'a Node<'a>>;
    fn set_prev(&self, Option<&'a Node<'a>>);
    fn next(&self) -> Option<&'a Node<'a>>;
    fn set_next(&self, Option<&'a Node<'a>>);
    fn location(&self) -> NodeLocation;
}

//impl<'a> Node<'a> for List<'a, N> {
//    fn location(&self) -> NodeLocation {
//        match (self.prev(), self.next()) {
//            (None, None) => NodeLocation::NotInList,
//            (None, Some(..)) => NodeLocation::Back,
//            (Some(..), None) => NodeLocation::Front,
//            (Some(..), Some(..)) => NodeLocation::Middle,
//        }
//    }
//}

impl<'a> List<'a> {
    fn new<'a>() -> List<'a> {
        List {
            front: None,
            back: None,
            n: 0,
        }
    }

    fn len(&self) -> uint {
        self.n
    }

    fn remove(&mut self, node: &'a Node<'a>) {
        match node.location() {
            NodeLocation::NotInList => {},
            NodeLocation::Back => {
                match node.next() {
                    Some(n) => n.set_prev(None),
                    None => {},
                };
                self.back = node.next();
                node.set_next(None);
                self.n -= 1;
            },
            NodeLocation::Front => {
                match node.prev() {
                    Some(n) => n.set_next(None),
                    None => {},
                };
                self.front = node.prev();
                node.set_prev(None);
                self.n -= 1;
            },
            NodeLocation::Middle => {
                match node.prev() {
                    Some(n) => n.set_next(node.next()),
                    None => {},
                };
                match node.next() {
                    Some(n) => n.set_prev(node.prev()),
                    None => {},
                };
                node.set_prev(None);
                node.set_next(None);
                self.n -= 1;
            },
        }
    }

    fn push_front(&mut self, node: &'a Node<'a>) {
        match self.front {
            Some(n) => n.set_prev(Some(node)),
            None => {},
        };
        node.set_next(self.front);
        self.front = Some(node);
        self.n += 1;
    }
}
