import Lake
open Lake DSL
require aeneas from "/workspace/toolchain/aeneas-src/backends/lean"
package verification
lean_lib Adler2 where
  srcDir := "generated"
lean_lib Proofs
