# pallet-witnet-oracle

A substrate pallet that enables sending data requests to the Witnet oracle from any Polkadot parachain, as well as
getting the results of the requests relayed back.  

##  Usage

### Use it from a different pallet

In the `[dependencies]` section of the  `Cargo.toml` file of the pallet in your working directory, add:

```toml
pallet-witnet-oracle = { default-features = false, git = "https://github.com/aesedepece/witnet-polkadot.git" }
```

Inside `[features]` > `std = [ ... ]` in `Cargo.toml`:

```toml
'pallet-witnet-oracle/std',
```
In your configuration trait, create a type that is bound by `pallet_witnet_oracle::Config` type (in `src/lib.rs`):

```rust
pub trait MyPalletConfig: frame_system::Config {
    type WitnetOracle: pallet_witnet_oracle::Pallet<pallet_witnet_oracle::Pallet::Config>;
}
```

### Add it to your own parachain

As with any other pallet, adding it to your parachain is pretty straightforward and only requires adding a few lines
here and there.

Inside `[dependencies]` in `runtime/Cargo.toml` and `node/Cargo.toml`:

```toml
pallet-witnet-oracle = { default-features = false, git = "https://github.com/aesedepece/witnet-polkadot.git" }
```

Inside `[features]` > `std = [ ... ]` in `runtime/Cargo.toml`:

```toml
'pallet-witnet-oracle/std',
```

In `runtime/src/lib.rs`:

```rust
parameter_types! {
	pub const MaxWitnetByteSize: u16 = 2048;
}

impl pallet_witnet_oracle::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type MaxByteSize = MaxWitnetByteSize;
    type TimeProvider = pallet_timestamp::Pallet<Test>;
}
```

Inside `construct_runtime!` in `runtime/src/lib.rs` as well:

```rust
Witnet: pallet_witnet_oracle,
```

Finally, inside `testnet_genesis` (or equivalent if mainnet) in your `node/src/chain_spec.rs`:
```rust
witnet: WitnetConfig {
    // Use the first account (aka Alice) as the sole operator by default
    operators: endowed_accounts.iter().cloned().take(1).collect()
} 
```
