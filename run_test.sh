cargo run --example test http://localhost:8080 \
&& cargo fmt --all -- --check \
&& cargo clippy --all -- -W clippy::all -W clippy::pedantic