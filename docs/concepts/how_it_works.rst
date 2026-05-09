How Cubic Works
===============

Cubic is intentionally simple. It has no background daemon, requires no root
privileges, and builds on three established tools: official cloud images,
cloud-init, and QEMU.

What Happens When You Run Cubic
--------------------------------

1. **Download** — Cubic fetches the official cloud image for the chosen
   distribution directly from the vendor's mirror. The image is
   checksum-verified and cached locally under ``~/.cache/cubic/images/`` so
   subsequent instances reuse it without re-downloading.

2. **Provision** — Cubic generates a unique Ed25519 SSH key for the instance
   and packages a cloud-init configuration into an ISO 9660 image. QEMU mounts
   the ISO as a virtual CD-ROM drive on first boot.

3. **First boot** — cloud-init reads the ISO, creates the default user,
   installs the instance's SSH key, and runs any configured tasks. On all
   subsequent boots cloud-init detects that provisioning is complete and does
   not run again.

4. **Run** — QEMU launches the VM directly from the CLI. No daemon is involved
   — Cubic is a process that starts QEMU and exits.

Cloud Images
------------

Cubic always uses official, unmodified images downloaded directly from the
vendor. Images are fetched from each distribution's own mirror (Ubuntu, Debian,
Fedora, Arch Linux, and others) and checksum-verified before use. The cached
image under ``~/.cache/cubic/images/`` is shared across instances of the same
distribution and version so it is only downloaded once.

cloud-init Provisioning
-----------------------

`cloud-init <https://cloud-init.io>`_ is the industry-standard tool for
initialising cloud VMs on first boot. Cubic uses it to configure each instance
automatically without user interaction.

For each new instance Cubic assembles a cloud-init configuration and packs it
into a small ISO 9660 image. QEMU presents the ISO as a virtual CD-ROM;
cloud-init finds it on first boot, applies the configuration — including
creating the default user and installing the instance's SSH public key — and
marks provisioning as done. The ISO is no longer consulted on subsequent boots.

QEMU and Hardware Acceleration
-------------------------------

`QEMU <https://www.qemu.org>`_ is an open-source machine emulator and
virtualiser. Cubic translates the instance configuration into a QEMU command
line and executes it directly. The accelerator is selected automatically based
on the host platform:

* **KVM** on Linux
* **HVF** (Hypervisor Framework) on macOS
* **Hyper-V** on Windows
* **TCG** (software emulation) as a fallback

Each instance stores its own disk image under
``~/.local/share/cubic/instances/<name>/``, keeping instances fully isolated
from each other and from the shared image cache.

Daemonless and Rootless Security
---------------------------------

Most VM managers require a background daemon running as root or a privileged
helper. Cubic does not. Every ``cubic`` invocation
is a short-lived CLI process that starts or stops a QEMU process owned by the
current user and then exits.

Because QEMU runs in the user's context with no elevated privileges, a
compromised guest VM cannot escalate to root on the host. The security boundary
is enforced by the operating system's normal user isolation.
