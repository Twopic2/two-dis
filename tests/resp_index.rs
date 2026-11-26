use two_dis::protocal::{Resp, array, null_bulk_string, parse_buffer_to_resp};

#[test]
fn parses_mixed_array() {
    let buffer = b"*5\r\n+OK\r\n-ERR example\r\n:9223372036854775807\r\n$5\r\nhello\r\n*0\r\n";
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::SimpleString("OK".into()),
            Resp::SimpleError("ERR example".into()),
            Resp::Integer(i64::MAX),
            Resp::BulkString("hello".into()),
            Resp::Array(vec![]),
        ]))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn recursively_parses_nested_mixed_arrays() {
    let buffer = b"*3\r\n*2\r\n:1\r\n+two\r\n$5\r\nthree\r\n*1\r\n*1\r\n-ERR nested\r\n";
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::Array(vec![Resp::Integer(1), Resp::SimpleString("two".into()),]),
            Resp::BulkString("three".into()),
            Resp::Array(vec![Resp::Array(vec![Resp::SimpleError(
                "ERR nested".into()
            )]),]),
        ]))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn advances_shared_index_across_multiple_values() {
    let buffer = b"+OK\r\n*2\r\n:42\r\n$4\r\nPING\r\n$0\r\n\r\n";
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::SimpleString("OK".into()))
    );
    assert_eq!(index, b"+OK\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::Integer(42),
            Resp::BulkString("PING".into()),
        ]))
    );
    assert_eq!(index, b"+OK\r\n*2\r\n:42\r\n$4\r\nPING\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::BulkString(String::new()))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn parses_array_from_nonzero_offset_and_leaves_following_value() {
    let buffer = b"skip:*2\r\n+OK\r\n:7\r\n+next\r\n";
    let mut index = b"skip:".len();

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::SimpleString("OK".into()),
            Resp::Integer(7),
        ]))
    );
    assert_eq!(index, b"skip:*2\r\n+OK\r\n:7\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::SimpleString("next".into()))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn failed_values_leave_callers_index_unchanged() {
    let frames: &[&[u8]] = &[
        b"+bad\n",
        b"$4\r\nPING\rX",
        b"*2\r\n:1\r\n",
        b"*1\r\n?bad\r\n",
        b":9223372036854775808\r\n",
    ];

    for frame in frames {
        let mut buffer = b"skip".to_vec();
        buffer.extend_from_slice(frame);
        let mut index = 4;
        assert_eq!(parse_buffer_to_resp(&buffer, &mut index), Err(()));
        assert_eq!(index, 4);
    }
}

#[test]
fn rejects_indexes_at_or_beyond_the_end() {
    let buffer = b"+OK\r\n";

    for start in [buffer.len(), buffer.len() + 1, usize::MAX] {
        let mut index = start;
        assert_eq!(parse_buffer_to_resp(buffer, &mut index), Err(()));
        assert_eq!(index, start);
    }
}

#[test]
fn parses_signed_64_bit_integer_boundaries() {
    let cases: &[(&[u8], i64)] = &[
        (b":9223372036854775807\r\n", i64::MAX),
        (b":-9223372036854775808\r\n", i64::MIN),
        (b":+42\r\n", 42),
        (b":-0\r\n", 0),
        (b":0\r\n", 0),
    ];

    for &(buffer, value) in cases {
        let mut index = 0;
        assert_eq!(
            parse_buffer_to_resp(buffer, &mut index),
            Ok(Resp::Integer(value))
        );
        assert_eq!(index, buffer.len());
    }
}

#[test]
fn rejects_malformed_and_overflowing_integers() {
    let buffers: &[&[u8]] = &[
        b":\r\n",
        b":+\r\n",
        b":--1\r\n",
        b": 1\r\n",
        b":1 \r\n",
        b":word\r\n",
        b":9223372036854775808\r\n",
        b":-9223372036854775809\r\n",
    ];

    for buffer in buffers {
        let mut index = 0;
        assert_eq!(parse_buffer_to_resp(buffer, &mut index), Err(()));
        assert_eq!(index, 0);
    }
}

#[test]
fn rejects_all_truncated_prefixes_of_nested_mixed_array() {
    let buffer = b"*4\r\n+OK\r\n:42\r\n$2\r\nhi\r\n*2\r\n$0\r\n\r\n-ERR\r\n";

    for end in 0..buffer.len() {
        let mut index = 0;
        assert_eq!(
            parse_buffer_to_resp(&buffer[..end], &mut index),
            Err(()),
            "prefix: {end}"
        );
        assert_eq!(index, 0);
    }
}

#[test]
fn roundtrips_mixed_nested_array_through_formatter() {
    let values = vec![
        Resp::SimpleString("OK".into()),
        Resp::SimpleError("ERR example".into()),
        Resp::Integer(i64::MIN),
        Resp::BulkString("héllo\r\n\0".into()),
        Resp::Array(vec![Resp::Integer(7), Resp::SimpleString("nested".into())]),
        Resp::Array(vec![]),
    ];
    let encoded = array(&values);
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(encoded.as_bytes(), &mut index),
        Ok(Resp::Array(values))
    );
    assert_eq!(index, encoded.len());
}

#[test]
fn rejects_invalid_nested_values() {
    let buffers: &[&[u8]] = &[
        b"*1\r\n+hello\nworld\r\n",
        b"*1\r\n-ERR\rX",
        b"*1\r\n:\r\n",
        b"*1\r\n*1\r\n?bad\r\n",
    ];

    for buffer in buffers {
        assert_eq!(parse_buffer_to_resp(buffer, &mut 0), Err(()));
    }
}

#[test]
fn formats_null_array_elements_distinctly() {
    let null = null_bulk_string();

    assert_eq!(
        array(&[Resp::NullString(null), Resp::NullArray]),
        "*2\r\n$-1\r\n*-1\r\n"
    );
}

#[test]
fn advances_index_across_basic_values() {
    let buffer = b"+OK\r\n*1\r\n$4\r\nPING\r\n$0\r\n\r\n";
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::SimpleString("OK".into()))
    );
    assert_eq!(index, b"+OK\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![Resp::BulkString("PING".into())]))
    );
    assert_eq!(index, b"+OK\r\n*1\r\n$4\r\nPING\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::BulkString(String::new()))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn parses_basic_array_from_nonzero_offset() {
    let buffer = b"skip:*2\r\n+OK\r\n$1\r\n7\r\n+next\r\n";
    let mut index = b"skip:".len();

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::SimpleString("OK".into()),
            Resp::BulkString("7".into()),
        ]))
    );
    assert_eq!(index, b"skip:*2\r\n+OK\r\n$1\r\n7\r\n".len());
    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::SimpleString("next".into()))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn parses_nested_arrays_of_simple_and_bulk_strings() {
    let buffer = b"*3\r\n*2\r\n+one\r\n$3\r\ntwo\r\n$5\r\nthree\r\n*0\r\n";
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(buffer, &mut index),
        Ok(Resp::Array(vec![
            Resp::Array(vec![
                Resp::SimpleString("one".into()),
                Resp::BulkString("two".into()),
            ]),
            Resp::BulkString("three".into()),
            Resp::Array(vec![]),
        ]))
    );
    assert_eq!(index, buffer.len());
}

#[test]
fn roundtrips_nested_string_arrays() {
    let values = vec![
        Resp::SimpleString("OK".into()),
        Resp::BulkString("héllo\r\n\0".into()),
        Resp::Array(vec![
            Resp::SimpleString("nested".into()),
            Resp::BulkString(String::new()),
        ]),
        Resp::Array(vec![]),
    ];
    let encoded = array(&values);
    let mut index = 0;

    assert_eq!(
        parse_buffer_to_resp(encoded.as_bytes(), &mut index),
        Ok(Resp::Array(values))
    );
    assert_eq!(index, encoded.len());
}
