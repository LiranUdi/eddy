use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Error, StdoutLock, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "type")]
    msg_type: String,
    msg_id: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<String>,
}

impl Message {
    fn init(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("init_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: None,
            },
        }
    }

    fn do_echo(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("echo_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: self.body.echo.clone(),
            },
        }
    }
}

fn main() -> io::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();
    let mut in_id: u8 = 128;

    // {"id":0,"src":"c0","dest":"n0","body":{"type":"init","node_id":"n0","node_ids":["n0"],"msg_id":1}}
    let init_req: Message =
        serde_json::from_str(&stdin.next().expect("failed to read init").unwrap())
            .expect("failed to serialize");
    let init_reply = init_req.init();
    let json = serde_json::to_string(&init_reply)?;
    serde_json::to_writer(&mut stdout, &init_reply)?;
    &stdout.write_all(b"\n")?;

    drop(stdin);

    // Organize the loop, handle multiple events
    // {"src": "c1","dest": "n1","body": {"type": "echo","msg_id": 1,"echo": "Please echo 35"}}
    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let echo_req: Message = serde_json::from_str(&line.expect("failed to read echo"))
            .expect("failed to serialize echo");
        let echo_reply = echo_req.do_echo();
        let json = serde_json::to_string(&echo_reply)?;
        serde_json::to_writer(&mut stdout, &echo_reply)?;
        &stdout.write_all(b"\n")?;
    }
    Ok(())
}
