.. _ssh connect:

Connect with the Host SSH Client
=================================

This guide shows how to connect to a Cubic virtual machine using the host's
native SSH client instead of ``cubic ssh``.

Add Port Forwarding
-------------------

Expose guest port 22 on a host port so the SSH client can reach the VM:

.. code-block::

    $ cubic modify <instance> --port 2222:22

Changes take effect on the next restart.

Restart the Virtual Machine
----------------------------

.. code-block::

    $ cubic restart <instance>

Add Your SSH Public Key
-----------------------

Connect to the VM with ``cubic ssh`` and append your public key to the
authorized keys file:

.. code-block::

    $ cubic ssh <instance>
    $ mkdir -p ~/.ssh && echo '<your-public-key>' >> ~/.ssh/authorized_keys

Your public key is typically stored in ``~/.ssh/id_ed25519.pub`` or
``~/.ssh/id_rsa.pub`` on the host.

Connect with SSH
----------------

Look up the VM username with ``cubic show``, then connect:

.. code-block::

    $ cubic show <instance>
    ...
    User:        <username>
    ...

    $ ssh -p 2222 <username>@localhost
