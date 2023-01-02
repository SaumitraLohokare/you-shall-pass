# you-shall-pass
Password manager built in Rust using SurrealDB and MagicCrypt.

## Features
- [x] Store using file backed SurrealDB.
- [x] Encrypt passwords before storing them.
- [ ] Find a beter solution to generate a secret key for MagicCrypt.

## Usage
```
Usage: ysp [COMMAND]

Commands:
  save         Save a new password.
  get-pass     Get the password for a username.
  update-pass  Update the password for a username.
  delete       Delete entry for username.
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```
