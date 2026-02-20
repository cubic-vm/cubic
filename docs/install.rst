.. _Install Cubic:

Install Cubic
=============

Install on Linux
----------------

You can install Cubic on Linux with the following methods:

* `Install with Snap`_  (**recommend**)
* `Install with Cargo`_ 
* `Install with Homebrew`_ 

Install on macOS
----------------

You can install Cubic on macOS with the following methods:

* `Install with Homebrew`_ (**recommend**)
* `Install with Cargo`_ 

Install on Windows
------------------

You can install Cubic on Windows with the following methods:

* `Install with Snap`_ (in WSL; **recommend**)
* `Install with Cargo`_ 
* `Install with Homebrew`_ (in WSL)

Install with Cargo
------------------

1. Install Dependencies
^^^^^

Install rustup and Cubic dependencies (QEMU, genisoimage).

For Debian, Ubuntu and derivatives:

.. code-block::

    sudo apt install build-essential rustup qemu-system-x86 qemu-system-arm genisoimage

For Fedora and derivatives:

.. code-block::

    sudo dnf install @development-tools rustup qemu-system-x86 qemu-system-arm qemu-img genisoimage
    rustup-init -y
    . "$HOME/.cargo/env"

For OpenSUSE and derivatives:

.. code-block::

    sudo zypper install rustup qemu-x86 qemu-arm cdrtools

2. Install Rust toolchain
^^^^^

Install the latest stable Rust toolchain.

.. code-block::

    rustup toolchain add stable

3. Install Cubic
^^^^^

.. code-block::

    cargo install cubic

4. Update PATH Environment Variable
^^^^

Add Cargo bin directory to PATH environment variable.

For Linux distributions:

.. code-block::

    echo 'export PATH="$PATH:~/.cargo/bin"' >> ~/.profile
    source ~/.profile

5. Allow KVM Acceleration (Linux only, Optional)
^^^^^

It is recommend to add Kernel-based Virtual Machine (KVM) permisson to your user for optimal VM performance:

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

Connect the KVM interface to accelerate the virtual machine (recommend):

.. code-block::

       $ sudo snap connect cubic:kvm

.. _Snap: https://snapcraft.io/cubic
