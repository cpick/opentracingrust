use std::collections::BTreeMap;
use std::collections::HashMap;


/// TODO
pub trait MapCarrier {
    /// TODO
    ///
    /// NOTE:
    ///   This is not the most efficient interface to extract baggage items.
    ///   The iterator interface cannot cleanly be used because we want
    ///   `TextMapCarrier` trait objects (which do not allow generics).
    ///
    ///   If a better interface comes up re-evaluate this method.
    fn find_items(&self, f: Box<Fn(&String) -> bool>) -> Vec<(String, String)>;

    /// TODO
    fn get(&self, key: &str) -> Option<String>;

    /// TODO
    fn set(&mut self, key: &str, value: &str);
}

impl MapCarrier for HashMap<String, String> {
    fn find_items(&self, f: Box<Fn(&String) -> bool>) -> Vec<(String, String)> {
        self.iter()
            .filter(|&(k, _)| f(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| v.clone())
    }

    fn set(&mut self, key: &str, value: &str) {
        self.insert(String::from(key), String::from(value));
    }
}

impl MapCarrier for BTreeMap<String, String> {
    fn find_items(&self, f: Box<Fn(&String) -> bool>) -> Vec<(String, String)> {
        self.iter()
            .filter(|&(k, _)| f(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn get(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| v.clone())
    }

    fn set(&mut self, key: &str, value: &str) {
        self.insert(String::from(key), String::from(value));
    }
}


#[cfg(test)]
mod tests {
    mod tree_map {
        use std::collections::BTreeMap;
        use super::super::MapCarrier;

        #[test]
        fn extract_keys() {
            let mut tree: BTreeMap<String, String> = BTreeMap::new();
            tree.insert(String::from("aa"), String::from("d"));
            assert_eq!(tree.get("aa").unwrap(), "d");
        }

        #[test]
        fn find_keys() {
            let mut tree: BTreeMap<String, String> = BTreeMap::new();
            tree.insert(String::from("aa"), String::from("d"));
            tree.insert(String::from("ab"), String::from("e"));
            tree.insert(String::from("bc"), String::from("f"));

            let mut items = tree.find_items(Box::new(|k| k.starts_with("a")));
            items.sort();
            assert_eq!(items, [
                (String::from("aa"), String::from("d")),
                (String::from("ab"), String::from("e"))
            ]);
        }

        #[test]
        fn inject_keys() {
            let mut tree: BTreeMap<String, String> = BTreeMap::new();
            tree.set("a", "d");
            tree.set("b", "e");
            tree.set("c", "f");
            assert_eq!("d", tree.get("a").unwrap());
            assert_eq!("e", tree.get("b").unwrap());
            assert_eq!("f", tree.get("c").unwrap());
        }
    }

    mod hash_map {
        use std::collections::HashMap;
        use super::super::MapCarrier;

        #[test]
        fn extract_keys() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.insert(String::from("aa"), String::from("d"));
            assert_eq!(map.get("aa").unwrap(), "d");
        }

        #[test]
        fn find_keys() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.insert(String::from("aa"), String::from("d"));
            map.insert(String::from("ab"), String::from("e"));
            map.insert(String::from("bc"), String::from("f"));

            let mut items = map.find_items(Box::new(|k| k.starts_with("a")));
            items.sort();
            assert_eq!(items, [
                (String::from("aa"), String::from("d")),
                (String::from("ab"), String::from("e"))
            ]);
        }

        #[test]
        fn inject_keys() {
            let mut map: HashMap<String, String> = HashMap::new();
            map.set("a", "d");
            map.set("b", "e");
            map.set("c", "f");
            assert_eq!("d", map.get("a").unwrap());
            assert_eq!("e", map.get("b").unwrap());
            assert_eq!("f", map.get("c").unwrap());
        }
    }
}