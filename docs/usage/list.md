# List Images and Virtual Machines

## List Command
```
$ cubic list -h
List images and machines

Usage: cubic list [OPTIONS] [NAME]

Arguments:
  [NAME]  

Options:
  -a, --all   
  -h, --help  Print help
```
**Example**:

List virtual machines:
```
$ cubic list
Name             CPUs     Memory       Disk  State     
noble               1    1.0 GiB    2.0 GiB  STOPPED  
```

List all images:
```
$ cubic list images
ID                    ARCH      SIZE
ubuntu:jammy         amd64   2.2 GiB
```
