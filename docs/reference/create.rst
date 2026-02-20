.. _ref_cubic_create:

cubic create
=====

.. code-block::

    $ cubic create --help
    Create a new virtual machine instance
    
    Usage: cubic create [OPTIONS] --image <IMAGE> <INSTANCE_NAME>
    
    Arguments:
      <INSTANCE_NAME>  Name of the virtual machine instance
    
    Options:
      -i, --image <IMAGE>  Name of the virtual machine image
      -u, --user <USER>    Name of the user [default: cubic]
      -c, --cpus <CPUS>    Number of CPUs for the virtual machine instance [default: 4]
      -m, --mem <MEM>      Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte) [default: 4G]
      -d, --disk <DISK>    Disk size of the virtual machine instance (e.g. 10G for 10 gigabytes) [default: 100G]
      -p, --port <PORT>    Forward ports from guest to host (e.g. -p 8000:80 or -p 127.0.0.1:9000:90/tcp)
      -v, --verbose        Increase logging output
      -q, --quiet          Reduce logging output
      -h, --help           Print help
