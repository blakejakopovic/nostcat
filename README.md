# noscat

Quick and dirty websocket client for nostr relay scripting


## Examples

Using interactive input
```shell
$ noscat wss://relay.damus.io
["REQ", "RAND", {"kinds": [1], "limit": 8}]
```

Using in piped scripts
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' | nostcat wss://relay.damus.io
```

Using jq to query Nostr JSON events and select the content values
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
nostcat wss://relay.damus.io |
jq '.[2].content'
```

Sort and unique results with multiple servers (dedupe)
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  nostcat --unique wss://relay.damus.io wss://nostr.ono.re
```

Output info log messages which can assist with debugging
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
  RUST_LOG=info nostcat wss://relay.damus.io
```

Pipe events from one server to another (currently limited to 1 event at a time)
```shell
echo '["REQ", "CID", {"limit": 1}]' | nostcat wss://relay.damus.io |
  jq -c 'del(.[1])' |
  wss://nostr.ono.re
```

## Getting started
Using Cargo to install (requires ~/.cargo/bin to be in PATH)
```shell
$ cargo install nostcat
```

Building from source
```shell
$ git clone https://github.com/blakejakopovic/nostcat
$ cargo build --release
```
