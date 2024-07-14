# Configure Virtual Machines

## Config Command
```
$ cubic config -h
Read and write configuration parameters

Usage: cubic config [OPTIONS] <INSTANCE>

Arguments:
  <INSTANCE>  

Options:
  -c, --cpus <CPUS>        
  -m, --mem <MEM>          
  -d, --disk <DISK>        
  -s, --sandbox <SANDBOX>  [possible values: true, false]
  -h, --help               Print help
```
**Example:**

Show a virtual machine config:
```
$ cubic config example 
cpus: 4 
mem:  4.0 GiB
disk: 2.2 GiB
```
Change a virtual machine config:
```
$ cubic config --cpus 5 --mem 5G --disk 5G example
cpus: 5 
mem:  5.0 GiB
disk: 5.0 GiB
```
