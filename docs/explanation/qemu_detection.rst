.. _qemu detection:

QEMU and Firmware Detection
===========================

Cubic needs two things from the host that it does not ship itself, the QEMU
binaries and the UEFI firmware that boots a guest. Both are installed in a
different place by every platform and every packager. This page explains how
Cubic finds them and why it looks the way it does.

:ref:`Install Cubic` lists the packages to install, :ref:`qemu not found` helps
when the search fails, and :ref:`env vars` documents the variables named here.

One Search List
---------------

Cubic builds a single ordered list of directories where QEMU may live:

#. ``CUBIC_QEMU_DIR``, when it is set
#. every directory on your ``PATH``
#. built in fallback locations, which are ``C:\Program Files\qemu`` on Windows
   and ``/usr/bin``, ``/usr/local/bin``, ``/opt/homebrew/bin``,
   ``/home/linuxbrew/.linuxbrew/bin`` and ``/opt/local/bin`` on Unix

The fallbacks matter because a graphical session or a launcher often has a
shorter ``PATH`` than your shell, and because Homebrew and MacPorts install
outside the system directories.

That one list serves both purposes. For the binaries Cubic hands the list to
QEMU as its ``PATH`` and starts ``qemu-system-x86_64``, ``qemu-system-aarch64``
or ``qemu-img`` by name, letting the operating system resolve it. Only the child
process gets that ``PATH``, your own environment is never touched. For the
firmware Cubic takes the first directory of the list that holds a QEMU binary
and treats its parent as the QEMU install prefix, so ``/usr/bin`` gives
``/usr``. On Windows the directory itself is the prefix.

Descriptors Instead of File Names
---------------------------------

Firmware files are named differently everywhere. One distribution ships
``OVMF_CODE_4M.fd``, another ``OVMF_CODE.fd``, and Homebrew and Windows ship
``edk2-x86_64-code.fd``. Guessing from a list of names breaks on the next
distribution that picks another one.

QEMU and the distributions ship firmware descriptors instead, JSON files that
state what each firmware image is and what it targets. Cubic collects them from
the configuration directory of QEMU, from ``/etc/qemu/firmware`` and from the
install prefix, and :ref:`qemu not found` lists that order in full.

A descriptor is used when it maps a flash device with a UEFI interface and names
a target that matches the architecture and the machine type of the VM instance,
``q35`` for x86-64 and ``virt`` for aarch64. Descriptors with a feature that needs
setup a plain guest does not have are skipped, which covers secure boot, SMM,
AMD SEV-SNP and Intel TDX.

This is why Cubic works on distributions it has never seen. The distribution
describes its own firmware and Cubic reads that description.

Paths Anchored on the QEMU Install
----------------------------------

A descriptor records an absolute path to its firmware image, and that path is
not always the truth. Inside a strictly confined snap the real file sits under
``$SNAP``. A relocated Windows build does not live where its packager built it.

Cubic therefore resolves the file relative to the QEMU install it found,
anchored on the ``share`` directory of the prefix. The descriptor still says
which file to use, while the install says where that file is.

.. list-table::
   :header-rows: 1
   :widths: 30 30 40

   * - Setup
     - QEMU install prefix
     - Firmware file
   * - Linux package manager
     - ``/usr``
     - ``/usr/share/OVMF/OVMF_CODE_4M.fd`` and the equivalents of other
       distributions
   * - snap, strict confinement
     - ``$SNAP/usr``
     - ``$SNAP/usr/share/OVMF/OVMF_CODE_4M.fd``
   * - macOS Homebrew or MacPorts
     - ``/opt/homebrew``, ``/usr/local``, ``/opt/local``
     - ``<prefix>/share/qemu/edk2-<arch>-code.fd``
   * - Windows, winget
     - ``C:\Program Files\qemu``
     - ``C:\Program Files\qemu\share\edk2-<arch>-code.fd``

On Linux the firmware and its descriptor come in a separate package from QEMU,
``ovmf`` or ``edk2`` depending on the distribution, which is why QEMU can be
installed and the firmware still missing. On macOS and Windows both ship
together.

When Detection Is Wrong
-----------------------

Two escape hatches exist, and they work at different levels.

``CUBIC_QEMU_DIR`` names a QEMU install. It goes to the front of the search list
and is used for the binaries and for the firmware descriptors, so it moves the
whole detection to another install.

``CUBIC_QEMU_FW_AMD64`` and ``CUBIC_QEMU_FW_ARM64`` name a firmware file
directly and skip the descriptors entirely. Cubic trusts such a path as it is
and never checks that the file exists, so a typo turns this error into a QEMU
startup failure later on.

Prefer the first when a whole QEMU install lives somewhere unusual, and the
second only when the descriptors themselves are unusable. :ref:`qemu not found`
walks through both.

Related
-------

* :ref:`qemu not found` when Cubic cannot start a VM instance
* :ref:`Install Cubic` for the QEMU packages of each platform
* :ref:`env vars` for the variables named here
* :ref:`design` for what Cubic does with QEMU once it is found
