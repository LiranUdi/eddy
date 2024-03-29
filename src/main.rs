use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<Payload> {
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
    msg_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

impl<Payload> Message<Payload> {
    fn new_reply(self) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                msg_id: Some(self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                payload: self.body.payload,
            },
        }
    }

    fn do_reply(&self, stdout: &mut impl Write) -> io::Result<()>
    where
        Payload: Serialize,
    {
        serde_json::to_writer(&mut *stdout, self)?;
        stdout.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayloads {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayloads {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum AllPayloads {
    Init(Init),
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        id: usize,
    },
    Broadcast {
        message: Option<usize>,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

fn main() -> io::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    let init_req: Message<InitPayloads> =
        serde_json::from_str(&stdin.next().expect("failed to read init").unwrap())
            .expect("failed to serialize");
    let mut init_reply = init_req.clone().new_reply();
    init_reply.body.payload = InitPayloads::InitOk;
    init_reply.do_reply(&mut stdout)?;

    let mut _node_ids: Vec<String> = match init_req.body.payload {
        InitPayloads::Init(init) => init.node_ids,
        _ => {
            vec![]
        }
    };
    let mut messages: Vec<usize> = vec![];

    drop(stdin);
    drop(stdout);

    let handles = std::thread::spawn(move || {
        let mut stdout = std::io::stdout().lock();
        let mut rng = rand::thread_rng();
        let stdin = std::io::stdin().lock();

        // // Organize the loop, handle multiple events
        for line in stdin.lines() {
            let incoming: Message<AllPayloads> =
                serde_json::from_str(&line.expect("failed handling request"))
                    .expect("failed handling request");
            let mut reply = incoming.clone().new_reply();
            match incoming.body.payload {
                AllPayloads::Echo { echo } => {
                    reply.body.payload = AllPayloads::EchoOk { echo };
                    reply.do_reply(&mut stdout).unwrap();
                }
                AllPayloads::Generate => {
                    let id: usize = rng.gen();
                    reply.body.payload = AllPayloads::GenerateOk { id };
                    reply.do_reply(&mut stdout).unwrap();
                }
                AllPayloads::Broadcast { message } => {
                    if let Some(m) = message {
                        messages.push(m.clone());
                    }

                    reply.body.payload = AllPayloads::BroadcastOk;
                    reply.do_reply(&mut stdout).unwrap();
                }
                AllPayloads::Read => {
                    reply.body.payload = AllPayloads::ReadOk {
                        messages: messages.clone(),
                    };
                    reply.do_reply(&mut stdout).unwrap();
                }
                AllPayloads::Topology { topology: t } => {
                    reply.body.payload = AllPayloads::TopologyOk;
                    reply.do_reply(&mut stdout).unwrap();
                }
                _ => {}
            }
        }
    });

    handles.join().expect("thread panic");
    Ok(())
}
