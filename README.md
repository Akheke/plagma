# plugma for Windows11(ver0.1.2)
"plugma" is a CLI tool for encrypting and decrypting text.

# DEMO
This is a free tool that allows two people to exchange encrypted messages once they have exchanged a shared key.

# Features
This tool treats all files that perform encryption as plugins, allowing you to easily create your own encryption processes or combine them within a folder to build a custom encryption workflow. Additionally, since you specify which folders to encrypt, you can use different encryption processes for different folders.

# Changes from the previous version(ver0.1.1)
- The process generating keys was moved from the plugins to the plugma
- Added a file to store the generated key
- Added a temprate file for plugin developers

# Requirements

- Rust compiler: rustc 1.96.0 (ac68faa20 2026-05-25)
- Cargo (included with Rust)

# Dependencies (Cargo.toml)
The following crates are used in this project:

- base64 = "0.22.1"
- chacha20poly1305 = "0.10"
- clap = { version = "4.6.1", features = ["derive"] }
- hex = "0.4.3"
- rand = "0.8"
- hkdf = "0.12"
- sha2 = "0.10"
- x25519-dalek = "1"
- rand_core = "0.5"
- crossterm = "0.29.0"

Cargo automatically resolves and installs all dependencies.

# Setting up Dependencies

All dependencies listed in `Cargo.toml` can be installed automatically using Cargo:

```bash
cargo fetch
```
#  Usage

Build the project:

```bash
cargo build
```

Determine a folder that contains the files used for the encryption and decryption processes (hereinafter referred to as the “plugin” folder).
If you haven't made any changes, it will work if you specify “plugin.”

Create a private key for encryption

```
plugma key

```

Verify the public key generated at the same time as the private key, and send it to the other party.

```
type keys\pub.key
```
This applies to the Windows Command Prompt. Please adjust the command to open a text file according to your operating system.

Register the other party's public key in plugma

```
plugma register -E <plugin path>
```

Encrypt Text

When copying content
```
plugma encrypt -o std -E <plugin path> -t <your text>
```
When outputting to a file
```
plugma encrypt -o file --output-path <file path> -E <plugin path> -t <your text>
```
When loading a file to be encrypted
```
plugma encrypt -o std -E <plugin path> --target-path <your file path>
```

Decode Text

When copying content
```
plugma decode -o std -E <The path to the plugin> -t <your text>
```
When outputting to a file
```
plugma decode -o file --output-path <file path> -E <plugin path>
```
When loading a file to be encrypted
```
plugma decode -o std -E <plugin path> --target-path <your file path>
```

# Note
I don't test environments under Linux and Mac.
If you want to build for an operating system like Linux, some of the code I've commented out might be helpful.

# To Developers
The plugin folder containing the files for the encryption process must meet the following requirements:
・It must contain executable files and a .order file specifying the execution order of those files.
・There must be only one .order file.
The .order file records the paths to the executable files, with the folder as the current directory. Please separate the paths of the files with a semicolon (;). Note that while the process will run even if you create folders, a confirmation prompt may appear asking whether to ignore non-executable files and continue the process unless you specify the -f opti

# Author
* Akheke

# License

"plugma" is under [MIT license](https://en.wikipedia.org/wiki/MIT_License) or [Apache-2.0](https://en.wikipedia.org/wiki/Apache_License).

Let's enjoy some private chats!!
Thank you!