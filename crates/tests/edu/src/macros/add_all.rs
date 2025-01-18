#[macro_export]
macro_rules! add_as {
    (
        // repeated block
        $($a:expr)
        // seperator
        ,
        // zero or more
        *
    )=>{
           {
            // to handle the case without any arguments
            0
            // block to be repeated
            $(+$a)*
            }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_macro() {
        println!("{}", add_as!(1,2,3,4)); // => println!("{}",{0+1+2+3+4})
    }
}

