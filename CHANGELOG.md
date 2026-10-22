## What's Changed in 0.10.0
* build(deps): update html-escape requirement from 0.2 to 0.3 by @dependabot[bot] in [#309](https://github.com/Webtoon-Studio/webtoon/pull/309)
* fix: update expected error message for deactivated creator profile by @RoloEdits in [#308](https://github.com/Webtoon-Studio/webtoon/pull/308)
* refactor: narrow logic for webtoons url parsing by @RoloEdits in [#307](https://github.com/Webtoon-Studio/webtoon/pull/307)
* ci(daily): change profile to `test` by @RoloEdits in [#306](https://github.com/Webtoon-Studio/webtoon/pull/306)
* perf: make downloading panels truly parallel by @RoloEdits in [#305](https://github.com/Webtoon-Studio/webtoon/pull/305)
* test: add `no_run` to Comment and Reply doc tests by @RoloEdits in [#304](https://github.com/Webtoon-Studio/webtoon/pull/304)
* test: ignore originals daily schedule parse by @RoloEdits in [#303](https://github.com/Webtoon-Studio/webtoon/pull/303)
* build(deps): bump `assumptions` rev by @RoloEdits in [#302](https://github.com/Webtoon-Studio/webtoon/pull/302)
* refactor(http): retry on 500 Internal Server Error by @RoloEdits in [#301](https://github.com/Webtoon-Studio/webtoon/pull/301)
* feat: add more assertions/assumptions by @RoloEdits in [#300](https://github.com/Webtoon-Studio/webtoon/pull/300)
* core: clean up clippy lints by @RoloEdits in [#299](https://github.com/Webtoon-Studio/webtoon/pull/299)
* build(deps): bump `assumptions` rev by @RoloEdits in [#298](https://github.com/Webtoon-Studio/webtoon/pull/298)
* refactor: make assumption messages follow recommended style by @RoloEdits in [#297](https://github.com/Webtoon-Studio/webtoon/pull/297)
* build(deps): switch to `assumptions` crate by @RoloEdits in [#296](https://github.com/Webtoon-Studio/webtoon/pull/296)
* refactor: even more polishing by @RoloEdits in [#295](https://github.com/Webtoon-Studio/webtoon/pull/295)
* refactor: more polishing by @RoloEdits in [#294](https://github.com/Webtoon-Studio/webtoon/pull/294)
* refactor: more polishing by @RoloEdits in [#293](https://github.com/Webtoon-Studio/webtoon/pull/293)
* refactor: parse new analytics dashboard by @RoloEdits in [#292](https://github.com/Webtoon-Studio/webtoon/pull/292)
* test: add more integration tests by @RoloEdits in [#291](https://github.com/Webtoon-Studio/webtoon/pull/291)
* fix: ensure that only english webtoons are supported by @RoloEdits in [#290](https://github.com/Webtoon-Studio/webtoon/pull/290)
* refactor: polish up codebase by @RoloEdits in [#289](https://github.com/Webtoon-Studio/webtoon/pull/289)
* build(deps): bump actions/checkout from 6 to 7 by @dependabot[bot] in [#288](https://github.com/Webtoon-Studio/webtoon/pull/288)
* ci(smoke): remove non-english jobs by @RoloEdits in [#287](https://github.com/Webtoon-Studio/webtoon/pull/287)
* docs(README): change MSRV to `1.95` by @RoloEdits in [#286](https://github.com/Webtoon-Studio/webtoon/pull/286)
* refactor!(webtoons): only support english language by @RoloEdits in [#285](https://github.com/Webtoon-Studio/webtoon/pull/285)
* refactor!: remove naver support by @RoloEdits in [#283](https://github.com/Webtoon-Studio/webtoon/pull/283)
* feat(stdx): add `hint` module by @RoloEdits in [#284](https://github.com/Webtoon-Studio/webtoon/pull/284)
* build(deps): update scraper requirement from 0.26 to 0.27 by @dependabot[bot] in [#282](https://github.com/Webtoon-Studio/webtoon/pull/282)
* fix(webtoons): parse `หลังวันสิ้นโลก` as `PostApocalyptic` by @RoloEdits in [#280](https://github.com/Webtoon-Studio/webtoon/pull/280)
* test(webtoons/english): `everyday` story completed by @RoloEdits in [#281](https://github.com/Webtoon-Studio/webtoon/pull/281)
* refactor(webtoons): adjust dashboard episode published assumption by @RoloEdits in [#279](https://github.com/Webtoon-Studio/webtoon/pull/279)
* chore: fix clippy lints by @RoloEdits in [#276](https://github.com/Webtoon-Studio/webtoon/pull/276)
* test(webtoons/indonesia): remove test as webtoon is no longer reachable by @RoloEdits in [#275](https://github.com/Webtoon-Studio/webtoon/pull/275)
* feat(webtoons): add assumption that published year is at least 2014 by @RoloEdits in [#274](https://github.com/Webtoon-Studio/webtoon/pull/274)
* fix(webtoons): parse `Inspirasional` as `Inspirational` by @RoloEdits in [#273](https://github.com/Webtoon-Studio/webtoon/pull/273)
* fix(webtoons): handle `EVERYDAY` as `Daily` by @RoloEdits in [#272](https://github.com/Webtoon-Studio/webtoon/pull/272)
* fix(webtoons): parse `Post-apokalypse` as `PostApocalyptic` by @RoloEdits in [#271](https://github.com/Webtoon-Studio/webtoon/pull/271)
* refactor(README): fix lints and clean up sections by @RoloEdits in [#270](https://github.com/Webtoon-Studio/webtoon/pull/270)
* test(webtoons/thai): don't fail when epsiode is `NotViewable` by @RoloEdits in [#269](https://github.com/Webtoon-Studio/webtoon/pull/269)
* fix(webtoons): handle thai webtoons that are app only by @RoloEdits in [#267](https://github.com/Webtoon-Studio/webtoon/pull/267)
* fix(webtoons): handle empty webtoon summary by @RoloEdits in [#266](https://github.com/Webtoon-Studio/webtoon/pull/266)
* fix(webtoons): handle when webtoon flare Webtoon is deleted by @RoloEdits in [#265](https://github.com/Webtoon-Studio/webtoon/pull/265)
* fix(webtoons): parse `สัตว์` as `Animals` by @RoloEdits in [#264](https://github.com/Webtoon-Studio/webtoon/pull/264)
* fix(webtoons): parse `สร้างแรงบันดาลใจ` as `Inspirational` by @RoloEdits in [#263](https://github.com/Webtoon-Studio/webtoon/pull/263)
* fix(webtoons): parse `Historique` as `Historical` by @RoloEdits in [#262](https://github.com/Webtoon-Studio/webtoon/pull/262)
* fix(webtoons): parse `เหนือธรรมชาติ` as `Supernatural` by @RoloEdits in [#261](https://github.com/Webtoon-Studio/webtoon/pull/261)
* fix(webtoons): parse `ทุกวัย` as `AllAges` by @RoloEdits in [#260](https://github.com/Webtoon-Studio/webtoon/pull/260)
* fix(webtoons): parse `โรงเรียน` as `School` by @RoloEdits in [#259](https://github.com/Webtoon-Studio/webtoon/pull/259)
* fix(webtoons): parse `Réconfortant` as `Heartwarming` by @RoloEdits in [#258](https://github.com/Webtoon-Studio/webtoon/pull/258)
* fix(webtoons): parse `Hewan` as `Animals` by @RoloEdits in [#257](https://github.com/Webtoon-Studio/webtoon/pull/257)
* fix(webtoons): parse `Tout âge` as `AllAges` by @RoloEdits in [#256](https://github.com/Webtoon-Studio/webtoon/pull/256)
* fix(webtoons): parse `อาชญากรรม / ปริศนา` as `Mystery` by @RoloEdits in [#255](https://github.com/Webtoon-Studio/webtoon/pull/255)
* fix(webtoons): parse `Animaux` as `Animals` by @RoloEdits in [#254](https://github.com/Webtoon-Studio/webtoon/pull/254)
* fix(webtoons): parse `Inspirant` as `Inspirational` by @RoloEdits in [#253](https://github.com/Webtoon-Studio/webtoon/pull/253)
* fix(webtoons): parse `Vie scolaire` as `School` by @RoloEdits in [#252](https://github.com/Webtoon-Studio/webtoon/pull/252)
* fix(webtoons): parse `Zombi` as `Zombies` by @RoloEdits in [#251](https://github.com/Webtoon-Studio/webtoon/pull/251)
* fix(webtoons): parse `Cerita pendek` as `ShortStory` by @RoloEdits in [#250](https://github.com/Webtoon-Studio/webtoon/pull/250)
* fix(webtoons): parse `เรื่องไทย` as `Local` by @RoloEdits in [#249](https://github.com/Webtoon-Studio/webtoon/pull/249)
* fix(webtoons): parse `ซอมบี้` as `Zombies` by @RoloEdits in [#246](https://github.com/Webtoon-Studio/webtoon/pull/246)
* fix(webtoons): parse `เรื่องสั้น` as `ShortStory` by @RoloEdits in [#248](https://github.com/Webtoon-Studio/webtoon/pull/248)
* fix(webtoons): parse `Sekolah` as `School` by @RoloEdits in [#247](https://github.com/Webtoon-Studio/webtoon/pull/247)
* fix(webtoons): parse `Surnaturel` as `Supernatural` by @RoloEdits in [#245](https://github.com/Webtoon-Studio/webtoon/pull/245)
* fix(webtoons): parse `Post Apocalyptique` as `PostApocalyptic` by @RoloEdits in [#244](https://github.com/Webtoon-Studio/webtoon/pull/244)
* fix(webtoons): parse `Post Apocalypse` as `PostApocalyptic` by @RoloEdits in [#243](https://github.com/Webtoon-Studio/webtoon/pull/243)
* feat(webtoons): add thai support by @RoloEdits in [#241](https://github.com/Webtoon-Studio/webtoon/pull/241)
* fix(webtoons): parse `ROMANCE FANTASTIQUE` as `RomanticFantasy` by @RoloEdits in [#242](https://github.com/Webtoon-Studio/webtoon/pull/242)
* fix(webtoons): parse `Supranatural` as `Supernatural` by @RoloEdits in [#240](https://github.com/Webtoon-Studio/webtoon/pull/240)
* fix(webtoons): parse `Tiere` as `Animals` by @RoloEdits in [#239](https://github.com/Webtoon-Studio/webtoon/pull/239)
* fix(webtoons): parse `Zombis` as `Zombies` by @RoloEdits in [#238](https://github.com/Webtoon-Studio/webtoon/pull/238)
* fix(webtoons): parse `LGBTQI+` as `LGBTQ` by @RoloEdits in [#237](https://github.com/Webtoon-Studio/webtoon/pull/237)
* fix(webtoons): handle if Canvas creator has no profile by @RoloEdits in [#236](https://github.com/Webtoon-Studio/webtoon/pull/236)
* fix(webtoons): parse `hari` as `Daily` by @RoloEdits in [#235](https://github.com/Webtoon-Studio/webtoon/pull/235)
* fix(webtoons): parse `Semua Umur` as `AllAges` by @RoloEdits in [#234](https://github.com/Webtoon-Studio/webtoon/pull/234)
* fix(webtoons): parse `Olahraga` as `Sports` by @RoloEdits in [#233](https://github.com/Webtoon-Studio/webtoon/pull/233)
* fix(webtoons): parse `Crime/Mystère` as `Mystery` by @RoloEdits in [#232](https://github.com/Webtoon-Studio/webtoon/pull/232)
* fix(webtoons): parse `Todas las edades` as `AllAges` by @RoloEdits in [#231](https://github.com/Webtoon-Studio/webtoon/pull/231)
* build(deps): update scraper requirement from 0.25 to 0.26 by @dependabot[bot] in [#230](https://github.com/Webtoon-Studio/webtoon/pull/230)
* fix(webtoons): parse `Kriminal/Misteri` as `Mystery` by @RoloEdits in [#229](https://github.com/Webtoon-Studio/webtoon/pull/229)
* fix(webtoons): parse `Histoires Courtes` as `ShortStory` by @RoloEdits in [#227](https://github.com/Webtoon-Studio/webtoon/pull/227)
* fix(webtoons): parse `Animales` as `Animals` by @RoloEdits in [#228](https://github.com/Webtoon-Studio/webtoon/pull/228)
* dev(smoke): make smoke tests every 30min by @RoloEdits in [#226](https://github.com/Webtoon-Studio/webtoon/pull/226)
* fix(webtoons): parse `Escuela` as `School` by @RoloEdits in [#225](https://github.com/Webtoon-Studio/webtoon/pull/225)
* feat(webtoons): support indonesian version by @RoloEdits in [#224](https://github.com/Webtoon-Studio/webtoon/pull/224)
* fix(webtoons): parse `Sport` as `Sports` by @RoloEdits in [#223](https://github.com/Webtoon-Studio/webtoon/pull/223)
* fix(webtoons): clean html entities from panel url's by @RoloEdits in [#222](https://github.com/Webtoon-Studio/webtoon/pull/222)
* fix(webtoons): parse `LOCAL` as `Local` by @RoloEdits in [#221](https://github.com/Webtoon-Studio/webtoon/pull/221)
* fix(webtoons): parse `Schule` as `School` by @RoloEdits in [#220](https://github.com/Webtoon-Studio/webtoon/pull/220)
* fix(webtoons): parse `Inspirierend` as `Inspirational` by @RoloEdits in [#219](https://github.com/Webtoon-Studio/webtoon/pull/219)
* fix(webtoons): parse `Inspirador` as `Inspirational` by @RoloEdits in [#218](https://github.com/Webtoon-Studio/webtoon/pull/218)
* feat(webtoons): add french smoke test runs by @RoloEdits in [#217](https://github.com/Webtoon-Studio/webtoon/pull/217)
* feat(webtoons): support french version by @RoloEdits in [#216](https://github.com/Webtoon-Studio/webtoon/pull/216)
* perf: speed up slow tests by @RoloEdits in [#215](https://github.com/Webtoon-Studio/webtoon/pull/215)
* fix(webtoons): parse `Misterio` as `Mystery by @RoloEdits in [#213](https://github.com/Webtoon-Studio/webtoon/pull/213)
* fix(webtoons): parse `Alle Altersklassen` as `AllAges by @RoloEdits in [#214](https://github.com/Webtoon-Studio/webtoon/pull/214)
* fix(webtoons): parse `Postapocalíptico` as `PostApocalyptic by @RoloEdits in [#212](https://github.com/Webtoon-Studio/webtoon/pull/212)
* fix(webtoons): parse `Fantasía Romántica` as `RomanticFantasy` by @RoloEdits in [#211](https://github.com/Webtoon-Studio/webtoon/pull/211)
* fix(webtoons): parse `Kurzgeschichten` as `ShortStory` by @RoloEdits in [#210](https://github.com/Webtoon-Studio/webtoon/pull/210)
* fix(webtoons): parse `Crimen/Misterio` as `Mystery` by @RoloEdits in [#209](https://github.com/Webtoon-Studio/webtoon/pull/209)
* fix(webtoons): parse `Historia Corta` as `ShortStory` by @RoloEdits in [#208](https://github.com/Webtoon-Studio/webtoon/pull/208)
* feat(webtoons): support spanish version by @RoloEdits in [#207](https://github.com/Webtoon-Studio/webtoon/pull/207)
* refactor(webtoons): do language specific schedule parsing by @RoloEdits in [#206](https://github.com/Webtoon-Studio/webtoon/pull/206)
* fix(webtoons): parse `Übernatürlich` as `Supernatural` by @RoloEdits in [#205](https://github.com/Webtoon-Studio/webtoon/pull/205)
* fix(webtoons): parse `Romantasy` as `RomanticFantasy` by @RoloEdits in [#204](https://github.com/Webtoon-Studio/webtoon/pull/204)
* feat(webtoons): support german version by @RoloEdits in [#197](https://github.com/Webtoon-Studio/webtoon/pull/197)
* refactor(webtoons/homepage): make parsing more language generic by @RoloEdits in [#203](https://github.com/Webtoon-Studio/webtoon/pull/203)
* test(webtoons): ignore `everyday_is_daily_schedule` by @RoloEdits in [#202](https://github.com/Webtoon-Studio/webtoon/pull/202)
* refactor(webtoons/webtoon): clean up count parsing by @RoloEdits in [#201](https://github.com/Webtoon-Studio/webtoon/pull/201)
* refactor(webtoons): improve creator name parsing by @RoloEdits in [#200](https://github.com/Webtoon-Studio/webtoon/pull/200)
* refactor(webtoons)!: return Option<UserInfo>` from `Client::user_info_for_session` by @RoloEdits in [#199](https://github.com/Webtoon-Studio/webtoon/pull/199)
* feat(webtoons): add `is_canvas_creator` to `UserInfo` by @RoloEdits in [#198](https://github.com/Webtoon-Studio/webtoon/pull/198)
* refactor(webtoons): change `UserInfo` assumption to include json response by @RoloEdits in [#196](https://github.com/Webtoon-Studio/webtoon/pull/196)
* feat(webtoons): add extra info to `UserInfo` assumption by @RoloEdits in [#195](https://github.com/Webtoon-Studio/webtoon/pull/195)
* feat(webtoons/episode): add `is_published` method by @RoloEdits in [#194](https://github.com/Webtoon-Studio/webtoon/pull/194)
* refactor(webtoons/originals): support parsing `EVERYDAY` as schedule by @RoloEdits in [#191](https://github.com/Webtoon-Studio/webtoon/pull/191)
* refactor(webtoons/creator): document profile page can return `None` when community disabled by @RoloEdits in [#190](https://github.com/Webtoon-Studio/webtoon/pull/190)
* fix(webtoons/creator): handle creator page community violation by @RoloEdits in [#180](https://github.com/Webtoon-Studio/webtoon/pull/180)
* test(webtoons): add more profiles for invalid profile test by @RoloEdits in [#188](https://github.com/Webtoon-Studio/webtoon/pull/188)
* refactor(webtoons): handle invalid creator profiles by @RoloEdits in [#187](https://github.com/Webtoon-Studio/webtoon/pull/187)
* feat(webtoons): support parsing `Romance Fantasy` genre by @RoloEdits in [#186](https://github.com/Webtoon-Studio/webtoon/pull/186)
* fix: Reply::downvote test by @RoloEdits in [#185](https://github.com/Webtoon-Studio/webtoon/pull/185)
* build(deps): bump actions/checkout from 4 to 6 by @dependabot[bot] in [#184](https://github.com/Webtoon-Studio/webtoon/pull/184)
* dev(dependabot): increase interval and add github-actions by @RoloEdits in [#183](https://github.com/Webtoon-Studio/webtoon/pull/183)
* test(webtoons): adjust tests so they pass again by @RoloEdits in [#182](https://github.com/Webtoon-Studio/webtoon/pull/182)
* test: fix drifting in tests by @RoloEdits in [#181](https://github.com/Webtoon-Studio/webtoon/pull/181)
* build(deps): bump reqwest from 12 to 13 by @RoloEdits in [#178](https://github.com/Webtoon-Studio/webtoon/pull/178)
* refactor(webtoons/reply): fix doctest test as reply was upvoted by @RoloEdits in [#179](https://github.com/Webtoon-Studio/webtoon/pull/179)
* refactor(webtoons/creator): get username from meta tag by @RoloEdits in [#177](https://github.com/Webtoon-Studio/webtoon/pull/177)
* fix(webtoons/reply): use `last` in doctest by @RoloEdits in [#176](https://github.com/Webtoon-Studio/webtoon/pull/176)
* refactor(webtoons/smoke): only search as far as page 3000 by @RoloEdits in [#175](https://github.com/Webtoon-Studio/webtoon/pull/175)
* refactor(webtoons/creator): trim spaces in creator name by @RoloEdits in [#173](https://github.com/Webtoon-Studio/webtoon/pull/173)
* fix(webtoons/creator): escape html entities in creator name by @RoloEdits in [#172](https://github.com/Webtoon-Studio/webtoon/pull/172)
* refactor(webtoons/creator)!: move some network requests out from `Creator` methods by @RoloEdits in [#168](https://github.com/Webtoon-Studio/webtoon/pull/168)
* refactor(webtoons/reply): update timestamp to make test pass by @RoloEdits in [#171](https://github.com/Webtoon-Studio/webtoon/pull/171)
* refactor(webtoons/comment)!: pre-cache top comments by @RoloEdits in [#169](https://github.com/Webtoon-Studio/webtoon/pull/169)
* refactor(webtoons/webtoon)!: remove `is_subscribed` from `Webtoon` by @RoloEdits in [#170](https://github.com/Webtoon-Studio/webtoon/pull/170)
* chore(webtoons/post): remove implemented todo by @RoloEdits in [#167](https://github.com/Webtoon-Studio/webtoon/pull/167)
* refactor(webtoons/error): remove unused errors by @RoloEdits in [#166](https://github.com/Webtoon-Studio/webtoon/pull/166)
* refactor(webtoons)!: remove all user interaction functionality by @RoloEdits in [#164](https://github.com/Webtoon-Studio/webtoon/pull/164)
* chore(naver/client): move `allow` lint to entire `impl PartialOrd` block by @RoloEdits in [#163](https://github.com/Webtoon-Studio/webtoon/pull/163)
* feat(webtoons/client): add assumption for `UserInfo` data combinations by @RoloEdits in [#162](https://github.com/Webtoon-Studio/webtoon/pull/162)
* perf(webtoons/post): cache episode top comments for quicker `is_top` by @RoloEdits in [#161](https://github.com/Webtoon-Studio/webtoon/pull/161)
* refactor(webtoons): wrap `Post` with `Comment` and `Reply by @RoloEdits in [#158](https://github.com/Webtoon-Studio/webtoon/pull/158)
* fix(webtoons/creator): handle invalid creator profile response by @RoloEdits in [#160](https://github.com/Webtoon-Studio/webtoon/pull/160)
* fix(webtoons/episode): handle images with multple `.` in file name by @RoloEdits in [#159](https://github.com/Webtoon-Studio/webtoon/pull/159)
* refactor(webtoons): remove `Posts` and use `Vec<Post>` by @RoloEdits in [#156](https://github.com/Webtoon-Studio/webtoon/pull/156)
* fix(webtoons/homepage): handle names with trailing `...` by @RoloEdits in [#157](https://github.com/Webtoon-Studio/webtoon/pull/157)
* refactor(webtoons/episode): remove `season` from `Episode` by @RoloEdits in [#155](https://github.com/Webtoon-Studio/webtoon/pull/155)
* refactor(webtoons/episode)!: introduce a `Published` abstraction over date and datetime by @RoloEdits in [#154](https://github.com/Webtoon-Studio/webtoon/pull/154)
* chore: remove todo for adding verious genres to test by @RoloEdits in [#153](https://github.com/Webtoon-Studio/webtoon/pull/153)
* refactor(webtoons/episode): rename `Panels` to `DownloadedPanels` by @RoloEdits in [#152](https://github.com/Webtoon-Studio/webtoon/pull/152)
* fix(webtoons/episode): support gif image format by @RoloEdits in [#150](https://github.com/Webtoon-Studio/webtoon/pull/150)
* ci(smoke): run tests one at a time by @RoloEdits in [#151](https://github.com/Webtoon-Studio/webtoon/pull/151)
* chore(deny): remove unneeded advisory for scraper as updated by @RoloEdits in [#149](https://github.com/Webtoon-Studio/webtoon/pull/149)
* build(deps): update scraper requirement from 0.24 to 0.25 by @dependabot[bot] in [#148](https://github.com/Webtoon-Studio/webtoon/pull/148)
* refactor(webtoons)!: add new `assumption` contract-style conditional checks by @RoloEdits in [#145](https://github.com/Webtoon-Studio/webtoon/pull/145)
* refactor: use let-chains where possible by @RoloEdits in [#144](https://github.com/Webtoon-Studio/webtoon/pull/144)
* chore: bump MSRV to 1.88 by @RoloEdits in [#143](https://github.com/Webtoon-Studio/webtoon/pull/143)
* build: remove `.cargo/config.toml` by @RoloEdits in [#142](https://github.com/Webtoon-Studio/webtoon/pull/142)
* build(features)!: add `download` and `naver` features by @RoloEdits in [#140](https://github.com/Webtoon-Studio/webtoon/pull/140)
* dev: remove dev profile by @RoloEdits in [#141](https://github.com/Webtoon-Studio/webtoon/pull/141)
* dev: add `.helix/` to gitignore by @RoloEdits in [#139](https://github.com/Webtoon-Studio/webtoon/pull/139)
* fix(episode/note): don't assume None is an error by @RoloEdits in [#137](https://github.com/Webtoon-Studio/webtoon/pull/137)
* build(deps): remove `serde_with` from dependencies by @RoloEdits in [#138](https://github.com/Webtoon-Studio/webtoon/pull/138)
* refactor!: restructure `webtoons/` to be flatter by @RoloEdits in [#136](https://github.com/Webtoon-Studio/webtoon/pull/136)
* fix(lints): cargo clippy by @RoloEdits in [#134](https://github.com/Webtoon-Studio/webtoon/pull/134)
* perf(webtoons): parallelize `Client::originals` by @RoloEdits in [#133](https://github.com/Webtoon-Studio/webtoon/pull/133)
* perf(webtoons): parallelize `Creator::webtoons` by @RoloEdits in [#132](https://github.com/Webtoon-Studio/webtoon/pull/132)
* ci: update actions to v5 by @RoloEdits in [#131](https://github.com/Webtoon-Studio/webtoon/pull/131)
* ci(audit): add `cargo deny` action by @RoloEdits in [#130](https://github.com/Webtoon-Studio/webtoon/pull/130)

**Full Changelog**: https://github.com/Webtoon-Studio/webtoon/compare/0.9.0...0.10.0

## What's Changed in 0.9.0
* chore: bump version from 0.8.0 to 0.9.0 by @RoloEdits in [#129](https://github.com/Webtoon-Studio/webtoon/pull/129)
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
