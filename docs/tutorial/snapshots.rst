.. _first snapshot:

Restore a Snapshot
==================

This tutorial takes a snapshot of a VM instance, changes the guest on purpose
and rolls the change back. It follows on from :ref:`create vm`.

Create the VM Instance
----------------------

``cubic run`` creates the VM instance, starts it and opens a shell inside it:

.. code-block::

    $ cubic run demo --image ubuntu
    Welcome to Ubuntu 26.04.1 LTS (GNU/Linux 7.0.0-30-generic x86_64)
    [...]
    alice@demo:~$ exit
    logout

The first boot creates the SSH host key of the guest. Snapshot after it, so a
restore keeps the key Cubic already knows.

Take the Snapshot
-----------------

A snapshot is taken while the VM instance is stopped:

.. code-block::

    $ cubic stop demo --wait
    $ cubic snapshot demo/clean
    Created snapshot demo/clean

The name ``demo/clean`` is the snapshot ``clean`` of the VM instance ``demo``.
``cubic show`` lists it:

.. code-block::

    $ cubic show demo
    Running:      no
    Arch:         amd64
    vCPUs:        4
    Memory:       2048 M
    Disk Used:    408 M
    Disk Total:   100 G
    User:         alice
    Isolated:     no
    SSH Port:     40881
    Monitor Port: 42661
    Console Port: 37913
    Snapshots:    clean

Change the Guest
----------------

Write a file inside the guest so there is something to lose:

.. code-block::

    $ cubic ssh demo
    alice@demo:~$ echo 'my work' > notes.txt
    alice@demo:~$ ls
    notes.txt
    alice@demo:~$ exit
    logout

Roll It Back
------------

.. code-block::

    $ cubic stop demo --wait
    $ cubic restore demo/clean --yes
    info: The instance is stopped and all changes since the snapshot are lost.
    Successfully restored demo/clean

Look Again
----------

.. code-block::

    $ cubic ssh demo
    alice@demo:~$ ls
    alice@demo:~$ exit
    logout

The file is gone. The disk is back in the state it had when you took the
snapshot, so anything you break after a snapshot costs you one restore.

Next Steps
----------

Delete the VM instance when you are done. Its snapshots go with it:

.. code-block::

    $ cubic delete demo

The :ref:`snapshots` guide covers several snapshots per VM instance, what a
snapshot does not contain and how to delete one on its own.
