#!/bin/env -S just --justfile

_run_def: && (run "0")
    @echo "just: Defaulting to smoke test..."

alias r := run
run challenge level="debug" port="10000":
    RUST_LOG={{ level }} cargo run --release -- -c {{ challenge }} -p {{ port }}

nc data port="10000" addr="127.0.0.1":
    echo '{{ data }}' | nc --tcp {{ addr }} {{ port }}

c1test num port="10000" addr="127.0.0.1": (nc '{"method":"isPrime","number":' + num + '}' port addr)
