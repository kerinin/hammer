use std::cmp;
use std::hash;
use std::clone;

use std::collections::{HashMap};

use evicting_store::entry::Entry;
use evicting_store::list::{List, Node, NodeLocation};

struct CachedNode<'a, T, V: 'a> {
    prev: Option<&'a CachedNode<'a, T, V>>,
    next: Option<&'a CachedNode<'a, T, V>>,
    //entry: Entry<T, V> + 'a,
}
impl<'a, T, V> Node<'a> for CachedNode<'a, T, V> {
    fn prev(&self) -> Option<&'a CachedNode<'a, T, V>> {
        self.prev
    }
    fn set_prev(&self, v: Option<&'a CachedNode<'a, T, V>>) {
        self.prev = v;
    }
    fn next(&self) -> Option<&'a CachedNode<'a, T, V>> {
        self.next
    }
    fn set_next(&self, v: Option<&'a CachedNode<'a, T, V>>) {
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
    prev: Option<&'a GhostNode<'a>>,
    next: Option<&'a GhostNode<'a>>,
}
impl<'a> Node<'a> for GhostNode<'a> {
    fn prev(&self) -> Option<&'a GhostNode<'a>> {
        self.prev
    }
    fn set_prev(&self, v: Option<&'a GhostNode<'a>>) {
        self.prev = v;
    }
    fn next(&self) -> Option<&'a GhostNode<'a>> {
        self.next
    }
    fn set_next(&self, v: Option<&'a GhostNode<'a>>) {
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

enum Position<'a, T, V: 'a> {
    Top(CachedNode<'a, T, V>),
    Bottom(GhostNode<'a>),
}

struct GhostedList<'a, T: cmp::Eq + hash::Hash, V: 'a> {
    pub top:        List<'a>,
    top_index:      HashMap<T, CachedNode<'a, T, V>>,
    pub bottom:     List<'a>,
    bottom_index:   HashMap<T, GhostNode<'a>>,
}
impl<'a, T: hash::Hash + cmp::Eq + clone::Clone, V> GhostedList<'a, T, V> {
    fn get(&self, token: T) -> Option<Position<'a, T, V>> {
        match self.top_index.get(&token.clone()) {
            Some(node) => {
                return Some(Position::Top(*node));
            },
            None => {},
        }
        match self.bottom_index.get(&token) {
            Some(node) => {
                return Some(Position::Bottom(*node));
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
    fn replace(&mut self, target: &GhostNode) {
        // NOTE: Need to use target...
        // NOTE: Make sure this updates indices
        // NOTE: What if these are None?
        let &mut top_lru = self.top.back;
        let &mut bottom_lru = self.bottom.back;

        // Remove from lists
        self.top.remove(top_lru);
        self.bottom.remove(bottom_lru);

        // Remove from lookups
        self.top_lookup.remove(self.top.data); // ???
        self.bottom_lookup.remove(self.bottom.data); // ???

        // Shift data
        bottom_lru.data = top_lru.data; // ???

        // Add to list
        self.bottom.push_front(bottom_lru);

        // Add to lookup
        self.bottom_lookup.add(self.bottom.data, self.bottom); // ???

        return top_lru;
    }
}
