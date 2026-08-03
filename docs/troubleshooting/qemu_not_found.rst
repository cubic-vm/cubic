.. _qemu not found:

QEMU or Firmware Not Found
==========================

Cubic stops with the following error when it cannot start a virtual machine:

.. code-block::

    QEMU or its UEFI firmware was not found.

The error lists the install command for your platform, so follow that first.
This guide helps when the packages are installed and the error still appears.

The same error covers two different causes. Cubic needs the QEMU binaries and
it needs the UEFI firmware that boots the guest. Either one can be missing.

Find Out Which Part Is Missing
------------------------------

Start the machine again with ``--verbose``:

.. code-block::

    $ cubic start <instance> --verbose

Cubic reports where it found the QEMU install:

.. code-block::

    debug: Found QEMU install at '/usr'

When you see this line, QEMU is fine and the firmware is the problem.
When you see ``debug: No QEMU install found`` instead, Cubic never located the
binaries and the firmware search never ran.

QEMU Was Not Found
------------------

Cubic looks for ``qemu-system-x86_64``, ``qemu-system-aarch64`` and
``qemu-img`` in the directories described in :ref:`qemu detection`.
Check that the binaries exist and that their directory is on your ``PATH``:

.. code-block::

    $ qemu-system-x86_64 --version
    $ qemu-img --version

When QEMU lives somewhere else, point Cubic at the directory that contains the
binaries:

.. code-block::

    $ CUBIC_QEMU_DIR=/opt/qemu/bin cubic start <instance>

Cubic puts this directory at the front of its search list, so it wins over
your ``PATH`` and the built-in fallbacks.
Only the QEMU process gets this search path, your own shell stays untouched.

The Firmware Was Not Found
--------------------------

Cubic reads the QEMU firmware descriptors, which are JSON files that describe
the available UEFI images. It collects them from these directories, in order:

* ``$XDG_CONFIG_HOME/qemu/firmware``, or ``$HOME/.config/qemu/firmware`` when
  ``XDG_CONFIG_HOME`` is not set
* ``/etc/qemu/firmware``
* ``<prefix>/share/qemu/firmware``
* ``<prefix>/share/firmware``
* ``<prefix>/firmware``

The prefix is the parent of the directory that holds the QEMU binaries, so
``/usr/bin`` gives the prefix ``/usr``. On Windows the prefix is the directory
itself.

Descriptors are matched by file name and the first directory wins. A leftover
file in your home directory therefore hides the one shipped by your
distribution. List the descriptors and remove the stale copies:

.. code-block::

    $ ls ~/.config/qemu/firmware /etc/qemu/firmware /usr/share/qemu/firmware

A descriptor is only useful when the firmware image it points at is present on
disk. Cubic skips every descriptor whose image is missing, which is what
happens when the QEMU package is installed without the matching firmware
package.

Point Cubic at a Firmware Image
-------------------------------

When the descriptors are unusable you can name the firmware image directly:

.. code-block::

    $ CUBIC_QEMU_FW_AMD64=/usr/share/OVMF/OVMF_CODE.fd cubic start <instance>

Use ``CUBIC_QEMU_FW_ARM64`` for arm64 machines.
Cubic uses these paths as they are and never checks that the file exists.
A typo therefore replaces this error with a QEMU startup failure later on.
Verify the path before you set the variable:

.. code-block::

    $ ls -l /usr/share/OVMF/OVMF_CODE.fd

The variables are described with the rest of the settings in
:ref:`qemu detection`.
