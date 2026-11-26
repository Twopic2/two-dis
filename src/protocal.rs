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

/*
  ! Resp Formater
*/

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

pub fn string_array(parts: &[String]) -> String {
    format!("*{}\r\n", parts.len())
        + &parts
            .iter()
            .map(|p| bulk_string(p))
            .collect::<Vec<String>>()
            .join("")
}

pub fn array(parts: &[Resp]) -> String {
    let mut message = format!("*{}\r\n", parts.len());

    for part in parts {
        let formatted = match part {
            Resp::SimpleString(value) => simple_string(value),
            Resp::SimpleError(value) => simple_error(value),
            Resp::Integer(value) => simple_integer(*value),
            Resp::BulkString(value) => bulk_string(value),
            Resp::NullString(_) => null_bulk_string(),
            Resp::Array(values) => array(values),
            Resp::NullArray => "*-1\r\n".into(),
        };
        message.push_str(&formatted);
    }

    message
}

/*
    ! RespParser
*/

#[derive(Debug, PartialEq, Eq)]
pub enum Resp {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    NullString(String),
    Array(Vec<Self>),
    NullArray,
}

pub fn parse_buffer_to_resp(buffer: &[u8], index: &mut usize) -> Result<Resp, ()> {
    let mut next = *index;
    let resp_type = *buffer.get(next).ok_or(())?;
    next += 1;

    let value = match resp_type {
        b'+' => Resp::SimpleString(get_line(buffer, &mut next)?),
        b'-' => Resp::SimpleError(get_line(buffer, &mut next)?),
        b':' => Resp::Integer(get_line(buffer, &mut next)?.parse().map_err(|_| ())?),
        b'$' => parse_bulk_string(buffer, &mut next)?,
        b'*' => parse_array(buffer, &mut next)?,
        _ => return Err(()),
    };
    *index = next;
    Ok(value)
}

fn get_line(buffer: &[u8], index: &mut usize) -> Result<String, ()> {
    let data = buffer.get(*index..).ok_or(())?;

    let mut final_pos = 0;
    while final_pos < data.len() && data[final_pos] != b'\r' && data[final_pos] != b'\n' {
        final_pos += 1;
    }

    if !data[final_pos..].starts_with(b"\r\n") {
        return Err(());
    }
    let message = String::from_utf8(data[..final_pos].to_vec()).unwrap();
    *index += final_pos + 2;

    Ok(message)
}

fn parse_length(buffer: &[u8], index: &mut usize) -> Result<usize, ()> {
    let line = get_line(buffer, index)?;
    Ok(line.parse().unwrap())
}

fn parse_bulk_string(buffer: &[u8], index: &mut usize) -> Result<Resp, ()> {
    let length = parse_length(buffer, index)?;

    let data = &buffer[*index..];
    let bytes = data.get(..length).ok_or(())?;
    if !data[length..].starts_with(b"\r\n") {
        return Err(());
    }

    let message = String::from_utf8(bytes.to_vec()).unwrap();
    *index += length + 2;

    Ok(Resp::BulkString(message))
}

fn parse_array(buffer: &[u8], index: &mut usize) -> Result<Resp, ()> {
    let length = parse_length(buffer, index)?;

    let mut array = Vec::new();
    for _ in 0..length {
        array.push(parse_array_elements(buffer, index)?);
    }

    Ok(Resp::Array(array))
}

fn parse_array_elements(buffer: &[u8], index: &mut usize) -> Result<Resp, ()> {
    parse_buffer_to_resp(buffer, index)
}
