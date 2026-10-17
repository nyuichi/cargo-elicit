import CoreLemmas
open Aeneas Aeneas.Std Result
set_option maxRecDepth 2048
namespace Verification

theorem chunk_loop0_empty (i : Usize) (a b : U32)
    (av bv : adler2.algo.U32X4) (rem : Slice U8) :
    adler2.algo.Adler32.compute_loop0 i ⟨[], rem⟩ a b av bv = ok (b,av,bv) := by
  unfold adler2.algo.Adler32.compute_loop0
  unfold loop
  simp only [adler2.algo.Adler32.compute_loop0.body,
    core.slice.iter.IteratorChunksExact.next, bind_ok]
  simp

theorem chunk_loop1_empty (av bv : adler2.algo.U32X4) (rem : Slice U8) :
    adler2.algo.Adler32.compute_loop1 ⟨[], rem⟩ av bv = ok (av,bv) := by
  unfold adler2.algo.Adler32.compute_loop1
  unfold loop
  simp only [adler2.algo.Adler32.compute_loop1.body,
    core.slice.iter.IteratorChunksExact.next, bind_ok]
  simp

#print axioms chunk_loop0_empty
#print axioms chunk_loop1_empty
end Verification
