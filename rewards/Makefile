include ../Makefile

LIB_NAME=serum_rewards_program
#TEST_PROGRAM_ID="514xCmpQprVrNH2xJW46D5wecDNZemU458dTUuDUiEVM"
#TEST_REGISTRY_PROGRAM_ID="3bB3Utm2hW9G2jbz1o2Qydt157Q8nqa6Dw47woFf1TNm"
#TEST_DEX_PROGRAM_ID="DsWkk5EXVf3FV4sqdbdvkcUT7N1SooR5RocL1DbbJD4B"

.PHONY: test deploy-dex deploy-registry test-program

#
# Rewards testing requires the DEX and the Registry to be deployed in addition
# to the Rewards program, so we override the parent Makefile.
#
test: deploy-dex deploy-registry deploy-super test-program
	@ # no-op

deploy-dex: build-dex
	cd ../ && ./do.sh build dex && cd rewards
	$(eval TEST_DEX_PROGRAM_ID=$(shell solana deploy ../dex/target/bpfel-unknown-unknown/release/serum_dex.so --use-deprecated-loader | jq .programId -r))

deploy-registry:
	$(eval TMP=$(shell make -s -C ../registry deploy))
	$(eval TEST_REGISTRY_PROGRAM_ID=$(shell echo $(TMP) | sed 's/.*{programId: \(.*\)}.*/\1/g'))

test-program:
	make TEST_PROGRAM_ID=$(TEST_PROGRAM_ID) TEST_DEX_PROGRAM_ID=$(TEST_DEX_PROGRAM_ID) TEST_REGISTRY_PROGRAM_ID=$(TEST_REGISTRY_PROGRAM_ID) test-program-super
