.. _temporary vm:

Run a Throwaway VM
==================

``cubic run --rm`` creates a VM instance, opens a shell in it and deletes the
whole VM instance when you leave the shell. It is the quick way to try a command
in a clean guest without anything left on your host afterwards.

Start One
---------

.. code-block::

    $ cubic run --rm scratch --image ubuntu
    alice@scratch:~$ python3 --version
    Python 3.13.7
    alice@scratch:~$ exit
    logout

The VM instance is gone once the shell closes. ``cubic instances`` no longer
lists it:

.. code-block::

    $ cubic instances
    Name    Arch    vCPUs    Memory   Disk Used   Disk Total   Running

Cubic deletes the VM instance whenever the session ends, so a dropped connection
or a crash of the guest cleans up as well.

Fast and Small
--------------

The disk of a throwaway VM instance is a thin overlay on the cached image rather
than a full copy, so it is created at once and stores only what the guest
writes. This makes ``--rm`` cheap to run again and again.

The overlay refers back to the image in the cache, so a throwaway VM instance is
not self-contained. ``cubic prune`` clears the whole image cache, so avoid it
while a throwaway VM instance still runs, or the overlay loses the base image it
reads from. A VM instance created without ``--rm`` holds a full copy and is not
affected.

Combine with Isolation
----------------------

``--isolate`` keeps the guest off the network, which pairs well with a throwaway
VM instance for code you do not trust. It runs in a clean guest, reaches nothing,
and leaves nothing behind:

.. code-block::

    $ cubic run --rm --isolate scratch --image ubuntu

See :ref:`first isolation` for what isolation covers.

Only ``cubic run``
------------------

``--rm`` belongs to ``cubic run``, which owns the shell session from start to
finish. ``cubic create``, ``cubic start`` and ``cubic ssh`` have no ``--rm``,
because they do not know when you are done with the VM instance. Delete a VM
instance you started that way with ``cubic delete``.

Related
-------

* :ref:`create vm` for a VM instance that stays
* :ref:`first isolation` to cut a VM instance off the network
* :ref:`exec command` to run a single command without a shell
