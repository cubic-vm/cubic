.. _first template:

Use a Template
==============

This tutorial writes a small template file and creates two identical VM
instances from it. It follows on from :ref:`create vm`.

Write the Template
------------------

A template is a plain TOML file. Save this one as ``template.toml``:

.. code-block::

    version = 1

    image = "ubuntu"
    cpus = 2
    memory = "2G"

    run = [
      "echo built by cubic > /etc/cubic-tutorial"
    ]

Only ``version`` is mandatory. The other fields use the same form as the
command line, and the ``run`` list holds commands the guest executes as root on
its first boot, which is why the example can write to ``/etc``.

Create Two VM Instances
-----------------------

.. code-block::

    $ cubic create web-1 --template ./template.toml
    $ cubic create web-2 --template ./template.toml

Both commands stay quiet. The template carries no instance name, so you can
create as many VM instances from it as you like.

See What the Template Did
-------------------------

.. code-block::

    $ cubic instances
    Name    Arch    vCPUs   Memory      Disk   Running
    web-1   amd64       2   2048 M   0/100 G        no
    web-2   amd64       2   2048 M   0/100 G        no

Both VM instances have two vCPUs and two GiB of memory because the template said
so. Without a template Cubic would have picked those values from the resources
of your host.

Check the First Boot Command
----------------------------

.. code-block::

    $ cubic ssh web-1
    alice@web-1:~$ cloud-init status --wait
    status: done
    alice@web-1:~$ cat /etc/cubic-tutorial
    built by cubic
    alice@web-1:~$ exit
    logout

``cubic ssh`` returns as soon as the guest accepts connections, which can be
before the ``run`` commands are finished. ``cloud-init status --wait`` waits for
them, so the file is there when you look.

Next Steps
----------

Delete the two VM instances when you are done:

.. code-block::

    $ cubic delete web-1 web-2

A command line argument beats the template, which lets you reuse one file and
change a single detail:

.. code-block::

    $ cubic create web-3 --template ./template.toml --cpus 4

The :ref:`template guide <templates>` lists every field a template can set.
