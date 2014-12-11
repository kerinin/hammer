/*
 * Generic naming conventions:
 *
 * V: Value - the raw data being stored
 * T: Token - used to fetch a value 
 *
 */

struct Eviction<V, T> {
    token: T,
    value: V,
}

trait Store<V, T> {
    fn insert(&mut self, value: V) -> (token: T, evictions: Vec<Eviction>); 

    fn get(&mut self, token: T) -> Option<V>;
}

trait Index<V, T> {
    fn insert(&mut self, token: T, value: V);
    fn find(&self, query: V) -> Vec<T>; // This could be an option, but I'd prefer to KISS
    fn remove(&self, token: T, value: V);
}

trait Database<V, T> {
    fn insert(&mut self, value: V);
    fn query(&self, query: V) -> HashSet<V>;
    // No delete - we'll let the eviction algorithm handle that...
}

struct Database<V, T> {
    store: Store<V, T>,
    index: Index<V, T>,
}

// NOTE: This is going to be single threaded as it's currently conceived :(
impl Database<V, T> {

    // NOTE: mutability here means we have to serialize writes :(
    fn insert(&mut self, value: V) {
        // How do we prevent duplicate inserts? (do we?)

        // insert the data into the store so its lifetime can be managed
        // this operation may cause store eviction, which needs to be handled
        let (token, evictions) = self.store.insert(value);

        // index the data so it can be found efficiently based on related info
        self.index.insert(token, value);

        // remove evicted pages from other stores
        for eviction in evictions.iter() {
            self.index.remove(eviction.token, eviction.value);
        }
    }

    // NOTE: mutability here means we have to serialize reads :(
    fn query(&mut self, query: &V) -> HashSet<V> {
        // Fetch pages from the index given the query predicate
        let result_tokens = self.index.find(query);
        let result_values: HashSet<V> = HashSet::with_capacity(result_tokens.len());

        for token in result_tokens.iter() {
            // Fetch pointer to stored data for each related page
            let value = self.store.get(token);

            // Post-process/filter results here if needed

            return_data.push(value);
        }
        
        return_data
    }
}
