# Add and Delete Virtual Machines

## Add Command
Create a virtual machine:
```
$ cubic add -h
Add an image or a machine

Usage: cubic add [OPTIONS] --image <IMAGE>

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
$ cubic add --name example --image ubuntu:jammy:amd64 --cpus 4 --mem 4G --disk 5G
```

## Delete Command

Delete a virtual machine:
```
$ cubic delete -h
Delete images and machines

Usage: cubic delete [IDS]...

Arguments:
  [IDS]...  

Options:
  -h, --help  Print help
```
**Example**:
```
$ cubic delete example
```
