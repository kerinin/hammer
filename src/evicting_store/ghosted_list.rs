struct CachedNode<T, V> {
    prev: Option<&CachedEntry<T, V>>,
    next: Option<&CachedEntry<T, V>>,
    entry: Entry<T, V>,
}
impl Node for CachedNode<T, V> {
    fn prev(&self) -> Option<&CachedNode<T, V>> {
        self.prev
    }
    fn next(&self) -> Option<&CachedNode<T, V>> {
        self.next
    }
}

struct GhostNode {
    prev: Option<&GhostEntry>,
    next: Option<&GhostEntry>,
}
impl Node for GhostNode {
    fn prev(&self) -> Option<&GhostNode> {
        self.prev
    }
    fn next(&self) -> Option<&GhostNode> {
        self.next
    }
}

enum Postition<T, V> {
    Top(CachedNode<T,V>),
    Bottom(GhosNode),
}

struct GhostedList<T, V> {
    pub top:        List<T>,
    top_index:      HashMap<T, CachedNode<T, V>>,
    pub bottom:     List<T>,
    bottom_index:   HashMap<T, GhostNode>,
}
impl GhostedList<T> {
    fn find(&self, token: T) -> Option<Postition<T, V>> {
        match self.top_index.find(token) {
            Some(node) => {
                return Some(Top(node));
            },
            None => {},
        }
        match self.bottom_index.find(token) {
            Some(node) => {
                return Some(Bottom(node));
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
        let &mut top_ru = self.top_list.back;
        let &mut bottom_lru = self.bottom_list.back;

        // Remove from lists
        self.top_list.remove(top_lru);
        self.bottom_list.remove(bottom_lru);

        // Remove from lookups
        self.top_lookup.remove(top.data) // ???
        self.bottom_lookup.remove(bottom.data) // ???

        // Shift data
        bottom_lru.data = top_lru.data; // ???

        // Add to list
        self.bottom_list.push_front(bottom_lru);

        // Add to lookup
        self.bottom_lookup.add(bottom.data, bottom) // ???

        return top_lru;
    }
}
