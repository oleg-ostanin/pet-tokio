use std::time::Duration;
use lib_dto::order::{OrderContent, OrderItem, OrderStatus};
use crate::ITERATIONS;
use crate::requests::user_context::UserContext;
use crate::scenario::common::{BOOKS_SIZE, check_order, create_order};

// todo move to common
pub async fn load(users: Vec<UserContext>) -> Vec<UserContext> {
    let mut users_res = Vec::with_capacity(users.len());
    let mut jhs = Vec::with_capacity(users.len());
    for user in users.into_iter() {
        let jh = tokio::spawn(load_user(user));
        jhs.push(jh);
    }
    for jh in jhs.into_iter() {
        users_res.push(jh.await.expect("must be ok"));
    }
    users_res
}

async fn load_user(mut user: UserContext) -> UserContext {
    let user_idx = user.idx();
    let mut orders = vec![];
    for i in 1..=ITERATIONS {
        let mut items = Vec::with_capacity(BOOKS_SIZE);
        for j in 0..BOOKS_SIZE {
            let book_id = j as i64 + 1;
            let quantity = user_idx as i64 * i;
            let item = OrderItem::new(book_id, quantity);
            items.push(item);
        }
        let order_content = OrderContent::new(items);
        let order_id = create_order(&mut user, order_content).await;
        orders.push(order_id);
    }

    tokio::time::sleep(Duration::from_secs(5)).await;

    for order_id in orders.into_iter() {
        let order_stored = check_order(&mut user, order_id).await;
        assert_eq!(&OrderStatus::Delivered, order_stored.status());
    }

    user
}