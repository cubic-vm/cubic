# SH Command
Open shell over a serial connection on the virtual machine.
```
$ cubic sh -h
Open a shell in the machine

Usage: cubic sh [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>  

Options:
  -c, --console  
  -h, --help     Print help
```
**Example**:
```
$ cubic sh example
```

# SSH Command
Connect with SSH to a virtual machine:
```
$ cubic ssh -h
Connect to a machine with SSH

Usage: cubic ssh [OPTIONS] <INSTANCE> [CMD]

Arguments:
  <INSTANCE>  
  [CMD]       

Options:
      --ssh-args <SSH_ARGS>  
  -h, --help                 Print help
```
**Example**:
```
$ cubic ssh example
```
