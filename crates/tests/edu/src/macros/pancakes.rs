use procedural_macro::{HelloMacro};
use builder_macro::{HelloBuilder};

pub trait HelloMacro {
    fn hello_macro();
}

pub trait HelloBuilder {
    fn hello_builder();
}

#[derive(HelloMacro)]
struct Pancakes;

#[derive(HelloBuilder)]
struct NilsCakes {
    food: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        Pancakes::hello_macro();
        NilsCakes::hello_builder();
    }
}