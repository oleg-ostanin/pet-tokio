#[derive(Default, Debug, Clone)]
pub(crate) struct Customer {
    id: u32,
    phone: String,
}

const PHONE_TEMPLATE: &str = "+79112120000";

impl Customer {
    pub(crate) fn new(id: u32) -> Self {
        let id_str = id.to_string();
        let id_len = id_str.len();
        let zeros = PHONE_TEMPLATE.split_at(8 + id_len).1;
        zeros.to_string().push_str(id_str.as_str());
        let phone = PHONE_TEMPLATE.to_string().replace("0000", zeros);
        Self { id, phone }
    }
}