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

// A breaking-change note is `BREAKING CHANGE:` (or `BREAKING-CHANGE:`) as the
// Conventional Commits spec writes it: upper case, with the colon. The parser's default
// matches the keyword in any case with no colon, so a squash body line that merely
// begins "breaking change on ..." made the analyzer compute a major release (11.0.0
// on stable/10). Note that the base config's default (angular) preset does not parse
// `type!:` headers, so write a `BREAKING CHANGE:` footer to mark a break.
const parserOpts = {
  noteKeywords: ['BREAKING CHANGE', 'BREAKING-CHANGE'],
  notesPattern: (keywords) => new RegExp(`^[\\s|*]*(${keywords}):\\s*(.*)`),
};
const withParserOpts = (plugin) => {
  const [name, options = {}] = Array.isArray(plugin) ? plugin : [plugin];
  return [
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
  ].includes(name)
    ? [name, { ...options, parserOpts: { ...options.parserOpts, ...parserOpts } }]
    : plugin;
};

module.exports = {
  ...base,
  plugins: [
    ...base.plugins.map(withParserOpts),
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
