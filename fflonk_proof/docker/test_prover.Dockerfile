# syntax=docker/dockerfile:1.4
FROM rust:1.74.0 AS dependencies

WORKDIR /src/

# APT deps
RUN apt -qq update && \
  apt install -y -q apt-transport-https build-essential clang cmake curl gnupg libgmp-dev libsodium-dev m4 nasm nlohmann-json3-dev npm

WORKDIR /src/

# Build and install circom
RUN git clone https://github.com/iden3/circom.git && \
  cd circom && \
  git checkout e60c4ab8a0b55672f0f42fbc68a74203bdb6a700 && \
  cargo install --path circom

ENV CC=clang
ENV CXX=clang++

# Build rapidsnark
RUN git clone https://github.com/iden3/rapidsnark.git && \
  cd rapidsnark && \
  git checkout 547bbda73bea739639578855b3ca35845e0e55bf

WORKDIR /src/rapidsnark/
# Copied from: https://github.com/iden3/rapidsnark/blob/main/tasksfile.js
# to bypass the taskfile dep in NPM being dropped
RUN git submodule init && \
  git submodule update && \
  mkdir -p build && \
  (cd depends/ffiasm && npm install) && \
  cd build/ && \
  node ../depends/ffiasm/src/buildzqfield.js -q 21888242871839275222246405745257275088696311157297823662689037894645226208583 -n Fq && \
  node ../depends/ffiasm/src/buildzqfield.js -q 21888242871839275222246405745257275088548364400416034343698204186575808495617 -n Fr && \
  nasm -felf64 fq.asm && \
  nasm -felf64 fr.asm && \
  g++ -I. -I../src -I../depends/ffiasm/c -I../depends/json/single_include ../src/main_prover.cpp ../src/binfile_utils.cpp ../src/zkey_utils.cpp ../src/wtns_utils.cpp ../src/logger.cpp ../depends/ffiasm/c/misc.cpp ../depends/ffiasm/c/naf.cpp ../depends/ffiasm/c/splitparstr.cpp ../depends/ffiasm/c/alt_bn128.cpp fq.cpp fq.o fr.cpp fr.o -o prover -fmax-errors=5 -std=c++17 -pthread -lgmp -lsodium -O3 -fopenmp &&\
  cp ./prover /usr/local/sbin/rapidsnark

# Cache ahead of the larger build process
FROM dependencies AS builder

WORKDIR /src/
COPY groth16/circuits/aliascheck.circom ./groth16/circuits/aliascheck.circom
COPY groth16/circuits/binsum.circom ./groth16/circuits/binsum.circom
COPY groth16/circuits/bitify.circom ./groth16/circuits/bitify.circom
COPY groth16/circuits/comparators.circom ./groth16/circuits/comparators.circom
COPY groth16/circuits/compconstant.circom ./groth16/circuits/compconstant.circom
COPY groth16/circuits/risc0.circom ./groth16/circuits/risc0.circom
COPY groth16/circuits/test_journal.circom ./groth16/circuits/test_journal.circom
COPY groth16/circuits/test_stark_verify.circom ./groth16/circuits/test_stark_verify.circom
COPY groth16/circuits/test_verify_for_guest.circom ./groth16/circuits/test_verify_for_guest.circom
COPY groth16/circuits/sha256 ./groth16/circuits/sha256

# Build the witness generation
RUN (cd groth16/circuits; circom --c --r1cs test_verify_for_guest.circom) && \
  sed -i 's/g++/clang++/' groth16/circuits/test_verify_for_guest_cpp/Makefile && \
  sed -i 's/O3/O0/' groth16/circuits/test_verify_for_guest_cpp/Makefile && \
  (cd groth16/circuits/test_verify_for_guest_cpp; make)

# Download the proving key
# RUN (cd groth16; wget https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_23.ptau)

# RUN (cd groth16; snarkjs g16s verify_for_guest.r1cs pot23.ptau circuit.zkey)

# Create a final clean image with all the dependencies to perform stark->snark
FROM ubuntu:jammy-20231211.1@sha256:bbf3d1baa208b7649d1d0264ef7d522e1dc0deeeaaf6085bf8e4618867f03494 AS prover

RUN apt update -qq && \
  apt install -y libsodium23 nodejs npm && \
  npm install -g snarkjs@0.7.3

COPY scripts/test_prover.sh /app/test_prover.sh
COPY --from=builder /usr/local/sbin/rapidsnark /usr/local/sbin/rapidsnark
COPY --from=builder /src/groth16/circuits/test_verify_for_guest_cpp/test_verify_for_guest /app/test_verify_for_guest
COPY --from=builder /src/groth16/circuits/test_verify_for_guest_cpp/test_verify_for_guest.dat /app/test_verify_for_guest.dat
COPY groth16/test_verify_for_guest_final.zkey /app/test_verify_for_guest_final.zkey

WORKDIR /app
RUN chmod +x test_prover.sh
RUN ulimit -s unlimited

ENTRYPOINT ["/app/test_prover.sh"]