# SH Command
Open shell over a serial connection on the virtual machine instance.
```
$ cubic sh --help
Open a shell in a virtual machine instance

Usage: cubic sh [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>  Name of the virtual machine instance

Options:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Reduce logging output
  -h, --help     Print help
```
**Example**:
```
$ cubic sh example
```

# SSH Command
Connect with SSH to a virtual machine instance:
```
Usage: cubic ssh [OPTIONS] <INSTANCE> [CMD]

Arguments:
  <INSTANCE>  Name of the virtual machine instance
  [CMD]       Execute a command in the virtual machine

Options:
  -X                         Forward X over SSH
  -v, --verbose              Enable verbose logging
  -q, --quiet                Reduce logging output
      --ssh-args <SSH_ARGS>  Pass additional SSH arguments
  -h, --help                 Print help
```

**Example**:
```
$ cubic ssh example
```
