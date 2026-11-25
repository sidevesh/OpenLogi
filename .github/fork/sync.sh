#!/usr/bin/env bash
# Rebuilds this fork's `fork` branch: the newest upstream `master`, then `fork-ci` (the files in
# .github/fork and the fork's workflows), then every branch listed in .github/fork/branches.txt on
# `fork-ci`. Run by the fork-sync workflow, and by hand:
#
#   .github/fork/sync.sh            rebuild `fork` and push it
#   .github/fork/sync.sh --dry-run  rebuild it locally as `fork-new` and push nothing
#
# `master` is kept as a plain fast-forward mirror of upstream, so pull requests can start from it.
# A branch that no longer merges cleanly stops the run and leaves `fork` as it was.
set -euo pipefail

UPSTREAM_URL=${UPSTREAM_URL:-https://github.com/AprilNEA/OpenLogi.git}
DRY_RUN=false
[ "${1:-}" = "--dry-run" ] && DRY_RUN=true

git remote get-url upstream >/dev/null 2>&1 || git remote add upstream "$UPSTREAM_URL"
git fetch --quiet --no-tags upstream master
git fetch --quiet --no-tags origin '+refs/heads/*:refs/remotes/origin/*'

if ! git config user.email >/dev/null; then
  git config user.name "github-actions[bot]"
  git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
fi

branches=$(git show origin/fork-ci:.github/fork/branches.txt | sed 's/#.*//' | tr -s '[:space:]' ' ')

echo "Rebuilding fork from upstream/master $(git rev-parse --short upstream/master)"
git checkout --quiet --detach upstream/master
git branch --quiet -f fork-new HEAD
git checkout --quiet fork-new

merge() { # branch
  echo "  merging $1"
  if ! git merge --quiet --no-ff -m "Merge $1 into the fork" "origin/$1"; then
    git merge --abort || true
    echo "::error::origin/$1 no longer merges into upstream/master. Rebase it, or remove it from .github/fork/branches.txt on fork-ci." >&2
    exit 1
  fi
}

merge fork-ci
for branch in $branches; do
  git rev-parse --verify --quiet "origin/$branch" >/dev/null || { echo "::error::origin/$branch does not exist" >&2; exit 1; }
  merge "$branch"
done

changed=true
if git rev-parse --verify --quiet origin/fork >/dev/null &&
  [ "$(git rev-parse 'origin/fork^{tree}')" = "$(git rev-parse 'fork-new^{tree}')" ]; then
  changed=false
  echo "fork is already up to date."
fi

if [ "$DRY_RUN" = true ]; then
  echo "Dry run: left the result in the local branch fork-new (changed=$changed)."
else
  if [ "$changed" = true ]; then
    old=$(git rev-parse --verify --quiet origin/fork || true)
    git push --force-with-lease="fork:$old" origin fork-new:refs/heads/fork
  fi
  # A fast-forward only: a fork whose master has diverged is left for a person to look at.
  git push origin upstream/master:refs/heads/master || echo "::warning::master was not fast-forwarded to upstream."
fi

[ -n "${GITHUB_OUTPUT:-}" ] && echo "changed=$changed" >> "$GITHUB_OUTPUT"
exit 0
