# Publishing lino-transformer

This repository ships a reusable Rust library and CLI and is intended for
crates.io. Keep the initial version at `0.1.0` until its first publication.

## Initial publication

The first publication and trusted-publisher registration require a crates.io
account with a verified email. GitHub repository access alone does not grant
registry access. After merging the release workflow into `main`, run from a
checkout of `main` with [package-registry-manager](https://github.com/link-foundation/package-registry-manager):

```sh
package-registry-manager setup --repository . --registry crates-io --workflow release.yml --execute
```

The tool publishes the first version with a short-lived crate-scoped token,
registers the trusted publisher, then revokes and verifies revocation of that
token. Do not store the bootstrap token in GitHub. Keep any existing fallback
secrets; do not opt into token-secret deletion.

Verify the registered publisher has these exact values:

| Field | Value |
| --- | --- |
| Owner | `link-foundation` |
| Repository | `transformer` |
| Workflow filename | `release.yml` |
| Environment | none |

See the [crates.io trusted publishing announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
and the [authentication action](https://github.com/rust-lang/crates-io-auth-action).

## Later releases

1. Update `Cargo.toml` and `Cargo.lock` together in a PR and merge after CI passes.
2. Create a GitHub release targeting `main`, using the tag `v<VERSION>` matching
   `Cargo.toml` (for example, `v0.1.1`). Publishing the release runs the workflow.
   Alternatively, dispatch the Release workflow on `main` to publish its current
   unpublished version. Do not dispatch a version already on crates.io.
3. Confirm the Release workflow succeeds and the version appears on crates.io.

The workflow runs formatting, Clippy, all tests, release policy checks, and
`cargo publish --locked --dry-run` before publishing. The publish job verifies
that its commit is on `main` and that a release tag matches the package version.
Publications are serialized and are never canceled by a newer publication.

Authentication tries OIDC first. Its short-lived token is revoked by the auth
action's post step. If OIDC is unavailable, the publish step uses the optional
`CARGO_REGISTRY_TOKEN` secret, then `CARGO_TOKEN`. No secret is required once trust
is registered. A missing OIDC token and missing fallback secrets fail publishing
with an explicit bootstrap instruction; PR CI needs no registry credentials.

## Local checks

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
python3 -m pip install PyYAML==6.0.3
python3 experiments/test_release_workflow.py
cargo publish --locked --dry-run
```
