.. _create vm:

Create your First Virtual Machine
=================================

Select a Linux Distribution
---------------------------

Cubic can create a virtual machine for you from a common Linux distribution.
First of all, you need to decide which distribution you want to use.
You can list all supported distributions with:

.. code-block::

    $ cubic image ls
    Name               Arch         Size
    archlinux:latest   amd64   519.3 MiB
    [...]
    debian:12          amd64   424.0 MiB
    debian:bookworm    amd64   424.0 MiB
    [...]
    fedora:42          amd64   507.6 MiB
    fedora:42          arm64   508.4 MiB
    [...]
    opensuse:15.6      amd64   670.3 MiB
    opensuse:15.6      arm64   634.9 MiB
    [...]
    ubuntu:24.04       amd64   243.2 MiB
    ubuntu:noble       amd64   243.2 MiB
    ubuntu:24.04       arm64   212.0 MiB
    ubuntu:noble       arm64   212.0 MiB
    ubuntu:24.10       amd64   249.4 MiB
    [...]

Create the Virtual Machine
--------------------------

In the next step you can create the virtual machine with a single command:

.. code-block::

    $ cubic create example --image debian:bookworm

List all Virtual Machine
------------------------

You can list all your virtual machine with the following command:

.. code-block::

    $ cubic instances
    PID    Name         Arch    CPUs     Memory       Disk   State
           example      amd64      1    1.0 GiB    1.0 GiB   STOPPED

Additional Settings
-------------------

You can also override the default virtual machine settings and define the number of CPUs, the memory size and disk size.
For example to create a virtual machine with 4 CPU cores, 4 GiB of memory and 10 GiB of disk storage use:

.. code-block::

    $ cubic create example --image debian:bookworm --cpus 4 --mem 4G  --disk 10G


Start the virtual machine
-------------------------

Once the virtual machine was created you can start it with:

.. code-block::

    $ cubic start example

Connect with SSH
----------------

When the virtual machine is running you can simply connect over SSH by:

.. code-block::

    $ cubic ssh example
