enum CachedPage<V,T> {
    FoundT1(Page<V,T>),
    FoundT2(Page<V,T>),
    FoundB1(Page<V,T>),
    FoundB2(Page<V,T>),
    Missing,
}

struct ARC<T, V> {
    // Cache size
    c: uint,

    // Frequency/Recency tradeoff parameter
    p: uint,

    // Pages - 2c
    pages: Array<Page<T,V>>,

    // Entries - c
    entries: Array<Entry<T,V>>,

    // Page lookup
    pages_lookup: HashMap<T, &Page<T,V>>,

    // List Pointers
    t1_lru: Option<&Page<T,V>>,
    t2_lru: Option<&Page<T,V>>,
    b1_lru: Option<&Page<T,V>>,
    b2_lru: Option<&Page<T,V>>,

    // Counters
    t1_len: uint,
    t2_len: uint,
    b1_len: uint,
    b2_len: uint,
    used_pages: uint,
}

impl ARC<V,T> {
    fn with_capacity(c: uint) -> &ARC {
        let pages = [Entry<T,V> ..2 * c];
        let entries = [Entry<T,V> ..c]
        let page_lookup: HashMap<T, &Page<T,V>> = HashMap::with_capacity(2 * c);

        &ARC {
            c: c, 
            p: 0,
            pages: pages,
            entries: entries,
            page_lookup: page_lookup,
            t1_lru: None,
            t2_lru: None,
            b1_lru: None,
            b2_lru: None,
            t1_len: 0,
            t2_len: 0,
            b1_len: 0,
            b2_len: 0,
            used_pages: 0,
        }
    }

    fn get_page(token: T) -> CachedPage<T, V> {
        match self.page_lookup.get(token) {
            Some(page) => match page.state {
                T1 => FoundT1(page),
                T2 => FoundT2(page),
                B1 => FoundB1(page),
                B2 => FoundB2(page),
            },
            None => Missing,
        }
    }

    fn replace(&mut self, page: Page<V,T>) {
        if !self.t1.empty() && (self.t1.len() > self.p || (self.b2.include(token) && self.t1.len() == self.p)) {
            page.flush();
            self.move_t1_to_b1(page);
        } else {
            page.flush();
            self.move_t2_to_b2(token)
        }
    }
}

impl<V, T> EvictingStore<V, T> for ARC<V, T> {
    fn insert(&mut self, value: V) -> (token: T, evictions: Vec<Eviction<V,T>>) {
    }

    /*
     * NOTE: This method is taken almost verbatim from page 123 of
     * https://www.usenix.org/legacy/event/fast03/tech/full_papers/megiddo/megiddo.pdf
     */
    fn get(&mut self, token: T) -> Option<V> {
        match self.get_page(token) {
            FoundT1(page), FoundT2(page) => {
                /*
                 * Token was found in one of the 'hot' caches, and should be ready
                 * to return
                 */
                self.make_t2_mru(page);
                return Some(page.data);
            },

            FoundB1(page) => {
                /*
                 * Token was found in the buffer cache for 'Recent' data
                 *
                 * This means the token was fetched once, but hasn't been fetched 
                 * in awhile, and is on the way towards being evicted completely
                 * from the system
                 */

                // Adaptation
                let delta = if self.b1_len >= self.b2_len {
                    1
                } else {
                    self.b2_len / self.b1_len
                }
                self.p = min(self.p + delta, c);

                // OK now...
                self.replace(page);
                self.make_t2_mru(page);

                page.fetch();
                return page.data;
            },

            FoundB2(page) => {
                /*
                 * Token was found in the buffer cache for 'Frequent' data
                 *
                 * This means the token was fetched a couple times, but hasn't 
                 * been fetched in awhile, and is on the way towards being evicted 
                 * completely from the system
                 */
                
                let delta = if self.b1_len <= self.b2_len {
                    1
                } else {
                    self.b1_len / self.b2_len
                }
                self.p = max(self.p - delta, 0);
                self.replace(page);
                self.make_t2_mru(page);

                page.fetch();
                return page.data;
            },

            Missing => {
                if self.t1_len + self.b1_len == self.c {
                    if self.t1_len < self.c {
                        /*
                         * Token wasn't found at all, and the 'Recenct data' 
                         * cache is at capacity with some elements in the buffer
                         */
                        let mut page = self.zero_b1_lru() // Rather than deleting...
                        self.replace(page);
                        page.fetch(token);
                        self.make_t1_mru(page);
                        return page.data;

                    } else {
                        /*
                         * Token wasn't found at all, and the 'Recent data' cache 
                         * is at capacity with an empty buffer cache
                         */
                        let mut page = self.zero_b1_lru() // Rather than deleting...
                        page.fetch(token);
                        self.make_t1_mru(page);
                        return page.data;
                    }

                } else {
                    if self.t1.len() + self.t2.len() + self.b1.len() + self.b2.len() == self.c {
                        /*
                         * Token wasn't found at all. The 'Recenct data' cache is 
                         * below capacity and the whole system is at capacity
                         */
                        let mut page = self.zero_b2_lru();
                        self.replace(page);
                        page.fetch(token);
                        self.make_t1_mru(page);
                        return page.data;

                    } else if self.t1.len() + self.t2.len() + self.b1.len() + self.b2.len() >= self.c {
                        /*
                         * Token wasn't found at all. The 'Recenct data' cache is 
                         * below capacity and the whole system is below capacity
                         */
                        let mut page = self.next_zero_page();
                        self.replace(page);
                        page.fetch(token);
                        self.make_t1_mru(page);
                        return page.data;
                    }
                }

            }
        }
    }
}
