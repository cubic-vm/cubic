.. _guest config:

Guest Configuration
===================

Cubic never modifies a distribution image. Everything a guest gets is handed to
cloud-init on the first boot through a small seed image, ``cloud-init.iso`` in
the instance directory. This page lists what that configuration contains.

The seed image is built once, when the VM instance is created, and cloud-init
reads it on the first boot only. Later boots ignore it, so a change here needs a
new VM instance.

Metadata
--------

.. list-table::
   :header-rows: 1
   :widths: 25 75

   * - Key
     - Value
   * - ``instance-id``
     - Name of the VM instance.
   * - ``local-hostname``
     - Name of the VM instance, so the guest hostname matches it.

User Account
------------

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Setting
     - Value
   * - ``name``
     - Your user name on the host, or the one you passed to ``--user``. See
       :ref:`values` for the fallback.
   * - ``lock_passwd``
     - ``false``. The ``hashed_passwd`` below keeps password logins closed.
   * - ``hashed_passwd``
     - ``*``. The account has no valid password, so there is no password to
       guess and no password login over SSH.
   * - ``ssh_authorized_keys``
     - Public key of the VM instance. The private half is ``ssh_client_key`` in
       the instance directory and belongs to this VM instance alone.
   * - ``shell``
     - Login shell of the distribution, such as ``/bin/bash``.
   * - ``sudo`` or ``doas``
     - Privilege tool of the distribution. ``sudo`` gets
       ``ALL=(ALL) NOPASSWD:ALL`` and ``doas`` gets ``permit nopass`` for the
       account. The account is a
       full administrator inside the guest and needs no password, which is why
       it has no password at all.

System Settings
---------------

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Setting
     - Value
   * - ``resize_rootfs``
     - ``noblock``. The root filesystem grows to the disk size in the
       background, so the first boot is not held up by it.
   * - ``ssh_genkeytypes``
     - ``ed25519``. The guest generates one host key type. Cubic pins that key
       on the first connect, see :ref:`instance file`.
   * - ``timezone``
     - Time zone of the host at the time the VM instance is created. ``UTC``
       when the host does not report one.
   * - ``/etc/ssh/sshd_config.d/10-cubic.conf``
     - Written with ``AcceptEnv *``, so the guest accepts every variable that
       ``cubic ssh --env`` and ``cubic exec --env`` send. See
       :ref:`forward env vars`.
   * - ``runcmd``
     - The commands of ``--exec``, joined with ``&&``, or the ``run`` list of a
       :ref:`template file`. cloud-init runs them as root once at the end of the
       first boot, so they need no ``sudo``.

What Cubic Does Not Configure
-----------------------------

* No packages are installed and no repository is added. The guest is the
  official image with the account above.
* No other user is created and the root account stays as the image left it.
* Nothing is mounted from the host. Use ``cubic scp`` to move files, see
  :ref:`copy files`.
* No network configuration is applied inside the guest. The image brings up the
  virtio network device with DHCP on its own.

Related
-------

* :ref:`security` and why the account has no password
* :ref:`forward env vars` into a running guest
* :ref:`instance file` where the settings of a VM instance live
