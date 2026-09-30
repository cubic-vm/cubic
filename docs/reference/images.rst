.. _image names:

Image Names
===========

An image name selects the distribution, the release and the architecture of a
new VM instance. Pass it to ``cubic create`` and ``cubic run`` with ``--image``,
or set ``image`` in a :ref:`template file`. ``cubic images`` lists everything
that is available.

Format
------

.. code-block::

    distro[:name][:arch]

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Part
     - Description
   * - ``distro``
     - Name of the distribution, such as ``debian``. Mandatory.
   * - ``name``
     - Release, given as a version, a codename or a tag. Defaults to ``stable``.
   * - ``arch``
     - ``amd64`` or ``arm64``. Defaults to the architecture of the host.

Examples:

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Name
     - Meaning
   * - ``ubuntu``
     - Newest long term release of Ubuntu for the host architecture.
   * - ``ubuntu:26.04``
     - Ubuntu 26.04 by version.
   * - ``ubuntu:resolute``
     - The same release by codename.
   * - ``ubuntu:latest``
     - Newest Ubuntu release, long term or not.
   * - ``debian:trixie:arm64``
     - Debian trixie as an arm64 guest.
   * - ``archlinux:rolling``
     - Arch Linux, which has one rolling image instead of releases.

A guest only runs with hardware acceleration when its architecture matches the
host. Cubic starts a guest of the other architecture in software emulation,
which is much slower.

Tags
----

Every distribution has the tags ``stable`` and ``latest``. Cubic derives them
from the release list each time it reads the image list, separately for each
distribution and each architecture, so a tag never goes stale and is never
cached.

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Tag
     - Meaning
   * - ``stable``
     - Newest long term release. Distributions without long term releases use
       their newest release. This is what a bare distribution name resolves to.
   * - ``latest``
     - Newest release, whether it is long term or not.
   * - ``rolling``
     - Version of a distribution that has no releases at all. It carries both
       ``stable`` and ``latest``.

Ubuntu is the only distribution with a long term rule of its own. A release is
long term when it was published in April of an even year, so ``ubuntu`` gives
you the last LTS while ``ubuntu:latest`` gives you the newest interim release.

Distributions
-------------

.. list-table::
   :header-rows: 1
   :widths: 18 12 30 40

   * - Name
     - Releases
     - Source
     - Checksum
   * - ``almalinux``
     - versions
     - ``raw.repo.almalinux.org``
     - SHA256
   * - ``alpine``
     - versions
     - ``dl-cdn.alpinelinux.org``
     - SHA512
   * - ``archlinux``
     - rolling
     - ``geo.mirror.pkgbuild.com``
     - SHA256
   * - ``centos``
     - versions
     - ``cloud.centos.org``
     - SHA256
   * - ``debian``
     - versions and codenames
     - ``cloud.debian.org``
     - SHA512
   * - ``fedora``
     - versions
     - ``dl.fedoraproject.org``
     - SHA256
   * - ``gentoo``
     - rolling
     - ``distfiles.gentoo.org``
     - SHA256
   * - ``opensuse``
     - versions
     - ``download.opensuse.org``
     - SHA256
   * - ``rockylinux``
     - versions
     - ``dl.rockylinux.org``
     - SHA256
   * - ``ubuntu``
     - versions and codenames
     - ``cloud-images.ubuntu.com``
     - SHA256

Cubic downloads the official image from the mirror of the distribution and
verifies it against the checksum the distribution publishes next to it. See
:ref:`security` for what that protects against.

Cached Images
-------------

A downloaded image is shared by every VM instance that uses it. ``cubic images``
shows which ones are cached. :ref:`file locations` names the cache directory and
the file name of each image.

Related
-------

* :ref:`create vm` from an image
* :ref:`template file` to set the image of a template
* :ref:`file locations` of the image cache
