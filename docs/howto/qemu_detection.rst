.. _qemu detection:

QEMU and Firmware Detection
============================

Cubic relies on QEMU and UEFI firmware to create and run virtual machines.
When you start a VM, Cubic automatically searches for these tools in standard
locations. This page explains where Cubic looks and how to override the
detected paths when needed.

QEMU Binary
-----------

Cubic searches for the QEMU system emulator on your ``PATH``. The binary name
depends on the architecture of the virtual machine:

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
     - ``sudo apt install qemu-system``
   * - Fedora / RHEL
     - ``sudo dnf install qemu-system-x86``
   * - Arch Linux
     - ``sudo pacman -S qemu-full``
   * - openSUSE
     - ``sudo zypper install qemu``
   * - macOS (Homebrew)
     - ``brew install qemu``
   * - Windows
     - Install from https://www.qemu.org/download/#windows

Override
~~~~~~~~

Set ``CUBIC_QEMU`` to use a QEMU binary that is not on ``PATH``:

.. code-block::

    $ CUBIC_QEMU=/opt/qemu/bin/qemu-system-x86_64 cubic start my-vm

qemu-img
--------

Cubic searches for ``qemu-img`` on your ``PATH``. This tool is used to create
and resize virtual machine disk images. It is included with QEMU on all
platforms.

Install ``qemu-img`` if it is not already present:

.. list-table::
   :header-rows: 1
   :widths: 30 50

   * - Platform
     - Command
   * - Debian / Ubuntu
     - ``sudo apt install qemu-utils``
   * - Fedora / RHEL
     - ``sudo dnf install qemu-img``
   * - Arch Linux
     - ``sudo pacman -S qemu-img``
   * - openSUSE
     - ``sudo zypper install qemu-tools``
   * - macOS (Homebrew)
     - ``brew install qemu``
   * - Windows
     - Install from https://www.qemu.org/download/#windows

Override
~~~~~~~~

Set ``CUBIC_QEMU_IMG`` to use a ``qemu-img`` binary that is not on ``PATH``:

.. code-block::

    $ CUBIC_QEMU_IMG=/opt/qemu/bin/qemu-img cubic create my-vm --image ubuntu:noble

UEFI Firmware
-------------

Cubic requires a UEFI firmware image (OVMF for amd64, AAVMF for arm64) to
boot virtual machines. Cubic searches a list of well-known paths in order and
uses the first file that exists.

AMD64 (OVMF)
~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 40

   * - Path
     - Platform
   * - ``$SNAP/usr/share/OVMF/OVMF_CODE_4M.fd``
     - Snap package
   * - ``<qemu-root>/share/qemu/edk2-x86_64-code.fd``
     - QEMU install directory
   * - ``/usr/share/OVMF/OVMF_CODE_4M.fd``
     - Debian / Ubuntu
   * - ``/usr/share/edk2/ovmf/OVMF_CODE.4m.fd``
     - Fedora / RHEL
   * - ``/usr/share/edk2-ovmf/OVMF_CODE.fd``
     - Fedora (older)
   * - ``/usr/share/edk2/x64/OVMF_CODE.4m.fd``
     - Arch Linux
   * - ``/usr/share/qemu/ovmf-x86_64-code.bin``
     - openSUSE
   * - ``/opt/homebrew/share/qemu/edk2-x86_64-code.fd``
     - Homebrew (Apple Silicon)
   * - ``/usr/local/share/qemu/edk2-x86_64-code.fd``
     - Homebrew (Intel)
   * - ``/home/linuxbrew/.linuxbrew/share/qemu/edk2-x86_64-code.fd``
     - Linux Homebrew (non-root)
   * - ``C:/Program Files/QEMU/share/edk2-x86_64-code.fd``
     - Windows

Install OVMF if it is not already present:

.. list-table::
   :header-rows: 1
   :widths: 30 50

   * - Platform
     - Command
   * - Debian / Ubuntu
     - ``sudo apt install ovmf``
   * - Fedora / RHEL
     - ``sudo dnf install edk2-ovmf``
   * - Arch Linux
     - ``sudo pacman -S edk2-ovmf``
   * - openSUSE
     - ``sudo zypper install qemu-ovmf-x86_64``
   * - macOS (Homebrew)
     - ``brew install qemu``
   * - Windows
     - Install from https://www.qemu.org/download/#windows

ARM64 (AAVMF)
~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 40

   * - Path
     - Platform
   * - ``$SNAP/usr/share/AAVMF/AAVMF_CODE.fd``
     - Snap package
   * - ``<qemu-root>/share/qemu/edk2-aarch64-code.fd``
     - QEMU install directory
   * - ``/usr/share/AAVMF/AAVMF_CODE.fd``
     - Debian / Ubuntu
   * - ``/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw``
     - Fedora / RHEL
   * - ``/usr/share/edk2-armvirt/aarch64/QEMU_EFI.fd``
     - Arch Linux
   * - ``/usr/share/qemu/aavmf-aarch64-code.bin``
     - openSUSE
   * - ``/opt/homebrew/share/qemu/edk2-aarch64-code.fd``
     - Homebrew (Apple Silicon)
   * - ``/usr/local/share/qemu/edk2-aarch64-code.fd``
     - Homebrew (Intel)
   * - ``/home/linuxbrew/.linuxbrew/share/qemu/edk2-aarch64-code.fd``
     - Linux Homebrew (non-root)
   * - ``C:/Program Files/QEMU/share/edk2-aarch64-code.fd``
     - Windows

Install AAVMF if it is not already present:

.. list-table::
   :header-rows: 1
   :widths: 30 50

   * - Platform
     - Command
   * - Debian / Ubuntu
     - ``sudo apt install qemu-efi-aarch64``
   * - Fedora / RHEL
     - ``sudo dnf install edk2-aarch64``
   * - Arch Linux
     - ``sudo pacman -S edk2-armvirt``
   * - openSUSE
     - ``sudo zypper install qemu-uefi-aarch64``
   * - macOS (Homebrew)
     - ``brew install qemu``
   * - Windows
     - Install from https://www.qemu.org/download/#windows

Override
~~~~~~~~

Set ``CUBIC_FW`` to use a firmware file that was not found by autodetection.
This bypasses the entire candidate search:

.. code-block::

    $ CUBIC_FW=/usr/share/OVMF/OVMF_CODE.fd cubic start my-vm

Environment Variable Reference
-------------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Variable
     - Tool
     - Description
   * - ``CUBIC_QEMU``
     - QEMU emulator
     - Path to the ``qemu-system-*`` binary to use instead of the one found on
       ``PATH``
   * - ``CUBIC_QEMU_IMG``
     - qemu-img
     - Path to the ``qemu-img`` binary to use instead of the one found on
       ``PATH``
   * - ``CUBIC_FW``
     - UEFI firmware
     - Path to the OVMF or AAVMF firmware file to use instead of the
       autodetected path
