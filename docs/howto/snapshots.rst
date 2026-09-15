.. _snapshots:

Snapshot and Restore
====================

``cubic snapshot`` saves the disk of a VM instance and ``cubic restore`` rolls
it back. Snapshots of the same VM instance share every block that did not
change, so keeping several of them stays cheap. :ref:`first snapshot` walks
through one snapshot step by step.

Create a Snapshot
-----------------

A snapshot can only be taken while the VM instance is stopped:

.. code-block::

    $ cubic stop example --wait
    $ cubic snapshot example/clean

A snapshot is always addressed as ``<instance>/<snapshot>``, so
``example/clean`` is the snapshot ``clean`` of the VM instance ``example``.

List the Snapshots
------------------

``cubic show`` lists the snapshots of a VM instance:

.. code-block::

    $ cubic show example
    Running:      no
    Arch:         amd64
    vCPUs:        4
    Memory:       2048 M
    Disk Used:    941 M
    Disk Total:   100 G
    User:         alice
    Isolated:     no
    SSH Port:     40881
    Monitor Port: 42661
    Console Port: 37913
    Snapshots:    clean
                  before-upgrade

Restore a Snapshot
------------------

Restoring rolls the disk back to the state it had when the snapshot was taken:

.. code-block::

    $ cubic restore example/clean

Everything written since the snapshot is lost. A running VM instance is stopped
first, so make sure you no longer need its current state. Add ``--yes`` to skip
the confirmation.

The settings of a VM instance, such as vCPUs, memory and forwarded ports, are
not part of a snapshot and stay as they are. The disk is the exception, because a
restore brings back the size the disk had when the snapshot was taken while the
setting keeps the newer value. ``cubic modify --disk`` grows a disk past the
size in the settings, so pick a larger size to bring the two back in line.

Delete a Snapshot
-----------------

A snapshot you no longer need can be deleted without touching the VM instance:

.. code-block::

    $ cubic delete example/clean

Deleting the VM instance itself removes all of its snapshots as well:

.. code-block::

    $ cubic delete example

Related
-------

* :ref:`resources` of an existing VM instance
* :ref:`recover disk` reads files from a VM instance that no longer boots
