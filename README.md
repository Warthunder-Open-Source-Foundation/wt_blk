# wt_blk
[![codecov](https://codecov.io/github/Warthunder-Open-Source-Foundation/wt_blk/graph/badge.svg?token=FWUP23Q2FH)](https://codecov.io/github/Warthunder-Open-Source-Foundation/wt_blk)
 <!-- [![LOC](https://tokei.rs/b1/github/Warthunder-Open-Source-Foundation/wt_blk)]([https://github.com/XAMPPRocky/tokei_rs](https://github.com/Warthunder-Open-Source-Foundation/wt_blk)). -->

> **War Thunder block-file and auxiliary binary formats parsing library**

Licensed under the [Apache 2.0](https://github.com/Warthunder-Open-Source-Foundation/wt_blk/blob/master/LICENSE) license

# Format documentation
(Vromf format)[https://warthunder-open-source-foundation.github.io/wt_blk/wt_blk/vromf/index.html]  
(Blk format)[https://warthunder-open-source-foundation.github.io/wt_blk/wt_blk/blk/index.html]  


# Bindings

### Python
Bindings are located in `wt_blk_pybindings` and published to [pypi](https://pypi.org/project/wt-blk-pybindings)

### WASM
Located in `wasm_bindings` and published to [npm](https://www.npmjs.com/package/wt_blk)

## For the end-user
For high-level consumption, please visit [the reference implementation](https://github.com/Warthunder-Open-Source-Foundation/wt_ext_cli).

## Library architecture and progress
![architecture](https://raw.githubusercontent.com/Warthunder-Open-Source-Foundation/wt_blk/assets/blk_conversions.svg)

## About the API
This crate provides low level parsing for various Binary formats that the game utilizes internally.
The public interfaces are currently overexposed and will be restricted once a stable API can be formed.
