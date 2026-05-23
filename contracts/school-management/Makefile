-include .env

all: test

test: build
	cargo test

debug: build
	RUST_BACKTRACE=full cargo test
	
t: test
	RUST_BACKTRACE=1 cargo test  -- --nocapture

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean 

upload: build
	@echo "📦 Uploading contract..."
	@stellar contract install \
	  --network ${TESTNET} \
	  --source ${SOURCE_TESTNET} \
	  --wasm ../../target/wasm32v1-none/release/liquidity_provider_contract.wasm > upload.log 2>&1 || (cat upload.log && exit 1)
	@grep -oE "[a-f0-9]{64}" upload.log | tail -n 1 > wasm_hash.txt
	@echo "✅ Wasm hash saved to wasm_hash.txt"
	@cat wasm_hash.txt

deploy:
	stellar contract deploy \
	  --wasm-hash ${} \
	  --network ${TESTNET} \
	  --source-account ${SOURCE_TESTNET} \
	  -- \
	  --usdc_asset ${USDC_TESTNET_ASSET} \
	  --settings_contract ${LM_CONTRACT_ADDRESS}

interact:
	stellar contract invoke \
		--id ${} \
		--source-account ${SOURCE_TESTNET} \
		--network ${TESTNET} \
		--fee ${FEE} \
		-- \
		get_token_balance \
		--user ${ADMIN_TESTNET}
