.. _template file:

Template File
=============

A template is a TOML file with default values for a new VM instance. Pass it to
``cubic create`` or ``cubic run`` with ``--template``. :ref:`templates` shows
how to use one.

Every field except ``version`` is optional and falls back to the Cubic default
when it is left out, see :ref:`values`. A command line argument always wins over
the template.

.. list-table::
   :header-rows: 1
   :widths: 15 25 60

   * - Field
     - Value
     - Description
   * - ``version``
     - ``1``
     - Format version of the template. Mandatory.
   * - ``image``
     - name and tag, ``"ubuntu:26.04"``
     - Distribution image of the VM instance. A plain name means the ``stable``
       tag. Defaults to ``ubuntu``. See :ref:`image names`.
   * - ``user``
     - user name, ``"john"``
     - User account inside the guest. Defaults to your user name on the host.
   * - ``cpus``
     - number, ``4``
     - vCPUs of the VM instance. Defaults to a value derived from the resources
       of the host.
   * - ``memory``
     - size, ``"4G"``
     - Memory of the VM instance. Defaults to a value derived from the
       resources of the host.
   * - ``disk``
     - size, ``"100G"``
     - Disk capacity of the VM instance. Defaults to 100 GiB. A disk can only
       grow later.
   * - ``isolate``
     - ``true`` or ``false``
     - Cuts the VM instance off the network. Defaults to ``false``.
   * - ``ports``
     - list of ``"host:guest"``
     - Forwarded ports, with an optional protocol such as ``"5353:53/udp"``.
   * - ``run``
     - list of shell commands
     - Commands the guest runs once on its first boot, as root.

Example
-------

.. code-block::

    version = 1              # mandatory

    image = "debian"         # optional
    user = "john"            # optional
    cpus = 4                 # optional
    memory = "4G"            # optional
    disk = "100G"            # optional
    isolate = false          # optional

    ports = [                # optional
      "8000:80",             # forward guest HTTP port to host TCP port 8000
      "5353:53/udp"          # forward guest DNS port to host UDP port 5353
    ]

    run = [                  # optional, run once on the first boot as root
      "apt update",
      "apt full-upgrade -y",
      "apt install -y vim"
    ]
