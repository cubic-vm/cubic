.. _console login:

Use the Serial Console
======================

The serial console shows the login prompt of the guest and that prompt accepts
only a password, so set one now, while SSH still works. A new VM instance has no
account password at all, and once SSH is broken it is too late to add one.
:ref:`security` explains why the account starts out without a password.

Set a Password
--------------

Connect over SSH and set a password for your own account:

.. code-block::

    $ cubic ssh demo
    alice@demo:~$ sudo passwd alice

The command asks for the new password twice. Nothing else is needed, because
the account already has passwordless sudo.

On Alpine, run ``doas passwd alice`` in place of ``sudo``.

Look Up the User Name
---------------------

The login prompt wants the guest user name. Use ``cubic show`` if you are not
sure which one the VM instance uses:

.. code-block::

    $ cubic show demo
    [...]
    User:         alice
    [...]

Open the Console
----------------

.. code-block::

    $ cubic console demo

Log in with the user name and the password you just set. To leave the console
press Enter, then ``~``, then ``.``. The VM instance keeps running.

Related
-------

* :ref:`security` covers the SSH key and the disabled account password
* :ref:`qemu not found` helps when a VM instance does not start at all
