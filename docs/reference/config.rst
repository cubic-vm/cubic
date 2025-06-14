.. _ref_cubic_config:

cubic config
============

.. code-block::

    $ cubic config --help
    Read and write virtual machine instance configuration parameters

    Usage: cubic config [OPTIONS] <INSTANCE>

    Arguments:
      <INSTANCE>  Name of the virtual machine instance

    Options:
      -c, --cpus <CPUS>  Number of CPUs for the virtual machine instance
      -m, --mem <MEM>    Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
      -d, --disk <DISK>  Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
      -h, --help         Print help
