.. _ref_cubic:

cubic
=====

.. code-block::

    $ cubic --help
    Cubic is a lightweight command line manager for virtual machines. It has a
    simple, daemon-less and rootless design. All Cubic virtual machines run
    isolated in the user context. Cubic is built on top of QEMU, KVM and cloud-init.

    Show all supported images:
    $ cubic image ls

    Create a new virtual machine instance:
    $ cubic add mymachine --image ubuntu:noble

    List all virtual machine instances:
    $ cubic ls

    Start an instance:
    $ cubic start <instance name>

    Stop an instance:
    $ cubic stop <instance name>

    Open a shell in the instance:
    $ cubic ssh <machine name>

    Copy a file from the host to the instance:
    $ cubic scp <path/to/host/file> <machine>:<path/to/guest/file>

    Copy a file from the instance to the hots:
    $ cubic scp <machine>:<path/to/guest/file> <path/to/host/file>


    Usage: cubic [COMMAND]

    Commands:
      run      Create, start and open a shell in a new virtual machine instance
      add      Create a new virtual machine instance
      ls       List all virtual machine instances
      rm       Delete virtual machine instances
      info     Get information about an virtual machine instance
      console  Open the console of an virtual machine instance
      ssh      Connect to a virtual machine instance with SSH
      scp      Copy a file from or to a virtual machine instance with SCP
      start    Start virtual machine instances
      stop     Stop virtual machine instances
      restart  Restart virtual machine instances
      config   Modify virtual machine instance configuration
      rename   Rename a virtual machine instance
      clone    Clone a virtual machine instance
      image    Image subcommands
      help     Print this message or the help of the given subcommand(s)

    Options:
      -h, --help     Print help
      -V, --version  Print version
