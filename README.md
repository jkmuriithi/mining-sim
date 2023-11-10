# Blockchain Mining Strategy Simulator

A library for simulating strategic blockchain mining outcomes based on game theoretical models. 

## Built With
- [Rust (stable)](https://www.rust-lang.org/)
- [rand](https://docs.rs/rand/latest/rand/)

## Based on
The development of this crate was guided by the content of the following
research papers and theses.
- [Searching for Optimal Strategies in Proof-of-Stake Mining Games with Access to External Randomness](https://thesis.anthonyhein.com/)
- [Proof-of-Stake Mining Games with Perfect
Randomness](https://arxiv.org/abs/2107.04069)

## Roadmap
- ~~Create blockchain framework~~
- ~~Create simulation framework and simulation builder interface~~
- Implement known strategies
  - ~~Honest~~
  - ~~Selfish Mining~~
  - Nothing At Stake
  - N-Deficit

## Building Locally
Making sure you have a working stable Rust install (with Cargo support), you
should be able to build this crate locally as follows:
```bash
git clone https://github.com/jkmuriithi/strategicmining.git
cd strategicmining
cargo build
```

Project documentation can be viewed via the `cargo doc` command:
```bash
cargo doc --open
```
