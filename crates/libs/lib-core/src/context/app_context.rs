use std::ops::Deref;
use std::sync::Arc;

use tokio_postgres::Client;


pub struct ModelManager {
    client: Arc<Client>,
}

impl ModelManager {
    pub fn create(client: Arc<Client>) -> ModelManager {
        ModelManager {
            client
        }
    }

    pub fn client(&self) -> &Client {
        self.client.deref()
    }
}