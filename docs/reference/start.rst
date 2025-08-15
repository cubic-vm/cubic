.. _ref_cubic_start:

cubic start
===========

.. code-block::

    $ cubic start --help
    Start virtual machine instances

    Usage: cubic start [OPTIONS] [INSTANCES]...

    Arguments:
      [INSTANCES]...  Name of the virtual machine instances to start

    Options:
          --qemu-args <QEMU_ARGS>  Pass additional QEMU arguments
      -w, --wait                   Wait for the virtual machine instance to be started
      -v, --verbose                Increase logging output
      -q, --quiet                  Reduce logging output
      -h, --help                   Print help
