# Changelog

## v0.3.4
* Added nostcat support for library use
* Added support to omit_eose output (enabled by default)
* General refactoring and code improvements

## v0.3.3
* Fixed --connect-timeout default to 10 seconds (not ms)

## v0.3.2
* Switched from threads back to async
* Increased --connect-timeout default to 10 seconds
* General code improvements

## v0.3.1
* Added --stream flag
* Added --connect-timeout option
* Added multi-line stdin support
* Added support for piping server to server
* Added websocket PONG response
* Added default one second connection timeout (non-streaming) (Thanks @jb55)
* Updated app description
* Migrate from async to threads
* Refactored main.rs + lib.rs
* Added rustfmt
* Fixed README.md example typo
* Removed tokio dependancy

## v0.3.0
* Added CLI handler
* Added multiple server connections
* Added --unique flag

## v0.2.0
* Added RUST_LOG info messages
* Refactored main.rs

## v0.1.0 - Initial release
* Simple websocket write and receive
