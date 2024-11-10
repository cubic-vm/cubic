# Start, Stop and Restart Virtual Machines

## Start Command
```
$ cubic start --help
Start instances

Usage: cubic start [OPTIONS] [IDS]...

Arguments:
  [IDS]...

Options:
      --qemu-args <QEMU_ARGS>
  -c, --console
  -v, --verbose
  -h, --help                   Print help
```

## Stop Command
```
$ cubic stop --help
Stop instances

Usage: cubic stop [OPTIONS] [IDS]...

Arguments:
  [IDS]...

Options:
  -a, --all
  -h, --help  Print help
```

## Restart Command
```
$ cubic restart --help
Restart instances

Usage: cubic restart [OPTIONS] [IDS]...

Arguments:
  [IDS]...

Options:
  -c, --console
  -v, --verbose
  -h, --help     Print help
```
