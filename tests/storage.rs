use two_dis::storage::Storage;

#[test]
fn returns_none_for_a_missing_key() {
    let mut storage = Storage::default();

    assert_eq!(storage.get("missing".into()), None);
}

#[test]
fn sets_and_gets_a_string() {
    let mut storage = Storage::default();

    assert_eq!(
        storage.set("name".into(), "two-dis".into()),
        Ok("OK".into())
    );
    assert_eq!(storage.get("name".into()), Some("two-dis".into()));
}

#[test]
fn overwrites_only_the_requested_key() {
    let mut storage = Storage::default();
    storage.set("name".into(), "old".into()).unwrap();
    storage.set("other".into(), "untouched".into()).unwrap();

    assert_eq!(storage.set("name".into(), "new".into()), Ok("OK".into()));
    assert_eq!(storage.get("name".into()), Some("new".into()));
    assert_eq!(storage.get("other".into()), Some("untouched".into()));
}

#[test]
fn treats_keys_as_case_sensitive() {
    let mut storage = Storage::default();
    storage.set("Key".into(), "upper".into()).unwrap();
    storage.set("key".into(), "lower".into()).unwrap();

    assert_eq!(storage.get("Key".into()), Some("upper".into()));
    assert_eq!(storage.get("key".into()), Some("lower".into()));
    assert_eq!(storage.get("KEY".into()), None);
}

#[test]
fn roundtrips_empty_unicode_whitespace_and_control_characters() {
    let cases = [
        ("", "empty key"),
        ("empty value", ""),
        ("clé 🦀", "héllo 世界"),
        (" key with spaces ", " value with spaces "),
        ("key\r\n\0", "first\r\nsecond\0last"),
    ];
    let mut storage = Storage::default();

    for (key, value) in cases {
        assert_eq!(storage.set(key.into(), value.into()), Ok("OK".into()));
    }

    for (key, value) in cases {
        assert_eq!(storage.get(key.into()), Some(value.into()), "key: {key:?}");
    }
}

#[test]
fn repeated_reads_and_changes_to_returned_values_do_not_change_storage() {
    let mut storage = Storage::default();
    storage.set("key".into(), "original".into()).unwrap();

    let mut value = storage.get("key".into()).unwrap();
    value.push_str(" changed");

    assert_eq!(value, "original changed");
    assert_eq!(storage.get("key".into()), Some("original".into()));
    assert_eq!(storage.get("key".into()), Some("original".into()));
}

#[test]
fn missing_reads_do_not_change_existing_values_or_create_keys() {
    let mut storage = Storage::default();
    storage.set("existing".into(), "value".into()).unwrap();

    assert_eq!(storage.get("missing".into()), None);
    assert_eq!(storage.get("existing".into()), Some("value".into()));
    assert_eq!(storage.get("missing".into()), None);
}

#[test]
fn stores_values_larger_than_the_socket_read_buffer() {
    let mut storage = Storage::default();
    let value = "héllo 🦀\r\n\0".repeat(1024);

    storage.set("large".into(), value.clone()).unwrap();

    assert_eq!(storage.get("large".into()), Some(value));
}

#[test]
fn separate_storage_instances_do_not_share_values() {
    let mut first = Storage::default();
    let mut second = Storage::default();
    first.set("key".into(), "first".into()).unwrap();

    assert_eq!(second.get("key".into()), None);
    second.set("key".into(), "second".into()).unwrap();

    assert_eq!(first.get("key".into()), Some("first".into()));
    assert_eq!(second.get("key".into()), Some("second".into()));
}

#[test]
fn deletes_only_the_requested_key() {
    let mut storage = Storage::default();
    storage.set("remove".into(), "value".into()).unwrap();
    storage.set("keep".into(), "untouched".into()).unwrap();

    assert_eq!(storage.del("remove".into()), 1);
    assert_eq!(storage.get("remove".into()), None);
    assert_eq!(storage.get("keep".into()), Some("untouched".into()));
}

#[test]
fn deleting_missing_or_already_deleted_keys_returns_zero() {
    let mut storage = Storage::default();
    storage.set("key".into(), "value".into()).unwrap();

    assert_eq!(storage.del("missing".into()), 0);
    assert_eq!(storage.get("key".into()), Some("value".into()));
    assert_eq!(storage.del("key".into()), 1);
    assert_eq!(storage.del("key".into()), 0);
}

#[test]
fn can_reinsert_a_deleted_key() {
    let mut storage = Storage::default();
    storage.set("key".into(), "old".into()).unwrap();

    assert_eq!(storage.del("key".into()), 1);
    storage.set("key".into(), "new".into()).unwrap();

    assert_eq!(storage.get("key".into()), Some("new".into()));
}
