#[macro_export]
macro_rules! all_true {
    ( $( $x:expr ),* ) => {
        {
            let mut res = true;
            'block: {
            $(
                if(!$x) {
                    res = false;
                    break 'block;
                }
            )*
            }
            res
        }
    };
}

fn always_true() -> bool {
    true
}

fn equals(x: i32, y: i32) -> bool {
    x == y
}

fn expanded() {
    let res = all_true!(always_true(), equals(2, 3), equals(3, 3), equals(3, 3));
    println!("res: {:?}", res);
}

#[cfg(test)]
mod tests {
    use crate::macros::all_true::{always_true, equals};

    #[test]
    fn check_macro() {
        let res = all_true!(always_true(), equals(2, 3), equals(3, 3), equals(3, 3));
        println!("res: {:?}", res);
    }
}

