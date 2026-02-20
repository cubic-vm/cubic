.. _ref_cubic_ssh:

cubic ssh
=====

.. code-block::

    $ cubic ssh --help
    Connect to a virtual machine instance with SSH
    
    Usage: cubic ssh [OPTIONS] <TARGET> [CMD]
    
    Arguments:
      <TARGET>  Target instance (format: [username@]instance, e.g. 'cubic@mymachine' or 'mymachine')
      [CMD]     Execute a command in the virtual machine
    
    Options:
      -v, --verbose  Increase logging output
      -q, --quiet    Reduce logging output
      -h, --help     Print help
