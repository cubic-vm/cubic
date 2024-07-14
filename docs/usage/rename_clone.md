# Rename and Clone Virtual Machines

## Rename Command
Rename a virtual machine:
```
$ cubic rename -h
Rename a machine

Usage: cubic rename <OLD_NAME> <NEW_NAME>

Arguments:
  <OLD_NAME>  
  <NEW_NAME>  

Options:
  -h, --help  Print help
```
**Example:**
```
$ cubic rename example2 example_new
```

## Clone Command
Clone a virtual machine:
```
$ cubic clone -h
Clone a machine

Usage: cubic clone <NAME> <NEW_NAME>

Arguments:
  <NAME>      
  <NEW_NAME>  

Options:
  -h, --help  Print help
```
**Example:**
```
$ cubic clone example example2
```
