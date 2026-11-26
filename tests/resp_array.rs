use two_dis::protocal::{Resp, parse_buffer_to_resp as parse_with_index};

fn parse_buffer_to_resp(buffer: &[u8]) -> Result<Resp, ()> {
    parse_with_index(buffer, &mut 0)
}

#[test]
fn parses_redis_cli_ping_command() {
    assert_eq!(
        parse_buffer_to_resp(b"*1\r\n$4\r\nPING\r\n"),
        Ok(Resp::Array(vec![Resp::BulkString("PING".into())]))
    );
}

#[test]
fn parses_redis_cli_set_command() {
    assert_eq!(
        parse_buffer_to_resp(b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n"),
        Ok(Resp::Array(vec![
            Resp::BulkString("SET".into()),
            Resp::BulkString("key".into()),
            Resp::BulkString("value".into()),
        ]))
    );
}

#[test]
fn parses_empty_array() {
    assert_eq!(parse_buffer_to_resp(b"*0\r\n"), Ok(Resp::Array(vec![])));
}

#[test]
fn parses_multidigit_element_count() {
    let message = format!("*12\r\n{}", "$1\r\nx\r\n".repeat(12));
    let expected = (0..12).map(|_| Resp::BulkString("x".into())).collect();

    assert_eq!(
        parse_buffer_to_resp(message.as_bytes()),
        Ok(Resp::Array(expected))
    );
}

#[test]
fn preserves_empty_multiline_and_nul_arguments() {
    assert_eq!(
        parse_buffer_to_resp(b"*3\r\n$3\r\nSET\r\n$0\r\n\r\n$5\r\na\r\n\0b\r\n"),
        Ok(Resp::Array(vec![
            Resp::BulkString("SET".into()),
            Resp::BulkString(String::new()),
            Resp::BulkString("a\r\n\0b".into()),
        ]))
    );
}

#[test]
fn parses_unicode_arguments_by_byte_length() {
    assert_eq!(
        parse_buffer_to_resp("*2\r\n$4\r\nECHO\r\n$6\r\nhéllo\r\n".as_bytes()),
        Ok(Resp::Array(vec![
            Resp::BulkString("ECHO".into()),
            Resp::BulkString("héllo".into()),
        ]))
    );
}

#[test]
fn rejects_every_truncated_command_prefix() {
    let message = b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n";

    for end in 0..message.len() {
        assert_eq!(
            parse_buffer_to_resp(&message[..end]),
            Err(()),
            "prefix: {end}"
        );
    }
}

#[test]
fn rejects_incomplete_or_invalid_array_separators() {
    let messages: &[&[u8]] = &[b"*", b"*1\r", b"*1\rX$4\r\nPING\r\n", b"*1\n$4\r\nPING\r\n"];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn rejects_incomplete_or_invalid_arguments() {
    let messages: &[&[u8]] = &[
        b"*1\r\n",
        b"*2\r\n$4\r\nPING\r\n",
        b"*1\r\n$4\r\nPIN\r\n",
        b"*1\r\n$4\r\nPING\rX",
        b"*1\r\n?PING\r\n",
    ];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn parses_first_array_without_including_next_frame() {
    assert_eq!(
        parse_buffer_to_resp(b"*1\r\n$4\r\nPING\r\n*1\r\n$4\r\nPING\r\n"),
        Ok(Resp::Array(vec![Resp::BulkString("PING".into())]))
    );
}
