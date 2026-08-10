How Cubic Works
===============

Cubic is intentionally simple. It has no background daemon, requires no root
privileges, and builds on four established tools: official cloud images,
cloud-init, QEMU, and EDK2 UEFI firmware.

What Happens When You Run Cubic
--------------------------------

1. **Download** — Cubic fetches the official cloud image for the chosen
   distribution directly from the vendor's mirror. The image is
   checksum-verified and cached locally under ``~/.cache/cubic/images/`` so
   subsequent instances reuse it without re-downloading.

2. **Provision** — Cubic generates a unique Ed25519 SSH key for the instance
   and packages a cloud-init configuration into an ISO 9660 image. QEMU
   attaches the image as a small virtio disk.

3. **First boot** — cloud-init reads the seed disk, creates the default user,
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
into a small ISO 9660 image, stored as ``cloud-init.iso`` in the instance
directory. QEMU attaches that image as a virtio disk, and
cloud-init finds it on first boot. It then applies the configuration —
including creating the default user and installing the instance's SSH public
key — and marks provisioning as done. The seed disk is no longer consulted on
subsequent boots.

QEMU and Hardware Acceleration
-------------------------------

`QEMU <https://www.qemu.org>`_ is an open-source machine emulator and
virtualiser. Cubic translates the instance configuration into a QEMU command
line and executes it directly. The accelerator is selected automatically based
on the host platform:

* **KVM** on Linux
* **HVF** (Hypervisor Framework) on macOS
* **NVMM** on the BSDs
* **TCG** (software emulation) as a fallback

On Windows the **WHPX** accelerator is currently disabled and instances run on
TCG software emulation. WHPX requires the ``host`` CPU model, while the CPU
model Cubic passes is shared with the TCG path, and the mismatch makes the
guest hang at the OVMF firmware screen. See `issue #500
<https://github.com/cubic-vm/cubic/issues/500>`_.

Each instance keeps everything it owns in one directory under
``~/.local/share/cubic/machines/<name>/``. That directory holds the
configuration, the disk image, the SSH key, the cloud-init seed image and,
while the machine runs, the QEMU pid file. This keeps instances fully isolated
from each other and from the shared image cache.

UEFI Firmware (EDK2)
--------------------

Cloud images boot through UEFI, so Cubic boots every VM with `EDK2
<https://github.com/tianocore/edk2>`_ UEFI firmware. Cubic finds the firmware
automatically in the host's QEMU installation and picks the right build for the
guest architecture. When the firmware lives somewhere unusual you can point Cubic
at it with the ``CUBIC_QEMU_FW_AMD64`` and ``CUBIC_QEMU_FW_ARM64`` environment
variables. See :ref:`qemu detection` for details.

Daemonless and Rootless Security
---------------------------------

Most VM managers require a background daemon running as root or a privileged
helper. Cubic does not. Every ``cubic`` invocation
is a short-lived CLI process that starts or stops a QEMU process owned by the
current user and then exits.

Because QEMU runs in the user's context with no elevated privileges, a
compromised guest VM cannot escalate to root on the host. The security boundary
is enforced by the operating system's normal user isolation.
