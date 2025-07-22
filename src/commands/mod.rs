use crate::db::Database;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
mod cmds;

//
// ─── Utility Functions ───────────────────────────────────────────────────────────
//

#[inline]
fn normalize_command(input: &[u8]) -> &[u8] {
    let mut end = input.len();
    while end > 0 && (input[end - 1] == b'\r' || input[end - 1] == b'\n') {
        end -= 1;
    }
    &input[..end]
}

//
// ─── Command Enum and Matching ───────────────────────────────────────────────────
//

#[derive(Debug)]
enum Command {
    Ping,
    Hello,
    Exit,
    Set,
    Get,
    Del,
    Drop,
    Memory,
    Size,
    Unknown,

}

fn dispatch_command(cmd: &[u8]) -> Command {
    let s = std::str::from_utf8(cmd).unwrap_or("").to_ascii_lowercase();
    match s.as_str() {
        // basic
        "ping" => Command::Ping,
        "hello" => Command::Hello,
        "exit" => Command::Exit,
        "quit" => Command::Exit,
        "memory" => Command::Memory,
        "size" => Command::Size,

        // core commands
        "set" => Command::Set,
        "get" => Command::Get,
        "del" => Command::Del,
        "drop" => Command::Drop,
        "incr" => Command::Unknown,
        "decr" => Command::Unknown,

        // many operations
        "mget" => Command::Unknown,
        "mset" => Command::Unknown,

        // regional commands
        "regiget" => Command::Unknown, // regiget client:users:emails a@gmail.com b@gmail.com .....
        "regiset" => Command::Unknown, // regiset client:users:emails a@gmail.com val1 b@gmail.com val2 .....
        "regidel" => Command::Unknown, // regidel client:users:emails a@gmail.com b@gmail.com .....
        "regigetall" => Command::Unknown,
        "regigetn" => Command::Unknown,
        "regiincr" => Command::Unknown,
        "regidecr" => Command::Unknown,

        // list based operations
        "lpush" => Command::Unknown,
        "rpush" => Command::Unknown,
        "lset" => Command::Unknown,
        "linsert" => Command::Unknown,
        "lremove" => Command::Unknown,
        "lindex" => Command::Unknown,
        "lcount" => Command::Unknown,

        // many list operations
        "lmpush" => Command::Unknown,
        "rmpush" => Command::Unknown,
        "lmset" => Command::Unknown,
        "lmindex" => Command::Unknown,

        // set based operations
        "sadd" => Command::Unknown, // sadd path wdwjndw
        "srem" => Command::Unknown,
        "smove" => Command::Unknown,     // smove path newnjk path2
        "sismember" => Command::Unknown, // presence of an item in a set
        "scount" => Command::Unknown,
        "smembers" => Command::Unknown, // returns all set items
        "spop" => Command::Unknown,
        "srandmember" => Command::Unknown,
        "sunion" => Command::Unknown,
        "sinter" => Command::Unknown,
        "sdiff" => Command::Unknown,
        "sunionstore" => Command::Unknown,
        "sinterstore" => Command::Unknown,
        "sdiffstore" => Command::Unknown,

        // Unknown
        _ => Command::Unknown,
    }
}

//
// ─── Main Entry Point: Command Handler ──────────────────────────────────────────
//

pub async fn handle_command(
    line: &str,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<bool> {
    let raw = normalize_command(line.as_bytes());
    let parts = cmds::CommandParts::new(raw);
    if parts.len() == 0 {
        return cmds::write_response(writer, b"Empty command\n")
            .await
            .map(|_| false);
    }
    let cmd = parts.get(0).unwrap();
    match dispatch_command(cmd) {
        Command::Ping => cmds::write_response(writer, b"PONG\n").await?,
        Command::Hello => cmds::write_response(writer, b"Hi there! FlashTree v0.1\n").await?,
        Command::Exit => {
            cmds::write_response(writer, b"Bye!\n").await?;
            return Ok(true);
        }
        Command::Set => cmds::handle_set(&parts, writer, database).await?,
        Command::Get => cmds::handle_get(&parts, writer, database).await?,
        Command::Del => cmds::handle_del(&parts, writer, database).await?,
        Command::Drop => cmds::handle_drop(writer, database).await?,
        Command::Memory => cmds::handle_memory(writer, database).await?,
        Command::Size => cmds::handle_size(writer, database).await?,
        Command::Unknown => cmds::write_response(writer, b"Unknown command\n").await?,
    }
    Ok(false)
}
