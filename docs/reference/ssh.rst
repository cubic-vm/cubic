.. _ref_cubic_ssh:

cubic ssh
=========

.. code-block::

    $ cubic ssh --help
    Connect to a virtual machine instance with SSH

    Usage: cubic ssh [OPTIONS] <INSTANCE> [CMD]

    Arguments:
      <INSTANCE>  Name of the virtual machine instance
      [CMD]       Execute a command in the virtual machine

    Options:
      -X                         Forward X over SSH
      -v, --verbose              Enable verbose logging
      -q, --quiet                Reduce logging output
          --ssh-args <SSH_ARGS>  Pass additional SSH arguments
      -h, --help                 Print help
