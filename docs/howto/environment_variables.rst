.. _environment variables:

Build with a Private GitHub Token Inside a VM
=============================================

This guide shows how to forward a ``GITHUB_TOKEN`` from the host into a Cubic
virtual machine so a build script can fetch private dependencies without storing
the token inside the VM.

Create the Virtual Machine
--------------------------

.. code-block::

    $ cubic create builder --image ubuntu:noble

Start the Virtual Machine
--------------------------

.. code-block::

    $ cubic start builder

Set the Token on the Host
--------------------------

Store the token in your current shell session:

.. code-block::

    $ export GITHUB_TOKEN=ghp_yourTokenHere

Connect and Forward the Token
------------------------------

Pass the variable by name. Cubic reads its value from the host environment and
makes it available inside the VM:

.. code-block::

    $ cubic ssh builder --env GITHUB_TOKEN

Verify Inside the VM
--------------------

Confirm the variable arrived:

.. code-block::

    $ echo $GITHUB_TOKEN
    ghp_yourTokenHere

Run the Build
-------------

The token is now available to any tool that reads it from the environment.
For example, configure Git to use it and install a private Python package:

.. code-block::

    $ git config --global url."https://${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"
    $ pip install git+https://github.com/your-org/private-lib.git

The token is only present for the duration of the SSH session and is never
written to disk inside the VM.

Pass an Explicit Value
----------------------

Use ``KEY=VALUE`` when the value is not already set in the host environment:

.. code-block::

    $ cubic ssh builder --env BUILD_TARGET=release

Both forms can be combined and repeated:

.. code-block::

    $ cubic ssh builder --env GITHUB_TOKEN --env BUILD_TARGET=release

Run a One-Off Command with Environment Variables
------------------------------------------------

Use ``cubic exec`` to run a single command inside the VM without opening an
interactive shell. The ``--env`` flag works the same way:

.. code-block::

    $ cubic exec builder --env GITHUB_TOKEN "pip install git+https://github.com/your-org/private-lib.git"
