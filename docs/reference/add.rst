.. _ref_cubic_add:

cubic add
=========

.. code-block::

    $ cubic add --help
    Create a new virtual machine instance

    Usage: cubic add [OPTIONS] --image <IMAGE> [INSTANCE_NAME]

    Arguments:
      [INSTANCE_NAME]  Name of the virtual machine instance

    Options:
      -i, --image <IMAGE>  Name of the virtual machine image
      -u, --user <USER>    Name of the user [default: cubic]
      -c, --cpus <CPUS>    Number of CPUs for the virtual machine instance
      -m, --mem <MEM>      Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
      -d, --disk <DISK>    Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
      -h, --help           Print help
