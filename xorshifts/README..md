# XORshift Random Number generators

This small create contains a few Rust implementation of Xorshift & Xoroshiro _("XOR-Rotate-Shift-Rotate")_ random number generators. <br>
One goal was purely pedagogical: I wanted to know how (pseudo) random number generators work under the hood, and to have the same implementation work both
for WASM & for desktop.<br>
Moreover, it made testing different RNGs easier, as they implement my custom traits by construction.<br>
The following RNGs are implemented:
* Xorshift64
* Xoroshiro128p
* Xoshiro256p
* Xoshiro256pp
# Seeding
For WASM I use the UTC time (mapped to epoch) to seed the RNG.<br>
On desktop, I use the rand::ThreadRng (rand::rng()) to seed from the OS. <br> Indeed, this is how the rand crate also seeds their rngs

## Sources
The implementations is based on the one found on the C implementations [https://prng.di.unimi.it/ ](https://prng.di.unimi.it/), which can also be found
on [wikipedia](https://en.wikipedia.org/wiki/Xorshift ).<br>
## Difference with C/C++ implementations
In Rust, additions or bitwise left shifts can lead to overflow, which will trigger integer *overflow* error. The workaround is
use "wrapping_add" as well as the "unbounded_shl" (shift-left) to mimick what you get in C & avoid undef-behaviour.<br> Indeed, these will put the "overflowing" bits to zero.
For instance the "rol64" operation, which "wraps a number from right to left", so that the "overflowing numbers appear on the right looks in C like
```C
uint64_t rol64(uint64_t x, size_t k) 
{
    return (x << k) | (x >> (64 - k));
}
```
will in Rust become:
```rust
fn rol64(x: u64, k: u32) -> u64
{
	(x.unbounded_shl(k)) | (x.unbounded_shr(64 - k))
}
```