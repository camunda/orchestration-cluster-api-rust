# [11.0.0-alpha.4](https://github.com/camunda/orchestration-cluster-api-rust/compare/v11.0.0-alpha.3...v11.0.0-alpha.4) (2026-11-22)


### Bug Fixes

* **worker:** keep polling with back-off after a failed job activation ([#201](https://github.com/camunda/orchestration-cluster-api-rust/issues/201)) ([1a3fcd2](https://github.com/camunda/orchestration-cluster-api-rust/commit/1a3fcd26d933cd82707ef4eabfa7fa983939eccc)), closes [#200](https://github.com/camunda/orchestration-cluster-api-rust/issues/200)

# [11.0.0-alpha.3](https://github.com/camunda/orchestration-cluster-api-rust/compare/v11.0.0-alpha.2...v11.0.0-alpha.3) (2026-11-21)


### Bug Fixes

* inject default tenant on suspend/resumeProcessDefinitions by id ([#189](https://github.com/camunda/orchestration-cluster-api-rust/issues/189)) ([ee6fb30](https://github.com/camunda/orchestration-cluster-api-rust/commit/ee6fb309fe6e170632274520a0dd0fa9ddcb2b37)), closes [#188](https://github.com/camunda/orchestration-cluster-api-rust/issues/188)

# [11.0.0-alpha.2](https://github.com/camunda/orchestration-cluster-api-rust/compare/v11.0.0-alpha.1...v11.0.0-alpha.2) (2026-11-20)


### Continuous Integration

* count only spec-form BREAKING CHANGE notes as breaking in releases ([#186](https://github.com/camunda/orchestration-cluster-api-rust/issues/186)) ([3265c22](https://github.com/camunda/orchestration-cluster-api-rust/commit/3265c22453e811cb44ed64b7f654fc395a2b7295)), closes [#184](https://github.com/camunda/orchestration-cluster-api-rust/issues/184)


### BREAKING CHANGES

* , upper case, with the colon.

A dry run on stable/10 now computes 10.0.1.

# [11.0.0-alpha.1](https://github.com/camunda/orchestration-cluster-api-rust/compare/v11.0.0-alpha.0...v11.0.0-alpha.1) (2026-11-20)


### Bug Fixes

* **deps:** update rust crate tokio to v1.53.3 [security] ([#95](https://github.com/camunda/orchestration-cluster-api-rust/issues/95)) ([2c14af9](https://github.com/camunda/orchestration-cluster-api-rust/commit/2c14af938709b663babd8aaf2a28aafa8293fa75))
* **gen:** regenerate for nullable legacy result fields and batch filter docs ([#67](https://github.com/camunda/orchestration-cluster-api-rust/issues/67)) ([0d55572](https://github.com/camunda/orchestration-cluster-api-rust/commit/0d555725b026fbe9d637bbc7940b0b8304eabced))
* handle ProcessInstanceCreationInstructionByVersionTag; regenerate client for spec main ([#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75)) ([864b666](https://github.com/camunda/orchestration-cluster-api-rust/commit/864b666a210467cf141bcabc1cf8b188afd0a729)), closes [#71](https://github.com/camunda/orchestration-cluster-api-rust/issues/71) [#71](https://github.com/camunda/orchestration-cluster-api-rust/issues/71)
* redact credentials from config and token debug output ([#140](https://github.com/camunda/orchestration-cluster-api-rust/issues/140)) ([3e97f2b](https://github.com/camunda/orchestration-cluster-api-rust/commit/3e97f2bddace7b6492dc919cfed0c7200dd7f4a9)), closes [hi#severity](https://github.com/hi/issues/severity)
* send cluster-admin operations to the server root, not under /v2 ([#78](https://github.com/camunda/orchestration-cluster-api-rust/issues/78)) ([88910db](https://github.com/camunda/orchestration-cluster-api-rust/commit/88910db1c12b7ac96dcbc12e4796007abc997fb6)), closes [#72](https://github.com/camunda/orchestration-cluster-api-rust/issues/72) [#76](https://github.com/camunda/orchestration-cluster-api-rust/issues/76)
* send truncateValues on search_variables and inject the default tenant into generated operations ([#77](https://github.com/camunda/orchestration-cluster-api-rust/issues/77)) ([bada1d4](https://github.com/camunda/orchestration-cluster-api-rust/commit/bada1d468e796ff84fb64e0f2c806975f3bdf98e)), closes [#74](https://github.com/camunda/orchestration-cluster-api-rust/issues/74)
* serialize date-time query params as RFC 3339 ([#76](https://github.com/camunda/orchestration-cluster-api-rust/issues/76)) ([3b74e4f](https://github.com/camunda/orchestration-cluster-api-rust/commit/3b74e4f6748e07012e2e5fa38e4d90fd2f716cd0)), closes [#73](https://github.com/camunda/orchestration-cluster-api-rust/issues/73)
* stop echoing the token-endpoint response body into auth errors ([#139](https://github.com/camunda/orchestration-cluster-api-rust/issues/139)) ([b4b2ab5](https://github.com/camunda/orchestration-cluster-api-rust/commit/b4b2ab50abf9902f5e0c77aca85673367c2d2e94))
* write the oauth token cache owner-only ([#142](https://github.com/camunda/orchestration-cluster-api-rust/issues/142)) ([dce90dc](https://github.com/camunda/orchestration-cluster-api-rust/commit/dce90dc36e3a472129abae64bc0d78eaacfa22ec)), closes [#138](https://github.com/camunda/orchestration-cluster-api-rust/issues/138)


### Continuous Integration

* cut releases with sdk-infra's shared semantic-release config ([#184](https://github.com/camunda/orchestration-cluster-api-rust/issues/184)) ([300ca6a](https://github.com/camunda/orchestration-cluster-api-rust/commit/300ca6a3b5176b7c0af74a8c3590002b937101b5)), closes [#180](https://github.com/camunda/orchestration-cluster-api-rust/issues/180)


### Features

* add example coverage for getClock ([#69](https://github.com/camunda/orchestration-cluster-api-rust/issues/69)) ([9db5a39](https://github.com/camunda/orchestration-cluster-api-rust/commit/9db5a39c12b174505a511e414e1b37b7cc6126a9))
* send job results from job workers ([#122](https://github.com/camunda/orchestration-cluster-api-rust/issues/122)) ([7b93645](https://github.com/camunda/orchestration-cluster-api-rust/commit/7b93645c11427741419c1d3d648631ceda38bbda))
* send the default tenant on tenant-scoped commands ([#160](https://github.com/camunda/orchestration-cluster-api-rust/issues/160)) ([a5e4d5b](https://github.com/camunda/orchestration-cluster-api-rust/commit/a5e4d5b5121c87d73965ec017d2e932c52ce7935)), closes [#159](https://github.com/camunda/orchestration-cluster-api-rust/issues/159)
* support clearing user task due and follow-up dates ([#125](https://github.com/camunda/orchestration-cluster-api-rust/issues/125)) ([80e6609](https://github.com/camunda/orchestration-cluster-api-rust/commit/80e660938b530c45eb4fccfd5299c9b0cc9fead7)), closes [camunda/camunda#68710](https://github.com/camunda/camunda/issues/68710)
* support lookupUserTask and typed expression tenantId ([#119](https://github.com/camunda/orchestration-cluster-api-rust/issues/119)) ([fa41e02](https://github.com/camunda/orchestration-cluster-api-rust/commit/fa41e0288182c72bb8b53fdc9a7fa9f11aef780d)), closes [camunda/camunda#67949](https://github.com/camunda/camunda/issues/67949)


### BREAKING CHANGES

* on stable/N stops the release instead of bumping the major.
