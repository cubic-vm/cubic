# SH Command
Open shell over a serial connection on the virtual machine instance.
```
$ cubic sh --help
Open a shell in an instance

Usage: cubic sh [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>

Options:
  -c, --console
  -v, --verbose
  -h, --help     Print help
```
**Example**:
```
$ cubic sh example
```

# SSH Command
Connect with SSH to a virtual machine instance:
```
$ cubic ssh --help
Connect to an instance with SSH

Usage: cubic ssh [OPTIONS] <INSTANCE> [CMD]

Arguments:
  <INSTANCE>
  [CMD]

Options:
  -X                         Forward X over SSH
  -v, --verbose
      --ssh-args <SSH_ARGS>
  -h, --help                 Print help
```

**Example**:
```
$ cubic ssh example
```
