.. _recover disk:

Recover Data from a VM Disk Image
==================================

This guide shows how to retrieve files from a Cubic virtual machine that can
no longer boot or is otherwise inaccessible via ``cubic ssh``. The VM disk is
a standard ``qcow2`` image, so ``qemu-img`` and ``qemu-nbd`` can read it
directly from the host.

Stop the Virtual Machine
------------------------

The disk image must not be in use by a running VM before it is mounted on the
host:

.. code-block::

    $ cubic stop <instance>

Locate the Disk Image
---------------------

Cubic stores each instance under the user data directory:

* Snap: ``$SNAP_USER_COMMON/cubic/machines/<instance>/machine.img``
  (typically ``~/snap/cubic/common/cubic/machines/<instance>/machine.img``)
* Linux: ``$XDG_DATA_HOME/cubic/machines/<instance>/machine.img``
  (typically ``~/.local/share/cubic/machines/<instance>/machine.img``)
* macOS: ``~/Library/cubic/machines/<instance>/machine.img``
* Windows: ``%LOCALAPPDATA%\cubic\machines\<instance>\machine.img``

Copy Files with ``virt-cat`` (read-only, recommended)
-----------------------------------------------------

If the host has ``libguestfs-tools`` installed, files can be extracted without
mounting the image:

.. code-block::

    $ virt-filesystems -a ~/.local/share/cubic/machines/<instance>/machine.img --all --long -h
    $ virt-cat -a ~/.local/share/cubic/machines/<instance>/machine.img /etc/hostname
    $ virt-copy-out -a ~/.local/share/cubic/machines/<instance>/machine.img /home/<user>/work .

``virt-copy-out`` writes the listed guest paths into the current host
directory. The image is opened read-only, so the running guest state cannot
be corrupted.

Mount the Image with ``qemu-nbd``
---------------------------------

When ``libguestfs-tools`` is not available, expose the disk as a block device
on Linux with the Network Block Device kernel module:

.. code-block::

    $ sudo modprobe nbd max_part=8
    $ sudo qemu-nbd --read-only --connect=/dev/nbd0 \
        ~/.local/share/cubic/machines/<instance>/machine.img
    $ lsblk /dev/nbd0

Mount the guest root partition (typically ``/dev/nbd0p1``) somewhere on the
host and copy files out:

.. code-block::

    $ sudo mkdir -p /mnt/cubic
    $ sudo mount -o ro /dev/nbd0p1 /mnt/cubic
    $ cp -a /mnt/cubic/home/<user>/work ~/recovered/

Always mount with ``-o ro`` to avoid writing to the guest filesystem.

Detach the Image
----------------

When the recovery is done, unmount and disconnect the device:

.. code-block::

    $ sudo umount /mnt/cubic
    $ sudo qemu-nbd --disconnect /dev/nbd0
