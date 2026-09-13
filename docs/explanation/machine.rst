.. _the machine:

The Virtual Machine
===================

Cubic builds the smallest machine that boots a cloud image and does real work.
It names every device it wants and adds nothing else, so a VM instance is the
same simple machine on Linux, macOS and Windows.

That keeps a VM instance secure and quick. Every emulated device is code running
in the QEMU process on your host, so leaving one out removes both attack surface
and startup time.

Devices
-------

Cubic creates no device by default and adds only the ones below. All of them are
virtio, so there is no legacy hardware in a guest and every official cloud image
already carries the drivers.

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Device
     - Purpose
   * - ``virtio-net-pci``
     - Network interface of the guest. See :ref:`networking`.
   * - virtio disk
     - Disk image of the VM instance, in ``qcow2`` format.
   * - virtio disk
     - cloud-init seed image, read on the first boot. See :ref:`guest config`.
   * - ``virtio-rng-pci``
     - Entropy, so early boot never stalls waiting for it. Cubic pins the
       builtin backend of QEMU, which works the same on every platform.
   * - ``virtio-balloon-pci``
     - Returns unused guest memory to the host. See below.
   * - Serial console
     - Character device on a loopback TLS socket, which ``cubic console``
       attaches to.

The UEFI firmware is attached read only and there is no display at all, since a
VM instance is reached over SSH or the serial console.

CPU and Machine Type
--------------------

A guest gets the machine type its architecture and its UEFI firmware expect,
``q35`` on amd64 and ``virt`` on arm64.

With hardware acceleration a guest runs on the CPU of the host itself and sees
its real features. Windows is the one exception, where the hypervisor accepts a
named model only. Software emulation uses the widest model available, so an
emulated guest loses speed but no features.

.. _hardware acceleration:

Hardware Acceleration
---------------------

Cubic does not decide from the platform whether acceleration is available. It
asks, by starting a throwaway machine and waiting for it to come up. A host can
look capable and still fail, for example when the KVM device is not reachable
inside a snap, and only starting a machine settles it.

When the answer is no, a guest runs in software emulation instead of failing.
``--accel auto`` is that default. ``--accel on`` insists on hardware
acceleration and fails without it, and ``--accel off`` always emulates.
Acceleration also needs the guest architecture to match the host, so an arm64
guest on an amd64 host is always emulated.

A VM Instance Costs What It Uses
--------------------------------

A VM instance is sized generously by default, with 100 GiB of disk and a share
of the host memory. Neither is paid for up front.

The disk grows as the guest writes and shrinks again when the guest frees a
block, because Cubic passes discards through to the host image. Official images
already mount the root filesystem that way, so deleting a file inside the guest
gives the space back with nothing to set up. The default of 100 GiB is a
ceiling, not an allocation.

Memory works the same way. The guest reports the pages it no longer uses and the
host takes them back on its own, with no command on either side and without the
guest losing its full memory size. The memory a VM instance holds on the host is
therefore often far below the size you gave it, and it moves while the guest
works.

Related
-------

* :ref:`design` for the decisions behind this machine
* :ref:`networking` for the network device and its forwards
* :ref:`values` for the default vCPU and memory sizes
* :ref:`recover disk` to read a disk image from the host
