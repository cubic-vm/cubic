# Rename and Clone Virtual Machines

## Instance Rename Command
Rename a virtual machine instance:
```
$ cubic rename --help
Rename a virtual machine instance

Usage: cubic rename <OLD_NAME> <NEW_NAME>

Arguments:
  <OLD_NAME>  Name of the virtual machine instance to rename
  <NEW_NAME>  New name of the virutal machine instance

Options:
  -h, --help  Print help
```

**Example:**
```
$ cubic rename example example_new
```

## Instance Clone Command
Clone a virtual machine instance:
```
$ cubic clone --help
Clone a virtual machine instance

Usage: cubic clone <NAME> <NEW_NAME>

Arguments:
  <NAME>      Name of the virtual machine instance to clone
  <NEW_NAME>  Name of the copy

Options:
  -h, --help  Print help
```

**Example:**
```
$ cubic clone example example2
```
