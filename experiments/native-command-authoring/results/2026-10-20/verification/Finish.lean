import Mix
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 2000000
namespace Verification
open adler2

lemma cast16_val {ty : UScalarTy} (x : UScalar ty) :
    (UScalar.cast .U16 x).val = x.val % 65536 := by
  change (x.bv.setWidth 16).toNat = _
  exact BitVec.toNat_setWidth 16 x.bv

def finish (tail : Slice U8) (a b : U32) : Result Adler32 := do
  let iter3 ← core.slice.Slice.iter tail
  let (a6,b5) ← algo.Adler32.compute_loop4 iter3 a b
  let i22 ← a6 % 65521#u32
  let i24 ← b5 % 65521#u32
  ok ⟨UScalar.cast .U16 i22, UScalar.cast .U16 i24⟩

@[step] theorem finish_correct (tail : Slice U8) (a b : U32)
    (ht : tail.val.length ≤ 3) (ha : a.val ≤ 327615) (hb : b.val ≤ 1506966) :
    finish tail a b ⦃ s => (s.a.val,s.b.val) = residues (run (a.val,b.val) tail.val) ⦄ := by
  unfold finish
  simp only [core.slice.Slice.iter,bind_ok]
  step with tail_correct tail 0 a b (by omega) (by omega) as ⟨a6,b5,hm⟩
  step as ⟨a7,ha7⟩
  step as ⟨b6,hb6⟩
  have ha7' : a7.val = a6.val % 65521 := ha7
  have hb6' : b6.val = b5.val % 65521 := hb6
  have hm' : ((a6.val:F),(b5.val:F)) = run (a.val,b.val) tail.val := by simpa only [List.drop_zero] using hm
  try simp only [spec_ok]
  rw [← hm']
  simp only [cast16_val,residues,ZMod.val_natCast]
  rw [Nat.mod_eq_of_lt (by omega), Nat.mod_eq_of_lt (by omega),ha7',hb6']
end Verification
