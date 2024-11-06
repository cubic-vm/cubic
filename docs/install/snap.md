# Install Cubic as Snap

Cubic can be installed from the Snap Store:
```
$ sudo snap install cubic
$ sudo snap connect cubic:kvm
$ sudo snap connect cubic:ssh-keys
```

## How to enable Kernel Virtual Machine (KVM) acceleration?

Virtual machines perform a lot better with KVM support.
It is recommend to allow KVM access to the Cubic snap.

Steps:
1. Make sure your current user is in the `kvm` group (show groups of your user: `groups`)
  1. You can add your user to the `kvm` group by: `sudo usermod -a -G kvm $USER` and then reboot.
2. Permit access to the kernel virtual machine (KVM) for hardware acceleration:
`sudo snap connect cubic:kvm`


