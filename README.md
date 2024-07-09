# risc0-to-fflonk

## Instructions

To clone this repo with submodules:

```
git clone --recurse-submodules https://github.com/chainwayxyz/risc0-to-fflonk.git
cd risc0-to-fflonk
```

To create the work directory:
```
mkdir work_dir
```

### Download the setup parameters:
For `test-groth16` proof:
```
 wget -O ./proof/groth16/pot19.ptau https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_19.ptau
```

For `groth16` proof:
```
 wget -O ./proof/groth16/pot23.ptau https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_23.ptau
```

For `fflonk` proof:
```
 wget -O ./proof/groth16/pot24.ptau https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_24.ptau
```

### Setup:
To run the ceremony for `test-groth16` proof:
```
cd proof
docker build -f docker/test_ceremony.Dockerfile . -t test-snark-ceremony
docker run --rm -v $(pwd)/groth16:/test_ceremony/proof/groth16 test-snark-ceremony
```

To build the prover:
```
docker build -f docker/test_prover.Dockerfile . -t risc0-test-groth16-prover
```

For `groth16` proof, use the same commands without `test` prefix.

To run the preprocessing for `fflonk` proof:
```
docker build -f docker/fflonk_pp.Dockerfile . -t fflonk-preprocess
docker run --rm -v $(pwd)/fflonk:/test_preprocess/proof/fflonk fflonk-preprocess
```

To build the prover:
```
docker build -f docker/fflonk_prover.Dockerfile . -t risc0-fflonk-prover
```

### Execution:
To run the code for any `proof-type`:
```
cd ..
PROOF_TYPE=<proof-type> RISC0_WORK_DIR=./work_dir cargo run --release
```
