use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use lib_dto::book::BookList;
    use lib_utils::json::value;

    use crate::context::context::{ServiceType, TestContext};
    use crate::dev::web::login;
    use crate::dev::web::request;
    use crate::utils::file_utils::from_file;

    #[tokio::test]
    async fn add_books() {
        let mut ctx = TestContext::new(ServiceType::Web).await;
        login(&mut ctx).await;

        let book_list: BookList = from_file("books_refactored.json");
        let request = request("add_books", Some(book_list));

        let rpc_response = ctx.post("/api/rpc", request).await;

        println!("{:?}", &rpc_response);
        let value = value(rpc_response).await;
        println!("{:?}", &value);
    }
}