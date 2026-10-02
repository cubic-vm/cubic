.. _file locations:

File Locations
==============

Cubic keeps everything a VM instance owns in one directory and shares the
downloaded images in a cache. Both live in your own user directories, so no
command needs elevated rights.

Instance Directory
------------------

Each VM instance has its own directory named after it:

.. list-table::
   :header-rows: 1
   :widths: 15 42 43

   * - Platform
     - Path
     - Typical value
   * - Linux
     - ``$XDG_DATA_HOME/cubic/machines/<instance>``
     - ``~/.local/share/cubic/machines/<instance>``
   * - Snap
     - ``$SNAP_USER_COMMON/cubic/machines/<instance>``
     - ``~/snap/cubic/common/cubic/machines/<instance>``
   * - macOS
     - ``$HOME/Library/cubic/machines/<instance>``
     - ``~/Library/cubic/machines/<instance>``
   * - Windows
     - ``%LOCALAPPDATA%\cubic\machines\<instance>``
     - ``C:\Users\<user>\AppData\Local\cubic\machines\<instance>``

Files in an Instance Directory
------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - File
     - Content
   * - ``instance.toml``
     - Settings of the VM instance and the stored SSH host key of the guest.
       Every field is listed in :ref:`instance file`.
   * - ``machine.img``
     - Disk image in ``qcow2`` format, including its snapshots.
   * - ``ssh_client_key``
     - Private SSH key of the VM instance.
   * - ``cloud-init.iso``
     - Seed image cloud-init reads on the first boot. Its content is listed in
       :ref:`guest config`.
   * - ``qemu.pid``
     - Process id of QEMU. Present while the VM instance runs.
   * - ``monitor.sock``, ``console.sock``
     - Unix domain sockets of the QEMU monitor and the serial console. Present
       while the VM instance runs.

Image Cache
-----------

Downloaded distribution images are shared by every VM instance that uses them:

.. list-table::
   :header-rows: 1
   :widths: 15 42 43

   * - Platform
     - Path
     - Typical value
   * - Linux
     - ``$XDG_CACHE_HOME/cubic``
     - ``~/.cache/cubic``
   * - macOS
     - ``$HOME/Library/Caches/cubic``
     - ``~/Library/Caches/cubic``
   * - Windows
     - ``%TEMP%\cubic``
     - ``C:\Users\<user>\AppData\Local\Temp\cubic``

The directory holds ``images`` with one file per image, named
``<distribution>_<release>_<architecture>`` such as ``ubuntu_resolute_amd64``.
