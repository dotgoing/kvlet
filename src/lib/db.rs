use anyhow::{anyhow, Result};
use rusqlite::named_params;
use rusqlite::Connection;

#[derive(Debug)]
pub struct Record {
    pub id: String,
    pub state: String,
    pub url: Option<String>,
    pub method: Option<String>,
    pub response_code: Option<usize>,
    pub response: Option<String>,
}

fn get_db() -> Result<Connection> {
    let conn = Connection::open("kvlet.db")?;
    conn.execute(
        "create table if not exists kvlet (
             id TEXT primary key,
             state TEXT not null,
             url TEXT,
             method TEXT,
             response_code INTEGER,
             response TEXT
         )",
        [],
    )?;
    Ok(conn)
}

pub fn set(id: &str, state: &str) -> Result<()> {
    let conn = get_db()?;
    let err = |_| anyhow!(format!("fail to save {}->{}", id, state));

    let mut stmt = conn
        .prepare("INSERT INTO kvlet (id, state) VALUES (:id, :state)")
        .map_err(err)?;
    stmt.execute(named_params! { ":id": id, ":state": state })
        .map_err(err)?;
    Ok(())
}

pub fn get(id: &str) -> Result<Vec<Record>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet where id = :id")?;
    let mut rows = stmt.query(&[(":id", id)])?;
    let mut records = vec![];
    while let Some(row) = rows.next()? {
        let r = Record {
            id: row.get(0)?,
            state: row.get(1)?,
            url: None,
            method: None,
            response_code: None,
            response: None,
        };
        records.push(r);
    }
    Ok(records)
}

pub fn list(num: usize) -> Result<Vec<Record>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet limit :num")?;
    let mut rows = stmt.query(&[(":num", &num.to_string())])?;
    let mut records = vec![];
    while let Some(row) = rows.next()? {
        let r = Record {
            id: row.get(0)?,
            state: row.get(1)?,
            url: None,
            method: None,
            response_code: None,
            response: None,
        };
        records.push(r);
    }
    Ok(records)
}
