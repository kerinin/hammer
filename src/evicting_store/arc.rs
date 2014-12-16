struct ARC<T, V> {
    // Cache size
    c:                      uint,
    // Frequency/Recency tradeoff parameter
    p:                      uint,
    // Counters
    used_top_nodes:         uint,
    used_bottom_nodes:      uint,

    top:                    Array<CachedNode<T, V>>,
    bottom:                 Array<GhostNode<T, V>>,

    recent:                 GhostList<T, V>,
    frequent:               GhostList<T, V>,
}

impl ARC<T, V> {
    fn with_capacity(c: uint) -> &ARC {
        let top =               Array::with_capacity(c);
        let bottom =            Array::with_capacity(c);
        let recent =            GhostList::new();
        let frequent =          GhostList::new();

        &ARC {
            c:                  c,
            p:                  0,
            used_top_nodes:     0,
            used_bottom_nodes:  0,
            top:                top,
            bottom:             bottom,
            recent:             recent,
            frequent:           frequent,
        }
    }

    /*
     * Shifts an entry from the LRU position in one of the top caches to the MRU
     * position in the corresponding bottom cache (L1->B1 or L2->B2), and returns 
     * the newly emptied entry from the top cache
     */
    fn replace(&mut self, token_in_bottom_frequent: bool, target: &GhostNode) -> &CachedPage<T, V> {
        if !self.t1.empty() && (self.t1.len() > self.p || (token_in_bottom_frequent && self.t1.len() == self.p)) {
            return self.recent.replace(target);
        } else {
            return self.frequent.replace(target);
        }
    }
}

impl<T, V> EvictingStore<T, V> for ARC<T, V> {
    /*
     * NOTE: This method is taken almost verbatim from page 123 of
     * https://www.usenix.org/legacy/event/fast03/tech/full_papers/megiddo/megiddo.pdf
     */
    fn get(&mut self, token: T) -> (Option<V>, Vec<Eviction<T, V>>) {
        match self.recent.find(token) {
            Some(Top(top_node)) => {
                self.recent.remove_top(top_node);

                self.frequent.top.push_front(top_node);
                return (top_node.value(), vec![]);
            },
            Some(Bottom(bottom_node)) => {
                // Adaptation
                let delta = if self.recent.bottom.len() >= self.frequent.bottom.len() {
                    1
                } else {
                    self.frequent.bottom.len() / self.recent.bottom.len()
                }
                self.p = min(self.p + delta, c);

                // OK now...
                self.recent.bottom.remove(bottom_node);
                let &mut top_node = self.replace(false, bottom_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.frequent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);
            },
            None => {},
        };
        match self.frequent.find(token) {
            Some(Top(top_node)) => {
                self.frequent.top.remove(top_node);

                self.frequent.top.push_front(top_node);
                return top_node.value();
            },
            Some(Bottom(bottom_node)) => {
                // Adaptation
                let delta = if self.frequent.bottom.len() >= self.recent.bottom.len() {
                    1
                } else {
                    self.recent.bottom.len() / self.frequent.bottom.len()
                }
                self.p = max(self.p - delta, 0);

                // OK now...
                self.frequent.bottom.remove(bottom_node);
                let &mut top_node = self.replace(true, bottom_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.frequent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);
            },
            None => {},
        };

        /*
         * Cache miss
         */
        if self.recent.top.len() + self.recent.bottom.len() == self.c {
            if self.recent.top.len() < self.c {
                let &mut bottom_node = self.recent.bottom.back;
                self.recent.bottom.remove(bottom_node);
                let &mut top_node = self.replace(false, bottom_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.recent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);

            } else {
                let &mut top_node = self.recent.top.back;
                self.recent.top.remove(top_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.recent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);
            }

        } else if self.recent.top.len() + self.recent.bottom.len() < self.c {
            if self.t1.len() + self.t2.len() + self.b1.len() + self.b2.len() == 2 * self.c {
                let &mut bottom_node = self.frequent.bottom.back;
                self.frequent.bottom.remove(bottom_node);
                let &mut top_node = self.replace(false, bottom_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.recent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);

            } else if self.t1.len() + self.t2.len() + self.b1.len() + self.b2.len() >= self.c {
                let &mut bottom_node = self.bottom[self.used_bottom_nodes];
                self.used_bottom_nodes++;
                let &mut top_node = self.replace(false, bottom_node);

                let eviction = Eviction::new(top_node.token(), top_node.value());

                self.recent.top.push_front(top_node);
                top_node.fetch(token);
                return (top_node.value(), vec![eviction]);
            }
        }

        let &mut top_node = self.top[self.used_top_nodes];
        self.used_top_nodes++;

        self.recent.push_front(top_node);
        top_node.fetch(token);
        return top_node.value();
    }
}
