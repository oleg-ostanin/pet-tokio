use std::collections::HashMap;
use crate::common::Types;

fn default_map(size: usize) -> HashMap<i32, Types> {
    let mut res = HashMap::new();
    for i in 0..size {
        res.insert(i as i32, Types::new(i as i32));
    }
    res
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::mem;
    use crate::common::Types;
    use crate::iter::map::default_map;

    #[test]
    // iter does not consume values
    fn iter() {
        let map = default_map(7);
        map.iter().for_each(
            // t is a reference
            |(k, t)| println!("{:?}", t)
        );
        assert_eq!(7, map.len());
    }

    #[test]
    fn iter_mut_swap() {
        let mut map = default_map(7);
        let mut taken: Option<String> = None;
        map.iter_mut().for_each(
            |(k, t)| {
                println!("{:?}", t);
                if *k == 5 {
                    mem::swap(&mut t.line_option, &mut taken);
                }
            }
        );
        assert_eq!(7, map.len());
        assert!(map.get(&5).is_some());
        assert!(map.get(&5).unwrap().line_option.is_none());
        assert!(taken.is_some());
        assert_eq!(taken.unwrap(), "5");
    }

    #[test]
    fn iter_mut_replace() {
        let mut map = default_map(7);
        let mut replaced = Some(Types::new(56));
        map.iter_mut().for_each(
            |(k, t)| {
                println!("{:?}", t);
                if *k == 5 {
                    replaced = Some(mem::replace(t, replaced.take().unwrap()));
                }
            }
        );
        assert_eq!(7, map.len());
        assert!(map.get(&5).is_some());
        assert!(map.get(&5).unwrap().line_option.is_some());
        assert_eq!(map.get(&5).unwrap().clone().line_option.unwrap(), "56");
        assert_eq!(replaced.unwrap().line_option.unwrap(), "5");
    }

    #[test]
    fn iter_mut_swap_maps() {
        let mut map = default_map(7);
        let mut other_map: HashMap<i32, Types> = HashMap::new();
        other_map.insert(0, Types::new(100));
        other_map.insert(1, Types::new(101));

        for i in 0..other_map.len() {
            let key = i as i32;
            let mut from_map = map.get_mut(&key);
            let mut from_other_map = other_map.get_mut(&key);
            if let Some(from_map) = from_map {
                if let Some(from_other_map) = from_other_map {
                    mem::swap(from_map, from_other_map);
                }
            }
        }
        assert_eq!(map.get(&0).unwrap().num, 100);
        assert_eq!(other_map.get(&0).unwrap().num, 0);
    }
}