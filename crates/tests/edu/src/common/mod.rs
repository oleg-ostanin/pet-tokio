pub mod order;
pub mod customer;
pub mod storage;
pub mod error;
pub mod processors;

#[derive(Default, Debug, Clone)]
pub(crate) struct Types {
    pub(crate) num: i32,
    num_option: Option<i32>,
    line: String,
    pub(crate) line_option: Option<String>,
}

impl Types {
    pub(crate) fn new(num: i32) -> Self {
        let line = num.to_string();
        let line_for_opt = num.to_string();
        Types {
            num,
            num_option: Some(num),
            line,
            line_option: Some(line_for_opt),
        }
    }
}