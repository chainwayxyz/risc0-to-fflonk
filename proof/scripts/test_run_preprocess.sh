#!/bin/bash

set -eoux

# Make all the node commands actually have 64GB of room
export NODE_OPTIONS="--max-old-space-size=65536"

# Preprocess the keys
(cd proof/fflonk; snarkjs fflonk setup $1 $2 test_fflonk.zkey)

# Export verification key
(cd proof/fflonk; snarkjs zkey export verificationkey test_fflonk.zkey test_fflonk_verification_key.json)

# Export solidity smart contract
(cd proof/fflonk; snarkjs zkey export solidityverifier test_fflonk.zkey test_fflonk_verifier.sol)
