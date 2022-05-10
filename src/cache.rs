use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

pub struct LRUClockCache<K, V> {
    size: usize,
    mutex: Mutex<()>,
    key_buf: Vec<K>,
    value_buf: Vec<V>,
    chance_to_survive_buf: Vec<u8>,
    is_edited_buf:Vec<u8>,
    mapping: HashMap<K,usize>,
    load_data: Box<dyn Fn(K) -> V>,
    save_data: Box<dyn FnMut(K, V)>,
    ctr: usize,
    ctr_evict: usize,
}

enum Opt {
    Read,
    Write,
}

impl<K, V> LRUClockCache<K, V>
    where K: Clone + Default + Hash + Eq,
          V: Clone + Default,
{
    pub fn new(size: usize, load_data: Box<dyn Fn(K) -> V>, save_data: Box<dyn FnMut(K, V)>) -> Self {
        Self{
            size,
            mutex: Mutex::new(()),
            key_buf: vec![K::default(); size],
            value_buf: vec![V::default(); size],
            chance_to_survive_buf: vec![0; size],
            is_edited_buf: vec![0; size],
            mapping: HashMap::with_capacity(size),
            load_data,
            save_data,
            ctr: 0,
            ctr_evict: size / 2,
        }
    }

    pub fn set(&mut self, key: K, val: V) {
        self.mutex.lock();
        self.access_clock(key, Some(val), Opt::Write);
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        self.access_clock(key, None, Opt::Read)
    }

    fn access_clock(&mut self, key: K, val: Option<V>, op: Opt) -> Option<V> {
        match self.mapping.get(&key){
            Some(&index) => {
                self.chance_to_survive_buf[index] = 1;
                match op {
                    Opt::Write => {
                        self.is_edited_buf[index] = 1;
                        if let Some(val) = val {self.value_buf[index] = val;}
                    }
                    Opt::Read =>{return Some(self.value_buf[index].clone());},
                }
            },
            None =>{
                let mut ctr_found = self.size+1;
                let mut old_val: V = V::default();
                let mut old_key: K = K::default();

                while ctr_found > self.size{
                    if self.chance_to_survive_buf[self.ctr] > 0 {
                        self.chance_to_survive_buf[self.ctr] = 0;
                    }

                    self.ctr +=1;
                    if self.ctr >= self.size {self.ctr = 0;}

                    if self.chance_to_survive_buf[self.ctr_evict] == 0 {
                        ctr_found = self.ctr_evict;
                        old_val = self.value_buf[ctr_found].clone();
                        old_key = self.key_buf[ctr_found].clone();
                    }

                    self.ctr_evict += 1;
                    if self.ctr_evict >= self.size {self.ctr_evict = 0;}
                }

                if self.is_edited_buf[ctr_found] == 1 {
                    (self.save_data)(old_key, old_val);
                }

                match op {
                    Opt::Write => {
                        self.mapping.remove(&self.key_buf[ctr_found]);
                        self.value_buf[ctr_found] = val.unwrap();
                        self.chance_to_survive_buf[ctr_found] = 0;
                        self.mapping.insert(key.clone(), ctr_found);
                        self.key_buf[ctr_found] = key;
                    }
                    Opt::Read => {
                        let loaded_data = (self.load_data)(key.clone());
                        self.is_edited_buf[ctr_found] = 0;
                        self.mapping.remove(&self.key_buf[ctr_found]);
                        self.value_buf[ctr_found] = loaded_data.clone();
                        self.chance_to_survive_buf[ctr_found] = 0;
                        self.mapping.insert(key.clone(), ctr_found);
                        self.key_buf[ctr_found] = key;
                        return Some(loaded_data);
                    }
                }
            }

        }
        None
    }
}

