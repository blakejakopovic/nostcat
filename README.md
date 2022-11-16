# noscat

Quick and dirty websocket client for nostr relay scripting


## Examples

Using interactive input
```shell
$ noscat wss://relay.damus.io
["REQ", "RAND", {"kinds": [1], "limit": 8}]
```

Using piped input
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' | nostcat wss://relay.damus.io
```

Using jq to query Nostr JSON events
```shell
$ echo '["REQ", "RAND", {"kinds": [1], "limit": 8}]' |
nostcat wss://relay.damus.io |
jq '.[2].content'
```

## Getting started
```shell
$ git clone https://github.com/blakejakopovic/nostcat
$ cargo build --release
```
