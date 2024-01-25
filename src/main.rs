use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

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
                id: None,
                node_id: None,
                node_ids: None,
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
                id: None,
                node_id: None,
                node_ids: None,
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
                echo: None,
                id: Some(id),
                node_id: None,
                node_ids: None,
            },
        }
    }

    fn do_reply(&self, stdout: &mut impl Write) -> io::Result<()> {
        serde_json::to_writer(&mut *stdout, self)?;
        stdout.write_all(b"\n")?;
        Ok(())
    }
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
    init_req.do_reply(&mut stdout)?;

    drop(stdin);

    // Organize the loop, handle multiple events
    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        let incoming: Message = serde_json::from_str(&line.expect("failed to read echo"))
            .expect("failed to serialize echo");

        if incoming.body.msg_type == String::from("echo") {
            let echo_reply = incoming.do_echo();
            echo_reply.do_reply(&mut stdout)?;
        } else if incoming.body.msg_type == String::from("generate") {
            let id: u32 = rng.gen();
            let gen_reply = incoming.do_generate(id);
            gen_reply.do_reply(&mut stdout)?;
        }
    }
    Ok(())
}

// init
// {src: "c1", dest: "n1", body: {msg_id: 1, type: "init", node_id: "n1", node_ids: ["n1"]}}
// echo
// {"src": "c1","dest": "n1","body": {"type": "echo","msg_id": 1,"echo": "Please echo 35"}}
// gen
// {"src": "c1","dest": "n1","body": {"type": "generate","msg_id": 1}}
//
