use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use magic_crypt::{new_magic_crypt, MagicCrypt256, MagicCryptTrait};
use surrealdb::{
    sql::{thing, Object, Value},
    Datastore, Response, Session,
};

pub struct Store {
    ds: Datastore,
    sess: Session,
    mc: MagicCrypt256,
}

// TODO: Make the key hidden
impl Store {
    pub async fn new(db_type: &str) -> Result<Self> {
        let mc = new_magic_crypt!("key", 256);
        let ds = Datastore::new(db_type).await?;
        let sess = Session::for_db("ns", "db");
        Ok(Self { ds, sess, mc })
    }

    pub async fn store_password(&self, username: String, password: String) -> Result<String> {
        let exists = self.get_id_for(username.clone()).await;
        if let Ok(_) = exists {
            return Err(anyhow!("Username already exists"));
        }

        let sql = "CREATE password CONTENT $data";

        let password = self.mc.encrypt_str_to_base64(password);
        let data: BTreeMap<String, Value> = [
            ("username".into(), username.into()),
            ("password".into(), password.into()),
        ]
        .into();

        let vars: BTreeMap<String, Value> = [("data".into(), data.into())].into();

        let res = self.ds.execute(sql, &self.sess, Some(vars), false).await?;

        into_iter_objects(res)?
            .next()
            .transpose()?
            .and_then(|obj| obj.get("id").map(|id| id.to_string()))
            .ok_or_else(|| anyhow!("No id returned."))
    }

    pub async fn get_password_for(&self, username: String) -> Result<String> {
        let sql = "SELECT * FROM password WHERE username = $username";
        let vars: BTreeMap<String, Value> = [("username".into(), username.into())].into();
        // let vars: BTreeMap<String, Value> = [()]

        let res = self.ds.execute(sql, &self.sess, Some(vars), false).await?;

        let pass = into_iter_objects(res)?
            .next()
            .transpose()?
            .and_then(|obj| obj.get("password").map(|pass| pass.to_string()))
            .ok_or_else(|| anyhow!("No password returned."))?;

        let pass = pass
            .split("\"")
            .filter(|x| x.len() > 0)
            .next()
            .ok_or_else(|| anyhow!("Could not trim \" from password string."))?;

        self.mc
            .decrypt_base64_to_string(pass)
            .map_err(|_| anyhow!("Could not decrypt password."))
    }

    pub async fn get_id_for(&self, username: String) -> Result<String> {
        let sql = "SELECT * FROM password WHERE username = $username";
        let vars: BTreeMap<String, Value> = [("username".into(), username.into())].into();
        // let vars: BTreeMap<String, Value> = [()]

        let res = self.ds.execute(sql, &self.sess, Some(vars), false).await?;
        into_iter_objects(res)?
            .next()
            .transpose()?
            .and_then(|obj| obj.get("id").map(|id| id.to_string()))
            .ok_or_else(|| anyhow!("No id returned."))
    }

    pub async fn update_password_for(
        &self,
        username: String,
        new_password: String,
    ) -> Result<String> {
        let id = self.get_id_for(username).await?;
        let new_password = self.mc.encrypt_str_to_base64(new_password);

        let sql = "UPDATE $th MERGE $data RETURN id;";
        let data: BTreeMap<String, Value> = [("password".into(), new_password.into())].into();
        let vars: BTreeMap<String, Value> = [
            ("th".into(), thing(&id)?.into()),
            ("data".into(), data.into()),
        ]
        .into();

        let res = self.ds.execute(sql, &self.sess, Some(vars), true).await?;

        into_iter_objects(res)?
            .next()
            .transpose()?
            .and_then(|obj| obj.get("id").map(|id| id.to_string()))
            .ok_or_else(|| anyhow!("No id returned."))
    }

    pub async fn delete_entry(&self, username: String) -> Result<()> {
        let id = self.get_id_for(username).await?;

        let sql = "DELETE $th";
        let vars: BTreeMap<String, Value> = [("th".into(), thing(&id)?.into())].into();

        self.ds.execute(sql, &self.sess, Some(vars), true).await?;

        Ok(())
    }
}

fn into_iter_objects(ress: Vec<Response>) -> Result<impl Iterator<Item = Result<Object>>> {
    let res = ress.into_iter().next().map(|rp| rp.result).transpose()?;

    match res {
        Some(Value::Array(arr)) => {
            let it = arr.into_iter().map(|v| match v {
                Value::Object(object) => Ok(object),
                _ => Err(anyhow!("A record was not an Object")),
            });
            Ok(it)
        }
        _ => Err(anyhow!("No records found.")),
    }
}
