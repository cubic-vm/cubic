.. _security:

Security
========

Cubic is designed with three security goals in mind:

* **Protect the host from the guest.** A virtual machine may run untrusted or
  even malicious software, so the guest must never be able to take control of
  the host it runs on.

* **Protect your virtual machines from other users of the same computer.** One
  user should not be able to watch or control the virtual machines of another.

* **Protect your virtual machines from the network** the host is connected to.

To meet these goals Cubic keeps the amount of trusted code small. It runs
without a background service, it never asks for extra privileges, and it
authenticates every connection that reaches a virtual machine.

.. _no privileged service:

No Privileged System Service
----------------------------

Many virtual machine managers rely on a background service that runs with
system wide privileges. Cubic does not. Each time you run a cubic command it
starts a short lived process that launches or stops a QEMU process owned by your
own user account, and then it exits.

Because QEMU runs as your normal user with no extra rights, a guest that breaks
out of its virtual machine cannot gain system wide privileges on the host. Any
such escape stays inside your own user account, where the operating system keeps
it isolated from other users in the usual way.

.. _verified images:

Verified Distribution Images
----------------------------

Cubic only uses official images that come straight from each distribution.
Before an image is used, Cubic compares it against the checksum that the distro
publishes next to it, using either SHA256 or SHA512. If the value does not
match, Cubic rejects the download and stops with an ``InvalidChecksum`` error.

This guards against downloads that are corrupted or altered on the way to your
machine. The verified image is cached and shared by every virtual machine of the
same distribution and version, so it is fetched and checked once.
See :ref:`file locations` for the cache directory of each platform.


Bound to Localhost
------------------

Every service that Cubic opens for a virtual machine, including the SSH port,
the QEMU monitor, and the serial console, listens only on the loopback address
``127.0.0.1``. None of them are reachable from the local network or from the
internet.

This already protects your virtual machines from outside attackers. A remote
computer cannot connect to a service that listens only on loopback, so a virtual
machine started with Cubic is never exposed to the network just by running.

Loopback can still be reached by other people who are logged in to the same
computer. Binding to ``127.0.0.1`` therefore keeps virtual machines off the
network, but on its own it does not separate one local user from another. That
separation comes from the authentication on each connection, which the sections
below describe.

Binding to loopback says nothing about the other direction. A guest still
reaches the internet by default, so it can install packages, and ``--isolate``
cuts that off for workloads that must stay contained. :ref:`networking` explains
both directions.

Port Forwarding
---------------

Cubic can forward a port from the guest to the host so that a service
running inside the guest becomes reachable from the host. You describe a forward
as ``host_port:guest_port``, for example ``8000:80``, and you can put a host
address in front of it to choose where the port listens.

This choice matters. When you leave the address out, Cubic binds the forwarded
port to ``127.0.0.1``, so the service stays on loopback and keeps the protection
described above. When you set the address to ``0.0.0.0`` or to a public address
of the host, the service inside the guest becomes reachable from the local
network and possibly from the internet.

Only expose a port beyond loopback when you really mean to, and make sure the
service behind it is meant to be public and is properly secured. A port opened
on ``0.0.0.0`` gives an outside attacker a direct path into the guest.

.. _ssh access:

SSH Access
----------

For each new virtual machine Cubic creates a unique Ed25519 SSH key using the
operating system's secure random generator. The matching public key is placed
inside the guest during first boot, and the private key stays on the host
together with the virtual machine. You connect over the loopback port that
belongs to that machine.

The intended way to reach a guest is SSH key authentication, which depends on
holding the private key that Cubic stores alongside the virtual machine rather
than on a password that could be guessed or shared.

A new machine has no account password at all. The user account is created with a
locked password, so the SSH key is the only way in.

This shapes how you use the serial console. The console shows the login prompt
of the guest, and that prompt accepts only a password, so it stays closed until
you set one from an SSH session. Set a password while SSH works if you want the
console available as a way in later. See :ref:`console login` for the steps.

Cubic also remembers the host key of each guest. The first connection stores the
key the guest presents in the machine's ``instance.toml`` file, and every later
connection compares against it, the same idea as a known hosts file kept per
machine.

A key that does not match stops the connection. Cubic shows both fingerprints,
points out that this may be a malicious attempt to take over the connection, and
asks whether to trust the new key. Answering yes stores it, which is how you
carry on after you recreated the guest yourself.

.. _encrypted control channels:

Encrypted QEMU Control Channels
-------------------------------

The QEMU monitor and the serial console talk to Cubic over loopback connections
that are protected with mutual TLS. Both ends have to present a valid
certificate before any data flows, and the traffic itself is encrypted.

The first time a virtual machine starts, Cubic creates a small certificate
authority just for that machine and uses it to issue a server certificate for
QEMU and a client certificate for Cubic. These are stored with the machine and
reused on later starts. Cubic uses rustls to load its own certificate and to
check the one that QEMU presents.

This is what stops another user on the same computer from driving your virtual
machine's monitor or console. Even though they can reach the loopback port, they
cannot complete the secure handshake without the client certificate that belongs
to your machine.

Keeping Instances Apart
-----------------------

Everything that belongs to a virtual machine, including its SSH private key, its
TLS certificates, its disk images and its cloud-init seed image, is stored under
your own data directory, which :ref:`file locations` names for each platform.
Reaching a running machine means holding the SSH key or the TLS client
certificate that is kept inside it.

What Cubic Does Not Protect Against
-----------------------------------

The goals above draw a line around what Cubic defends. These things sit outside
it, and knowing that is part of using it safely.

**Anything running as you.** The protection is between user accounts, not inside
one. Any process of your own user can read the SSH private key, the TLS client
certificate and the disk image of every virtual machine you own, and can
therefore reach every guest you can reach.

**A port you publish yourself.** A forward bound to ``0.0.0.0`` or to a public
address of the host puts the service inside the guest on the network, with
whatever authentication that service brings and no more.

**Data at rest.** Disk images, snapshots, SSH keys and certificates are stored
as plain files. Anyone who can read your data directory, including a backup that
leaves the machine, gets the contents of every VM instance. Use encryption on
the host if that matters.

**A mirror that serves a matching checksum.** Verification proves the image
matches the checksum the distribution published next to it. It does not prove
that either file came from the distribution. Verifying a distribution signature
over the checksum file would, and would remove the need to trust the connection
to the mirror at all. Cubic does not do that yet.

**A network cut with ``--isolate``.** Isolation stops guest traffic. It is a
containment feature rather than a boundary against a guest that has already
broken out of QEMU.

Security Issues
---------------

If you believe you have found a security problem in Cubic, please report it
privately as described in the ``SECURITY.md`` file.
