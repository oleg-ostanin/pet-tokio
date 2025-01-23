use uuid::Uuid;
use lib_dto::user::UserForCreate;
use crate::bmc::scheme::Scheme;
use crate::context::app_context::ModelManager;

use super::{Result, Error};

pub struct UserBmc;
//todo return identity type
const INSERT_USER: &str = r#"
INSERT INTO users
(identity, first_name, last_name, pwd
  , pwd_salt, token_salt
  --, created_at, updated_at todo
)
VALUES
($1, $2, $3, $4
  , $5, $6
  -- , $7, $8 todo
)
RETURNING id;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM users WHERE id=$1;
"#;

const SELECT_BY_IDENTITY: &str = r#"
SELECT * FROM users WHERE identity=$1;
"#;

impl UserBmc {
    pub async fn create(
        mm: &ModelManager,
        user: UserForCreate,
    ) -> Result<()> {
        let pwd_salt = Uuid::new_v4();
        let to_hash = ContentToHash {
            content: user.password.clone(),
            salt: pwd_salt,
        };

        let pwd_hashed = hash_pwd(to_hash).await?;

        let token_salt = Uuid::new_v4().to_string();

        // mm.client().execute(INSERT_USER, &[&user.identity, &user.first_name,
        //     &user.last_name, &pwd_hashed, &pwd_salt.to_string(), &token_salt]).await?;

        Ok(())
    }

    // pub async fn get_by_id(
    //     mm: &ModelManager,
    //     id: i64,
    // ) -> Result<UserStored> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_ID, &[&id]).await?;
    //
    //     println!("{:?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserStored::try_from(v)
    // }
    //
    // pub async fn get_for_auth(
    //     mm: &ModelManager,
    //     identity: &String,
    // ) -> Result<UserForAuth> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_IDENTITY, &[identity]).await?;
    //
    //     println!("{:?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserForAuth::try_from(v)
    // }
    //
    // // todo make these two functions generic
    // pub async fn get_for_login(
    //     mm: &ModelManager,
    //     identity: &String,
    // ) -> Result<UserForLogin> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_IDENTITY, &[identity]).await?;
    //
    //     println!("{:?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserForLogin::try_from(v)
    // }
}

/// The clean content to hash, with the salt.
///
/// Notes:
///    - Since content is sensitive information, we do NOT implement default debug for this struct.
///    - The clone is only implement for testing
#[cfg_attr(test, derive(Clone))]
pub struct ContentToHash {
    pub content: String, // Clear content.
    pub salt: Uuid,
}

// endregion: --- Types

// region:    --- Public Functions

/// Hash the password with the default scheme.
pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
    let s = Scheme;

    tokio::task::spawn_blocking(move || s.hash(&to_hash))
        .await
        .map_err(|_| Error::SomeError)?

}