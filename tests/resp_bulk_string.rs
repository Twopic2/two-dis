use two_dis::protocal::{Resp, parse_buffer_to_resp as parse_with_index};

fn parse_buffer_to_resp(buffer: &[u8]) -> Result<Resp, ()> {
    parse_with_index(buffer, &mut 0)
}

#[test]
fn parses_bulk_string() {
    assert_eq!(
        parse_buffer_to_resp(b"$4\r\nPING\r\n"),
        Ok(Resp::BulkString("PING".into()))
    );
}

#[test]
fn parses_empty_bulk_string() {
    assert_eq!(
        parse_buffer_to_resp(b"$0\r\n\r\n"),
        Ok(Resp::BulkString(String::new()))
    );
}

#[test]
fn parses_bulk_string_with_multidigit_length() {
    assert_eq!(
        parse_buffer_to_resp(b"$12\r\nhello world!\r\n"),
        Ok(Resp::BulkString("hello world!".into()))
    );
}

#[test]
fn parses_unicode_bulk_string_by_byte_length() {
    assert_eq!(
        parse_buffer_to_resp("$6\r\nhéllo\r\n".as_bytes()),
        Ok(Resp::BulkString("héllo".into()))
    );
}

#[test]
fn preserves_line_breaks_and_nul_bytes() {
    assert_eq!(
        parse_buffer_to_resp(b"$5\r\na\r\n\0b\r\n"),
        Ok(Resp::BulkString("a\r\n\0b".into()))
    );
}

#[test]
fn rejects_every_truncated_bulk_string_prefix() {
    let message = b"$4\r\nPING\r\n";

    for end in 0..message.len() {
        assert_eq!(
            parse_buffer_to_resp(&message[..end]),
            Err(()),
            "prefix: {end}"
        );
    }
}

#[test]
fn rejects_mismatched_lengths_and_invalid_terminators() {
    let messages: &[&[u8]] = &[
        b"$4\r\nPIN\r\n",
        b"$3\r\nPING\r\n",
        b"$4\r\nPING\rX",
        b"$0\r\nX\r\n",
        "$5\r\nhéllo\r\n".as_bytes(),
    ];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn parses_first_bulk_string_without_including_next_frame() {
    assert_eq!(
        parse_buffer_to_resp(b"$4\r\nPING\r\n+OK\r\n"),
        Ok(Resp::BulkString("PING".into()))
    );
}
