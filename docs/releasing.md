# Releasing cargo-elicit

The Release workflow runs on every push to main and can also be started manually
from GitHub Actions. It runs the same formatting, Clippy, tests and Rust 1.86 MSRV
checks as CI, then runs `cargo publish --dry-run --locked`. Only after these
checks succeed does it publish a version absent from crates.io. Registry failures
other than HTTP 404 stop the workflow. Already published versions are skipped.

## Authentication

Prefer crates.io Trusted Publishing. Configure the publisher on crates.io for:

- GitHub owner: `proofs-rs`
- Repository: `cargo-elicit`
- Workflow filename: `release.yml`
- Environment: leave empty (this workflow does not use a GitHub environment)

For an initial publication, configure a pending/new-crate trusted publisher if
crates.io offers it for your account. Otherwise, create a crates.io API token
permitted to create/publish `cargo-elicit` and save it as the GitHub repository
Actions secret `CARGO_REGISTRY_TOKEN`. Do not commit credentials. The workflow
uses this secret when present and Trusted Publishing otherwise. After initial
publication, configure Trusted Publishing on the crate and remove the long-lived
secret to switch to OIDC.

## Next version

1. Update `package.version` in Cargo.toml and refresh Cargo.lock with Cargo.
2. Merge the change into main after review. The Release workflow publishes it.
3. Confirm that the Release workflow succeeds and the version appears on crates.io.

To retry after configuring credentials or a transient failure, rerun the failed
Release job or use Run workflow on main. Never change the source of an already
published version; crates.io versions are immutable.
