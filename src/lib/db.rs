use super::hGet;
use super::post;
use super::Method;
use super::Response;
use anyhow::Result;
use chrono::prelude::*;
use log::info;
use rusqlite::named_params;
use rusqlite::{Connection, Rows};
use tabled::Tabled;

/// 输出给用户可见的对象
#[derive(Debug, Tabled)]
pub struct OutRecord {
    pub id: String,
    pub state: String,
    pub info: String,
    pub method: Method,
    pub url: String,
    pub response_code: String,
    pub response: String,
    pub create_at: String,
    pub update_at: String,
}
#[derive(Debug)]
pub struct InRecord {
    pub id: String,
    pub state: String,
    pub info: Option<String>,
    pub notify: Option<Notify>,
}

/// 是否要通知
#[derive(Debug)]
pub struct Notify {
    pub method: Method,
    pub url: String,
}

/// 数据库每一行对应的对象
#[derive(Debug)]
struct TableItem {
    pub id: String,
    pub state: String,
    pub info: Option<String>,
    pub method: Option<Method>,
    pub url: Option<String>,
    pub response_code: Option<u16>,
    pub response: Option<String>,
    pub create_at: i64,
    pub update_at: i64,
}

impl TableItem {
    fn show(self) -> OutRecord {
        OutRecord {
            id: self.id,
            state: self.state,
            info: self.info.unwrap_or("".to_string()),
            method: self.method.unwrap_or_else(|| Method::None),
            url: self.url.unwrap_or_else(|| "".to_string()),
            response_code: self
                .response_code
                .map(|it| it.to_string())
                .unwrap_or_else(|| "".to_string()),
            response: self.response.unwrap_or_else(|| "".to_string()),
            create_at: Self::from_time(self.create_at),
            update_at: Self::from_time(self.update_at),
        }
    }

    fn from_time(time: i64) -> String {
        Local.timestamp_millis(time).format("%F %T").to_string()
    }
}
use std::env;
fn get_db() -> Result<Connection> {
    let db_name = "kvlet.db";
    let db_path = match env::var("KVLET_DB_PATH") {
        Ok(dir) => format!("{}/{}", dir, db_name),
        _ => format!("./{}", db_name),
    };

    let conn = Connection::open(db_path)?;
    conn.execute(
        "create table if not exists kvlet (
             id TEXT primary key,
             state TEXT not null,
             info TEXT,
             method TEXT,
             url TEXT,
             response_code INTEGER,
             response TEXT,
             create_at INTEGER not null,
             update_at INTEGER not null
         )",
        [],
    )?;
    Ok(conn)
}

fn set_into_db(conn: &Connection, record: InRecord) -> Result<InRecord> {
    fn compose(m: Option<Method>, u: &Option<String>) -> Option<Notify> {
        match (m, u) {
            (Some(method), Some(url)) => Some(Notify {
                method: method,
                url: url.to_string(),
            }),
            _ => None,
        }
    }
    match get_table_item(&record.id, None)? {
        Some(TableItem {
            method, url, info, ..
        }) => update(
            &conn,
            InRecord {
                notify: record.notify.or(compose(method, &url)),
                info: record.info.or(info),
                ..record
            },
        ),
        None => create(&conn, record),
    }
}

// notify if url exist
fn notify(record: InRecord) -> Result<Option<Response>> {
    let InRecord {
        id,
        state,
        notify,
        info,
    } = record;
    match notify {
        Some(Notify {
            method: Method::Get,
            url,
        }) => hGet(&id, &state, &url).and_then(|it| Ok(Some(it))),
        Some(Notify {
            method: Method::Post,
            url,
        }) => post(&id, &state, &info, &url).and_then(|it| Ok(Some(it))),
        _ => Ok(None),
    }
}

pub fn set(record: InRecord) -> Result<Option<Response>> {
    let conn = get_db()?;
    set_into_db(&conn, record)
        .and_then(notify)
        .and_then(|op| match op {
            Some(Response {
                id,
                status_code,
                body,
            }) => update_response(&conn, &id, status_code, &body).and_then(|_| {
                Ok(Some(Response {
                    id,
                    status_code,
                    body,
                }))
            }),
            None => Ok(None),
        })
}

fn create(conn: &Connection, record: InRecord) -> Result<InRecord> {
    info!("create {:#?}", record);
    let now = Local::now().timestamp_millis();
    let mut stmt = conn.prepare("INSERT INTO kvlet (id, state, info,method,url, create_at,update_at) VALUES (:id, :state,:info, :method,:url, :create_at,:update_at)")?;
    let InRecord {
        id,
        state,
        notify,
        info,
    } = record;
    let info_str = match &info {
        Some(d) => d.to_string(),
        None => "".to_string(),
    };
    match &notify {
        Some(Notify { method, url }) => stmt.execute(named_params! {
           ":id": &id,
           ":state": &state,
           ":info": info_str,
           ":method": method.to_string(),
           ":url": url,
           ":create_at": now,
           ":update_at": now
        })?,
        None => stmt.execute(named_params! {
           ":id": &id,
           ":state": &state,
           ":info": info_str,
           ":method": "",
           ":url": "",
           ":create_at": now,
           ":update_at": now
        })?,
    };
    Ok(InRecord {
        id,
        state,
        notify,
        info,
    })
}

fn update_response(conn: &Connection, id: &str, status_code: u16, body: &str) -> Result<usize> {
    let now = Local::now().timestamp_millis();
    let affected =   conn
            .prepare("UPDATE kvlet set response_code = :response_code, response = :response, update_at = :update_at where id = :id")?
            .execute(named_params! {
               ":id": id,
               ":response_code": status_code,
               ":response": body,
               ":update_at": now
            })?;
    Ok(affected)
}

fn update(conn: &Connection, record: InRecord) -> Result<InRecord> {
    info!("update {:#?}", record);
    let now = Local::now().timestamp_millis();
    let InRecord {
        id,
        state,
        notify,
        info,
    } = record;
    let info_str = match &info {
        Some(d) => d.to_string(),
        None => "".to_string(),
    };
    match &notify {
        Some(Notify { method, url }) => conn
            .prepare("UPDATE kvlet set state = :state, info = :info, method = :method, url = :url, update_at = :update_at where id = :id")?
            .execute(named_params! {
               ":id": id,
               ":state": state,
               ":info": info_str,
               ":method": method.to_string(),
               ":url": url,
               ":update_at": now
            })?,
        None => conn
            .prepare("UPDATE kvlet set state = :state, info = :info, update_at = :update_at  where id = :id")?
            .execute(named_params! {
               ":id": id,
               ":state": state,
               ":info": info_str,
               ":update_at": now
            })?,
    };
    Ok(InRecord {
        id,
        state,
        notify,
        info,
    })
}

fn update_url(conn: &Connection, id: &str, notify: Notify) -> Result<()> {
    info!("update url {} {:?}", id, &notify);
    let now = Local::now().timestamp_millis();
    let Notify { method, url } = notify;
    conn.prepare(
        "UPDATE kvlet set method = :method, url = :url, update_at = :update_at where id = :id",
    )?
    .execute(named_params! {
       ":id": id,
       ":method": method.to_string(),
       ":url": url,
       ":update_at": now
    })?;
    Ok(())
}

pub fn get(id: &str, notify: Option<Notify>) -> Result<Option<OutRecord>> {
    get_table_item(id, notify).map(|o| o.map(|t| t.show()))
}

fn get_table_item(id: &str, notify: Option<Notify>) -> Result<Option<TableItem>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet where id = :id")?;
    let mut rows = stmt.query(&[(":id", id)])?;
    let records = parse_rows(&mut rows)?;
    for record in records {
        if let Some(notify) = notify {
            update_url(&conn, id, notify)?
        }
        return Ok(Some(record));
    }
    Ok(None)
}

pub fn list(num: usize, state: &Option<String>) -> Result<Vec<OutRecord>> {
    match state {
        Some(s) => list_filter(num, s),
        None => list_num(num),
    }
}

fn list_num(num: usize) -> Result<Vec<OutRecord>> {
    let conn = get_db()?;
    let mut stmt = conn.prepare("SELECT * FROM kvlet ORDER BY create_at DESC limit :num")?;
    let mut rows = stmt.query(&[(":num", &num.to_string())])?;
    let records = parse_rows(&mut rows)?;
    Ok(records.into_iter().map(|r| r.show()).collect())
}

fn list_filter(num: usize, state: &str) -> Result<Vec<OutRecord>> {
    let conn = get_db()?;
    let mut stmt = conn
        .prepare("SELECT * FROM kvlet where state = :state ORDER BY create_at DESC limit :num")?;
    let mut rows = stmt.query(&[(":num", &num.to_string()), (":state", &state.to_string())])?;
    let records = parse_rows(&mut rows)?;
    Ok(records.into_iter().map(|r| r.show()).collect())
}

fn parse_rows(rows: &mut Rows) -> Result<Vec<TableItem>> {
    let mut records = vec![];

    while let Some(row) = rows.next()? {
        let method: Option<String> = row.get(3)?;
        let method: Option<Method> = match method {
            Some(m) => Some(m.parse()?),
            None => None,
        };

        let info: Option<String> = row.get(2)?;
        records.push(TableItem {
            id: row.get(0)?,
            state: row.get(1)?,
            info: info,
            method: method,
            url: row.get(4)?,
            response_code: row.get(5)?,
            response: row.get(6)?,
            create_at: row.get(7)?,
            update_at: row.get(8)?,
        });
    }
    Ok(records)
}
