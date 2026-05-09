.. _http server:

Run an HTTP Server in a VM
==========================

This guide shows how to create a Debian virtual machine, install nginx, and
expose its HTTP port to the host so you can reach it from a browser or curl.

Create the Virtual Machine
--------------------------

Create a VM with port forwarding so that host port 8080 maps to guest port 80:

.. code-block::

    $ cubic create webserver --image debian:trixie --port 8080:80

Start the Virtual Machine
--------------------------

.. code-block::

    $ cubic start webserver

Install nginx
-------------

Connect to the VM over SSH and install nginx:

.. code-block::

    $ cubic ssh webserver
    $ sudo apt update && sudo apt install -y nginx

nginx starts automatically after installation and listens on port 80 inside
the VM.

Verify from the Host
--------------------

Open a new terminal on the host and send a request to port 8080:

.. code-block::

    $ curl http://localhost:8080
    <!DOCTYPE html>
    <html>
    <head>
    <title>Welcome to nginx!</title>
    ...

You can also open ``http://localhost:8080`` in a browser.

Show VM Settings
----------------

Use ``cubic show`` to inspect the VM configuration including all active port
forwarding rules:

.. code-block::

    $ cubic show webserver
    Arch:        amd64
    CPUs:        4
    Memory:      4.0 GiB
    Disk Used:   1.2 GiB
    Disk Total:  100.0 GiB
    User:        alice
    Isolated:    no
    SSH Port:    10022
    SSH:         ssh -p 10022 alice@localhost
    Forward:     127.0.0.1:8080:80/tcp

Modify VM Settings
------------------

Port forwarding rules and other VM settings can be changed at any time with
``cubic modify``. Changes take effect on the next restart.

Add a forwarding rule for HTTPS (port 443):

.. code-block::

    $ cubic modify webserver --port 8443:443

Remove the HTTP forwarding rule:

.. code-block::

    $ cubic modify webserver --rm-port 8080:80

Increase the number of CPUs and memory:

.. code-block::

    $ cubic modify webserver --cpus 2 --memory 2G
