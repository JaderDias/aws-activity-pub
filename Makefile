.PHONY: all clean test \
	build \
	build_with_profile \
	check \
	clippy \
	grcov \
	integration_test \
	nightly_toolchain \
	refresh_database \
	run_service_in_background \
	scan_table \
	stable_toolchain \
	test_with_coverage \
	test_with_html_coverage \
	test_with_lcov \
	unit_test \
	watch
.SHELLFLAGS = -ec # -e for exiting on errors and -c so that -e doesn't cause unwanted side effects

build:
	cargo clean -p web_service || true
	cargo build --workspace --all-targets

build_with_profile:
	cargo clean
	CARGO_INCREMENTAL=0 \
		RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" \
		RUSTDOCFLAGS="-Cpanic=abort" \
		cargo build --workspace --all-targets

check: clippy
	cargo fmt --all -- --check

clippy: stable_toolchain
	cargo clippy --all -- \
		-D "clippy::all" \
		-D clippy::pedantic \
		-D clippy::cargo \
		-D clippy::nursery \
		-W clippy::cast-possible-wrap \
		-W clippy::cognitive_complexity \
		-W clippy::missing_errors_doc \
		-W clippy::missing_panics_doc \
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
		--ignore "/*" \
		--ignore-not-existing \
		-o ./target/debug/$(OUTPUT)

integration_test:
	RUST_LOG="web_service_test=info" \
	LOCAL_DYNAMODB_URL=http://localhost:8000 \
		./target/debug/web_service_test localhost:8080 target_username localhost:8080 signer_username LocalDynamodbTable

nightly_toolchain:
	rustup update nightly
	rustup default nightly

refresh_database:
	docker-compose -f docker/test/docker-compose.yml kill
	docker-compose -f docker/test/docker-compose.yml up --build --detach

run_service_in_background:
	@./kill_web_service.sh
	CUSTOM_DOMAIN=localhost:8080 \
		DYNAMODB_TABLE=LocalDynamodbTable \
		LOCAL_DYNAMODB_URL=http://localhost:8000 \
		PROTOCOL=http \
		REGION=eu-west-1 \
		RUST_LOG="rocket=warn,web_service=info" \
		./target/debug/web_service &

scan_table:
	@if ! grep -F '[profile localhost]' <~/.aws/config; then \
		echo "[profile localhost]\nregion = us-east-1" >>~/.aws/config; \
	fi
	@if ! grep -F '[localhost]' <~/.aws/credentials; then \
		echo "[localhost]\naws_access_key_id = ANY_ACCESS_KEY_WILL_DO\naws_secret_access_key = ANY_SECRET_KEY_WILL_DO" >>~/.aws/credentials; \
	fi
	aws dynamodb scan --table-name LocalDynamodbTable --endpoint-url http://localhost:8000 --profile localhost

stable_toolchain:
	rustup update stable
	
test: \
	stable_toolchain \
	refresh_database \
	build \
	unit_test \
	run_service_in_background \
	integration_test

test_with_coverage: \
	nightly_toolchain \
	refresh_database \
	build_with_profile \
	unit_test \
	run_service_in_background \
	integration_test
	@./kill_web_service.sh

test_with_html_coverage: \
	test_with_coverage
	$(MAKE) grcov TYPE_PARAM=html OUTPUT=coverage

test_with_lcov: \
	test_with_coverage
	$(MAKE) grcov TYPE_PARAM=lcov OUTPUT=lcov.info

unit_test: stable_toolchain
	cargo test --workspace || true

watch: stable_toolchain
	cargo watch --clear -x 'build --workspace --all-targets'