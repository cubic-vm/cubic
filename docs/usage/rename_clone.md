# Rename and Clone Virtual Machines

## Rename Command
Rename a virtual machine:
```
$ cubic instance rename --help
Rename an instance

Usage: cubic instance rename <OLD_NAME> <NEW_NAME>

Arguments:
  <OLD_NAME>
  <NEW_NAME>

Options:
  -h, --help  Print help
```

**Example:**
```
$ cubic instance rename example example_new
```

## Clone Command
Clone a virtual machine:
```
$ cubic instance clone --help
Clone an instane

Usage: cubic instance clone <NAME> <NEW_NAME>

Arguments:
  <NAME>
  <NEW_NAME>

Options:
  -h, --help  Print help
```

**Example:**
```
$ cubic instance clone example example2
```
