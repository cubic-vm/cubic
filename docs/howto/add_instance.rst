.. _create vm:

Create your First Virtual Machine
=================================

Select a Linux Distribution
---------------------------

Cubic can create a virtual machine for you from a common Linux distribution.
First of all, you need to decide which distribution you want to use.
You can list all supported distributions with:

.. code-block::

    $ cubic images
    Name                       Arch         Size   Cached
    archlinux:latest           amd64   516.7 MiB       no
    debian:{12, bookworm}      amd64   426.0 MiB       no
    debian:{11, bullseye}      amd64   343.5 MiB       no
    debian:{10, buster}        amd64   301.7 MiB       no
    debian:{13, trixie}        amd64   411.2 MiB       no
    fedora:41                  amd64   468.9 MiB       no
    fedora:41                  amd64   468.9 MiB       no
    fedora:42                  amd64   507.6 MiB       no
    fedora:43                  amd64   556.3 MiB       no
    opensuse:15.2              amd64   544.1 MiB       no
    opensuse:15.3              amd64   560.4 MiB       no
    opensuse:15.4              amd64   683.7 MiB       no
    opensuse:15.5              amd64   643.1 MiB       no
    opensuse:15.6              amd64   683.6 MiB       no
    rockylinux:10              amd64   548.8 MiB       no
    rockylinux:8               amd64     1.9 GiB       no
    rockylinux:9               amd64   618.8 MiB       no
    ubuntu:{18.04, bionic}     amd64   206.0 MiB       no
    ubuntu:{18.10, cosmic}     amd64   289.7 MiB       no
    ubuntu:{19.04, disco}      amd64   153.3 MiB       no
    ubuntu:{19.10, eoan}       amd64   193.1 MiB       no
    ubuntu:{20.04, focal}      amd64   251.9 MiB       no
    ubuntu:{20.10, groovy}     amd64   250.6 MiB       no
    ubuntu:{21.04, hirsute}    amd64   258.3 MiB       no
    ubuntu:{21.10, impish}     amd64   254.4 MiB       no
    ubuntu:{22.04, jammy}      amd64   293.9 MiB       no
    ubuntu:{22.10, kinetic}    amd64   294.5 MiB       no
    ubuntu:{23.04, lunar}      amd64   335.9 MiB       no
    ubuntu:{23.10, mantic}     amd64   228.1 MiB       no
    ubuntu:{24.04, noble}      amd64   251.5 MiB       no
    ubuntu:{24.10, oracular}   amd64   249.6 MiB       no
    ubuntu:{25.04, plucky}     amd64   261.1 MiB       no
    ubuntu:{25.10, questing}   amd64   394.4 MiB       no

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
    PID    Name         Arch    CPUs     Memory       Disk   Running
           example      amd64      1    1.0 GiB    1.0 GiB       yes

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
