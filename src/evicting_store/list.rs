struct List<N> {
    front: Option<&N>,
    back: Option<&N>,
    n: uint,
}

enum NodeLocation {
    Front,
    Back,
    Middle,
    NotInList,
}

trait Node {
    fn prev(&self) -> Option<&Node>;
    fn next(&self) -> Option<&Node>;
}

impl Node {
    fn location(&self) -> NodeLocation {
        match (self.prev(), self.next()) {
            (None, None) => NotInList,
            (None, Some(..)) => Back,
            (Some(..), None) => Front,
            (Some(..), Some(..)) => Middle,
        }
    }
}

impl<T: Node> List<T> {
    fn new() -> &Self {
        &List {
            front: None,
            back: None,
            n: 0,
        }
    }

    fn len(&self) -> uint {
        self.n
    }

    fn remove(&mut self, node: &T) {
        match node.location() {
            NotInList => {},
            Back => {
                node.next.prev = None;
                self.back = node.next;
                node.next = None;
                self.n -= 1;
            },
            Front => {
                node.prev.next = None;
                self.front = node.prev;
                node.prev = None;
                self.n -= 1;
            },
            Middle => {
                node.prev.next = node.next;
                node.next.prev = node.prev;
                node.prev = None;
                node.next = None;
                self.n -= 1;
            },
        }
    }

    fn push_front(&mut self, node: &Node<T>) {
        self.front.prev = node;
        node.next = self.front;
        self.front = node;
        self.n += 1;
    }
}
