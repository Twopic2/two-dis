/*
 * Protocol notes
 * This implementation targets RESP2.
 * The first byte identifies the RESP data type.
 * All types below are supported in RESP2.
 *
 * | First byte | Data type        | Category  |
 * |------------|------------------|-----------|
 * | +          | Simple string    | Simple    |
 * | -          | Simple error     | Simple    |
 * | :          | Integer          | Simple    |
 * | $          | Bulk string      | Bulk      |
 * | $          | Null bulk string | Bulk      |
 * | *          | Array            | Aggregate |
 *
 * A null bulk string also starts with $. Its length of -1 identifies it as null.
 * Its complete encoding is $-1\r\n.
 */

pub struct RespFormatter;

impl RespFormatter {
    pub fn simple_string(s: &str) -> String {
        format!("+{s}\r\n")
    }

    pub fn simple_error(s: &str) -> String {
        format!("-{s}\r\n")
    }

    pub fn simple_integer(i: i64) -> String {
        format!(":{i}\r\n")
    }

    pub fn bulk_string(s: &str) -> String {
        format!("${}\r\n{s}\r\n", s.len())
    }

    pub fn null_bulk_string() -> String {
        "$-1\r\n".into()
    }
}
