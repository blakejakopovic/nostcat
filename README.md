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

Output info log messages which can help when debugging
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' | RUST_LOG=info nostcat wss://relay.damus.io
```

## Getting started
```shell
$ git clone https://github.com/blakejakopovic/nostcat
$ cargo build --release
```
