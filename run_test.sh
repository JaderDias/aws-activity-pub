cargo run --example test http://localhost:8080 \
&& cargo fmt --all -- --check \
&& cargo clippy --all -- \
  -D clippy::all \
  -D clippy::pedantic \
  -D clippy::cargo \
  -D clippy::nursery \
  -W clippy::no_effect_underscore_binding \
  -W clippy::multiple_crate_versions \
  -W clippy::future_not_send