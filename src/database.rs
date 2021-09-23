use rusqlite::Result;

#[derive(Debug)]
pub struct ItemData {
    pub id: i32,
    pub time_to_prepare: i32,
    pub name: String,
}

#[derive(Debug)]
pub struct TableData {
    pub id: i32,
    pub item_ids: String,
}

pub fn setup_db() -> Result<String, String> {
    let db_connection = match rusqlite::Connection::open("data.sqlite") {
        Ok(connection) => connection,
        Err(_) => {
            return Err("Cannot connect to database.".into());
        }
    };
    match db_connection.execute_batch(
        "create table if not exists item (
                id integer primary key,
                name varchar(64) not null,
                preperation_time integer not null
            );
            create table if not exists restaurant_table (
                id integer primary key,
                items varchar(64) not null
            );",
    ) {
        Ok(_) => Ok("Successfully created database tables.".into()),
        Err(_) => return Err("Could not run create table sql".into()),
    }
}

pub fn get_connection() -> rusqlite::Connection {
    rusqlite::Connection::open("data.sqlite").expect("Failed to get db connection.")
}

pub fn build_statement<'a>(connection: &'a rusqlite::Connection, statement:&str) -> rusqlite::Statement<'a> {
    connection.prepare(statement).expect("Failed to prepare query.")
}

pub fn setup_test_db(statement: &str) {
    setup_db().expect("Set up database.");
    add_test_items(statement);
}

pub fn close_test_db(statement: &str) {
    delete_test_items(statement);
}

fn add_test_items(statement: &str) {
    let db_connection = rusqlite::Connection::open("data.sqlite").expect("Failed to get db.");
    match db_connection.execute_batch(statement) {
        Ok(_) => {
            println!("Inserted test values")
        }
        Err(e) => {
            println!("Failed to inserted test values {}", e)
        }
    };
    println!("setup test db so we can test now");
}

fn delete_test_items(statement: &str) {
    let db_connection = rusqlite::Connection::open("data.sqlite").expect("Failed to get db.");
    match db_connection.execute_batch(statement) {
        Ok(_) => {
            println!("Deleted test values")
        }
        Err(e) => {
            println!("Failed to delete test values {}", e)
        }
    };
    println!("Deleted test items from db.");
}
