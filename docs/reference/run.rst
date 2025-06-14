.. _ref_cubic_run:

cubic run
=========

.. code-block::

    $ cubic run --help
    Setup and run a new instance

    Usage: cubic run [OPTIONS] --image <IMAGE> --name <NAME>

    Options:
      -i, --image <IMAGE>  Name of the virtual machine image
      -n, --name <NAME>    Name of the virtual machine instance
      -c, --cpus <CPUS>    Number of CPUs for the virtual machine instance
      -m, --mem <MEM>      Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
      -d, --disk <DISK>    Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
      -v, --verbose        Enable verbose logging
      -q, --quiet          Reduce logging output
      -h, --help           Print help
