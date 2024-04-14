# toktor

---
[![Build Status](https://https://github.com/Hodkinson/toktor/workflows/Build/badge.svg)](https://github.com/Hodkinson/toktor/actions)

---

A small Actor framework for use within tokio.

Features:
* both graceful and hard shutdown options for actors
* spawning of child actors
* spawning of associated tasks, that may be shutdown with the actor

This was created to cater for a need in a project for allowing creation of complex actor
structures that could be easily shutdown when required. For example in UIs, when a user
closes a window, or in telephony when an internet call is dropped.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.