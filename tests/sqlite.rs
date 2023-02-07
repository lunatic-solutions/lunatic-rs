use lunatic::sqlite::{Query, SqliteClient, Value};
use lunatic_test::test;

#[test]
fn query() {
    let client = SqliteClient::connect("").unwrap();

    let mut rows = client.query("select \"Hello\"");
    assert_eq!(rows.next(), Some(vec![Value::Text("Hello".to_string())]));
    assert_eq!(rows.next(), None);
}

#[test]
fn prepared_query() {
    let client = SqliteClient::connect("").unwrap();

    let mut stmt = client.prepare_query("select ?");
    stmt = stmt.bind("Foo!");
    let mut rows = stmt.execute();
    assert_eq!(rows.next(), Some(vec![Value::Text("Foo!".to_string())]));
    assert_eq!(rows.next(), None);
}

#[test]
fn execute() {
    let client = SqliteClient::connect("").unwrap();

    client.execute("select \"Hello\"").unwrap();
}
