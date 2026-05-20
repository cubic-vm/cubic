#!/usr/bin/env bash
set -euo pipefail

# Resolve the pull request range from CI arguments or local environment.
base_sha="${1:-${BASE_SHA:-}}"
head_sha="${2:-${HEAD_SHA:-HEAD}}"

if [[ -z "$base_sha" ]]; then
    echo "::error::Base commit SHA is required."
    exit 2
fi

# Make sure both endpoints are present before walking the PR commits.
for revision in "$base_sha" "$head_sha"; do
    if ! git cat-file -e "${revision}^{commit}" 2>/dev/null; then
        echo "::error::Commit '${revision}' is not available in the checkout."
        exit 2
    fi
done

# Match the commit type prefixes documented in CONTRIBUTING.md.
commit_subject='^(feat|fix|refactor|docs|chore|revert): .+'

has_error=0

while read -r commit_hash; do
    short_sha="$(git rev-parse --short "$commit_hash")"
    message="$(git show -s --format=%B "$commit_hash" | tr -d '\r')"
    subject="$(printf '%s\n' "$message" | sed -n '1p')"
    last_line="$(printf '%s\n' "$message" | sed '/^[[:space:]]*$/d' | tail -n 1)"
    line_count="$(wc -l <<< "$message")"

    if ! printf '%s\n' "$subject" | grep -Eq "$commit_subject" ||
        [[ "$line_count" -lt 4 ]] ||
        [[ "$last_line" != Signed-off-by:* ]]; then
        echo "::error::${short_sha} is not formatted according to the CONTRIBUTING.md."
        has_error=1
    fi
done < <(git log "${base_sha}..${head_sha}" --pretty=tformat:"%H")

exit "$has_error"
