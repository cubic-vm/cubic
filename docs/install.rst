.. _Install Cubic:

Install Cubic
=============

Linux
-----

Install Cubic on Linux as `Snap`_ with the following command:

.. code-block::

       $ sudo snap install cubic && sudo snap connect cubic:kvm && sudo snap connect cubic:ssh-keys

.. _Snap: https://snapcraft.io/cubic

macOS
-----

Install Cubic on macOS via `Homebrew`_ with the following command:

.. code-block::

       $ brew install cubic-vm/cubic/cubic

.. _Homebrew: https://brew.sh

Windows
-------

Install Cubic on Windows in `WSL`_ as `Snap`_ with the following command:

.. code-block::

       $ sudo snap install cubic && sudo snap connect cubic:kvm && sudo snap connect cubic:ssh-keys
 
.. _WSL: https://learn.microsoft.com/en-us/windows/wsl/install
.. _Snap: https://snapcraft.io/cubic

Others
------

Install Cubic from `crates.io`_:

.. note:: Cubic requires the following dependencies: ``QEMU``, ``OpenSSH`` and either ``cdrtools`` or ``cdrkit``.


.. code-block::

       $ cargo install cubic

.. _crates.io: https://crates.io/crates/cubic
