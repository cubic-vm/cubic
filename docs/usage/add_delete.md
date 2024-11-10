# Add and Delete Virtual Machines

## Add Command
Create a virtual machine:
```
$ cubic instance add -h
Add an instance

Usage: cubic instance add [OPTIONS] --image <IMAGE>

Options:
  -i, --image <IMAGE>
  -n, --name <NAME>
  -c, --cpus <CPUS>
  -m, --mem <MEM>
  -d, --disk <DISK>
  -h, --help           Print help
```
**Example**:
```
$ cubic instance add --name example --image ubuntu:jammy:amd64 --cpus 4 --mem 4G --disk 5G
```

## Delete Command

Delete a virtual machine:
```
$ cubic instance del --help
Delete instances

Usage: cubic instance del [INSTANCES]...

Arguments:
  [INSTANCES]...

Options:
  -h, --help  Print help
```
**Example**:
```
$ cubic instance del example
```
