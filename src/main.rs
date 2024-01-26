use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::{self, BufRead, Write}};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

// Cleanup required!
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<Payload> {
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
    msg_id: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

// This is ugly, clean this up!
impl <Payload> Message<Payload> {
    fn new_reply(self) -> Self{
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                msg_id: Some(self.body.msg_id.unwrap() + 1),
                in_reply_to: self.body.msg_id,
                payload: self.body.payload,
            }
        }
    }

    fn do_reply(&self, stdout: &mut impl Write) -> io::Result<()> 
    where 
        Payload: Serialize
    {
        serde_json::to_writer(&mut *stdout, self)?;
        stdout.write_all(b"\n")?;
        Ok(())
    }


    //     Message {
    //         src: self.dest.clone(),
    //         dest: self.src.clone(),
    //         body: Body {
    //             payload: Events::TopologyOk,
    //             msg_id: Some(&self.body.msg_id.unwrap() + 1),
    //             in_reply_to: self.body.msg_id,
    //             node_id: None,
    //             node_ids: None,
    //             message: None,
    //             topology: None,
    //         },
    //     }
    // }
}

// #[derive(Clone, Serialize, Deserialize, Debug)]
// #[serde(rename_all = "snake_case")]
// #[serde(tag = "type")]
// enum Events {
//     Init,
//     InitOk,
//     Echo { echo: String },
//     Generate,
//     Broadcast,
//     EchoOk { echo: String },
//     GenerateOk { id: usize },
//     BroadcastOk,
//     ReadOk { messages: Vec<usize> },
//     Read,
//     Topology,
//     TopologyOk,
// }

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
    Echo { echo: String},
    EchoOk { echo: String},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum AllPayloads {
    Init(Init),
    InitOk,
    Echo { echo: String},
    EchoOk { echo: String},
    Generate,
    GenerateOk { id: usize },
    Broadcast { message: Option<usize> },
    BroadcastOk,
    Read,
    ReadOk { messages: Vec<usize> },
    Topology { topology: HashMap<String, Vec<String>> },
    TopologyOk,
}


fn main() -> io::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();
    let mut rng = rand::thread_rng();

    let init_req: Message<InitPayloads> =
        serde_json::from_str(&stdin.next().expect("failed to read init").unwrap())
            .expect("failed to serialize");
    let mut init_reply = init_req.new_reply();
    init_reply.body.payload = InitPayloads::InitOk;
    init_reply.do_reply(&mut stdout)?;

    //let mut _node_ids: Vec<String> = init_req.body.node_ids.unwrap();
    let mut messages: Vec<usize> = vec![];

    drop(stdin);

    // // Organize the loop, handle multiple events
    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let incoming: Message<AllPayloads> = serde_json::from_str(&line.expect("failed handling request"))
            .expect("failed handling request");
        let mut reply = incoming.clone().new_reply();
        match incoming.body.payload {
            AllPayloads::Echo { echo } => {
                reply.body.payload = AllPayloads::EchoOk { echo };
                reply.do_reply(&mut stdout)?;
            },
            AllPayloads::Generate => {
                let id: usize = rng.gen();
                reply.body.payload = AllPayloads::GenerateOk { id };
                reply.do_reply(&mut stdout)?;
            },
            AllPayloads::Broadcast { message } =>{
                if let Some(m) = message {
                    messages.push(m.clone());
                }

                reply.body.payload = AllPayloads::BroadcastOk;
                reply.do_reply(&mut stdout)?;
            },
            AllPayloads::Read => {
                reply.body.payload = AllPayloads::ReadOk { messages: messages.clone() };
                reply.do_reply(&mut stdout)?;
            },
            AllPayloads::Topology { topology: _ } => {
                reply.body.payload = AllPayloads::TopologyOk;
                reply.do_reply(&mut stdout)?;
            }
            _ => {}
        }
    }
    Ok(())
}

// init
// {"src":"c1","dest":"n1","body":{"msg_id":1,"type":"init","node_id":"n1","node_ids":["n1"]}}
// echo
// {"src": "c1","dest": "n1","body": {"type": "echo", "msg_id": 1, "echo": "Please echo 35"}}
// gen
// {"src": "c1","dest": "n1","body": {"type": "generate","msg_id": 1}}
// broadcast
// {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "broadcast","message": 210}}
// read
//  {"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "read"}}
//  topology
//  {"src":"c1","dest":"n1","body":{"msg_id":3,"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
