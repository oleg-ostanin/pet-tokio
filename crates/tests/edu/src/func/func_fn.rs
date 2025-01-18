struct Calc<T: Fn(i32, i32) -> i32> {
    pub calc: T,
}

impl<T: Fn(i32, i32) -> i32> Calc<T> {
    pub fn new(add_calc: T) -> Self {
        Self { calc: add_calc }
    }
}

struct BoxedCalc {
    pub boxed_calc: Box<dyn Fn(i32, i32) -> i32>,
}

impl BoxedCalc {
    pub fn new(boxed_calc: Box<dyn Fn(i32, i32) -> i32>) -> Self {
        Self { boxed_calc }
    }
}

fn calc_vec() -> Vec<BoxedCalc> {
    let add: fn(i32, i32) -> i32 = |a, b| a + b;
    let sub: fn(i32, i32) -> i32 = |a, b| a - b;
    let multiply: fn(i32, i32) -> i32 = |a, b| a * b;
    let add_instance= BoxedCalc::new(Box::new(add));
    let sub_instance = BoxedCalc::new(Box::new(sub));
    let multiply_instance = BoxedCalc::new(Box::new(multiply));
    let vec = vec![add_instance, sub_instance, multiply_instance];
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let add: fn(i32, i32) -> i32 = |a, b| a + b;
        let sum = add(3, 5);
        assert_eq!(8, sum)
    }

    #[test]
    fn add_struct() {
        let add: fn(i32, i32) -> i32 = |a, b| a + b;
        let add_instance = Calc::new(add);
        // does not compile!
        // let sum = add_instance.add_calc(3, 5);
        let sum = (add_instance.calc)(3, 5);
        assert_eq!(8, sum)
    }

    #[test]
    fn vec_iter() {
        let add: fn(i32, i32) -> i32 = |a, b| a + b;
        let sub: fn(i32, i32) -> i32 = |a, b| a - b;
        let multiply: fn(i32, i32) -> i32 = |a, b| a * b;
        let add_instance = Calc::new(add);
        let sub_instance = Calc::new(sub);
        let multiply_instance = Calc::new(multiply);
        let vec = vec![add_instance, sub_instance, multiply_instance];
        let values = vec![1, 2, 3];
        let mut res = 0;
        vec.iter().enumerate().for_each(
            |(i, instance)| {
                let value = values[i];
                res = (instance.calc)(res, value);
            }
        );
        assert_eq!(-3, res)
    }

    #[test]
    fn vec_into_iter() {
        let add: fn(i32, i32) -> i32 = |a, b| a + b;
        let sub: fn(i32, i32) -> i32 = |a, b| a - b;
        let multiply: fn(i32, i32) -> i32 = |a, b| a * b;
        let add_instance = Calc::new(add);
        let sub_instance = Calc::new(sub);
        let multiply_instance = Calc::new(multiply);
        let vec = vec![add_instance, sub_instance, multiply_instance];
        let values = vec![1, 2, 3];
        let mut res = 0;
        vec.into_iter().enumerate().for_each(
            |(i, instance)| {
                let value = values[i];
                res = (instance.calc)(res, value);
            }
        );
        assert_eq!(-3, res)
    }

    #[test]
    fn vec_boxed_iter() {
        let vec = calc_vec();
        let values = vec![1, 2, 3];
        let mut res = 0;
        vec.iter().enumerate().for_each(
            |(i, instance)| {
                let value = values[i];
                // does not compile, cannot move, need to borrow
                //let func = instance.boxed_calc;
                res = (instance.boxed_calc)(res, value);
            }
        );
        assert_eq!(-3, res)
    }

    #[test]
    fn vec_boxed_into_iter() {
        let vec = calc_vec();
        let values = vec![1, 2, 3];
        let mut res = 0;
        vec.into_iter().enumerate().for_each(
            |(i, instance)| {
                let value = values[i];
                let func = instance.boxed_calc;
                res = func(res, value);
            }
        );
        assert_eq!(-3, res)
    }

    // Does not compile! see func_fn_l.rs for the solution
    // #[test]
    // fn vec_stack() {
    //     let add: fn(i32, i32) -> i32 = |a, b| a + b;
    //     let add_instance = BoxedCalc::new(Box::new(add));
    //
    //     let on_the_stack = 42;
    //     let add_from_the_stack = |a, _| { a + on_the_stack};
    //
    //     let on_the_stack_instance = BoxedCalc::new(Box::new(add_from_the_stack));
    //     let vec = vec![add_instance, on_the_stack_instance];
    //     let values = vec![1, 2];
    //     let mut res = 0;
    //     vec.into_iter().enumerate().for_each(
    //         |(i, instance)| {
    //             let value = values[i];
    //             let func = instance.boxed_calc;
    //             res = func(res, value);
    //         }
    //     );
    //     assert_eq!(-3, res)
    // }
}