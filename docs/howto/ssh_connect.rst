.. _ssh connect:

Use the Host SSH Client
=======================

``cubic ssh`` is the short way into a guest. The SSH client of your host works
just as well, which is what you want for an editor, an agent or a tool that
speaks SSH on its own.

Use the Key Cubic Made
----------------------

Every VM instance already has its own user, its own key and a forwarded SSH
port. ``cubic show --all`` prints all three and the command that puts them
together:

.. code-block::

    $ cubic show --all demo
    Running:      yes
    Arch:         amd64
    vCPUs:        4
    Memory:       4096 M
    Disk Used:    1229 M
    Disk Total:   100 G
    User:         alice
    Isolated:     no
    SSH Port:     40881
    Monitor Port: 40882
    Console Port: 40883
    PID:          12345
    Disk Image:   ~/.local/share/cubic/machines/demo/machine.img
    Config:       ~/.local/share/cubic/machines/demo/instance.toml
    SSH Key:      ~/.local/share/cubic/machines/demo/ssh_client_key
    SSH:          ssh -i ~/.local/share/cubic/machines/demo/ssh_client_key -p 40881 alice@localhost

The ``User`` field names the account inside the guest. A new VM instance takes
the user name of your host account, so ``alice`` in these examples stands for
your own user name. ``cubic create --user`` picks another one.

Copy the ``SSH`` line and you are in:

.. code-block::

    $ ssh -i ~/.local/share/cubic/machines/demo/ssh_client_key -p 40881 alice@localhost
    alice@demo:~$

The VM instance has to be running, because the host client does not start it.
Use ``cubic start demo`` first.

Check the Host Key
------------------

``cubic ssh`` pins the host key of a guest in the config of the VM instance and
checks it at every login. Your host client keeps its own list in
``~/.ssh/known_hosts`` instead, so the first login asks you to confirm a
fingerprint:

.. code-block::

    The authenticity of host '[localhost]:40881 ([127.0.0.1]:40881)' can't be established.
    ED25519 key fingerprint is SHA256:ZkAslGjFiUHdGf/WUL8rQvkib4PTvQatUV0OUQSncCA.
    Are you sure you want to continue connecting (yes/no/[fingerprint])?

Compare it with the ``SSH Host Key`` line of ``cubic show --all`` and answer
``yes`` once both match. Cubic learns the key of a guest at its own first login,
so run ``cubic ssh demo`` once when that line is missing.

Your client stores the answer under ``[localhost]:<port>``. Ports come back into
use, so a later VM instance on the same port makes the client report a changed
host key. Drop the old entry and confirm the new fingerprint:

.. code-block::

    $ ssh-keygen -R '[localhost]:40881'

A fixed port of your own keeps these entries apart, as shown further below.

Use Your Own Key
----------------

Add your own public key to the guest when you would rather use the key of your
agent, or when a tool cannot be told which key file to take. ``ssh-copy-id``
does it in one step, with the key of Cubic for the login and your own key as the
one to install:

.. code-block::

    $ ssh-copy-id -i ~/.ssh/id_ed25519.pub \
        -o IdentityFile=~/.local/share/cubic/machines/demo/ssh_client_key \
        -p 40881 alice@localhost

The same by hand on a host without ``ssh-copy-id``:

.. code-block::

    $ cubic ssh demo
    alice@demo:~$ mkdir -p ~/.ssh && chmod 700 ~/.ssh
    alice@demo:~$ echo '<your-public-key>' >> ~/.ssh/authorized_keys
    alice@demo:~$ exit

Your public key is usually ``~/.ssh/id_ed25519.pub`` on the host. Both keys work
side by side afterwards.

Give the VM Instance a Fixed Port
---------------------------------

Cubic picks a free SSH port when it creates a VM instance and keeps that port
from then on. A start that finds the port taken by something else moves the VM
instance to a free one, and an entry in ``~/.ssh/config`` then points at the
wrong place. Add a forward of your own for a port that stays:

.. code-block::

    $ cubic modify demo --port 2222:22

A running VM instance opens the host port right away, so no restart is needed.
The entry then stays valid:

.. code-block::

    Host demo
        HostName localhost
        Port 2222
        User alice
        IdentityFile ~/.local/share/cubic/machines/demo/ssh_client_key

With that in place ``ssh demo``, ``scp`` and ``rsync`` all reach the guest.

Reach the Guest from Another Machine
------------------------------------

Cubic binds the SSH port of a VM instance to ``127.0.0.1``, so it answers on the
host itself and nowhere else. A forward of your own takes a host address in
front of the host port, and ``0.0.0.0`` opens it on every network of the host:

.. code-block::

    $ cubic modify demo --port 0.0.0.0:2222:22

Everyone who reaches your host can knock on that port afterwards, so keep it on
a network you trust. A jump over the host is the other way, and it leaves the
guest on loopback:

.. code-block::

    $ ssh -J alice@my-host -i ./ssh_client_key -p 40881 alice@localhost

The key file has to sit on the machine you start from, so copy it over or add
your own key to the guest as shown above.

Related
-------

* :ref:`copy files` with ``cubic scp`` instead
* :ref:`port forward` to reach other services in the guest
* :ref:`file locations` of the SSH key of a VM instance
* :ref:`console login` reaches a guest whose SSH server stopped working
