Security
========

Cubic is designed with three security goals in mind:

 - The first goal is to protect the host from the guest. A virtual machine may run
   untrusted or even malicious software, so the guest must never be able to take
   control of the host it runs on.

 - The second goal is to protect each user's virtual machines from other people who
   use the same computer. One user should not be able to watch or control the
   virtual machines that belong to another user.

 - The third goal is to protect the user's virtual machines from attacks coming
   over any network the host is connected to.

To meet these goals Cubic keeps the amount of trusted code small. It runs
without a background service, it never asks for root rights, and it
authenticates every connection that reaches a virtual machine.

Daemonless and Rootless Design
------------------------------

Many virtual machine managers rely on a background service that runs as root.
Cubic does not. Each time you run a cubic command it starts a short lived
process that launches or stops a QEMU process owned by your own user account,
and then it exits.

Because QEMU runs as your normal user with no extra rights, a guest that breaks
out of its virtual machine cannot gain root access on the host. Any such escape
stays inside your own user account, where the operating system keeps it isolated
from other users in the usual way.

Verified Cloud Images
---------------------

Cubic only uses official images that come straight from each distribution.
Before an image is used, Cubic compares it against the checksum that the vendor
publishes next to it, using either SHA256 or SHA512. If the value does not
match, Cubic rejects the download and stops with an ``InvalidChecksum`` error.

This guards against downloads that are corrupted or altered on the way to your
machine. The verified image is cached under ``~/.cache/cubic/images/`` and
shared by every virtual machine that uses the same distribution and version, so
it only needs to be fetched and checked once.

In the future Cubic could go one step further and verify a vendor signature over
the checksum file. That would remove the need to trust the connection to the
mirror at all.

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

By default a guest still has outbound access so it can install packages and
reach the internet. Starting a machine with ``--isolate`` cuts off this outbound
access for workloads that should stay fully contained.

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

For convenience while a machine is still being provisioned, new instances also
have a default account password enabled for the moment. The SSH port is bound to
loopback, so this is only reachable from the same host and never from the
network, but moving to key-only authentication and dropping the default password
is a planned improvement.

One area that can still be improved is host key verification. Today the host key
that the guest presents is accepted as it is, which is reasonable while
connections only ever go to ``127.0.0.1``. Remembering each machine's host key on
the first connection would let Cubic notice if it ever reached the wrong
endpoint.

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
TLS certificates, and its disk images, is stored under your own data directory,
for example ``~/.local/share/cubic/instances/<name>/`` on Linux. Reaching a
running machine therefore comes down to two things: the file permissions on that
directory, and holding the SSH key or the TLS client certificate that is kept
inside it.

Cubic currently relies on the file permissions that the operating system applies
by default. Restricting the private keys and certificates so that only their
owner can read them, and the instance directories so that only their owner can
enter them, would make this protection independent of how each user's system is
configured.

Security Issues
---------------

If you believe you have found a security problem in Cubic, please report it
privately as described in the ``SECURITY.md`` file.
