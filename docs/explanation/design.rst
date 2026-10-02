.. _design:

Design Decisions
================

Cubic spins up a Linux VM with one command and then gets out of the way. This
page collects the decisions that make that possible, and what each one buys.

Every one of them points the same way. Cubic owns as little as it can, so there
is less to trust, less to break and less to keep up to date.

No Background Service
---------------------

A ``cubic`` command is a short lived process. It reads the settings of a VM
instance, starts or stops a QEMU process, and exits. Nothing of Cubic keeps
running and nothing of Cubic runs as root.

Most VM managers put a privileged service in the middle instead. That service
has to be installed, kept running, kept up to date and trusted, and every VM on
the host depends on it. Cubic has no such component, so there is nothing to
trust and nothing to fail. A guest that breaks out of QEMU lands with your own
rights rather than with the rights of a system service.

The price is that VM instances belong to a user rather than to the host. There
is no system wide list of machines and nothing starts them at boot.

Unmodified Official Images
--------------------------

Cubic boots the cloud image the distribution publishes and changes nothing in
it. The guest you get is the one the distribution ships and supports.

This is why Cubic can offer eight distributions without maintaining anything
inside any of them. Nothing has to be rebuilt.

Cubic ships with a list of the official images and their ``stable`` and
``latest`` tags. Maintainers refresh the list before each release, so a new
Cubic version brings the new distribution releases. Reading the list from the
binary keeps Cubic fast. It also lets Cubic create a VM instance offline once
the image is cached. :ref:`image names` covers the naming.

cloud-init Instead of Custom Provisioning
-----------------------------------------

A guest needs a user account, an SSH key and sometimes a command on the first
boot. Cubic could reach into the disk image to arrange that. It does not.

Official cloud images already ship `cloud-init <https://cloud-init.io>`_, which
exists to do exactly this work. Cubic writes a small configuration, hands it to
the guest on a seed image and lets cloud-init apply it on the first boot. The
setup is declarative, it is the same on every distribution, and Cubic never
touches a filesystem it does not own. :ref:`guest config` lists what goes in.

A Plain QEMU Process
--------------------

Cubic turns the settings of a VM instance into a QEMU command line and runs it.
There is no library binding and no management layer in between.

That means what runs on your host is an ordinary QEMU process. ``--verbose``
prints the exact command, you can read it, and you can run it yourself. When
something goes wrong the thing to debug is QEMU, with everything the QEMU
community already knows about it, rather than an abstraction Cubic invented.

It also keeps the guest small. Cubic asks for a handful of virtio devices and
nothing else, which is what :ref:`the machine` describes.

The cost is that Cubic has to find QEMU and its UEFI firmware on a host where
every platform puts them somewhere else. :ref:`qemu detection` is that work.

One Directory per VM Instance
-----------------------------

A VM instance is a directory. It holds the settings, the disk image, the SSH
key, the seed image and, while it runs, the pid file and the control sockets
of QEMU. Nothing is shared with another VM instance except the image cache, which
holds verified read only downloads.

So the lifecycle needs no bookkeeping. Deleting the directory deletes the VM
instance, copying it takes everything along, and a backup of it is a complete
backup. There is no database that can disagree with what is on disk.
:ref:`file locations` names the directory and :ref:`instance file` lists the
settings.

Related
-------

* :ref:`the machine` for the virtual hardware a guest gets
* :ref:`networking` for how a guest reaches the network and how you reach it
* :ref:`security` for the threat model and its limits
* :ref:`qemu detection` for how QEMU and the firmware are located
