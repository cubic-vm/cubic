.. _Install Cubic:

Install Cubic
=============

Install on Linux
----------------

You can install Cubic on Linux with the following methods:

* `Install with Snap`_  (**recommended**)
* `Install with Cargo`_
* `Install with Homebrew`_

Install on macOS
----------------

You can install Cubic on macOS with the following methods:

* `Install with Homebrew`_ (**recommended**)
* `Install with Cargo`_

Install on Windows
------------------

You can install Cubic on Windows with the following methods:

* `Install with Snap`_ (in WSL; **recommended**)
* `Install with Cargo`_
* `Install with Homebrew`_ (in WSL)

Install with Cargo
------------------

1. Install Dependencies
^^^^^

Cubic needs QEMU and its UEFI firmware on the host. Install them with your
package manager:

For Debian, Ubuntu and derivatives:

.. code-block::

    sudo apt install qemu-system qemu-utils ovmf qemu-efi-aarch64

For Fedora and derivatives:

.. code-block::

    sudo dnf install qemu-system-x86 qemu-img edk2-ovmf edk2-aarch64

For Arch Linux and derivatives:

.. code-block::

    sudo pacman -S qemu-full edk2-ovmf edk2-armvirt

For openSUSE and derivatives:

.. code-block::

    sudo zypper install qemu qemu-tools qemu-ovmf-x86_64 qemu-uefi-aarch64

For macOS:

.. code-block::

    brew install qemu

For Windows:

.. code-block::

    winget install SoftwareFreedomConservancy.QEMU

2. Install Rust toolchain
^^^^^

Install the `Rust toolchain`_:

.. code-block::

    rustup toolchain install stable

.. _Rust toolchain: https://rustup.rs

3. Install Cubic
^^^^^

.. code-block::

    cargo install cubic

4. Update PATH Environment Variable
^^^^

Add Cargo bin directory to PATH environment variable.

For Linux distributions:

.. code-block::

    echo 'export PATH="$PATH:$HOME/.cargo/bin"' >> ~/.profile
    source ~/.profile

5. Allow KVM Acceleration (Linux only, Optional)
^^^^^

It is recommended to add Kernel-based Virtual Machine (KVM) permission to your user for optimal VM performance:

.. code-block::

    sudo usermod -a -G kvm $USER

This requires to exit the current user session and to relogin to make the change active.

Test Cubic
^^^^

Check if Cubic is installed correctly:

.. code-block::

    cubic --help

Install with Homebrew
---------------------

Use the following command to install Cubic via `Homebrew`_:

.. code-block::

       $ brew install cubic-vm/cubic/cubic

.. _Homebrew: https://github.com/cubic-vm/homebrew-cubic



Install with Snap
-----------------

Use the following command to install Cubic with `Snap`_:

.. code-block::

       $ sudo snap install cubic

Connect the KVM interface to accelerate the virtual machine (recommended):

.. code-block::

       $ sudo snap connect cubic:kvm

.. _Snap: https://snapcraft.io/cubic
