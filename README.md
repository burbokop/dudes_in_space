# dudes_in_space

## Run
```bash
CXX=/usr/bin/clang++-19 cargo run --release --package dev_presenter
```

## Profile
```bash
sudo sysctl kernel.perf_event_paranoid=2
CXX=/usr/bin/clang++-19 CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --package dev_presenter
```
