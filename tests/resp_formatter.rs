use two_dis::protocal::{Resp, array};

#[test]
fn formats_empty_array() {
    assert_eq!(array(&[]), "*0\r\n");
}

#[test]
fn formats_command_array_as_bulk_strings() {
    let values = [
        Resp::BulkString("ECHO".into()),
        Resp::BulkString("hello".into()),
    ];

    assert_eq!(array(&values), "*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n");
}

#[test]
fn preserves_types_and_order_in_mixed_array() {
    let values = [
        Resp::SimpleString("OK".into()),
        Resp::SimpleError("ERR example".into()),
        Resp::Integer(-42),
        Resp::BulkString("hello".into()),
    ];

    assert_eq!(
        array(&values),
        "*4\r\n+OK\r\n-ERR example\r\n:-42\r\n$5\r\nhello\r\n"
    );
}

#[test]
fn formats_nested_arrays_with_independent_element_counts() {
    let values = [
        Resp::Integer(1),
        Resp::Array(vec![
            Resp::BulkString("hi".into()),
            Resp::Array(vec![Resp::SimpleString("OK".into())]),
        ]),
        Resp::Array(vec![]),
    ];

    assert_eq!(
        array(&values),
        "*3\r\n:1\r\n*2\r\n$2\r\nhi\r\n*1\r\n+OK\r\n*0\r\n"
    );
}

#[test]
fn uses_byte_lengths_for_unicode_and_multiline_bulk_strings() {
    let values = [
        Resp::BulkString("héllo".into()),
        Resp::BulkString("a\r\nb".into()),
    ];

    assert_eq!(array(&values), "*2\r\n$6\r\nhéllo\r\n$4\r\na\r\nb\r\n");
}
