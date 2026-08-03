.. _console login:

Log In on the Serial Console
============================

Set a console password now, while SSH still works. The console is what you
reach for when SSH is broken, and at that point it is too late to set one.

Why the Console Needs a Password
--------------------------------

A new virtual machine has no account password at all. Cubic installs a unique
SSH key during first boot, so ``cubic ssh`` works right away without one.

The serial console is different. It shows the login prompt of the guest, and
that prompt only accepts a password. SSH keys do not apply there. So the
account stays locked on the console until you set a password from inside the
guest.

Set a Password
--------------

Connect over SSH and set a password for your own account:

.. code-block::

    $ cubic ssh <instance>
    $ sudo passwd $USER

The command asks for the new password twice. Nothing else is needed, because
the account already has passwordless sudo.

Look Up the User Name
---------------------

The login prompt wants the guest user name. Use ``cubic show`` if you are not
sure which one the machine uses:

.. code-block::

    $ cubic show <instance>
    ...
    User:        <username>
    ...

Open the Console
----------------

.. code-block::

    $ cubic console <instance>

Log in with the user name and the password you just set. To leave the console
press Enter, then ``~``, then ``.``. The virtual machine keeps running.

When the Console Helps
----------------------

The console is useful when you want to watch a machine boot, or when you need
to reach a guest whose network or SSH server stopped working. Both cases are
easier to handle if the password is already in place.
