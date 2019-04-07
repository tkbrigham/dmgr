### Ordered ToDo list
1. Subcommands:
  - Register
  - Start
  - Stop
  - Logs
1. Config file overrides
1. Routing: enable requests to be sent elsewhere

### Tasks
1. [routing](https://docs.rs/iron/0.6.0/iron/request/index.html)
    1. This is to enable connecting to non-local services
    1. should also include `dmgr auth` or similar. Need to auth might lead to multiple layers
       of auth, from per-service, to per-env, to per-service-and-env
1. [Logging](https://docs.rs/log4rs/*/log4rs/)
    1. Use [solo logging](https://github.com/socrata/solo/blob/master/solo/solo_logging.py)
       for examples if necessary
    1. [Formatter documentation](https://docs.rs/log4rs/0.8.1/log4rs/encode/pattern/index.html)
    1. [rust strftime vars](https://docs.rs/chrono/0.4.0/chrono/format/strftime/index.html)
1. [arg parsing](https://docs.rs/clap/*/clap/)
    1. [examples](https://github.com/clap-rs/clap/blob/master/examples/01b_quick_example.rs)
1. [running commands](https://doc.rust-lang.org/std/process/struct.Command.html)
    1. [CHECK THIS OUT](https://rust-lang-nursery.github.io/rust-cookbook/os/external.html)
1. Configurable, either `dmgr config` or otherwise
    1. Configured with files, I would think
1. Dependency trees?
1. Daemon to check on status of services?
1. `bin` dir?
1. tests
1. installation
1. versioning (Cargo.toml, probs)
1. constants
1. registry class
1. service class
1. group class
1. Config file 
    1. ~/.config
    1. [.toml files](https://docs.rs/toml/0.4.10/toml/)
1. subcommands
    1. config
    1. group
    1. health
    1. list
    1. logs
    1. pull
    1. register
    1. restart
    1. start
    1. stop
1. migration from Python `solo`
1. Harden, aka no `unwrap()`


### Other topics
1. [Build/release](https://doc.rust-lang.org/stable/book/ch14-00-more-about-cargo.html)
1. [Rust book](https://doc.rust-lang.org/stable/book/)
1. [std lib docs](https://doc.rust-lang.org/std/index.html)
1. [rust API checklist](https://rust-lang-nursery.github.io/api-guidelines/checklist.html)
1. [arg completion](https://docs.rs/clap/*/clap/struct.App.html#method.gen_completions)
