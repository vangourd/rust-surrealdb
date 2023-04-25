#![allow(unused)]
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use surrealdb::sql::{thing, Datetime, Object, Thing, Value};
use surrealdb::kvs::Datastore;
use surrealdb::dbs::{Session, Response};

type DB = (Datastore, Session);

#[tokio::main]
async fn main() -> Result<()> {
    let db: &DB = &(Datastore::new("memory").await?, Session::for_db("my_ns","my_db"));
    let (ds, ses) = db;

    // --- Create
    create_task(db, "Task 01", 10).await?;
    create_task(db, "Task 02", 7).await?;
    
    // --- Select
    let sql = "SELECT * from task";
    let ress = ds.execute(sql, ses, None, false).await?;
    for object in into_iter_objects(ress)? {
        println!("record {}", object?);
    }

    Ok(())
}

async fn create_task((ds, ses): &DB, title: &str, priority:i32) -> Result<String> {
    let sql = "CREATE task CONTENT $data";

    let data: BTreeMap<String, Value> = [
        ("title".into(), title.into()),
        ("priority".into(), priority.into()),
    ]
    .into();
    let vars: BTreeMap<String, Value> = [("data".into(), data.into())].into();

    let ress = ds.execute(sql, ses, Some(vars), false).await?;

    into_iter_objects(ress)?
        .next()
        .transpose()?
        .and_then(|obj| obj.get("id").map(|id| id.to_string()))
        .ok_or_else(|| anyhow!("No id returned."))

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
        },
        _ => Err(anyhow!("No records found.")),
    }
}