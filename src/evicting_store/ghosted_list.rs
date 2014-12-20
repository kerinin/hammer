use std::cmp;
use std::hash;
use std::clone;

use std::collections::{HashMap};

use evicting_store::entry::Entry;
use evicting_store::list::{List, Node, NodeLocation};

struct CachedNode<'a, T: 'a, V: 'a> {
    prev: Option<&'a mut CachedNode<'a, T, V>>,
    next: Option<&'a mut CachedNode<'a, T, V>>,
    //entry: Entry<T, V> + 'a,
}
impl<'a, T, V> Node<'a> for CachedNode<'a, T, V> {
    fn prev(&self) -> Option<&'a mut CachedNode<'a, T, V>> {
        self.prev
    }
    fn set_prev(&mut self, v: Option<&'a mut CachedNode<'a, T, V>>) {
        self.prev = v;
    }
    fn next(&self) -> Option<&'a mut CachedNode<'a, T, V>> {
        self.next
    }
    fn set_next(&mut self, v: Option<&'a mut CachedNode<'a, T, V>>) {
        self.next = v;
    }
    fn location(&self) -> NodeLocation {
        match (self.prev, self.next) {
            (None, None) => NodeLocation::NotInList,
            (None, Some(..)) => NodeLocation::Back,
            (Some(..), None) => NodeLocation::Front,
            (Some(..), Some(..)) => NodeLocation::Middle,
        }
    }
}

struct GhostNode<'a> {
    prev: Option<&'a mut GhostNode<'a>>,
    next: Option<&'a mut GhostNode<'a>>,
}
impl<'a> Node<'a> for GhostNode<'a> {
    fn prev(&self) -> Option<&'a mut GhostNode<'a>> {
        self.prev
    }
    fn set_prev(&mut self, v: Option<&'a mut GhostNode<'a>>) {
        self.prev = v;
    }
    fn next(&self) -> Option<&'a mut GhostNode<'a>> {
        self.next
    }
    fn set_next(&mut self, v: Option<&'a mut GhostNode<'a>>) {
        self.next = v;
    }
    fn location(&self) -> NodeLocation {
        match (self.prev, self.next) {
            (None, None) => NodeLocation::NotInList,
            (None, Some(..)) => NodeLocation::Back,
            (Some(..), None) => NodeLocation::Front,
            (Some(..), Some(..)) => NodeLocation::Middle,
        }
    }
}

enum Position<'a, T: 'a, V: 'a> {
    Top(CachedNode<'a, T, V>),
    Bottom(GhostNode<'a>),
}

struct GhostedList<'a, T: 'a + cmp::Eq + hash::Hash, V: 'a> {
    pub top:        List<'a, CachedNode<'a, T, V>>,
    top_index:      HashMap<T, CachedNode<'a, T, V>>,
    pub bottom:     List<'a, GhostNode<'a>>,
    bottom_index:   HashMap<T, GhostNode<'a>>,
}
impl<'a, T: hash::Hash + cmp::Eq + clone::Clone, V> GhostedList<'a, T, V> {
    fn get(&self, token: T) -> Option<Position<'a, T, V>> {
        match self.top_index.get(&token.clone()) {
            Some(node) => {
                //return Some(Position::Top(*node));
            },
            None => {},
        }
        match self.bottom_index.get(&token) {
            Some(node) => {
                //return Some(Position::Bottom(*node));
                return None;
            },
            Nonde => {
                return None;
            },
        }
    }

    /*
     * Shifts an entry from the LRU position in the top list to the MRU position 
     * in the bottom list, and returns the newly emptied entry from the top
     */
    fn replace(&mut self, target: &GhostNode) -> &CachedNode<'a, T, V> {
        // NOTE: Need to use target...
        // NOTE: Make sure this updates indices
        // NOTE: What if these are None?

        match self.top.back {
            None => {
                // We shouldn't ever end up here - this function should
                // only ever be called when the top has data.
                panic!("Attempted to shift data from an empty list")
            },
            Some(top_lru) => {
                // Clear out pointers
                //self.top.remove(top_lru);
                //self.bottom.remove(target);

                // Remove from lookups
                //self.top_index.remove(self.top.data); // ???
                //self.bottom_index.remove(self.bottom.data); // ???

                // Shift data
                //target.data = top_lru.data; // ???

                // Add to list
                //self.bottom.push_front(target);

                // Add to lookup
                //self.bottom_index.insert(self.bottom.data, self.bottom); // ???

                return top_lru;
            },
        }
    }
}
