# This fork's builds

This fork builds OpenLogi for macOS on GitHub-hosted runners and updates itself from the fork, so it
can run next to upstream. Nothing here touches upstream's files: it only adds the files under
`.github/fork` and the two `fork-*` workflows, which is why a merge from upstream never conflicts.

## Branches

| Branch | What it is |
| --- | --- |
| `master` | A fast-forward mirror of upstream's `master`. Start pull requests from it. |
| `feat/...` | Pull request branches. They never contain anything from here. |
| `fork-ci` | Upstream's `master` plus these files. |
| `fork` | The default branch, and what gets built: upstream's `master`, then `fork-ci`, then every branch in `branches.txt`. The sync rebuilds it, so never commit to it. |

## Workflows

- **Fork sync** (daily, or by hand) rebuilds `fork`, turns off upstream's workflows (they need paid
  runners a fork does not have) and starts a build when `fork` changed. Run it after pushing to a
  branch in `branches.txt`. A branch that no longer merges stops it and leaves `fork` unchanged.
- **Fork build** (by hand, or from the sync) builds the arm64 and x86_64 DMGs, publishes them as a
  GitHub Release, and publishes the update feed on GitHub Pages.

## What the build is

The same `cargo xtask macos package` as upstream's release. Versions are upstream's next patch
release as a beta, `0.9.1-beta.1007`, because the updater only offers a strictly newer version.
Upstream's release is higher than any of them, so a fork build never outranks it.

It differs from upstream's release in what needs upstream's Apple account:

- **Signature.** Signed with the fork's own certificate, not notarized: the first launch needs
  right-click, Open (or `xattr -dr com.apple.quarantine /Applications/OpenLogi.app`). The same
  certificate signs every build, so Accessibility and Input Monitoring grants survive updates.
- **Typing.** Upstream's agent types shortcuts through a virtual HID keyboard that an Apple-granted
  provisioning profile allows. Without it the agent uses `CGEventPost`, the fallback upstream ships.
- **Updates.** Upstream embeds Sparkle. The fork uses the built-in engine, which upstream keeps for
  older installs: it reads `channels/<channel>/latest.json` from the fork's Pages site and checks the
  DMG's SHA-256 and a minisign signature made with the fork's key.
- **LTO.** `thin` by default so the build fits a hosted runner. Choose `fat` when dispatching to
  build exactly as upstream does.

The bundle identifiers are upstream's, so a fork build replaces an installed upstream OpenLogi and
shares its settings.

## Setup (once)

Repository settings: Pages with source "GitHub Actions", Actions enabled, default branch `fork`.

| Name | Kind | What |
| --- | --- | --- |
| `FORK_SIGNING_P12` | secret | Base64 of the self-signed code signing identity (`.p12`). |
| `FORK_SIGNING_P12_PASSWORD` | secret | Its password. |
| `FORK_MINISIGN_SECRET_KEY` | secret | Base64 of the minisign secret key file. |
| `FORK_MINISIGN_PUBLIC_KEY` | variable | The minisign public key (the `RW...` line). |
| `FORK_SYNC_TOKEN` | secret, optional | A personal access token with `repo` and `workflow`. Needed only when upstream changes its own workflow files, which the built-in token may not push. |

Keep a copy of the two keys. Installed builds trust only the minisign key they were built with, and
macOS keeps permissions per signing certificate, so replacing either strands the installs.

## By hand

```sh
.github/fork/sync.sh --dry-run   # rebuild fork locally as fork-new
.github/fork/sync.sh             # and push it
gh workflow run fork-build.yml --ref fork
```
