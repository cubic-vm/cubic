.. _instance file:

Instance File
=============

Every VM instance stores its settings in ``instance.toml`` in its own directory,
see :ref:`file locations`. Cubic writes the file when the VM instance is created
and updates it on every change.

Use ``cubic show`` to read the settings and ``cubic modify`` to change them.
Editing the file by hand works but is not checked, and a change only takes
effect after a restart. Edit it while the VM instance is stopped, because Cubic
rewrites the file on its own, for example when it assigns a port at the start.

Fields
------

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Field
     - Value
     - Description
   * - ``arch``
     - ``"AMD64"`` or ``"ARM64"``
     - Architecture of the guest. Set when the VM instance is created and never
       changed afterwards.
   * - ``os``
     - os name
     - Guest OS, such as ``debian:13``. Set from the image when the VM instance
       is created and refreshed from ``/etc/os-release`` in the guest on every
       ssh, scp and exec connection. A guest that keeps the file to itself keeps
       the stored value.
   * - ``user``
     - user name
     - User account inside the guest. cloud-init created it on the first boot,
       so a later change locks you out.
   * - ``cpus``
     - number
     - vCPUs handed to QEMU.
   * - ``mem``
     - bytes
     - Memory of the guest, in bytes rather than a size string.
   * - ``disk_capacity``
     - bytes
     - Size the disk image may grow to, in bytes. A disk can only grow.
   * - ``ssh_port``
     - port
     - Host port forwarded to port 22 of the guest. Assigned when the VM
       instance is created. Cubic assigns a new one when the port is taken at
       the next start.
   * - ``monitor_port``
     - port
     - Host port of the QEMU monitor. Assigned at every start, so it changes
       from run to run.
   * - ``console_port``
     - port
     - Host port of the serial console. Assigned at every start as well.
   * - ``hostfwd``
     - list of strings
     - Forwarded ports in QEMU notation, written as
       ``protocol:host_ip:host_port-:guest_port``. ``cubic modify --port`` takes
       the shorter ``host:guest`` form instead.
   * - ``execute``
     - shell command
     - Commands the guest runs once on its first boot. Several ``--exec``
       arguments are joined with ``&&``.
   * - ``isolate``
     - ``true`` or ``false``
     - Cuts the guest off the network.
   * - ``ssh_host_key``
     - public key
     - SSH host key of the guest. Cubic pins it on the first connect and asks
       before it accepts a different key later.

``arch``, ``user``, ``cpus``, ``mem``, ``disk_capacity`` and ``ssh_port`` are
always written. The rest is left out while it is unset.

Example
-------

.. code-block::

    arch = "AMD64"
    user = "john"
    cpus = 4
    mem = 4294967296
    disk_capacity = 107374182400
    ssh_port = 37635
    monitor_port = 42661
    console_port = 37913
    hostfwd = ["tcp:127.0.0.1:8080-:80"]
    execute = "sudo apt update && sudo apt install -y nginx"
    isolate = false
    ssh_host_key = "ssh-ed25519 AAAA..."

Cubic Never Stores
------------------

Some state lives elsewhere and is not part of the file:

* Snapshots are read back from the disk image itself.
* The disk usage is measured on the fly.
* Whether the VM instance runs follows from ``qemu.pid``.
* Image tags are derived from the image list at every read.

Related
-------

* :ref:`template file` for the defaults of a new VM instance
* :ref:`file locations` of the instance directory
* :ref:`values` for the size units and the name rules
