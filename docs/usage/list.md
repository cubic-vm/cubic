# List Images and Virtual Machines

## List Virtual Machine Instances

List virtual machines:
```
$ cubic list
Name             CPUs     Memory       Disk  State
noble               1    1.0 GiB    2.0 GiB  STOPPED
```

## List Virtual Machine Images

List all images:
```
$ cubic image list --all
Vendor      Version  Name         Arch       Size
archlinux    latest  latest      amd64
debian            9  stretch     amd64
debian           10  buster      amd64
debian           11  bullseye    amd64
debian           12  bookworm    amd64
debian           13  trixie      amd64
debian           14  forky       amd64
fedora           39  39          amd64
fedora           40  40          amd64
fedora           41  41          amd64
ubuntu        18.04  bionic      amd64
ubuntu        18.10  cosmic      amd64
ubuntu        19.04  disco       amd64
ubuntu        19.10  eoan        amd64
ubuntu        20.04  focal       amd64
ubuntu        20.10  groovy      amd64
ubuntu        21.04  hirsute     amd64
ubuntu        21.10  impish      amd64
ubuntu        22.04  jammy       amd64  284.6 MiB
ubuntu        22.10  kinetic     amd64
ubuntu        23.04  lunar       amd64
ubuntu        23.10  mantic      amd64
ubuntu        24.04  noble       amd64
ubuntu        24.10  oracular    amd64
```
