.. _resources:

Change vCPUs, Memory and Disk
=============================

``cubic modify`` changes the size of a VM instance after it was created.

Stop the VM Instance
--------------------

vCPUs, memory and disk apply when a VM instance starts, so stop it first:

.. code-block::

    $ cubic stop demo --wait

Change the Settings
-------------------

.. code-block::

    $ cubic modify demo --cpus 2 --memory 2G --disk 200G

The command is quiet. Every flag is optional, so a single value can be changed
on its own. Modifying a running VM instance works too and reminds you that the
change needs a restart:

.. code-block::

    $ cubic modify demo --cpus 2
    info: Note: changes may require a restart to take effect.

Check the Result
----------------

.. code-block::

    $ cubic show demo
    Running:      no
    Arch:         amd64
    vCPUs:        2
    Memory:       2048 M
    Disk Used:    448 M
    Disk Total:   200 G
    User:         alice
    Isolated:     no
    SSH Port:     33033
    Monitor Port: 38283
    Console Port: 42883

Start the VM instance again:

.. code-block::

    $ cubic start demo

The Disk Can Only Grow
----------------------

The guest picks up a larger disk on the next boot, because the official images
grow their root filesystem themselves:

.. code-block::

    $ cubic exec demo "df -h / | tail -1"
    /dev/root       193G  724M  193G   1% /

Shrinking is refused, because it would cut into data that is already on the
disk:

.. code-block::

    $ cubic modify demo --disk 10G
    error: Cannot shrink the disk of the instance 'demo'

Related
-------

* :ref:`templates` to set these values for every new VM instance
* :ref:`port forward` with ``cubic modify``
