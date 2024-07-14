# Start, Stop and Restart Virtual Machines

## Start Command
```
$ cubic start -h
Start machines

Usage: cubic start [OPTIONS] [IDS]...

Arguments:
  [IDS]...  

Options:
      --qemu-args <QEMU_ARGS>  
  -c, --console                
  -h, --help                   Print help
```

## Stop Command
```
$ cubic stop -h
Stop machines

Usage: cubic stop [OPTIONS] [IDS]...

Arguments:
  [IDS]...  

Options:
  -a, --all   
  -h, --help  Print help
```

## Restart Command
```
$ cubic restart -h
Restart a machine

Usage: cubic restart [OPTIONS] [IDS]...

Arguments:
  [IDS]...  

Options:
  -c, --console  
  -h, --help     Print help
```
