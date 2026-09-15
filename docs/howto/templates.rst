.. _templates:

Create from a Template
======================

A template is a TOML file with default values for a new VM instance, so the same
setup can be created again and again without retyping a long command line.
:ref:`first template` walks through a small one step by step.

Write a Template
----------------

A template is a plain TOML file. Only ``version`` is mandatory, every other
field falls back to the Cubic default:

.. code-block::

    version = 1

    image = "debian"
    cpus = 4
    memory = "4G"

    ports = ["8000:80"]

    run = ["apt install -y vim"]

The fields use the same form as the command line, for example ``4G`` for memory
and ``8000:80`` for a forwarded port. The ``run`` list holds shell commands the
guest runs once on its first boot. :ref:`template file` lists every field and
:ref:`values` the units and defaults behind them.

Create a VM Instance from a Template
------------------------------------

Pass the template to ``cubic create`` or ``cubic run`` with ``--template``:

.. code-block::

    $ cubic create my-instance --template ./template.toml

A template carries no instance name, so one file serves as many VM instances as
you like.

Override Template Values
------------------------

Every command line argument overrides the matching value in the template, which
lets you reuse one file and change a single detail, such as the image or the
number of vCPUs:

.. code-block::

    $ cubic create my-instance --template ./template.toml --image ubuntu --cpus 8

A template that sets ``isolate = true`` keeps the VM instance off the network.
``--no-isolate`` gives a single VM instance network access anyway:

.. code-block::

    $ cubic create my-instance --template ./template.toml --no-isolate

Related
-------

* :ref:`template file` lists every field of a template
* :ref:`resources` after the VM instance was created
* :ref:`port forward` to reach a service from the host
