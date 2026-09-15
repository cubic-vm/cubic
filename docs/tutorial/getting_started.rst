.. _create vm:

Getting Started
===============

This tutorial creates your first VM instance and opens a shell inside it.
Cubic has to be installed on your host first, see :ref:`Install Cubic`.

We use Ubuntu here. Run ``cubic images`` later to see every distribution Cubic
can boot.

Create the VM Instance
----------------------

.. code-block::

    $ cubic create example --image ubuntu

The first run downloads the Ubuntu image, so the command takes a moment.

See What You Created
--------------------

.. code-block::

    $ cubic instances
    Name      Arch    vCPUs   Memory      Disk   Running
    example   amd64       4   2048 M   1/100 G        no

The VM instance exists but nothing runs yet, so ``Running`` is ``no``. Cubic
picked the number of vCPUs and the memory from the resources of your host, so
your values can differ.

Every command can be shortened while the short form stays unique, so
``cubic in`` does the same as ``cubic instances``.

Connect with SSH
----------------

The VM instance is not running yet. ``cubic ssh`` starts it, waits for the boot
to finish and then opens the shell, so the first output takes a few seconds:

.. code-block::

    $ cubic ssh example
    Welcome to Ubuntu 26.04.1 LTS (GNU/Linux 7.0.0-30-generic x86_64)

     * Documentation:  https://docs.ubuntu.com
     * Management:     https://landscape.canonical.com
     * Support:        https://ubuntu.com/pro
    [...]
    alice@example:~$

You are inside the VM instance now. Ask the guest which system it runs:

.. code-block::

    alice@example:~$ cat /etc/os-release
    PRETTY_NAME="Ubuntu 26.04.1 LTS"
    NAME="Ubuntu"
    VERSION_ID="26.04"
    VERSION="26.04.1 LTS (Resolute Raccoon)"
    [...]

Leave the guest and return to your host:

.. code-block::

    alice@example:~$ exit
    logout

The VM instance keeps running after you leave it:

.. code-block::

    $ cubic instances
    Name      Arch    vCPUs   Memory      Disk   Running
    example   amd64       4   2048 M   1/100 G       yes

Next Steps
----------

Connect again at any time with ``cubic ssh example``. Stop the VM instance with
``cubic stop example`` and delete it when you no longer need it:

.. code-block::

    $ cubic delete example

``cubic run example --image ubuntu`` creates and connects in one command.

From here you can:

* :ref:`snapshots` before you try something risky
* :ref:`port forward` and reach a service from your host
* :ref:`templates` and stop retyping long command lines
* :ref:`console login` when SSH is not available
