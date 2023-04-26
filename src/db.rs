use redis::{Client, Connection};

const DB_ERR_MSG: &str = "failed to init db";

pub async fn init(db_url: &str) -> Result<Connection, &str> {
    let Ok(client) = Client::open(db_url) else {
        return Err(DB_ERR_MSG)
    };

    let Ok(conn) = client.get_connection() else {
        return Err(DB_ERR_MSG)
    };

    return Ok(conn);
}
