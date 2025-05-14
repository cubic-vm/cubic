# Start, Stop and Restart Virtual Machines

## Start Command
```
$ cubic start --help
Start virtual machine instances

Usage: cubic start [OPTIONS] [INSTANCES]...

Arguments:
  [INSTANCES]...  Name of the virtual machine instances to start

Options:
      --qemu-args <QEMU_ARGS>  Pass additional QEMU arguments
  -v, --verbose                Enable verbose logging
  -q, --quiet                  Reduce logging output
  -h, --help                   Print help
```

## Stop Command
```
$ cubic stop --help
Stop virtual machine instances

Usage: cubic stop [OPTIONS] [INSTANCES]...

Arguments:
  [INSTANCES]...  Name of the virtual machine instances to stop

Options:
  -a, --all      Stop all virtual machine instances
  -v, --verbose  Enable verbose logging
  -q, --quiet    Reduce logging output
  -h, --help     Print help
```

## Restart Command
```
$ cubic restart --help
Restart virtual machine instances

Usage: cubic restart [OPTIONS] [INSTANCES]...

Arguments:
  [INSTANCES]...  Name of the virtual machine instances to restart

Options:
  -v, --verbose  Enable verbose logging
  -q, --quiet    Reduce logging output
  -h, --help     Print help
```
