.. _shell completions:

Install Shell Completions
=========================

Cubic can generate completion scripts for several shells. The following
commands install a script for the current user. Regenerate it after updating
Cubic so it matches the installed commands and options.

Bash
----

Generate the script in a user-owned directory:

.. code-block::

    $ mkdir -p ~/.local/share/cubic
    $ cubic completions bash > ~/.local/share/cubic/cubic.bash

Add the following line to ``~/.bashrc``:

.. code-block::

    source ~/.local/share/cubic/cubic.bash

Reload the configuration:

.. code-block::

    $ source ~/.bashrc

Zsh
---

Zsh discovers completion functions through its ``fpath``. Generate the script
with the ``_cubic`` function name that Zsh expects:

.. code-block::

    $ mkdir -p ~/.zfunc
    $ cubic completions zsh > ~/.zfunc/_cubic

Add these lines to ``~/.zshrc`` before any existing ``compinit`` call:

.. code-block::

    fpath=(~/.zfunc $fpath)
    autoload -Uz compinit
    compinit

If ``~/.zshrc`` already initializes completion, keep its existing ``compinit``
lines and add only the ``fpath`` line before them. Start a new shell to load the
function:

.. code-block::

    $ exec zsh

Fish
----

Fish loads completion files from its user configuration directory on demand:

.. code-block::

    $ mkdir -p ~/.config/fish/completions
    $ cubic completions fish > ~/.config/fish/completions/cubic.fish

No change to ``config.fish`` is required. Start a new shell if the completion
file is not picked up in the current session.

Verify the Setup
----------------

Type ``cubic``, add a space and press Tab. The shell should offer Cubic's
subcommands and options.
