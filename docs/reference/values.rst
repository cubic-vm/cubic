.. _values:

Names, Sizes and Defaults
=========================

The rules every command shares: what a name may contain, how a size is read,
and what Cubic picks when you leave a setting out.

Names
-----

.. list-table::
   :header-rows: 1
   :widths: 20 35 45

   * - Name
     - Allowed
     - Notes
   * - VM instance
     - letters, numbers, underscore and dash
     - Used as the directory name of the VM instance.
   * - Snapshot
     - letters, numbers, underscore and dash
     - Always written as ``<instance>/<snapshot>``, such as
       ``builder/before-upgrade``.
   * - Guest user
     - letters, numbers, underscore and dash
     - Must start with a letter or an underscore. Cubic turns the name into
       lowercase, so ``Tux`` becomes ``tux``.
   * - Image
     - see :ref:`image names`
     - Written as ``distro[:name][:arch]``.

Sizes
-----

A size is a whole number followed by one uppercase letter. The letters are
binary, so ``1G`` is one gibibyte and not one gigabyte.

.. list-table::
   :header-rows: 1
   :widths: 15 25 60

   * - Suffix
     - Unit
     - Bytes
   * - ``B``
     - byte
     - 1
   * - ``K``
     - kibibyte
     - 1024
   * - ``M``
     - mebibyte
     - 1048576
   * - ``G``
     - gibibyte
     - 1073741824
   * - ``T``
     - tebibyte
     - 1099511627776

Lowercase letters and decimals are rejected, so use ``512M`` rather than
``0.5G`` or ``512m``. Cubic prints sizes back with a single letter ``B``, ``K``,
``M``, ``G`` or ``T`` and no decimal, dropping to the next smaller unit when the
value would fall below ten, so ``1G`` of memory prints as ``1024 M``. Sizes are
stored as plain bytes in the :ref:`instance file`.

Defaults of a New VM Instance
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 80

   * - Setting
     - Default
   * - Image
     - None. ``--image`` or a template is required.
   * - User
     - Your user name on the host, taken from ``USER``, ``LOGNAME`` or
       ``USERNAME``. ``root`` and a name that breaks the rule above fall back
       to ``cubic``.
   * - Architecture
     - Architecture of the host.
   * - Disk
     - 100 GiB. A disk can only grow later.
   * - vCPUs and memory
     - Derived from the host, see below.
   * - Isolation
     - Off.
   * - Ports
     - Only SSH, on a free host port bound to ``127.0.0.1``.

vCPUs and Memory
----------------

A fixed default would be wasteful on a small host and harmful on a laptop, since
QEMU reserves the full memory at startup. Cubic therefore picks a size from the
host.

The sizes form levels. Level N gives 2N vCPUs and N GiB of memory, so every
level keeps half a GiB per vCPU. A level needs the host to have at least
4 * (N + 1) threads and the same number of GiB of memory. Cubic takes the
largest level both totals allow. A host too small for level 1 gets 1 vCPU and
512 MiB.

.. list-table::
   :header-rows: 1
   :widths: 30 30 40

   * - Host threads
     - Host memory
     - Default of a new VM instance
   * - 4
     - 4 GiB
     - 1 vCPU, 512 MiB
   * - 8
     - 8 GiB
     - 2 vCPUs, 1 GiB
   * - 16
     - 16 GiB
     - 6 vCPUs, 3 GiB
   * - 32
     - 64 GiB
     - 14 vCPUs, 7 GiB

The smaller of the two totals decides, so a host with many threads and little
memory gets the memory level. A VM instance therefore always stays below half of
the host threads and below a quarter of the host memory. Set ``--cpus`` and
``--memory`` to override it, either at create time or later with
``cubic modify``.

Cubic keeps 1 GiB of memory aside for the host when it checks whether a VM
instance can start, and warns when the host has less than 5 GiB of free disk
space.

Related
-------

* :ref:`resources` of a VM instance
* :ref:`instance file` where the values are stored
* :ref:`template file` to set your own defaults
