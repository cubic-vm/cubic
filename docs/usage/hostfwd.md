# Guest to Host Port Forwarding

```
Guest to host port forwarding commands

List forwarded ports for all instances:
$ cubic net hostfwd list

Forward guest SSH port (TCP port 22) to host on port 8000:
$ cubic net hostfwd add myinstance tcp:127.0.0.1:8000-:22

Remove port forwarding:
$ cubic net hostfwd del myinstance tcp:127.0.0.1:8000-:22

Usage: cubic net hostfwd <COMMAND>

Commands:
  list  List forwarded host ports
  add   Add host port forwarding rule
  del   Delete host port forwarding rule
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')
```
