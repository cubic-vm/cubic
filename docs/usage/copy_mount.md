# Transfer Directories and Files

## SCP Command
```
$ cubic scp --help
Copy a file from or to a virtual machine instance with SCP

Usage: cubic scp [OPTIONS] <FROM> <TO>

Arguments:
  <FROM>  Source of the data to copy
  <TO>    Target of the data to copy

Options:
  -v, --verbose              Enable verbose logging
  -q, --quiet                Reduce logging output
      --scp-args <SCP_ARGS>  Pass additional SCP arguments
  -h, --help                 Print help
```

**Example:**
Copy a file from the host to an virtual machine instance:
```
$ touch test
$ cubic scp test example:~/
```

Copy a directory from the virtual machine instance to the host:
```
$ cubic scp example:~/Documents/ .
```

## Mount Command
```
$ cubic mount --help
Mount commands

Usage: cubic mount <COMMAND>

Commands:
  list  List mount mounts
  add   Add a directory mount
  del   Delete a directory mount
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

**Example:**
Mount a host directory to the virtual machine instance:
```
$ cubic mount add example /home/tux/Documennts /home/cubic/Documents
```

List mounts of virtual machine instance:
```
$ cubic mount list example
HOST                           GUEST
/home/tux/Documennts           /home/cubic/Documents
```

Unmount a directory:
```
$ cubic mount del example /home/cubic/Documents
```
