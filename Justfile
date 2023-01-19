#!/bin/env -S just --justfile

_run_def: && (run "0")
    @echo "just: Defaulting to smoke test..."

alias r := run
run challenge port="10000" level="debug":
    RUST_LOG={{ level }} cargo run --release -- -c {{ challenge }} -p {{ port }}

c1test num port="10000":
    echo '{"method":"isPrime","number":{{ num }}}' | nc --tcp 127.0.0.1 {{ port }}
