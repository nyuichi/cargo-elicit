import Lake
open Lake DSL
package adlerVerification
require aeneas from "/workspace/toolchain/aeneas-src/backends/lean"
lean_lib Generated where
  roots := #[`Generated.Adler2]
@[default_target] lean_lib Verification where
  roots := #[`Proofs, `Codec, `Vector, `Algebra, `Lanes, `Bounds, `Inner, `Chunks, `Outer, `Iterators, `Next, `Mix, `Core, `Finish, `Api, `Laws]
