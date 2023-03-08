use lunatic::sqlite::{Query, SqliteClient, Value};
use lunatic_test::test;

#[test]
fn query() {
    let client = SqliteClient::connect("").unwrap();

    let rows = client.query("select \"Hello\"");
    assert_eq!(rows, vec![vec![Value::Text("Hello".to_string())]]);
}

#[test]
fn prepared_query() {
    let client = SqliteClient::connect("").unwrap();

    let mut stmt = client.prepare_query("select ?");
    stmt = stmt.bind("Foo!");
    let rows = stmt.execute();
    assert_eq!(rows, vec![vec![Value::Text("Foo!".to_string())]]);
}

#[test]
fn execute() {
    let client = SqliteClient::connect("").unwrap();

    client.execute("select \"Hello\"").unwrap();
}
