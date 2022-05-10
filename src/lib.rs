extern crate core;

mod cache;

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};
    use crate::cache::LRUClockCache;

    #[test]
    fn it_works() {
        let mut  data = Vec::with_capacity(100000);
        for i in 0..100000 {
            data.push(i*10);
        }
        let data_arc1 = Arc::new(Mutex::new(data));
        let data_arc2 = data_arc1.clone();
        let load_data =  move|key: usize|-> usize {

            let content = data_arc1.lock().unwrap();
            content[key]};
        let set_data =  move |key: usize, val: usize| {
            let mut content = data_arc2.lock().unwrap();
            content[key] = val};

        let mut cache = LRUClockCache::new(1000, Box::new(load_data), Box::new(set_data));

        for i in 0..100000 {
            assert_eq!(Some(i*10), cache.get(i));
        }
    }
}
