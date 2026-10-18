import Sums
open Aeneas Aeneas.Std Result ControlFlow
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

def rawStep (s : Nat × Nat) (d : U8) : Nat × Nat :=
 (s.1+d.val, s.2+(s.1+d.val))
def rawFold (s : Nat × Nat) (l : List U8) : Nat × Nat := l.foldl rawStep s

def RawSafe : List U8 → Nat → Nat → Prop
 | [], _, _ => True
 | d::ds, a, b =>
   a+d.val ≤ 4294967295 ∧ b+(a+d.val) ≤ 4294967295 ∧ RawSafe ds (a+d.val) (b+(a+d.val))

theorem raw_safe_budget (l : List U8) (a b : Nat)
 (ha : a+255*l.length ≤ 4294967295)
 (hb : b+l.length*(a+255*l.length) ≤ 4294967295) : RawSafe l a b := by
 induction l generalizing a b with
 | nil => trivial
 | cons d ds ih =>
   have hd : d.val ≤ 255 := Nat.le_of_lt_succ d.hmax
   have hm := Nat.mul_le_mul_left ds.length hd
   simp only [List.length_cons] at ha hb
   have hsA : a+d.val ≤ 4294967295 := by omega
   have hsB : b+(a+d.val) ≤ 4294967295 := by nlinarith
   refine ⟨hsA, hsB, ih _ _ ?_ ?_⟩
   · omega
   · nlinarith

theorem rawFold_cast (l : List U8) (a b : Nat) :
 (((rawFold (a,b) l).1 : R), ((rawFold (a,b) l).2 : R)) = rfold ((a:R),(b:R)) l := by
 induction l generalizing a b with
 | nil => rfl
 | cons d ds ih =>
   simp only [rawFold, List.foldl_cons, rawStep, rfold, rstep]
   simpa only [Nat.cast_add, add_assoc, rawFold, rfold] using ih (a+d.val) (b+(a+d.val))

theorem tail_loop (s : Slice U8) (i : Nat) (a b : U32)
 (hsafe : RawSafe (s.val.drop i) a.val b.val) :
 adler2.algo.Adler32.compute_loop4 ⟨s,i⟩ a b
 ⦃ r => (r.1.val, r.2.val) = rawFold (a.val,b.val) (s.val.drop i) ⦄ := by
 unfold adler2.algo.Adler32.compute_loop4
 rw [loop]
 simp only [adler2.algo.Adler32.compute_loop4.body,
   core.slice.iter.IteratorSliceIter.next]
 split
 · rename_i h
   have h' : i < s.val.length := h
   rw [List.drop_eq_getElem_cons h'] at hsafe
   rcases hsafe with ⟨ha,hb,hrest⟩
   step as ⟨d, hd⟩
   change d.val = s.val[i].val at hd
   step with U32.add_spec (x:=a) (y:=d) (by simpa [U32.max,U32.numBits,hd] using ha) as ⟨a',ha'⟩
   step with U32.add_spec (x:=b) (y:=a') (by simpa [U32.max,U32.numBits,ha',hd] using hb) as ⟨b',hb'⟩
   change adler2.algo.Adler32.compute_loop4 ⟨s,i+1⟩ a' b' ⦃ _ ⦄
   have ih := tail_loop s (i+1) a' b' (by simpa [ha',hb',hd] using hrest)
   apply WP.spec_mono ih
   intro r hr
   rw [List.drop_eq_getElem_cons h']
   simpa only [rawFold, List.foldl_cons, rawStep, ha',hb',hd] using hr
 · rename_i h
   have hempty : s.val.drop i = [] := List.drop_eq_nil_of_le (by simpa using Nat.le_of_not_gt h)
   simp [hempty, rawFold, WP.spec_ok]
termination_by s.val.length - i
decreasing_by simp_all; omega
