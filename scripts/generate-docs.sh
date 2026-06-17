#!/bin/bash
set -euo pipefail

version="$1"

CMDS="run create instances images ports show modify console ssh scp exec start \
    stop restart rename clone delete prune completions"

function generate_cmd_doc() {
    name="$1"
    cmd="$2"
    ref="$3"
    file="$4"

    echo -e ".. $ref:\n\n$name\n=====\n\n.. code-block::\n\n    \$ $name --help" > "$file"
    cargo run -- $cmd --help | sed "s/^/    /" >> "$file"
}

# Set version
sed "s/^release = .*$/release = '$version'/g" -i docs/conf.py

# Create Reference Doc Directory
mkdir -p docs/reference

# Generate cubic help
generate_cmd_doc "cubic" "" "_ref_cubic" "docs/reference/cubic.rst"

# Generate cubic subcommands help
for cmd in ${CMDS}; do
    generate_cmd_doc "cubic $cmd" "$cmd" "_ref_cubic_$cmd" "docs/reference/$cmd.rst"
done

# Generate reference/index.rst as the Command Reference landing page
cat > docs/reference/index.rst << 'REFEOF'
Command Reference
=================

.. toctree::
   :hidden:

   cubic
REFEOF

for cmd in ${CMDS}; do
    echo "   $cmd" >> docs/reference/index.rst
done

# Generate index.rst with a single root toctree
cat > docs/index.rst << 'EOF'
Cubic
=====

.. toctree::
   :caption: How-To
   :hidden:

   howto/install
   howto/getting_started
   howto/http_server
   howto/ssh_connect
   howto/environment_variables

.. toctree::
   :caption: Troubleshooting
   :hidden:

   troubleshooting/recover_disk

.. toctree::
   :caption: Internals
   :hidden:

   internals/how_it_works
   internals/security
   internals/qemu_detection

.. toctree::
   :caption: Command Reference
   :hidden:

EOF

for cmd in ${CMDS}; do
    echo "   reference/$cmd" >> docs/index.rst
done

cat >> docs/index.rst << 'EOF'

Cubic spins up Linux virtual machines on Linux, macOS and Windows with a single command.

Every distribution comes as a prebuilt cloud image and is ready to use within seconds, so you skip the long installation. Cubic keeps things simple and secure by acting as lightweight glue over proven tools. No privileged system service is required and every VM runs as your normal user, so you never need admin or root rights.
Cubic is built on top of ``QEMU``, ``EDK2``, official cloud images and ``cloud-init``.

Features
---------
* Simple command-line interface
* Supports Alma Linux, Arch Linux, Debian, Fedora, Gentoo, OpenSUSE, Rocky Linux and Ubuntu guest images
* Uses official, checksum-verified cloud images downloaded straight from each distribution
* Supports Linux, macOS and Windows hosts with amd64 and arm64 architecture
* Supports hardware acceleration with KVM (Linux), Hypervisor (macOS) and Hyper-V (Windows)
* Boots each VM with EDK2 UEFI firmware, discovered automatically per architecture
* No background privileged service and runs with standard user rights, no admin or root needed
* Written in Rust

Source Code
===========

The source code of Cubic is on `Github`_.

.. _Github: https://github.com/cubic-vm/cubic
EOF
