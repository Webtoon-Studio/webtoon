## What's Changed in 0.9.0
* build(deps): update scraper requirement from 0.23 to 0.24 by @dependabot[bot] in [#128](https://github.com/Webtoon-Studio/webtoon/pull/128)
* refactor(webtoons): remove `Option` and make `default` by @RoloEdits in [#127](https://github.com/Webtoon-Studio/webtoon/pull/127)
* refactor(webtoons): de-nest more project structure by @RoloEdits in [#126](https://github.com/Webtoon-Studio/webtoon/pull/126)
* refactor(webtoons): flatten episode file structure by @RoloEdits in [#125](https://github.com/Webtoon-Studio/webtoon/pull/125)
* fix(naver)!: change to new comment api by @RoloEdits in [#122](https://github.com/Webtoon-Studio/webtoon/pull/122)
* fix(webtoons): properly support readers for audio episodes by @RoloEdits in [#124](https://github.com/Webtoon-Studio/webtoon/pull/124)
* fix(webtoons): be less specific with episode title selector by @RoloEdits in [#123](https://github.com/Webtoon-Studio/webtoon/pull/123)
* refactor(webtoons): always return episodes by same order by @RoloEdits in [#121](https://github.com/Webtoon-Studio/webtoon/pull/121)
* fix(webtoons): change scope name from `superhero` to `super-hero` by @RoloEdits in [#120](https://github.com/Webtoon-Studio/webtoon/pull/120)
* fix(webtoons): change scope name from `sci-fi` to `sf` by @RoloEdits in [#119](https://github.com/Webtoon-Studio/webtoon/pull/119)
* fix(originals): completed -> complete by @RoloEdits in [#117](https://github.com/Webtoon-Studio/webtoon/pull/117)
* chore: clippy 1.88 lints by @RoloEdits in [#115](https://github.com/Webtoon-Studio/webtoon/pull/115)
* fix(webtoons): rating is no longer a data point so remove by @RoloEdits in [#114](https://github.com/Webtoon-Studio/webtoon/pull/114)
* ci: correct `faste` to `fast` by @RoloEdits in [#112](https://github.com/Webtoon-Studio/webtoon/pull/112)
* doc: change default `user-agent` for `Client` by @RoloEdits in [#111](https://github.com/Webtoon-Studio/webtoon/pull/111)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.8.0...0.9.0

## What's Changed in 0.8.0
* chore: bump version from 0.7.0 to 0.8.0 by @RoloEdits in [#97](https://github.com/Webtoon-Studio/webtoon/pull/97)
* doc: correct the default `user-agent` for clients by @RoloEdits in [#110](https://github.com/Webtoon-Studio/webtoon/pull/110)
* fix(webtoons): accommodate new UI changes for originals by @RoloEdits in [#109](https://github.com/Webtoon-Studio/webtoon/pull/109)
* chore: add `panels` to `.gitignore` by @RoloEdits in [#108](https://github.com/Webtoon-Studio/webtoon/pull/108)
* test(naver): add `episode` test by @RoloEdits in [#107](https://github.com/Webtoon-Studio/webtoon/pull/107)
* chore: add `test` task to `Justfile` by @RoloEdits in [#106](https://github.com/Webtoon-Studio/webtoon/pull/106)
* refactor(webtoons): remove `thumbnail` for `Originals` by @RoloEdits in [#105](https://github.com/Webtoon-Studio/webtoon/pull/105)
* ci: add `--no-fail-fast` to tests by @RoloEdits in [#103](https://github.com/Webtoon-Studio/webtoon/pull/103)
* chore: add `examples/panels` to `.gitignore` by @RoloEdits in [#98](https://github.com/Webtoon-Studio/webtoon/pull/98)
* fix(webtoons): make fields optional in `UserInfo` by @RoloEdits in [#101](https://github.com/Webtoon-Studio/webtoon/pull/101)
* test: separate download tests to `*_single|multi` by @RoloEdits in [#99](https://github.com/Webtoon-Studio/webtoon/pull/99)
* tests(webtoons): fix spurious tests by using oldest post by @RoloEdits in [#96](https://github.com/Webtoon-Studio/webtoon/pull/96)
* docs: refine `webtoons` like `naver` by @RoloEdits in [#90](https://github.com/Webtoon-Studio/webtoon/pull/90)
* refactor(naver): add consuming `IntoIterator` for `Episodes` by @RoloEdits in [#95](https://github.com/Webtoon-Studio/webtoon/pull/95)
* build: use `mold` as `linux-gnu` linker by @RoloEdits in [#94](https://github.com/Webtoon-Studio/webtoon/pull/94)
* fix(naver): parse creator followers with less specific selector by @RoloEdits in [#93](https://github.com/Webtoon-Studio/webtoon/pull/93)
* fix(webtoons): also parse `Graphic Novel` as a genre by @RoloEdits in [#91](https://github.com/Webtoon-Studio/webtoon/pull/91)
* feat!: add platform support for `naver` by @RoloEdits in [#89](https://github.com/Webtoon-Studio/webtoon/pull/89)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.7.0...0.8.0

## What's Changed in 0.7.0
* chore: bump version from `0.6.1` to `0.7.0` by @RoloEdits in [#88](https://github.com/Webtoon-Studio/webtoon/pull/88)
* chore: remove trailing spaces from end of lines by @RoloEdits in [#87](https://github.com/Webtoon-Studio/webtoon/pull/87)
* feat(genre): add supprot for new `graphic-novel` by @RoloEdits in [#85](https://github.com/Webtoon-Studio/webtoon/pull/85)
* perf: use `next_back` instead of `last` for `DoubleEndedIterator` by @RoloEdits in [#86](https://github.com/Webtoon-Studio/webtoon/pull/86)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.6.1...0.7.0

## What's Changed in 0.6.1
* chore: bump version from `0.6.0` to `0.6.1` by @RoloEdits in [#83](https://github.com/Webtoon-Studio/webtoon/pull/83)
* feat: add specialized `first_episode` to `Webtoon` by @RoloEdits in [#81](https://github.com/Webtoon-Studio/webtoon/pull/81)
* refactor(client): move retry logic to a trait mased solution by @RoloEdits in [#82](https://github.com/Webtoon-Studio/webtoon/pull/82)
* chore: remove `TODO` comments and make gh issues of them by @RoloEdits in [#79](https://github.com/Webtoon-Studio/webtoon/pull/79)
* test: remove `no_run` from doc tests by @RoloEdits in [#71](https://github.com/Webtoon-Studio/webtoon/pull/71)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.6.0...0.6.1

## What's Changed in 0.6.0
* chore: bump version from 0.5.0 to 0.6.0 by @RoloEdits in [#70](https://github.com/Webtoon-Studio/webtoon/pull/70)
* refactor(test): switch all session checks to validate checks by @RoloEdits
* refactor(examples): switch all session checks to validate checks by @RoloEdits in [#68](https://github.com/Webtoon-Studio/webtoon/pull/68)
* fix(client)!: add retry logic to internal and public impls by @RoloEdits in [#67](https://github.com/Webtoon-Studio/webtoon/pull/67)
* fix(posts): add retry loop when getting posts by @RoloEdits in [#66](https://github.com/Webtoon-Studio/webtoon/pull/66)
* fix(dashboard): adjust timer to prevent page rate limit by @RoloEdits in [#65](https://github.com/Webtoon-Studio/webtoon/pull/65)
* fix(doc): remove broken intra-doc link by @RoloEdits in [#64](https://github.com/Webtoon-Studio/webtoon/pull/64)
* fix(webtoon): adjust timer to prevent page rate limit by @RoloEdits in [#63](https://github.com/Webtoon-Studio/webtoon/pull/63)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.5.0...0.6.0

## What's Changed in 0.5.0
* chore: bump version from 0.4.0 to 0.5.0 by @RoloEdits in [#62](https://github.com/Webtoon-Studio/webtoon/pull/62)
* feat(poster): add super likes api on `Poster` by @RoloEdits in [#61](https://github.com/Webtoon-Studio/webtoon/pull/61)
* feat(creator): add `id` method by @RoloEdits in [#60](https://github.com/Webtoon-Studio/webtoon/pull/60)
* feat(webtoon): add `is_completed` method by @RoloEdits in [#59](https://github.com/Webtoon-Studio/webtoon/pull/59)
* refactor: change `Release` into more accurate `Schedule` by @RoloEdits in [#58](https://github.com/Webtoon-Studio/webtoon/pull/58)
* refactor: switch to `parking_lot::RwLock` inplace of `tokio` version by @RoloEdits in [#57](https://github.com/Webtoon-Studio/webtoon/pull/57)
* refactor(episode): change `Mutex` for `RwLock` by @RoloEdits in [#56](https://github.com/Webtoon-Studio/webtoon/pull/56)
* refactor(tests): use `has_session` instead of `has_valid_session` by @RoloEdits in [#55](https://github.com/Webtoon-Studio/webtoon/pull/55)
* refactor(episode)!: change `posts_for_each` to async closure by @RoloEdits in [#54](https://github.com/Webtoon-Studio/webtoon/pull/54)
* refactor(creator): change `Mutex` for `RwLock` on `Creator::page` by @RoloEdits in [#51](https://github.com/Webtoon-Studio/webtoon/pull/51)
* refactor(webtoon): change `Mutex` for `RwLock` on `Webtoon::page` by @RoloEdits in [#49](https://github.com/Webtoon-Studio/webtoon/pull/49)
* chore: update to rust 2024 edition by @RoloEdits in [#48](https://github.com/Webtoon-Studio/webtoon/pull/48)
* feat(webtoon): add `is_orginal` and `is_canvas` to `Webtoon` by @RoloEdits in [#47](https://github.com/Webtoon-Studio/webtoon/pull/47)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.4.0...0.5.0

## What's Changed in 0.4.0
* chore: bump version from 0.3.2 to 0.4.0 by @RoloEdits in [#46](https://github.com/Webtoon-Studio/webtoon/pull/46)
* dev: add Justfile by @RoloEdits in [#45](https://github.com/Webtoon-Studio/webtoon/pull/45)
* fix(client): add retry when encounter a 429 response by @RoloEdits
* fix: webtoon thumbnail parse by @RoloEdits in [#43](https://github.com/Webtoon-Studio/webtoon/pull/43)
* build(deps): update scraper requirement from 0.22 to 0.23 by @dependabot[bot] in [#42](https://github.com/Webtoon-Studio/webtoon/pull/42)
* fix(client): return proper deserialization error for `userInfo` endpoint by @RoloEdits in [#39](https://github.com/Webtoon-Studio/webtoon/pull/39)
* fix(lints): fix clippy lints by @RoloEdits in [#40](https://github.com/Webtoon-Studio/webtoon/pull/40)
* build(deps): update scraper requirement from 0.21 to 0.22 by @dependabot[bot] in [#38](https://github.com/Webtoon-Studio/webtoon/pull/38)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.3.2...0.4.0

## What's Changed in 0.3.2
* chore: bump version from 0.3.1 to 0.3.2 by @RoloEdits in [#34](https://github.com/Webtoon-Studio/webtoon/pull/34)
* fix: remove leftover `eprintln!` by @RoloEdits in [#33](https://github.com/Webtoon-Studio/webtoon/pull/33)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.3.1...0.3.2

## What's Changed in 0.3.1
* chore: bump version from 0.3.0 to 0.3.1 by @RoloEdits in [#32](https://github.com/Webtoon-Studio/webtoon/pull/32)
* fix(page): correct parsing of creator names by @RoloEdits in [#31](https://github.com/Webtoon-Studio/webtoon/pull/31)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.3.0...0.3.1

## What's Changed in 0.3.0
* chore: bump version from 0.2.3 to 0.3.0 by @RoloEdits in [#29](https://github.com/Webtoon-Studio/webtoon/pull/29)
* chore: bump version from 0.2.3 to 0.3.0 by @RoloEdits
* fix(tests): check if post is deleted before replying by @RoloEdits in [#30](https://github.com/Webtoon-Studio/webtoon/pull/30)
* fix(tests): check if post is deleted before replying by @RoloEdits
* feat: add integration tests by @RoloEdits in [#26](https://github.com/Webtoon-Studio/webtoon/pull/26)
* feat: add integration tests by @RoloEdits
* doc(posts): if no session default to `Reaction::None` by @RoloEdits in [#25](https://github.com/Webtoon-Studio/webtoon/pull/25)
* doc(posts): if no session default to `Reaction::None` by @RoloEdits
* feat(creator): add `has_patreon` function to `Creator` by @RoloEdits in [#24](https://github.com/Webtoon-Studio/webtoon/pull/24)
* feat(creator): add `has_patreon` function to `Creator` by @RoloEdits
* fix(creator): handle edge case for old accounts by @RoloEdits in [#23](https://github.com/Webtoon-Studio/webtoon/pull/23)
* fix(creator): handle edgecase for old accounts by @RoloEdits

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.2.3...0.3.0

## What's Changed in 0.2.3
* chore: bump version from 0.2.2 to 0.2.3 by @RoloEdits in [#22](https://github.com/Webtoon-Studio/webtoon/pull/22)
* fix(webtoon): return `None` for `banner` when webtoon is `Canvas` by @RoloEdits in [#21](https://github.com/Webtoon-Studio/webtoon/pull/21)
* fix(creator): update to recent changes made by @RoloEdits in [#20](https://github.com/Webtoon-Studio/webtoon/pull/20)
* build(deps): update thiserror requirement from 1 to 2 by @dependabot[bot] in [#18](https://github.com/Webtoon-Studio/webtoon/pull/18)
* build(deps): update scraper requirement from 0.20 to 0.21 by @dependabot[bot] in [#17](https://github.com/Webtoon-Studio/webtoon/pull/17)

## New Contributors
* @dependabot[bot] made their first contribution in [#18](https://github.com/Webtoon-Studio/webtoon/pull/18)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.2.2...0.2.3

## What's Changed in 0.2.2
* chore: bump version from `0.2.1` to `0.2.2` by @RoloEdits
* fix(posts): add support for `Super Like` post `section` by @RoloEdits in [#16](https://github.com/Webtoon-Studio/webtoon/pull/16)
* perf(replies): add early return when replies are zero by @RoloEdits in [#15](https://github.com/Webtoon-Studio/webtoon/pull/15)
* style(clippy): fix `rust_2018_idioms` lints by @RoloEdits in [#14](https://github.com/Webtoon-Studio/webtoon/pull/14)
* docs: add `docs.rs` flag for feature annotations by @RoloEdits in [#13](https://github.com/Webtoon-Studio/webtoon/pull/13)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.2.1...0.2.2

## What's Changed in 0.2.1
* chore: bump version to `0.2.0` to `0.2.1` by @RoloEdits

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.2.0...0.2.1

## What's Changed in 0.2.0
* chore: bump version from `0.1.0` to `0.2.0` by @RoloEdits in [#12](https://github.com/Webtoon-Studio/webtoon/pull/12)
* build: switch `reqwest` to `rustls` backend by @RoloEdits in [#8](https://github.com/Webtoon-Studio/webtoon/pull/8)
* feat(creator): add check for disabled profile page by @RoloEdits in [#9](https://github.com/Webtoon-Studio/webtoon/pull/9)
* refactor!: move `rss` and `download` behind feature flags by @RoloEdits in [#10](https://github.com/Webtoon-Studio/webtoon/pull/10)
* refactor(examples): use basic `Client` for downloads by @RoloEdits in [#7](https://github.com/Webtoon-Studio/webtoon/pull/7)
* chore: add more cargo package metadata by @RoloEdits in [#11](https://github.com/Webtoon-Studio/webtoon/pull/11)
* chore: clean up residual leftover commented code by @RoloEdits in [#6](https://github.com/Webtoon-Studio/webtoon/pull/6)
* chore: clean up TODO's by @RoloEdits
* chore: separate daily test from CI by @RoloEdits
* chore(gitignore): add `Cargo.lock` by @RoloEdits
* chore(dependabot): add `dependabot.yml` by @RoloEdits
* build: create `CI.yml` by @RoloEdits
* feat(README): Add link to `examples` folder by @RoloEdits
* feat(README): add badges by @RoloEdits
* refactor(Cargo.toml): edit keywords by @RoloEdits

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.1.0...0.2.0

## What's Changed in 0.1.0
* feat: clean project by @RoloEdits
* Create LICENSE-APACHE by @RoloEdits
* Rename LICENSE to LICENSE-MIT by @RoloEdits
* Initial commit by @RoloEdits

## New Contributors
* @RoloEdits made their first contribution

<!-- generated by git-cliff -->
