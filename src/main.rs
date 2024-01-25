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
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "type")]
    msg_type: Events,
    msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<usize>,
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
                msg_type: Events::InitOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: None,
                id: None,
                node_id: None,
                node_ids: None,
                messages: None,
                message: None,
                topology: None,
            },
        }
    }

    fn do_echo(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: Events::EchoOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: self.body.echo.clone(),
                id: None,
                node_id: None,
                node_ids: None,
                messages: None,
                message: None,
                topology: None,
            },
        }
    }

    fn do_generate(&self, id: usize) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: Events::GenerateOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                id: Some(id),
                echo: None,
                node_id: None,
                node_ids: None,
                messages: None,
                message: None,
                topology: None,
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
                msg_type: Events::BroadcastOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: None,
                id: None,
                node_id: None,
                node_ids: None,
                messages: None,
                message: None,
                topology: None,
            },
        }
    }

    fn do_broadcast_read(&self, values: &Vec<usize>) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: Events::ReadOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                messages: Some(values.clone()),
                echo: None,
                id: None,
                node_id: None,
                node_ids: None,
                message: None,
                topology: None,
            },
        }
    }

    fn do_topology(&self) -> Self {
        Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_type: Events::TopologyOk,
                msg_id: Some(&self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                echo: None,
                id: None,
                node_id: None,
                node_ids: None,
                message: None,
                topology: None,
                messages: None,
            },
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Events {
    Init,
    Echo,
    Generate,
    Broadcast,
    InitOk,
    EchoOk,
    GenerateOk,
    BroadcastOk,
    ReadOk,
    Read,
    Topology,
    TopologyOk,
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

    let mut messages: Vec<usize> = vec![];
    // Organize the loop, handle multiple events
    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let incoming: Message = serde_json::from_str(&line.expect("failed handling request"))
            .expect("failed handling request");

        match incoming.body.msg_type {
            Events::Echo => {
                let echo_reply = incoming.do_echo();
                echo_reply.do_reply(&mut stdout)?;
            }
            Events::Generate => {
                let id: usize = rng.gen();
                let gen_reply = incoming.do_generate(id);
                gen_reply.do_reply(&mut stdout)?;
            }
            Events::Broadcast => {
                if let Some(message) = incoming.body.message {
                    messages.push(message.clone());
                }
                let broadcast_reply = incoming.do_broadcast();
                broadcast_reply.do_reply(&mut stdout)?;
            }
            Events::Read => {
                let read_reply = incoming.do_broadcast_read(&messages);
                read_reply.do_reply(&mut stdout)?;
            }
            Events::Topology => {
                let topology_reply = incoming.do_topology();
                topology_reply.do_reply(&mut stdout)?;
            }
            _ => {}
        }
    }
    Ok(())
}

// init
// {"src":"c1","dest":"n1","body":{"msg_id":1,"type":"init","node_id":"n1","node_ids":["n1"]}}
// echn
// {"src": "c1","dest": "n1","body": {"type": "echo","msg_id": 1,"echo": "Please echo 35"}}
// gen
// {"src": "c1","dest": "n1","body": {"type": "generate","msg_id": 1}}
// broadcast
// {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "broadcast","message": 210}}
// read
//  {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "read"}}
//  topology
//  {"src":"c1","dest":"n1","body":{"msg_id":3,"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
