# Configure Virtual Machines

## Config Command
```
$ cubic instance config --help
Read and write configuration parameters

Usage: cubic instance config [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>

Options:
  -c, --cpus <CPUS>
  -m, --mem <MEM>
  -d, --disk <DISK>
  -h, --help         Print help
```
**Example:**

Show a virtual machine config:
```
$ cubic instance config example
cpus: 4
mem:  4.0 GiB
disk: 2.2 GiB
```

Change a virtual machine config:
```
$ cubic instance config --cpus 5 --mem 5G --disk 5G example
cpus: 5
mem:  5.0 GiB
disk: 5.0 GiB
```
