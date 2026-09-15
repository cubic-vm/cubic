.. _port forward:

Forward a Port
==============

A forwarded port makes a service inside a VM instance reachable from the host.
This guide forwards host port 8080 to guest port 80 and runs nginx as the
service behind it, so you can reach it from a browser or curl.

Create the VM Instance
----------------------

Create a VM instance with port forwarding, so that host port 8080 maps to guest
port 80:

.. code-block::

    $ cubic create webserver --image debian:trixie --port 8080:80

Start the VM Instance
----------------------

.. code-block::

    $ cubic start webserver

Install nginx
-------------

Connect to the VM instance over SSH and install nginx:

.. code-block::

    $ cubic ssh webserver
    $ sudo apt update && sudo apt install -y nginx

nginx starts automatically after installation and listens on port 80 inside
the VM instance.

Verify from the Host
--------------------

Open a new terminal on the host and send a request to port 8080:

.. code-block::

    $ curl http://localhost:8080
    <!DOCTYPE html>
    <html>
    <head>
    <title>Welcome to nginx!</title>
    [...]

You can also open ``http://localhost:8080`` in a browser.

Show the Settings
-----------------

Use ``cubic show --all`` to inspect the full configuration of a VM instance,
including every port forwarding rule:

.. code-block::

    $ cubic show --all webserver
    Running:      yes
    Arch:         amd64
    vCPUs:        4
    Memory:       4096 M
    Disk Used:    1229 M
    Disk Total:   100 G
    User:         alice
    Isolated:     no
    SSH Port:     10022
    Monitor Port: 10023
    Console Port: 10024
    Forward:      127.0.0.1:8080:80/tcp
    PID:          12345
    Disk Image:   ~/.local/share/cubic/machines/webserver/machine.img
    Config:       ~/.local/share/cubic/machines/webserver/instance.toml
    SSH Key:      ~/.local/share/cubic/machines/webserver/ssh_client_key
    SSH:          ssh -i .../webserver/ssh_client_key -p 10022 alice@localhost

The paths belong to a Linux host. :ref:`file locations` names the directory of
each platform.

Change a Forward
----------------

``cubic modify`` adds and removes forwards at any time. On a running VM instance
the host port opens and closes right away, so no restart is needed.

Add a forward for HTTPS:

.. code-block::

    $ cubic modify webserver --port 8443:443

Remove the HTTP forward:

.. code-block::

    $ cubic modify webserver --rm-port 8080:80

When the VM instance is running the command prints a note that changes may
require a restart. It applies to the other settings of a VM instance, such as
vCPUs and memory, not to the forwards.

List the Forwards
-----------------

``cubic ports`` lists the forwards of every VM instance and whether the host
port is in use:

.. code-block::

    $ cubic ports
    Instance    Host              Guest   Protocol   In Use
    webserver   127.0.0.1:8080    :80     /tcp       yes

Related
-------

* :ref:`resources` of the VM instance
* :ref:`ssh connect` instead of ``cubic ssh``
