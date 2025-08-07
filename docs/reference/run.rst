.. _ref_cubic_run:

cubic run
=========

.. code-block::

    $ cubic run --help
    Create, start and open a shell in a new virtual machine instance

    Usage: cubic run [OPTIONS] --image <IMAGE> [INSTANCE_NAME]

    Arguments:
      [INSTANCE_NAME]  Name of the virtual machine instance

    Options:
      -i, --image <IMAGE>  Name of the virtual machine image
      -u, --user <USER>    Name of the user [default: cubic]
      -c, --cpus <CPUS>    Number of CPUs for the virtual machine instance [default: 4]
      -m, --mem <MEM>      Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte) [default: 4G]
      -d, --disk <DISK>    Disk size of the virtual machine instance (e.g. 10G for 10 gigabytes) [default: 100G]
      -v, --verbose        Enable verbose logging
      -q, --quiet          Reduce logging output
      -h, --help           Print help
