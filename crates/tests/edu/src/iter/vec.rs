use crate::common::Types;

fn default_vec(size: usize) -> Vec<Types> {
    let mut res = Vec::new();
    for i in 0..size {
        res.push(Types::new(i as i32))
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::iter::vec::default_vec;

    #[test]
    // iter does not consume values
    fn iter() {
        let vec = default_vec(7);
        vec.iter().for_each(
            // t is a reference
            |t| println!("{:?}", t)
        );
        assert_eq!(7, vec.len());
    }

    #[test]
    // into_iter does consume values
    fn into_iter() {
        let vec = default_vec(7);
        vec.into_iter().for_each(
            // t is owned
            |t| println!("{:?}", t)
        );
        // assert_eq!(7, vec.len());  Compilation error value borrowed here after move
    }

    #[test]
    fn enumerate() {
        let vec = default_vec(3);
        let mut total = 0;
        vec.iter().enumerate().for_each(
            |(i, _)| {
                total = total + i;
            }
        );
        // 0 + 1 + 2
        assert_eq!(3, total);
    }

    #[test]
    fn enumerate_mut() {
        let mut vec = default_vec(7);
        let mut taken: Option<String> = None;
        vec.iter_mut().enumerate().for_each(
            |(i, types)| {
                if i == 5 {
                    taken = types.line_option.take();
                }
            }
        );
        assert!(vec.get(5).is_some());
        assert!(vec.get(5).unwrap().line_option.is_none());
        assert!(taken.is_some());
        assert_eq!(taken.unwrap(), "5");
    }
}