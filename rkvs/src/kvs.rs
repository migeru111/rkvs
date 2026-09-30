use std::sync::Mutex;
use std::collections::HashMap;

pub struct Kvs {
    m: Mutex<HashMap<String,String>>
}

impl Kvs {
    pub fn new()-> Self{
        Kvs{m:Mutex::new(HashMap::new())}
    }

    pub fn get(&self,key: &str) -> Option<String>{
        let mg=self.m.lock().unwrap();
        mg.get(key).cloned()
    }
    pub fn set(&self,key:&str,value:&str)->Option<String>{
        let mut mg = self.m.lock().unwrap();
        mg.insert(key.to_string(),value.to_string())
    }
    pub fn del(&self,key: &str) -> Option<String>{
        let mut mg= self.m.lock().unwrap();
        mg.remove(key)
    }
}
