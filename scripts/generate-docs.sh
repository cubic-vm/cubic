#!/bin/bash

CMDS="run create instances images ports show modify console ssh scp start stop \
		restart rename clone delete prune completions"

function generate_cmd_doc() {
    name="$1"
    cmd="$2"
    ref="$3"
    file="$4"

    echo -e ".. $ref:\n\n$name\n=====\n\n.. code-block::\n\n    \$ $name --help" > "$file"
    cargo run -- $cmd --help | sed "s/^/    /" >> "$file"
}

# Set version
sed "s/^release = .*$/release = '$(git describe --tags)'/g" -i docs/conf.py

# Create Reference Doc Directory
mkdir -p docs/reference

# Generate cubic help
generate_cmd_doc "cubic" "" "_ref_cubic" "docs/reference/cubic.rst"

# Generate cubic subcommands help
for cmd in ${CMDS}; do
    generate_cmd_doc "cubic $cmd" "$cmd" "_ref_cubic_$cmd" "docs/reference/$cmd.rst"
done
