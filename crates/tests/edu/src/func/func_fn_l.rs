struct BoxedCalc<'a> {
    pub boxed_calc: Box<(dyn Fn(i32, i32) -> i32 + 'a)>,
}

impl<'a> BoxedCalc<'a> {
    pub fn new(boxed_calc: Box<(dyn Fn(i32, i32) -> i32 + 'a)>) -> Self {
        Self { boxed_calc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_stack() {
        let add: fn(i32, i32) -> i32 = |a, b| a + b;
        let add_instance = BoxedCalc::new(Box::new(add));

        let on_the_stack = 42;
        let add_from_the_stack = |a, _| { a + on_the_stack};

        let on_the_stack_instance = BoxedCalc::new(Box::new(add_from_the_stack));
        let vec = vec![add_instance, on_the_stack_instance];
        let values = vec![1, 2];
        let mut res = 0;
        vec.into_iter().enumerate().for_each(
            |(i, instance)| {
                let value = values[i];
                let func = instance.boxed_calc;
                res = func(res, value);
            }
        );
        assert_eq!(43, res)
    }
}