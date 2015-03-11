pub struct List<'a, T: 'a + Node<'a>> {
    pub front: Option<&'a mut T>,
    pub back: Option<&'a mut T>,
    n: u64,
}

pub enum NodeLocation {
    Front,
    Back,
    Middle,
    NotInList,
}

pub trait Node<'a> {
    fn prev(&self) -> Option<&'a mut Self>;
    fn set_prev(&mut self, Option<&'a mut Self>);
    fn next(&self) -> Option<&'a mut Self>;
    fn set_next(&mut self, Option<&'a mut Self>);
    fn location(&self) -> NodeLocation;
}

impl<'a, T: Node<'a>> List<'a, T> {
    fn new() -> List<'a, T> {
        List {
            front: None,
            back: None,
            n: 0,
        }
    }

    fn len(&self) -> u64 {
        self.n
    }

    pub fn remove(&mut self, node: &mut T) {
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

    fn push_front(&mut self, node: &'a mut T) {
        match self.front {
            Some(ref mut n) => n.set_prev(Some(node)),
            None => {},
        };
        node.set_next(self.front);
        self.front = Some(node);
        self.n += 1;
    }
}
