check:
	USERP=0000000000000000 cargo check;


# USERP=********** make release_build
run:
	RUST_BACKTRACE=1 cargo run;


# Run tests
test:
	USERP=0000000000000000 cargo test;


# cargo install cargo-llvm-cov
test_coverage_in_browser:
	USERP=0000000000000000 cargo llvm-cov --html --open


# USERP=********** make release_build
release_build:
	cargo build --release --features hardening;
# 	cargo build --release;


# USERP=********** make release_build
release_run: release_build
# 	RUST_BACKTRACE=1 ./target/release/check_health_domain
	./target/release/check_health_domain


search_private_data_in_binary: release_build
	strings target/release/check_health_domain | grep -E "$(USERP)" -B 20 -A 20 || echo "Clean";
	strings target/release/check_health_domain | grep -E "@gmail|@mail" -B 20 -A 20 || echo "Clean";


# USERP=********** TARGET=user@server make deploy
deploy: check search_private_data_in_binary release_build
	scp target/release/check_health_domain $(TARGET):~/

#################################################

install_dev_depends:
	sudo apt update -y;
	sudo apt install libssl-dev -y;


# Cron command
show_cron_command:
	echo "1 * * * * /root/check_health_domain >> /var/log/check_health_domain.log 2>&1";
