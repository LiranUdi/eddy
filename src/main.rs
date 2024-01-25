use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

// Cleanup required!
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Body {
    #[serde(rename = "type")]
    msg_type: String,
    msg_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topology: Option<std::collections::HashMap<String, Vec<String>>>,
}

// This is ugly, clean this up!
impl Message {
    fn init(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("init_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                ..Default::default()
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
                ..Default::default()
            },
        }
    }

    fn do_generate(&self, id: u32) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("generate_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                id: Some(id),
                ..Default::default()
            },
        }
    }

    fn do_reply(&self, stdout: &mut impl Write) -> io::Result<()> {
        serde_json::to_writer(&mut *stdout, self)?;
        stdout.write_all(b"\n")?;
        Ok(())
    }

    fn do_broadcast(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("broadcast_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                ..Default::default()
            },
        }
    }

    fn do_broadcast_read(&self, values: &Vec<i32>) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("read_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                messages: Some(values.clone()),
                ..Default::default()
            },
        }
    }

    fn do_topology(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: String::from("topology_ok"),
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                ..Default::default()
            },
        }
    }
}

#[derive(PartialEq)]
enum Events {
    Init,
    Echo,
    Generate,
    Broadcast,
}

fn main() -> io::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();
    let mut rng = rand::thread_rng();

    let init_req: Message =
        serde_json::from_str(&stdin.next().expect("failed to read init").unwrap())
            .expect("failed to serialize");
    let init_reply = init_req.init();
    init_reply.do_reply(&mut stdout)?;

    let mut _node_ids: Vec<String> = init_req.body.node_ids.unwrap();

    drop(stdin);

    let mut messages: Vec<i32> = vec![];
    // Organize the loop, handle multiple events
    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let incoming: Message = serde_json::from_str(&line.expect("failed handling request"))
            .expect("failed handling request");
        if incoming.body.msg_type == String::from("echo") {
            let echo_reply = incoming.do_echo();
            echo_reply.do_reply(&mut stdout)?;
        } else if incoming.body.msg_type == String::from("generate") {
            let id: u32 = rng.gen();
            let gen_reply = incoming.do_generate(id);
            gen_reply.do_reply(&mut stdout)?;
        } else if incoming.body.msg_type == String::from("broadcast") {
            if let Some(message) = incoming.body.message {
                messages.push(message.clone());
            }
            let broadcast_reply = incoming.do_broadcast();
            broadcast_reply.do_reply(&mut stdout)?;
        } else if incoming.body.msg_type == String::from("read") {
            let read_reply = incoming.do_broadcast_read(&messages);
            read_reply.do_reply(&mut stdout)?;
        } else if incoming.body.msg_type == String::from("topology") {
            let topology_reply = incoming.do_topology();
            topology_reply.do_reply(&mut stdout)?;
        } else {
            incoming.do_reply(&mut stdout)?;
        }
    }
    Ok(())
}

// init
// {"src":"c1","dest":"n1","body":{"msg_id":1,"type":"init","node_id":"n1","node_ids":["n1"]}}
// echo
// {"src": "c1","dest": "n1","body": {"type": "echo","msg_id": 1,"echo": "Please echo 35"}}
// gen
// {"src": "c1","dest": "n1","body": {"type": "generate","msg_id": 1}}
// broadcast
// {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "broadcast","message": 210}}
// read
//  {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "read"}}
//  topology
//  {"src":"c1","dest":"n1","body":{"msg_id":3,"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
