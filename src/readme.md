Simple Restaurant API

This project is a simple rest application to model tables and items are a restaurant. It runs a server locally and then returns json data. 

The server can be run with cargo run and then calling the endpoints. Tests are run with cargo test

There are 5 endpoints that all use this path /api/v1/tables.
GET /api/v1/tables
get_all_tables returns the list of all the tables and items in those tables.

GET /api/v1/tables/<table>
get_items_for_table returns a single table containing the list of items

GET /api/v1/tables<table>/<item>
get_item_for_table returns a the item if it is in the table

POST /api/v1/tables
add_item Adds an item to a table and will create a new table if there is not one found

DELETE /api/v1/tables<table>/<item>
remove_item Removes an item from the table