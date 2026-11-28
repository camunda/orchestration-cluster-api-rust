## [10.0.5](https://github.com/camunda/orchestration-cluster-api-rust/compare/v10.0.4...v10.0.5) (2026-11-28)


### Bug Fixes

* retry job commands only when not applied (429/503) (backport [#219](https://github.com/camunda/orchestration-cluster-api-rust/issues/219) to stable/10) ([#220](https://github.com/camunda/orchestration-cluster-api-rust/issues/220)) ([85bf7b4](https://github.com/camunda/orchestration-cluster-api-rust/commit/85bf7b446450ab1d7d7cf57260d8756553d6a431))

## [10.0.4](https://github.com/camunda/orchestration-cluster-api-rust/compare/v10.0.3...v10.0.4) (2026-11-24)


### Bug Fixes

* **backpressure:** recover BALANCED gate under a steady trickle of 503s (backport [#204](https://github.com/camunda/orchestration-cluster-api-rust/issues/204) to stable/10) ([#209](https://github.com/camunda/orchestration-cluster-api-rust/issues/209)) ([2252885](https://github.com/camunda/orchestration-cluster-api-rust/commit/2252885af4cd0d687cceaf28fb30edad0b85f444)), closes [#208](https://github.com/camunda/orchestration-cluster-api-rust/issues/208)

## [10.0.3](https://github.com/camunda/orchestration-cluster-api-rust/compare/v10.0.2...v10.0.3) (2026-11-22)


### Bug Fixes

* **worker:** keep polling with back-off after a failed job activation (backport [#201](https://github.com/camunda/orchestration-cluster-api-rust/issues/201) to stable/10) ([#202](https://github.com/camunda/orchestration-cluster-api-rust/issues/202)) ([bd5ba09](https://github.com/camunda/orchestration-cluster-api-rust/commit/bd5ba09aaf426868df5fbf0220b46ada9e4de5dc)), closes [#145](https://github.com/camunda/orchestration-cluster-api-rust/issues/145)

## [10.0.2](https://github.com/camunda/orchestration-cluster-api-rust/compare/v10.0.1...v10.0.2) (2026-11-21)


### Bug Fixes

* send the default tenant on tenant-scoped commands ([#193](https://github.com/camunda/orchestration-cluster-api-rust/issues/193)) ([182592d](https://github.com/camunda/orchestration-cluster-api-rust/commit/182592d4f8de905ad818d5fc390d30be49d7db72)), closes [#160](https://github.com/camunda/orchestration-cluster-api-rust/issues/160) [#192](https://github.com/camunda/orchestration-cluster-api-rust/issues/192)

## [10.0.1](https://github.com/camunda/orchestration-cluster-api-rust/compare/v10.0.0...v10.0.1) (2026-11-20)


### Bug Fixes

* backport conformance fixes to stable/10 ([#80](https://github.com/camunda/orchestration-cluster-api-rust/issues/80)) ([c1e5606](https://github.com/camunda/orchestration-cluster-api-rust/commit/c1e5606a9030608804ee6f2a91249a3c76dee6bf)), closes [#73](https://github.com/camunda/orchestration-cluster-api-rust/issues/73) [#72](https://github.com/camunda/orchestration-cluster-api-rust/issues/72) [#79](https://github.com/camunda/orchestration-cluster-api-rust/issues/79) [#74](https://github.com/camunda/orchestration-cluster-api-rust/issues/74)
* backport hook_17 scalar-allOf-wrappers so exact-match and advanced enum filters work ([#163](https://github.com/camunda/orchestration-cluster-api-rust/issues/163)) ([35ac7f6](https://github.com/camunda/orchestration-cluster-api-rust/commit/35ac7f65615cfb5e62ac6b749c8ddd95f60bf9a7)), closes [#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75) [#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75) [#81](https://github.com/camunda/orchestration-cluster-api-rust/issues/81) [#162](https://github.com/camunda/orchestration-cluster-api-rust/issues/162)
* **deps:** update rust crate tokio to v1.53.3 [security] ([#97](https://github.com/camunda/orchestration-cluster-api-rust/issues/97)) ([d4484a1](https://github.com/camunda/orchestration-cluster-api-rust/commit/d4484a1e00687f9d649833899a373b6d48fa7748))
* redact credentials from config and token debug output ([#140](https://github.com/camunda/orchestration-cluster-api-rust/issues/140)) ([#175](https://github.com/camunda/orchestration-cluster-api-rust/issues/175)) ([5a45e37](https://github.com/camunda/orchestration-cluster-api-rust/commit/5a45e371ec6f3f0f57115b492ece3dabb67d06f5)), closes [hi#severity](https://github.com/hi/issues/severity)
* regenerate stable/10 for upstream stable/8.10 spec drift ([#81](https://github.com/camunda/orchestration-cluster-api-rust/issues/81)) ([9f03061](https://github.com/camunda/orchestration-cluster-api-rust/commit/9f03061208e2e4680494323593589090a59a03d9)), closes [#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75) [#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75)
* send the OAuth token as the bearer credential ([#178](https://github.com/camunda/orchestration-cluster-api-rust/issues/178)) ([6c79ce0](https://github.com/camunda/orchestration-cluster-api-rust/commit/6c79ce0a64260c3fc2ccd04d1a3c8491c0b08f42)), closes [#75](https://github.com/camunda/orchestration-cluster-api-rust/issues/75)
* stop echoing the token-endpoint response body into auth errors ([#176](https://github.com/camunda/orchestration-cluster-api-rust/issues/176)) ([35117e9](https://github.com/camunda/orchestration-cluster-api-rust/commit/35117e9ddf94b9daa3c2204e3f5f4b65847f2fb9)), closes [#139](https://github.com/camunda/orchestration-cluster-api-rust/issues/139)
* write the oauth token cache owner-only, with secret-leak regression tests ([#177](https://github.com/camunda/orchestration-cluster-api-rust/issues/177)) ([8ec0a0e](https://github.com/camunda/orchestration-cluster-api-rust/commit/8ec0a0ea8d70cc641494d6c5fbcd7180087a0423)), closes [#142](https://github.com/camunda/orchestration-cluster-api-rust/issues/142) [#146](https://github.com/camunda/orchestration-cluster-api-rust/issues/146) [#142](https://github.com/camunda/orchestration-cluster-api-rust/issues/142)
