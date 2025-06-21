Cubic
=====

.. toctree::
   :maxdepth: 2
   :caption: Contents:

Cubic is a lightweight command-line manager for virtual machines with focus on simplicity and security.

It has a simple, daemon-less and rootless design. All Cubic virtual machines run isolated in the user context.
Cubic is built on top of ``QEMU``, ``KVM`` and ``cloud-init``.

Features
---------
* Simple command-line interface
* Supports ArchLinux, Debian, Fedora, OpenSUSE and Ubuntu guest images
* Supports Linux, macOS and Window hosts with amd64 and arm64 architecture
* Supports hardware acceleration with KVM (Linux), Hypervisoer (macOS) and Hyper-V (Windows)
* Daemon-less design which does not require root rights
* Written in Rust

Source Code
===========

The source code of Cubic is on `Github`_.

.. _Github: https://github.com/cubic-vm/cubic


Getting Started
===============
* :ref:`Install Cubic`
* :ref:`create vm`

Reference
=========
* :ref:`ref_cubic`

  * :ref:`ref_cubic_run`
  * :ref:`ref_cubic_ls`
  * :ref:`ref_cubic_add`
  * :ref:`ref_cubic_rm`
  * :ref:`ref_cubic_clone`
  * :ref:`ref_cubic_rename`
  * :ref:`ref_cubic_info`
  * :ref:`ref_cubic_config`
  * :ref:`ref_cubic_console`
  * :ref:`ref_cubic_ssh`
  * :ref:`ref_cubic_scp`
  * :ref:`ref_cubic_start`
  * :ref:`ref_cubic_stop`
  * :ref:`ref_cubic_restart`
  * :ref:`ref_cubic_image`
  * :ref:`ref_cubic_net`
