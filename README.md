# mssh - Multi-Server SSH CLI

A fast, concurrent SSH command executor for running commands across multiple servers simultaneously.

## Requirements

- Rust 1.88.0 or later
- SSH client installed
- Proper SSH key configuration

## Features

- **Concurrent execution** - Run commands on multiple servers at once
- **Group management** - Organize servers into groups and nested groups
- **Connection limiting** - Max 5 concurrent connections to prevent overwhelming servers
- **Colored output** - Easy to distinguish between different servers
- **Simple configuration** - Plain text config file

## Installation

```bash
cargo build --release
cp target/release/mssh /usr/local/bin/
```

## Configuration

Create `~/.mssh_groups` file:

```ini
# Web servers
[web]
user@server1.example.com
user@server2.example.com

# Database servers
[db]
user@db1.example.com
user@db2.example.com

# Production environment (references other groups)
[prod:g]
web
db

```

### Group Types

- **Server groups**: `[web]` - Contains actual server addresses
- **Reference groups**: `[prod:g]` - Contains references to other groups

## Usage

```bash
mssh @<group> <command> [args...]
```

### Examples

```bash
# List files on all web servers
mssh @web ls -la

# Check process status on production servers
mssh @prod ps aux

# Check disk usage on all servers
mssh @all df -h
```

## Output Format

Successful commands:

```
[server1.example.com]
total 8
drwxr-xr-x  2 user user 4096 Jan 1 12:00 .
drwxr-xr-x 20 user user 4096 Jan 1 12:00 ..
```

Error output:

```
[server2.example.com] ERROR
ssh: connect to host server2.example.com port 22: Connection refused
```
