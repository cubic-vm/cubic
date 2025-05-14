# Configure Virtual Machines

## Instance Config Command
```
$ cubic config --help
Read and write virtual machine instance configuration parameters

Usage: cubic config [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>  Name of the virtual machine instance

Options:
  -c, --cpus <CPUS>  Number of CPUs for the virtual machine instance
  -m, --mem <MEM>    Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
  -d, --disk <DISK>  Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
  -h, --help         Print help
```
**Example:**
Change a virtual machine instance config:
```
$ cubic config --cpus 5 --mem 5G --disk 5G example
```

## Instance Info Command
```
Get information about an virtual machine instance

Usage: cubic info <INSTANCE>

Arguments:
  <INSTANCE>  Name of the virtual machine instance

Options:
  -h, --help  Print help
```

Show a virtual machine instance configuration:
```
$ cubic info example
cpus: 4
mem:  4.0 GiB
disk: 2.2 GiB
```
