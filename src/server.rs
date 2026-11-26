use std::{collections::HashMap, sync::Arc};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::Mutex};

use crate::{protocal::{Resp::{self, BulkString, Integer, NullString, SimpleError, SimpleString}, bulk_string, null_bulk_string, parse_buffer_to_resp, simple_error, simple_integer, simple_string}, storage::{DataTypes, Storage}};

/* 
    * The way RESP is used in Redis as a request-response protocol is the following:

    * Clients send commands to a Redis server as a RESP Array of Bulk Strings.
    * The server replies with one of the RESP types according to the command implementation.
*/

pub async fn hanlde_connection(stream: &mut TcpStream, storage: Arc<Mutex<Storage>>) {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                return;
            }
            Ok(size) => if size > 0 {
                let mut index: usize = 0;
                let request = match parse_buffer_to_resp(&buffer,&mut index) {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("Error with request parsing");
                        return;
                    },
                };

                let response = process_request(request, storage.clone()).await;

                match response {
                    SimpleString(value) => {
                        if let Err(e) = stream.write_all(value.as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);
                        }
                    },
                    BulkString(value) => {
                        if let Err(e) = stream.write_all(value.as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);
                        }  
                    },
                    SimpleError(value) => {
                        if let Err(e) = stream.write_all(value.as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);
                        }
                    },
                    NullString(value) => {
                        if let Err(e) = stream.write_all(value.as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);  
                        }
                    }
                    Integer(value) => {
                        let value = simple_integer(value);

                        if let Err(e) = stream.write_all(value.as_bytes()).await {
                            eprintln!("Error writing to socket: {}", e);
                        }  
                    },
                    _ => {
                        return;
                    }
                }
            }
            Err(error) => {
                eprintln!("Read error: {error}");
                break;
            }
        }
    }
}

/* 
    * The redis-cli sends requests based on Array of Bulk Strings
*/
pub async fn process_request(request: Resp, storage: Arc<Mutex<Storage>>) -> Resp {
    let elements = match request {
        Resp::SimpleString(_) => todo!(),
        Resp::SimpleError(_) => todo!(),
        Resp::Integer(_) => todo!(),
        Resp::BulkString(_) => todo!(),
        Resp::NullString(_) => todo!(),
        Resp::Array(resps) => {
            resps
        },
        Resp::NullArray => todo!(),
    };

    let mut commands = Vec::new();

    for elem in elements {
        if let Resp::BulkString(value) = elem {
            commands.push(value);
        }
    }

    let mut gaurd = storage.lock().await;
    process_commands(&commands, &mut gaurd)    
}

fn process_commands(commands: &[String], storage: &mut Storage) -> Resp {
    match commands[0].to_lowercase().as_str() {
        "ping" => {
            let ping = simple_string("PONG");
            return Resp::SimpleString(ping)
        },
        "echo" => {
            let echo = bulk_string(&commands[1]);
            return Resp::BulkString(echo)
        }
        "set" => {
            if commands.len() != 3 {
                let error = simple_error("(error) ERR wrong number of arguments for 'set' command");
                return Resp::SimpleError(error)
            }

            let value = match storage.set(commands[1].clone(), commands[2].clone()) {
                Ok(v) => v,
                Err(_) => todo!(),
            };

            let value = simple_string(&value);
            return Resp::SimpleString(value)
        },
        "get" => {
            if commands.len() != 2 {
                let error = simple_error("(error) ERR wrong number of arguments for 'get' command");
                return Resp::SimpleError(error)  
            }

            if let Some(v) = storage.get(commands[1].clone()) {
                let value = simple_string(&v);
                return Resp::SimpleString(value) 
            } else {
                let null = null_bulk_string();
                return Resp::NullString(null)
            }   
        },
        "del" => {
            if commands.len() < 2 {
                let error = simple_error("(error) ERR wrong number of arguments for 'del' command");
                return Resp::SimpleError(error)
            }

            let deletion_num: usize = commands[1..]
                .iter()
                .map(|key| storage.del(key.clone()))
                .sum();
            Resp::Integer(deletion_num as i64)            
        }
        "append" => {
            if commands.len() != 3 {
                let error = simple_error("(error) ERR wrong number of arguments for 'append' command");
                return Resp::SimpleError(error)
            }

            let insertion_num = storage.append(commands[1].clone(), commands[2].clone());
            Resp::Integer(insertion_num as i64)
        }
        _ => {
            let error = simple_string("command not found");
            return Resp::SimpleError(error)
        }
    }
}
