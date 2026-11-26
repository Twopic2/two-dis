use two_dis::protocal::{Resp, parse_buffer_to_resp as parse_with_index};

fn parse_buffer_to_resp(buffer: &[u8]) -> Result<Resp, ()> {
    parse_with_index(buffer, &mut 0)
}

#[test]
fn parses_simple_string() {
    assert_eq!(
        parse_buffer_to_resp(b"+OK\r\n"),
        Ok(Resp::SimpleString("OK".into()))
    );
}

#[test]
fn parses_empty_simple_string() {
    assert_eq!(
        parse_buffer_to_resp(b"+\r\n"),
        Ok(Resp::SimpleString(String::new()))
    );
}

#[test]
fn preserves_spaces_in_simple_string() {
    assert_eq!(
        parse_buffer_to_resp(b"+ hello world \r\n"),
        Ok(Resp::SimpleString(" hello world ".into()))
    );
}

#[test]
fn parses_utf8_simple_string() {
    assert_eq!(
        parse_buffer_to_resp("+héllo 🦀\r\n".as_bytes()),
        Ok(Resp::SimpleString("héllo 🦀".into()))
    );
}

#[test]
fn rejects_empty_input() {
    assert_eq!(parse_buffer_to_resp(b""), Err(()));
}

#[test]
fn rejects_invalid_type_markers() {
    let messages: &[&[u8]] = &[b"OK\r\n", b"?hello\r\n", b"\0hello\r\n"];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn rejects_every_truncated_simple_string_prefix() {
    let message = b"+hello\r\n";

    for end in 0..message.len() {
        assert_eq!(
            parse_buffer_to_resp(&message[..end]),
            Err(()),
            "prefix: {end}"
        );
    }
}

#[test]
fn rejects_malformed_separators() {
    let messages: &[&[u8]] = &[b"+OK\rX", b"+OK\n\r", b"+OK\n\n"];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn rejects_embedded_line_breaks() {
    let messages: &[&[u8]] = &[b"+hello\nworld\r\n", b"+hello\rworld\r\n"];

    for message in messages {
        assert_eq!(parse_buffer_to_resp(message), Err(()), "input: {message:?}");
    }
}

#[test]
fn parses_errors_and_integers() {
    let messages: &[(&[u8], Resp)] = &[
        (b"-ERR\r\n", Resp::SimpleError("ERR".into())),
        (b":42\r\n", Resp::Integer(42)),
    ];

    for (message, expected) in messages {
        assert_eq!(
            parse_buffer_to_resp(message).as_ref(),
            Ok(expected),
            "input: {message:?}"
        );
    }
}
