# Add and Delete Virtual Machine Instances

## Instance Add Command
Create a virtual machine instance:
```
$ cubic add --help
Add a virtual machine instance

Usage: cubic add [OPTIONS] --image <IMAGE> --name <NAME>

Options:
  -i, --image <IMAGE>  Name of the virtual machine image
  -n, --name <NAME>    Name of the virtual machine instance
  -c, --cpus <CPUS>    Number of CPUs for the virtual machine instance
  -m, --mem <MEM>      Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
  -d, --disk <DISK>    Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
  -h, --help           Print help
```
**Example**:
```
$ cubic add --name example --image ubuntu:noble:amd64 --cpus 4 --mem 4G --disk 5G
```

## Instance Delete Command

Delete a virtual machine instance:
```
$ cubic rm --help
Delete virtual machine instances

Usage: cubic rm [OPTIONS] [INSTANCES]...

Arguments:
  [INSTANCES]...  Name of the virtual machine instances to delete

Options:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Reduce logging output
  -f, --force    Delete the virtual machine instances without confirmation
  -h, --help     Print help
```
**Example**:
```
$ cubic rm example
```
