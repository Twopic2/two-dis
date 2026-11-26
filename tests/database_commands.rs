use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinSet, time::timeout};
use two_dis::{
    protocal::{Resp, parse_buffer_to_resp, string_array},
    server::process_request,
    storage::Storage,
};

fn database() -> Arc<Mutex<Storage>> {
    Arc::new(Mutex::new(Storage::default()))
}

// Exercise RESP command parsing and the real database together. The current
// process_request API returns already-encoded replies inside the Resp variants.
async fn execute(storage: &Arc<Mutex<Storage>>, arguments: &[&str]) -> Resp {
    let arguments: Vec<String> = arguments
        .iter()
        .map(|argument| (*argument).into())
        .collect();
    let frame = string_array(&arguments);
    let mut index = 0;
    let request = parse_buffer_to_resp(frame.as_bytes(), &mut index).unwrap();
    assert_eq!(index, frame.len());

    process_request(request, Arc::clone(storage)).await
}

#[tokio::test]
async fn set_returns_ok_and_writes_to_the_database() {
    let storage = database();

    assert_eq!(
        execute(&storage, &["SET", "name", "two-dis"]).await,
        Resp::SimpleString("+OK\r\n".into())
    );
    assert_eq!(
        storage.lock().await.get("name".into()),
        Some("two-dis".into())
    );
}

#[tokio::test]
async fn get_reads_an_existing_database_value_without_removing_it() {
    let storage = database();
    storage
        .lock()
        .await
        .set("name".into(), "two-dis".into())
        .unwrap();

    assert_eq!(
        execute(&storage, &["GET", "name"]).await,
        Resp::SimpleString("+two-dis\r\n".into())
    );
    assert_eq!(
        storage.lock().await.get("name".into()),
        Some("two-dis".into())
    );
}

#[tokio::test]
async fn get_returns_null_for_a_missing_key() {
    let storage = database();

    assert_eq!(
        execute(&storage, &["GET", "missing"]).await,
        Resp::NullString("$-1\r\n".into())
    );
    assert_eq!(storage.lock().await.get("missing".into()), None);
}

#[tokio::test]
async fn set_then_get_returns_the_latest_value_and_preserves_other_keys() {
    let storage = database();

    for arguments in [
        ["SET", "name", "old"],
        ["SET", "other", "untouched"],
        ["SET", "name", "new"],
    ] {
        assert_eq!(
            execute(&storage, &arguments).await,
            Resp::SimpleString("+OK\r\n".into())
        );
    }

    assert_eq!(
        execute(&storage, &["GET", "name"]).await,
        Resp::SimpleString("+new\r\n".into())
    );
    assert_eq!(
        execute(&storage, &["GET", "other"]).await,
        Resp::SimpleString("+untouched\r\n".into())
    );
}

#[tokio::test]
async fn command_names_are_case_insensitive_but_keys_are_case_sensitive() {
    let storage = database();

    for set_command in ["set", "SET", "sEt"] {
        assert_eq!(
            execute(&storage, &[set_command, "Key", "value"]).await,
            Resp::SimpleString("+OK\r\n".into())
        );
    }

    for get_command in ["get", "GET", "gEt"] {
        assert_eq!(
            execute(&storage, &[get_command, "Key"]).await,
            Resp::SimpleString("+value\r\n".into())
        );
    }

    assert_eq!(
        execute(&storage, &["GET", "key"]).await,
        Resp::NullString("$-1\r\n".into())
    );
}

#[tokio::test]
async fn roundtrips_empty_unicode_and_whitespace_keys_and_values() {
    let storage = database();
    let cases = [
        ("", "empty key", "+empty key\r\n"),
        ("empty value", "", "+\r\n"),
        ("clé 🦀", "héllo 世界", "+héllo 世界\r\n"),
        (" key ", " value ", "+ value \r\n"),
    ];

    for (key, value, _) in cases {
        assert_eq!(
            execute(&storage, &["SET", key, value]).await,
            Resp::SimpleString("+OK\r\n".into())
        );
    }

    for (key, _, expected) in cases {
        assert_eq!(
            execute(&storage, &["GET", key]).await,
            Resp::SimpleString(expected.into()),
            "key: {key:?}"
        );
    }
}

#[tokio::test]
async fn set_preserves_multiline_and_nul_bytes_in_the_database() {
    let storage = database();

    assert_eq!(
        execute(&storage, &["SET", "key\r\n\0", "first\r\nsecond\0last"]).await,
        Resp::SimpleString("+OK\r\n".into())
    );
    assert_eq!(
        storage.lock().await.get("key\r\n\0".into()),
        Some("first\r\nsecond\0last".into())
    );
}

#[tokio::test]
async fn invalid_set_argument_counts_return_errors_without_changing_data() {
    let storage = database();
    storage
        .lock()
        .await
        .set("key".into(), "original".into())
        .unwrap();
    let cases: &[&[&str]] = &[
        &["SET"],
        &["SET", "key"],
        &["SET", "key", "replacement", "extra"],
        &["SET", "missing"],
        &["SET", "missing", "value", "extra"],
    ];

    for arguments in cases {
        assert_eq!(
            execute(&storage, arguments).await,
            Resp::SimpleError(
                "-(error) ERR wrong number of arguments for 'set' command\r\n".into()
            ),
            "arguments: {arguments:?}"
        );
        let mut stored = storage.lock().await;
        assert_eq!(stored.get("key".into()), Some("original".into()));
        assert_eq!(stored.get("missing".into()), None);
    }
}

#[tokio::test]
async fn invalid_get_argument_counts_return_errors_without_changing_data() {
    let storage = database();
    storage
        .lock()
        .await
        .set("key".into(), "original".into())
        .unwrap();
    let cases: &[&[&str]] = &[&["GET"], &["GET", "key", "extra"]];

    for arguments in cases {
        assert_eq!(
            execute(&storage, arguments).await,
            Resp::SimpleError(
                "-(error) ERR wrong number of arguments for 'get' command\r\n".into()
            ),
            "arguments: {arguments:?}"
        );
        assert_eq!(
            storage.lock().await.get("key".into()),
            Some("original".into())
        );
    }
}

#[tokio::test]
async fn separate_database_instances_keep_command_results_isolated() {
    let first = database();
    let second = database();

    assert_eq!(
        execute(&first, &["SET", "key", "first"]).await,
        Resp::SimpleString("+OK\r\n".into())
    );
    assert_eq!(
        execute(&second, &["GET", "key"]).await,
        Resp::NullString("$-1\r\n".into())
    );
    assert_eq!(
        execute(&second, &["SET", "key", "second"]).await,
        Resp::SimpleString("+OK\r\n".into())
    );
    assert_eq!(
        execute(&first, &["GET", "key"]).await,
        Resp::SimpleString("+first\r\n".into())
    );
    assert_eq!(
        execute(&second, &["GET", "key"]).await,
        Resp::SimpleString("+second\r\n".into())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_requests_share_storage_without_losing_writes() {
    let storage = database();
    let mut tasks = JoinSet::new();

    timeout(Duration::from_secs(5), async {
        for client in 0..32 {
            let storage = Arc::clone(&storage);
            tasks.spawn(async move {
                let key = format!("key-{client}");
                let value = format!("value-{client}");

                assert_eq!(
                    execute(&storage, &["SET", &key, &value]).await,
                    Resp::SimpleString("+OK\r\n".into())
                );
                assert_eq!(
                    execute(&storage, &["GET", &key]).await,
                    Resp::SimpleString(format!("+{value}\r\n"))
                );
            });
        }

        while let Some(result) = tasks.join_next().await {
            result.expect("database request task panicked");
        }

        let mut stored = storage.lock().await;
        for client in 0..32 {
            assert_eq!(
                stored.get(format!("key-{client}")),
                Some(format!("value-{client}"))
            );
        }
    })
    .await
    .expect("concurrent database requests timed out");
}

#[tokio::test]
async fn del_removes_multiple_keys_and_counts_each_existing_key_once() {
    let storage = database();
    for key in ["first", "second", "keep"] {
        storage
            .lock()
            .await
            .set(key.into(), "value".into())
            .unwrap();
    }

    assert_eq!(
        execute(&storage, &["DEL", "first", "missing", "first", "second"]).await,
        Resp::Integer(2)
    );
    for key in ["first", "second"] {
        assert_eq!(
            execute(&storage, &["GET", key]).await,
            Resp::NullString("$-1\r\n".into())
        );
    }
    assert_eq!(
        execute(&storage, &["GET", "keep"]).await,
        Resp::SimpleString("+value\r\n".into())
    );
    assert_eq!(
        execute(&storage, &["DEL", "first", "second", "missing"]).await,
        Resp::Integer(0)
    );
}

#[tokio::test]
async fn del_command_names_are_case_insensitive_but_keys_are_case_sensitive() {
    let storage = database();
    for command in ["del", "DEL", "dEl"] {
        storage
            .lock()
            .await
            .set("Key".into(), "value".into())
            .unwrap();

        assert_eq!(execute(&storage, &[command, "key"]).await, Resp::Integer(0));
        assert_eq!(execute(&storage, &[command, "Key"]).await, Resp::Integer(1));
    }
}

#[tokio::test]
async fn del_without_keys_returns_an_error_without_changing_data() {
    let storage = database();
    storage
        .lock()
        .await
        .set("key".into(), "value".into())
        .unwrap();

    assert_eq!(
        execute(&storage, &["DEL"]).await,
        Resp::SimpleError("-(error) ERR wrong number of arguments for 'del' command\r\n".into())
    );
    assert_eq!(storage.lock().await.get("key".into()), Some("value".into()));
}

#[tokio::test]
async fn del_accepts_an_empty_key() {
    let storage = database();
    assert_eq!(
        execute(&storage, &["SET", "", "value"]).await,
        Resp::SimpleString("+OK\r\n".into())
    );

    assert_eq!(execute(&storage, &["DEL", ""]).await, Resp::Integer(1));
    assert_eq!(
        execute(&storage, &["GET", ""]).await,
        Resp::NullString("$-1\r\n".into())
    );
}
