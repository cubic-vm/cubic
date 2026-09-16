.. _use a proxy:

Use a Proxy for Downloads
=========================

Cubic downloads distribution images over HTTPS. On a network that only reaches
the internet through a proxy the download fails or hangs. Set the proxy in the
environment and Cubic uses it.

Only the image download goes through the proxy. A VM instance has its own
network through QEMU and does not inherit these settings.

Set the Proxy
-------------

Export the proxy URL and create a VM instance:

.. code-block::

    $ export HTTPS_PROXY=http://proxy.example.com:3128
    $ cubic create --image ubuntu:resolute

On Windows use ``set`` instead:

.. code-block::

    > set HTTPS_PROXY=http://proxy.example.com:3128
    > cubic create --image ubuntu:resolute

Set it for a single command when you do not want it in your shell:

.. code-block::

    $ HTTPS_PROXY=http://proxy.example.com:3128 cubic create --image ubuntu:resolute

A Proxy That Needs a Login
--------------------------

Put the user name and the password in the URL:

.. code-block::

    $ export HTTPS_PROXY=http://user:password@proxy.example.com:3128

The password then sits in your shell history, so prefer a proxy without a login
where you can, or set the variable from a file your shell reads instead of
typing it.

Skip the Proxy for Some Hosts
-----------------------------

``NO_PROXY`` lists the hosts that are reached directly:

.. code-block::

    $ export NO_PROXY=localhost,127.0.0.1,mirror.internal.example.com

Use a single ``*`` to bypass the proxy for every host.

Check That It Works
-------------------

``cubic images`` downloads the image list, so it is the quickest test:

.. code-block::

    $ cubic images

An error or a hang points at a proxy that is not reachable or at a wrong port.
Add ``--verbose`` to any command to see more.

Platform Notes
--------------

On Windows the proxy of the Internet Settings is used when no variable is set,
so a system wide proxy already works.

On macOS the system proxy settings are not read. Set the variables even when
System Settings already has a proxy.

TLS Interception
----------------

Cubic ships its own root certificates and does not read ``SSL_CERT_FILE`` or
``SSL_CERT_DIR``. A proxy that terminates TLS with its own certificate authority
makes the download fail with a certificate error. Download the image on another
host and place it in the image cache instead, see :ref:`file locations`.

Related
-------

* :ref:`env vars` that Cubic reads
* :ref:`file locations` of the image cache
