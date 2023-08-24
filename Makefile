BUILDDIR ?= $(CURDIR)/build
STRIDE_HOME ?= $(CURDIR)/../stride

KEY ?= ""
VALUE ?= ""

check-key-value:
	ifndef KEY
	$(error KEY is not set)
	endif

	ifndef VALUE
	$(error VALUE is not set)
	endif

.PHONY: build
build:
	cargo wasm

build-debug:
	cargo wasm-debug

build-optimized:
	docker run --rm -v "$(CURDIR)":/code \
		--mount type=volume,source="$(notdir $(CURDIR))_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.12.12

validate:
	cosmwasm-check ./artifacts/ica_oracle.wasm

# Uploads the contract to osmosis
store-contract: 
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/store_contract.sh

# Instantiates the contract directly with the osmosis dockernet validator as the admin
instantiate-contract: 
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/instantiate_contract.sh

# Adds a metric directly to the contract from the osmosis dockernet validator
add-metric:
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/add_metric.sh

# Add's an oracle to Stride and instantiates the contract with an ICA
add-oracle:
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/add_oracle.sh

# Queries all oracles on Stride
query-oracles:
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/query_oracles.sh

# Queries all metrics stored in the contract
query-metrics:
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/query_metrics.sh

# Queries all ICA errors
query-errors:
	@STRIDE_HOME=$(STRIDE_HOME) bash scripts/query_errors.sh

# Stores and instantiates the contract directly (bypassing stride)
# allowing metrics to be pushed manually
setup-dockernet-manual:
	@$(MAKE) store-contract && sleep 5
	@$(MAKE) instantiate-contract && sleep 5
	@$(MAKE) add-metric

# Stores and instantiates the contract through stride so
# that metrics can only be pushed when the redemption rate updates
setup-dockernet-dynamic:
	@$(MAKE) store-contract && sleep 5
	@$(MAKE) add-oracle