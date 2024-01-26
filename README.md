# eddy

An extremely simple, unfancy, solution to Fly.io's distributed systems challenge with Maelstrom using Rust.
https://fly.io/dist-sys/

Will add and improve when I'm bored

## Request-Response Examples:
Init:
```json
{"src":"c1","dest":"n1","body":{"msg_id":1,"type":"init","node_id":"n1","node_ids":["n1"]}}
```

Init Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":1,"msg_id":2,"type":"init_ok"}}
```

Echo:
```json
{"src": "c1","dest": "n1","body": {"type": "echo", "msg_id": 1, "echo": "Please echo 35"}}
```

Echo Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":1,"msg_id":2,"type":"echo_ok","echo":"Please echo 35"}}
```

Generate:
```json
{"src": "c1","dest": "n1","body": {"type": "generate","msg_id": 1}}
```
Generate Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":1,"msg_id":2,"type":"generate_ok","id":5225959968650978295}}
```

Broadcast:
```json
{"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "broadcast","message": 210}}
```

Broadcast Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":3,"msg_id":4,"type":"broadcast_ok"}}
```
P
Read:
```json
{"src": "c1","dest": "n1","body": {"msg_id": 3, "type": "read"}}
```

Read Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":3,"msg_id":4,"type":"read_ok","messages":[210]}}
```

Topology:
```json
{"src":"c1","dest":"n1","body":{"msg_id":3,"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
```

Topology Ok:
```json
{"src":"n1","dest":"c1","body":{"in_reply_to":3,"msg_id":4,"type":"topology_ok"}}
```

## Resources:
Maelstrom documentation:
https://github.com/jepsen-io/maelstrom/tree/main/doc

Fly.io's challenge page:
https://fly.io/dist-sys

Serde documentation:
https://serde.rs/
https://serde.rs/enum-representations.html
https://serde.rs/attr-flatten.html
https://serde.rs/attributes.html
https://serde.rs/impl-serializer.html
https://serde.rs/impl-deserializer.html