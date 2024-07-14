# Transfer Directories and Files

## SCP Command
```
$ cubic scp -h
Copy a file from or to a machine with SCP

Usage: cubic scp <FROM> <TO>

Arguments:
  <FROM>  
  <TO>    

Options:
  -h, --help  Print help
```
**Example:**
```
$ touch test
$ cubic scp mymachine test example:~/
```

## Mount Command
```
$ cubic mount -h
Mount host directory to guest

Usage: cubic mount <NAME> <HOST> <GUEST>

Arguments:
  <NAME>   
  <HOST>   
  <GUEST>  

Options:
  -h, --help  Print help
```


## Umount Command
```
$ cubic umount -h
Unmount guest directory

Usage: cubic umount <NAME> <GUEST>

Arguments:
  <NAME>   
  <GUEST>  

Options:
  -h, --help  Print help
```
