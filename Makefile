.PHONY: \
	all \
	build \
	build_with_profile \
	check \
	clean \
	clippy \
	grcov \
	html_coverage_report \
	integration_test \
	kill_service_running_in_background \
	lcov \
	nightly_toolchain \
	refresh_database \
	run_service_in_background \
	scan_table \
	test \
	test_with_coverage \
	unit_test \
	watch
.SHELLFLAGS = -ec # -e for exiting on errors and -c so that -e doesn't cause unwanted side effects

build:
	cargo clean -p rust_lambda
	cargo build --all-targets

build_with_profile:
	cargo clean
	CARGO_INCREMENTAL=0 \
		RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" \
		RUSTDOCFLAGS="-Cpanic=abort" \
		cargo build --all-targets

check: clippy
	cargo fmt --all -- --check

clippy:
	cargo clippy --all -- \
		-D "clippy::all" \
		-D clippy::pedantic \
		-D clippy::cargo \
		-D clippy::nursery \
		-W clippy::future_not_send \
		-W clippy::missing_errors_doc \
		-W clippy::module_name_repetitions \
		-W clippy::multiple_crate_versions \
		-W clippy::no_effect_underscore_binding

grcov:
	cargo install grcov
	grcov . \
		-s . \
		--binary-path ./target/debug/ \
		-t $(TYPE_PARAM) \
		--branch \
		--ignore-not-existing \
		-o ./target/debug/$(OUTPUT)

html_coverage_report:
	$(MAKE) grcov TYPE_PARAM=html OUTPUT=coverage

integration_test:
	RUST_LOG="test=info" \
	LOCAL_DYNAMODB_URL=http://localhost:8000 \
		./target/debug/examples/test localhost:8080 target_username localhost:8080 signer_username

kill_service_running_in_background:
	pkill rust_lambda || true
	pkill -9 rust_lambda || true

lcov:
	$(MAKE) grcov TYPE_PARAM=lcov OUTPUT=lcov.info

nightly_toolchain:
	rustup update nightly
	rustup default nightly

refresh_database:
	docker-compose -f docker/test/docker-compose.yml kill
	docker-compose -f docker/test/docker-compose.yml up --build --detach

run_service_in_background: kill_service_running_in_background
	CUSTOM_DOMAIN=localhost:8080 \
		DYNAMODB_TABLE=ServerlessActivityPub \
		LOCAL_DYNAMODB_URL=http://localhost:8000 \
		PROTOCOL=http \
		REGION=eu-west-1 \
		RUST_LOG="rust_lambda=info" \
		./target/debug/rust_lambda &

scan_table:
	@if ! grep -F '[profile localhost]' <~/.aws/config; then \
		echo "[profile localhost]\nregion = us-east-1" >>~/.aws/config; \
	fi
	@if ! grep -F '[localhost]' <~/.aws/credentials; then \
		echo "[localhost]\naws_access_key_id = ANY_ACCESS_KEY_WILL_DO\naws_secret_access_key = ANY_SECRET_KEY_WILL_DO" >>~/.aws/credentials; \
	fi
	aws dynamodb scan --table-name ServerlessActivityPub --endpoint-url http://localhost:8000 --profile localhost

test: \
	refresh_database \
	build \
	run_service_in_background \
	integration_test

test_with_coverage: \
	nightly_toolchain \
	refresh_database \
	build_with_profile \
	run_service_in_background \


unit_test:
	cargo test

watch:
	cargo watch --clear