.. _Install Cubic:

Install Cubic
=============

Install on Linux
----------------

You can install Cubic on Linux with the following methods:

* `Install with Cargo`_ 
* `Install with Homebrew`_ 
* `Install with Snap`_ 

Install on macOS
----------------

You can install Cubic on macOS with the following methods:

* `Install with Cargo`_ 
* `Install with Homebrew`_ 

Install on Windows
------------------

You can install Cubic on Windows with the following methods:

* `Install with Cargo`_ 
* `Install with Homebrew`_ (in WSL)
* `Install with Snap`_ (in WSL)

Install with Cargo
------------------

Use the following command to install Cubic with `Cargo`_:

.. note:: Cubic requires the following dependencies: ``QEMU``, ``OpenSSH`` and either ``cdrtools`` or ``cdrkit``.

.. code-block::

       $ cargo install cubic

.. _Cargo: https://crates.io/crates/cubic

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

Allow access to the host `.ssh/` directory if you want to connect to the VM with your personal SSH keys (optional):

.. code-block::

       $ sudo snap connect cubic:ssh-keys

.. _Snap: https://snapcraft.io/cubic
