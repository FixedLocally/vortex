# Vortex
Solana vote and block rewards tracking tool. We used to have the OG [Vortex](https://app.vx.tools) but that doesn't appear to work anymore.
## Prerequisites
- Install Rust.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- Have a Solana RPC with full API support and [Yellowstone GRPC](https://github.com/rpcpool/yellowstone-grpc/).
- Have a MariaDB instance. MySQL may work but is untested.
## Building
```bash
cargo build --release
```
## Running
Built executables can be found in `./target/release`.
- `vortex` - Data collector, needs to be running 24/7.
- `populate-leader-schedule` - Stores the current epoch's leader schedule to the database. Needs to be run every epoch.
- `populate-optimal-vote` - Stores calculated optimals to the database. Needs to be run every couple of minutes.
- `rev-report <epoch>` - Prints a table of median/average/total fee/tips and the respective percentage relative to the global value for the given epoch.
- `vote-report <vote> <epoch> [bucket]` - Prints the given vote account's vote performance in the form of `bucket votes_cast/slots total_latency credits`, where each bucket consists of 1000 slots. Use `OPTIMAL` for the optimal vote records.