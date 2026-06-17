.. _qemu detection:

QEMU and Firmware Detection
============================

Cubic relies on QEMU and UEFI firmware to create and run virtual machines.
When you start a VM, Cubic automatically searches for these tools in standard
locations. This page explains where Cubic looks and how to override the
detected paths when needed.

How Cubic looks
---------------

Cubic builds a single ordered list of directories where QEMU may live:

#. ``CUBIC_QEMU_DIR`` (if set),
#. every directory on your ``PATH``,
#. the built-in fallback locations:
   ``C:\Program Files\qemu`` on Windows, and ``/usr/bin``,
   ``/usr/local/bin``, ``/opt/homebrew/bin``,
   ``/home/linuxbrew/.linuxbrew/bin`` and ``/opt/local/bin`` on Unix.

That list is used two ways. For the **binaries**, Cubic hands the list to QEMU
as its ``PATH`` (only for the child process, your own environment is left
untouched) and launches ``qemu-system-x86_64`` / ``qemu-system-aarch64`` /
``qemu-img`` by name, letting the operating system resolve them. For the
**firmware**, Cubic finds the QEMU install from the same list and reads its
firmware descriptors (see below).

QEMU Binary
-----------

The binary name depends on the architecture of the virtual machine:

.. list-table::
   :header-rows: 1
   :widths: 20 40

   * - Architecture
     - Binary
   * - amd64
     - ``qemu-system-x86_64``
   * - arm64
     - ``qemu-system-aarch64``

Install QEMU if it is not already present:

.. list-table::
   :header-rows: 1
   :widths: 30 50

   * - Platform
     - Command
   * - Debian / Ubuntu
     - ``sudo apt install qemu-system qemu-utils ovmf qemu-efi-aarch64``
   * - Fedora / RHEL
     - ``sudo dnf install qemu-system-x86 qemu-img edk2-ovmf edk2-aarch64``
   * - Arch Linux
     - ``sudo pacman -S qemu-full edk2-ovmf edk2-armvirt``
   * - openSUSE
     - ``sudo zypper install qemu qemu-tools qemu-ovmf-x86_64 qemu-uefi-aarch64``
   * - macOS (Homebrew)
     - ``brew install qemu``
   * - Windows
     - ``winget install SoftwareFreedomConservancy.QEMU``

On Linux the UEFI firmware (OVMF for amd64, AAVMF for arm64) is a separate
package. The commands above install it alongside QEMU. On macOS (Homebrew) and
Windows the firmware is bundled with QEMU.

Override
~~~~~~~~

Set ``CUBIC_QEMU_DIR`` to the directory that holds your QEMU binaries. Cubic
puts it at the front of the search list (for the binaries and the firmware
lookup below):

.. code-block::

    $ CUBIC_QEMU_DIR=/opt/qemu/bin cubic start my-vm

qemu-img
--------

Cubic runs ``qemu-img`` from the same QEMU install as the emulator. This tool is
used to create and resize virtual machine disk images. It is included with QEMU
on all platforms, so installing QEMU (see above) is enough.

UEFI Firmware
-------------

Cubic requires UEFI firmware to boot virtual machines. Rather than guessing
filenames, Cubic reads QEMU's **firmware descriptor** files
(``share/qemu/firmware/*.json``, shipped by QEMU on Linux, Homebrew and Windows)
and selects the plain UEFI (pflash) firmware whose target matches the VM's
architecture and machine (``q35`` for amd64, ``virt`` for arm64). Other
special-purpose variants (such as secure boot) are skipped.

The firmware file named by the chosen descriptor is then resolved **relative to
the QEMU install** (anchored on its ``share/`` directory). This lets the
firmware that ships next to QEMU resolve even when the descriptor records an
absolute path that is not reachable, for example inside a confined snap, or in
a relocated Windows build. On Linux the firmware and its descriptor ship in a
separate package (``ovmf`` / ``edk2``, installed by the commands above). On
macOS (Homebrew) and Windows they are bundled with QEMU.

Where the firmware comes from, per setup:

.. list-table::
   :header-rows: 1
   :widths: 30 30 40

   * - Setup
     - QEMU install prefix
     - Firmware file
   * - Linux package manager
     - ``/usr``
     - ``/usr/share/OVMF/OVMF_CODE_4M.fd`` (and distro equivalents)
   * - snap (strict confinement)
     - ``$SNAP/usr``
     - ``$SNAP/usr/share/OVMF/OVMF_CODE_4M.fd``
   * - macOS Homebrew / MacPorts
     - ``/opt/homebrew``, ``/usr/local``, ``/opt/local``
     - ``<prefix>/share/qemu/edk2-<arch>-code.fd``
   * - Windows (winget)
     - ``C:\Program Files\qemu``
     - ``C:\Program Files\qemu\share\edk2-<arch>-code.fd``

Override
~~~~~~~~

Point ``CUBIC_QEMU_DIR`` at a QEMU install and Cubic reads its firmware
descriptors, or set the per-architecture ``CUBIC_QEMU_FW_AMD64`` /
``CUBIC_QEMU_FW_ARM64`` to use a specific firmware file directly:

.. code-block::

    $ CUBIC_QEMU_FW_AMD64=/opt/qemu/share/qemu/edk2-x86_64-code.fd cubic start my-vm

Environment Variable Reference
-------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Variable
     - Tool
     - Description
   * - ``CUBIC_QEMU_DIR``
     - QEMU emulator, qemu-img and UEFI firmware
     - Directory of the QEMU install. Placed at the front of the search list for
       the binaries, and used to locate the ``share/qemu/firmware`` descriptors.
   * - ``CUBIC_QEMU_FW_AMD64``
     - UEFI firmware (amd64)
     - Path to a specific amd64 UEFI firmware (CODE) file. Overrides
       descriptor-based firmware selection for amd64 VMs.
   * - ``CUBIC_QEMU_FW_ARM64``
     - UEFI firmware (arm64)
     - Path to a specific arm64 UEFI firmware (CODE) file. Overrides
       descriptor-based firmware selection for arm64 VMs.
