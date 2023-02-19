.PHONY: \
	all \
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
	unit_test \
	watch

build_with_profile:
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
		-W clippy::no_effect_underscore_binding \
		-W clippy::multiple_crate_versions \
		-W clippy::future_not_send

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
	LOCAL_DYNAMODB_URL=http://localhost:8000 \
		./target/debug/examples/test http://localhost:8080

kill_service_running_in_background:
	pkill rust_lambda

lcov:
	$(MAKE) grcov TYPE_PARAM=lcov OUTPUT=lcov.info

nightly_toolchain:
	rustup update nightly
	rustup default nightly

refresh_database:
	docker-compose -f docker/test/docker-compose.yml kill
	docker-compose -f docker/test/docker-compose.yml up --build --detach

run_service_in_background:
	CUSTOM_DOMAIN=example.com \
		DYNAMODB_TABLE=table_name \
		LOCAL_DYNAMODB_URL=http://localhost:8000 \
		REGION=eu-west-1 \
		./target/debug/rust_lambda &

scan_table:
	@if ! grep -F '[profile localhost]' <~/.aws/config; then \
		echo "[profile localhost]\nregion = us-east-1" >>~/.aws/config; \
	fi
	@if ! grep -F '[localhost]' <~/.aws/credentials; then \
		echo "[localhost]\naws_access_key_id = ANY_ACCESS_KEY_WILL_DO\naws_secret_access_key = ANY_SECRET_KEY_WILL_DO" >>~/.aws/credentials; \
	fi
	aws dynamodb scan --table-name table_name --endpoint-url http://localhost:8000 --profile localhost

test: \
	nightly_toolchain \
	refresh_database \
	build_with_profile \
	run_service_in_background \
	integration_test \
	kill_service_running_in_background

unit_test:
	cargo test

watch:
	cargo watch --clear