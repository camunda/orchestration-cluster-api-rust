// semantic-release configuration: the shared Camunda SDK base config (branch model and
// commit analysis) from @camunda8/sdk-infra, plus the Rust-specific steps.
//
//   - main       -> <next>.0.0-alpha.N prereleases (channel alpha)
//   - stable/N   -> N.x.y releases
// `fix`/`perf`/`revert` -> patch, `feat` -> minor (see the base config).
//
// semantic-release only versions, tags and writes the changelog. The `vX.Y.Z` tag it
// pushes triggers .github/workflows/release.yml, which runs the test and API
// compatibility gates and publishes both crates to crates.io.
const base = require('@camunda8/sdk-infra/configs/release.config.base.cjs');

module.exports = {
  ...base,
  plugins: [
    ...base.plugins,
    [
      '@semantic-release/exec',
      {
        verifyReleaseCmd: 'bash scripts/verify-release-major.sh "${nextRelease.version}"',
        prepareCmd: 'bash scripts/set-release-version.sh "${nextRelease.version}"',
      },
    ],
    [
      '@semantic-release/git',
      {
        assets: [
          'CHANGELOG.md',
          'Cargo.toml',
          'Cargo.lock',
          'client/Cargo.toml',
          'client/README.md',
          'openapi-generator-config.yaml',
        ],
        // No [skip ci]: GitHub would then skip the tag push too, and the tag is what
        // triggers the crates.io publish. version.yml skips its own release commits.
        message: 'chore(release): ${nextRelease.version}\n\n${nextRelease.notes}',
      },
    ],
    [
      '@semantic-release/github',
      {
        successComment: false,
      },
    ],
  ],
};
