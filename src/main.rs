#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rand::Rng;
use rocket_contrib::json::Json;
use rusqlite::Result;

mod data;
mod database;

#[get("/api/v1/tables")]
fn get_all_tables() -> Result<Json<Vec<data::Table>>, String> {
    println!("Getting all tables.");    let connection = &database::get_connection();
    let mut statement = database::build_statement(connection,  "select * from restaurant_table;");
    println!("Prepared statement {:?}.", statement);
    let results = statement.query_map([], |row| {
        Ok(database::TableData {
            id: row.get(0)?,
            item_ids: row.get(1)?,
        })
    });
    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();
            match collection {
                Ok(items) => {
                    let new_data = items
                        .iter()
                        .map(|table_data| data::Table {
                            id: table_data.id,
                            items: {
                                let id_list = Vec::from_iter(
                                    table_data.item_ids.split(",").map(|s| s.to_string()),
                                );
                                get_items(connection, id_list)
                            },
                        })
                        .collect();
                    println!("Finished getting all Tables {:?}.", new_data);
                    Ok(Json(new_data))
                }
                Err(_) => Err("Could not collect items".into()),
            }
        }
        Err(_) => Err("Failed to fetch items.".into()),
    }
}

#[get("/api/v1/tables/<table>")]
fn get_items_for_table(table: String) -> Result<Json<data::Table>, String> {
    println!("Getting items for table {}.", table);    let connection = &database::get_connection();
    let mut statement = database::build_statement(&connection, "select * from restaurant_table where id = $1;");
    println!("Prepared statement {:?}.", statement);
    let results = statement.query_map(&[&table], |row| {
        Ok(database::TableData {
            id: row.get(0)?,
            item_ids: row.get(1)?,
        })
    });
    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();
            match collection {
                Ok(items) => {
                    let table_data = items.first().expect("No tables found.");
                    let table = data::Table {
                        id: table_data.id,
                        items: {
                            let id_list = Vec::from_iter(
                                table_data.item_ids.split(",").map(|s| s.to_string()),
                            );
                            get_items(&connection, id_list)
                        },
                    };
                    println!("Finished getting items {:?}.", table);
                    Ok(Json(table))
                }
                Err(_) => Err("Could not collect items".into()),
            }
        }
        Err(_) => Err("Failed to fetch items.".into()),
    }
}

#[get("/api/v1/tables/<table>/<item>")]
fn get_item_for_table(table: String, item: String) -> Result<Json<data::Item>, String> {
    println!("Getting item {:?} for table {}.", item, table);
    let connection = &database::get_connection();
    let mut statement = database::build_statement(&connection, "select * from restaurant_table where id = $1;");
    println!("Prepared statement {:?}.", statement);
    let results = statement.query_map(&[&table], |row| {
        Ok(database::TableData {
            id: row.get(0)?,
            item_ids: row.get(1)?,
        })
    });
    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();
            match collection {
                Ok(items) => {
                    let table = items.first().expect("Table not found.");
                    println!("Found table: {:?}", table);
                    let item_ids = table
                        .item_ids
                        .split(",")
                        .filter(|s| item.eq(&s.to_string()))
                        .map(|s| s.to_string())
                        .collect();
                    println!("Getting items for ids: {:?}", item_ids);
                    let item = get_items(&connection, item_ids).into_iter().next().expect("Item not found.");
                    println!("Found items {:?}", item);
                    Ok(Json(item))
                }
                Err(_) => Err("Could not collect items".into()),
            }
        }
        Err(_) => Err("Failed to fetch items.".into()),
    }
}

fn get_items(connection: &rusqlite::Connection, ids: Vec<String>) -> Vec<data::Item> {
    println!("Getting items {:?}.", ids);
    let mut items: Vec<data::Item> = Vec::new();
    let mut iter = IntoIterator::into_iter(&ids);
    while let Some(s) = iter.next() {
        let mut statement = database::build_statement(&connection, "select * from item where id = $1;");
        let results = statement.query_map(&[&s], |row| {
            Ok(database::ItemData {
                id: row.get(0)?,
                name: row.get(1)?,
                time_to_prepare: row.get(2)?,
            })
        });
        match results {
            Ok(rows) => {
                let collection: rusqlite::Result<Vec<database::ItemData>> = rows.collect();
                match collection {
                    Ok(data) => {
                        for d in data.iter() {
                            items.push(data::Item {
                                id: d.id,
                                name: d.name.clone(),
                                time_to_prepare: d.time_to_prepare,
                            });
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        };
    }
    println!("Found items {:?}.", items);
    items
}

#[post("/api/v1/tables", format = "json", data = "<request>")]
fn add_item(request: Json<data::ItemRequest>) -> Result<Json<data::StatusMessage>, String> {
    println!(
        "Adding item {:?} in Table {}.",
        request.items, request.table_id
    );
    let connection = &database::get_connection();
    let mut statement = database::build_statement(&connection, "insert into item (id, name, preperation_time) values (null, $1, $2);");
    println!("Prepared statement {:?}.", statement);
    let mut random = rand::thread_rng();
    let item_ids = request
        .items
        .iter()
        .map(|item| {
            let preperation_time = random.gen_range(5..=15).to_string();
            statement
                .execute(&[&item, &preperation_time])
                .expect("Failed to insert items");
            connection.last_insert_rowid().to_string()
        })
        .collect::<Vec<String>>()
        .join(",");
    match add_item_to_table(&item_ids, &request.table_id.to_string()) {
        Ok(result) => {
            println!("Finished adding items to Table.");
            Ok(Json(data::StatusMessage {
                message: String::from(result),
            }))
        }
        Err(_) => Err("Failed to insert into items.".into()),
    }
}

pub fn add_item_to_table(item_id: &str, table_id: &str) -> Result<String, String> {
    let connection = &database::get_connection();
    let mut statement = database::build_statement(&connection, "select * from restaurant_table where id = :id;");
    println!("Prepared statement {:?}.", statement);
    let mut table_rows: rusqlite::Rows = statement
        .query(rusqlite::named_params! { ":id": table_id })
        .expect("Select item statement failed");
    let mut items = String::from("");
    while let Some(row) = table_rows.next().expect("Failed to select table.`") {
        let values: String = row.get(1).expect("Failed to gets items from table.");
        items.push_str(&values.to_string());
        items.push_str(",");
    }
    if items.len() < 1 {
        let mut insert_statement = match connection
            .prepare("insert into restaurant_table (id,items) values (null, $1);")
        {
            Ok(statement) => statement,
            Err(e) => return Err(format!("Failed with error: {}", e)),
        };
        items.push_str(&item_id.to_string());
        let results = insert_statement.execute(&[&items]);
        match results {
            Ok(count) => return Ok(format!("{} rows inserted.", count)),
            Err(e) => return Err(format!("Failed with error: {}", e)),
        }
    }
    items.push_str(&item_id.to_string());

    let mut update_statement = database::build_statement(&connection, "update restaurant_table set items = $1 where id = $2;");
    println!("Prepared statement {:?}.", update_statement);
    let results = update_statement.execute(&[&items, table_id]);
    match results {
        Ok(count) => Ok(format!("{} rows inserted.", count)),
        Err(e) => Err(format!("Failed with error: {}", e)),
    }
}

#[delete("/api/v1/tables/<table>/<item>")]
fn remove_item(table: String, item: String) -> Result<Json<data::StatusMessage>, String> {
    println!("Removing item {}, from table {}.", item, table);
    let connection = &database::get_connection();
    let mut statement = database::build_statement(&connection, "select * from restaurant_table where id = :id;");
    println!("Prepared statement {:?}.", statement);
    let table_rows = statement.query_map(&[&table], |row| {
        Ok(database::TableData {
            id: row.get(0)?,
            item_ids: row.get(1)?,
        })
    });
    let mut updated_items = String::new();
    match table_rows {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<database::TableData>> = rows.collect();
            match collection {
                Ok(data) => {
                    for d in data.iter() {
                        println!("Found table {:?}", data);
                        let table_items = d
                            .item_ids
                            .split(",")
                            .filter(|s| s != &item)
                            .collect::<String>();
                        updated_items.push_str(&table_items);
                        println!("New items are {:?}", table_items);
                        break;
                    }
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    };
    let mut update_statement = database::build_statement(&connection, "update restaurant_table set items = $1 where id = $2;");
    println!("Prepared statement {:?}.", update_statement);
    let results = update_statement.execute(&[&updated_items, &table]);
    match results {
        Ok(_) => {
            println!("Updated table {} and removed item {}.", table, item);
            let message = delete_item(&connection, item).unwrap();
            Ok(Json(data::StatusMessage { message: message }))
        }
        Err(e) => Err(format!("Failed with error: {}", e)),
    }
}

fn delete_item(connection: &rusqlite::Connection, id: String) -> Result<String, String> {
    let mut statement = database::build_statement(&connection, "delete from item where id = $1;");
    println!("Prepared statement {:?}.", statement);
    let deleted_rows = statement.execute(&[&id]);
    match deleted_rows {
        Ok(rows_affected) => Ok(format!("{} rows deleted.", rows_affected)),
        Err(_) => Err("Failed to delete into items.".into()),
    }
}

fn main() {
    database::setup_db().expect("Program failed to start");
    let rocket = luanch_server();
    rocket.launch();
}

fn luanch_server() -> rocket::Rocket {
    rocket::ignite().mount(
        "/",
        routes![
            get_all_tables,
            add_item,
            remove_item,
            get_item_for_table,
            get_items_for_table
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;

    #[test]
    fn should_setup_db() {
        let db_message = database::setup_db().unwrap();
        assert!(db_message.eq("Successfully created database tables."));
    }

    #[test]
    fn should_get_all() {
        database::setup_test_db(
            "INSERT INTO restaurant_table VALUES (999, '999,1000');
                 INSERT INTO item VALUES (999, 'pizza', 5);
                 INSERT INTO item VALUES (1000, 'cake', 9);",
        );
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        let mut response = client.get("/api/v1/tables").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.body_string().unwrap();
        let data =
            serde_json::from_str::<Vec<data::Table>>(&body).expect("Failed to convert json.");
        assert_ne!(data.is_empty(), true);
        database::close_test_db(
            "
        DELETE FROM restaurant_table WHERE id = 999;
        DELETE FROM item WHERE id = 999;
        DELETE FROM item WHERE id = 1000;",
        );
    }

    #[test]
    fn should_get_all_items_for_table() {
        database::setup_test_db(
            "INSERT INTO restaurant_table VALUES (1000, '1001,1002');
                 INSERT INTO item VALUES (1001, 'pizza', 5);
                 INSERT INTO item VALUES (1002, 'cake', 9);",
        );
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        let mut response = client.get("/api/v1/tables/1000").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.body_string().unwrap();
        let table = serde_json::from_str::<data::Table>(&body).expect("Failed to convert json.");
        assert_eq!(table.id, 1000);
        let items = table.items;
        assert_eq!(items.len(), 2);
        let item1 = items.get(0).unwrap();
        assert_eq!(item1.id, 1001);
        assert_eq!(item1.time_to_prepare, 5);
        assert!(item1.name.eq("pizza"));
        let item2 = items.get(1).unwrap();
        assert_eq!(item2.id, 1002);
        assert_eq!(item2.time_to_prepare, 9);
        assert!(item2.name.eq("cake"));
        database::close_test_db(
            "
        DELETE FROM restaurant_table WHERE id = 1000;
        DELETE FROM item WHERE id = 1001;
        DELETE FROM item WHERE id = 1002;",
        );
    }

    #[test]
    #[should_panic(expected = "No tables found.")]
    fn should_get_no_items_for_table() {
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        client.get("/api/v1/tables/9999999").dispatch();
    }

    #[test]
    fn should_add_item() {
        database::setup_test_db(
            "INSERT INTO restaurant_table VALUES (1001, '10003,1004');
                 INSERT INTO item VALUES (1003, 'pizza', 5);
                 INSERT INTO item VALUES (1004, 'cake', 9);",
        );
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        let items = vec![String::from("pasta")];
        let mut response = client
            .post("/api/v1/tables")
            .header(ContentType::JSON)
            .body(get_item_json(String::from("999"), items))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.body_string().unwrap();
        let data =
            serde_json::from_str::<data::StatusMessage>(&body).expect("Failed to convert json.");
        assert!(data.message.eq("1 rows inserted."));
        database::close_test_db(
            "DELETE FROM restaurant_table WHERE id = 1001;
        DELETE FROM item WHERE id = 1003;
        DELETE FROM item WHERE id = 1004;",
        );
    }

    #[test]
    fn should_get_item() {
        database::setup_test_db(
            "INSERT INTO restaurant_table VALUES (1002, '1005,1006');
                 INSERT INTO item VALUES (1005, 'pizza', 5);
                 INSERT INTO item VALUES (1006, 'cake', 9);",
        );
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        let mut response = client
            .get("/api/v1/tables/1002/1005")
            .header(ContentType::JSON)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let response_body = response.body_string().unwrap();
        let item = serde_json::from_str::<data::Item>(&response_body)
            .expect("Failed to deserialzie item response.");
        assert!(item.name.eq("pizza"));
        assert_eq!(item.id, 1005);
        assert_eq!(item.time_to_prepare, 5);
        database::close_test_db(
            "DELETE FROM restaurant_table WHERE id = 1002;
                    DELETE FROM item WHERE id = 1005;
                    DELETE FROM item WHERE id = 1006;",
        );
    }

    #[test]
    fn should_delete_item() {
        database::setup_test_db(
            "INSERT INTO restaurant_table VALUES (1003, '1007,1008');
                 INSERT INTO item VALUES (1007, 'pizza', 5);
                 INSERT INTO item VALUES (1008, 'cake', 9);",
        );
        let rocket = luanch_server();
        let client = Client::new(rocket).expect("Failed to start server");
        let mut response = client
            .delete("/api/v1/tables/1003/1007")
            .header(ContentType::JSON)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let response_body = response.body_string().unwrap();
        let status_message = serde_json::from_str::<data::StatusMessage>(&response_body)
            .expect("Failed to convert json.");
        assert_eq!(status_message.message, "1 rows deleted.");
        let mut response = client.get("/api/v1/tables/1003").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        let body = response.body_string().unwrap();
        let table = serde_json::from_str::<data::Table>(&body).expect("Failed to convert json.");
        assert_eq!(table.id, 1003);
        let items = table.items;
        assert_eq!(items.len(), 1);
        let item1 = items.get(0).unwrap();
        assert_eq!(item1.id, 1008);
        assert_eq!(item1.time_to_prepare, 9);
        assert!(item1.name.eq("cake"));
        database::close_test_db(
            "DELETE FROM restaurant_table WHERE id = 1003;
                    DELETE FROM item WHERE id = 1007;
                    DELETE FROM item WHERE id = 1008;",
        );
    }

    fn get_item_json(table_id: String, items: Vec<String>) -> String {
        let request = data::ItemRequest {
            table_id: table_id,
            items: items,
        };
        serde_json::to_string(&request).unwrap()
    }
}
