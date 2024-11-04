use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::str::FromStr;
use std::io::Read;
use std::io::Write;

pub struct FileCache {
    pub file: File,
    pub line_separator: String,
    pub kv_separator: String,

    m: HashMap<String, String>,
}


impl FileCache {
    pub fn new(path: &str) -> Self {
        let mut file = fs::OpenOptions::new().read(true).write(true).create(true).append(true).open(path).unwrap();
        let mut content = String::new();
        let file_size = file.read_to_string(&mut content).unwrap();
        println!("[FileCache] new with file:{:?}, size:{:?}", path, file_size);

        let mut fc = FileCache {
            file,
            line_separator: "\n".to_string(),
            kv_separator: "----".to_string(),
            m: Default::default(),
        };

        content.split(&fc.line_separator).for_each(
            |x| {
                let line = x.split(&fc.kv_separator).collect::<Vec<&str>>();
                if line.len() < 2 {
                    return;
                }
                fc.m.insert(line.get(0).unwrap().to_string(), line.get(1).unwrap().to_string());
            }
        );
        fc
    }

    pub fn get<K: ToString, V: FromStr>(&self, k: &K) -> Option<V>
    where
        <V as FromStr>::Err: std::fmt::Debug,
    {
        match self.m.get(k.to_string().as_str()) {
            None => None,
            Some(s) => {
                Some(V::from_str(s.as_str()).unwrap())
            }
        }
    }
    pub fn set<K: ToString, V: ToString>(&mut self, k: &K, v: &V) {
        let k = k.to_string();
        let v = v.to_string();
        let line = String::new() + k.as_str() + &self.kv_separator + v.as_str() + &self.line_separator;
        self.m.insert(k, v);
        self.file.write(line.as_bytes()).unwrap();
    }
}