# List Images and Virtual Machines

## List Virtual Machine Instances

List virtual machine instancess:
```
$ cubic ls
Name             CPUs     Memory       Disk  State
noble               1    1.0 GiB    2.0 GiB  STOPPED
```

## List Virtual Machine Images

List all virtual machine images:
```
$ cubic image ls --all
Name               Arch         Size   
archlinux:latest   amd64   516.0 MiB   
debian:12          amd64   424.2 MiB   
debian:bookworm    amd64   424.2 MiB   
debian:11          amd64   345.1 MiB   
debian:bullseye    amd64   345.1 MiB   
debian:10          amd64   301.7 MiB   
debian:buster      amd64   301.7 MiB   
fedora:41          amd64   468.9 MiB   
fedora:42          amd64   507.6 MiB   
ubuntu:18.04       amd64   206.0 MiB   
ubuntu:bionic      amd64   206.0 MiB   
ubuntu:18.10       amd64   289.7 MiB   
ubuntu:cosmic      amd64   289.7 MiB   
ubuntu:19.04       amd64   153.3 MiB   
ubuntu:disco       amd64   153.3 MiB   
ubuntu:19.10       amd64   193.1 MiB   
...
```
