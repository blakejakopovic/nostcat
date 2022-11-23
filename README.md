# nostcat

Websocket client for nostr relay scripting


## Examples

Using interactive input
```shell
$ nostcat wss://relay.damus.io <return>
["REQ", "RAND", {"kinds": [1], "limit": 8}] <return>
<ctrl-D>
```

Using stdin (supports multiple lines of commands)
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat wss://relay.damus.io

$ cat commands.txt
["REQ", "RAND", {"kinds": [1], "limit": 2}]
["REQ", "RAND2", {"kinds": [2], "limit": 2}]

$ cat commands.txt | nostcat wss://relay.damus.io

```

Using jq to query Nostr JSON events and select the event JSON
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat wss://relay.damus.io |
  jq '.[2]'

  $ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat wss://relay.damus.io |
  jq '.[2].content'
```

Unique (dedupe) results as they come in (note: no longer applies sorting events - FIFO)
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat --unique wss://relay.damus.io wss://nostr.ono.re
```

Stream websocket data (like tail -f)
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat --stream wss://relay.damus.io
```

Output info log messages which can assist with debugging
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  RUST_LOG=info nostcat wss://relay.damus.io
```

Pipe events from one server to another (currently limited to 1 event at a time)
```shell
$ echo '["REQ", "CID", {"limit": 1}]' |
  nostcat wss://relay.damus.io |
  jq -c 'del(.[1])' |
  nostcat wss://nostr.ono.re
```

Pipe events from one server to another (for multiple events, `ctrl-C` when finished)
```shell
$ echo '["REQ", "CID", {"limit": 3}]' |
  nostcat wss://relay.damus.io |
  jq -c 'del(.[1])' |
  nostcat --stream wss://nostr.ono.re
  <ctrl-C>
```


## Getting started
Using Cargo to install (requires ~/.cargo/bin to be in PATH)
```shell
$ cargo install nostcat
```

Building from source (may be unstable)
```shell
$ git clone https://github.com/blakejakopovic/nostcat
$ cargo build --release
```
