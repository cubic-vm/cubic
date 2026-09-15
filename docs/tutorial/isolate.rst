.. _first isolation:

Isolate a VM Instance
=====================

This tutorial creates a VM instance that cannot reach the network, shows that
the guest is cut off and then gives it access. It follows on from
:ref:`create vm`.

Create an Isolated VM Instance
------------------------------

.. code-block::

    $ cubic create sandbox --image ubuntu --isolate

``cubic show`` reports the state:

.. code-block::

    $ cubic show sandbox
    Running:    no
    Arch:       amd64
    vCPUs:      4
    Memory:     2048 M
    Disk Used:  408 M
    Disk Total: 100 G
    User:       alice
    Isolated:   yes
    SSH Port:   46831

Try to Reach the Network
------------------------

.. code-block::

    $ cubic ssh sandbox
    alice@sandbox:~$ curl -4 -sSI -m 8 https://ubuntu.com | head -1
    curl: (6) Could not resolve host: ubuntu.com
    alice@sandbox:~$ exit
    logout

The guest cannot even resolve a name. Nothing leaves the VM instance, which is
what you want while you try out software you do not trust. Your own connection
still works, because ``cubic ssh`` reaches the guest on the loopback address of
your host rather than over the network of the guest.

Give the VM Instance Access
---------------------------

.. code-block::

    $ cubic modify sandbox --no-isolate
    info: Note: changes may require a restart to take effect.

The switch applies on the next start, so restart the VM instance:

.. code-block::

    $ cubic restart sandbox

``cubic restart`` stops the VM instance and starts it again, which is what picks
up the new setting.

Try Again
---------

.. code-block::

    $ cubic ssh sandbox
    alice@sandbox:~$ curl -4 -sSI -m 8 https://ubuntu.com | head -1
    HTTP/2 200
    alice@sandbox:~$ exit
    logout

Same guest, same command, different answer. Isolation is a setting of the VM
instance, not something you install inside it, so a guest can never turn it off
for itself.

Next Steps
----------

Delete the VM instance when you are done:

.. code-block::

    $ cubic delete sandbox

``cubic create sandbox --image ubuntu --isolate`` and ``cubic modify sandbox
--isolate`` set the same switch, before and after creation. The
:ref:`security` page explains what isolation protects and what it does not.
