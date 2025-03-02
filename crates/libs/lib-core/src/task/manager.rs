use anyhow::{Result, Error};


pub(crate) struct TaskManager {

}

impl TaskManager {
    pub(crate) async fn create() -> Result<Self> {

        Ok(TaskManager {

        })
    }

    pub(crate) async fn start(&self) -> Result<()> {


        Ok(())
    }
}