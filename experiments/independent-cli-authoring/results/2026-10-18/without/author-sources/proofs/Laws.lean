import API
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000

theorem state_ext (s t : adler2.Adler32) (ha : s.a.val = t.a.val) (hb : s.b.val = t.b.val) : s = t := by
 have hA := UScalar.eq_of_val_eq ha
 have hB := UScalar.eq_of_val_eq hb
 cases s; cases t
 simp_all

def saveRestore (s : adler2.Adler32) : Result adler2.Adler32 := do
 let c ← adler2.Adler32.checksum s
 adler2.Adler32.from_checksum c

@[step] theorem codec_state_roundtrip (s : adler2.Adler32) : saveRestore s ⦃ t => t = s ⦄ := by
 unfold saveRestore
 step as ⟨c,hc⟩
 step as ⟨t,ha,hb⟩
 have haB := s.a.hmax
 have hbB := s.b.hmax
 change s.a.val < 65536 at haB
 change s.b.val < 65536 at hbB
 apply state_ext <;> omega

theorem codec_checksum_roundtrip (c : U32) :
 (do let s ← adler2.Adler32.from_checksum c
     adler2.Adler32.checksum s) ⦃ out => out = c ⦄ := by
 step as ⟨s,ha,hb⟩
 step as ⟨out,hout⟩
 apply UScalar.eq_of_val_eq
 omega

-- This precondition is exactly the representability of the concatenated slice.
def concatSlice (left right : Slice U8)
 (hcat : left.val.length + right.val.length ≤ Usize.max) : Slice U8 :=
 Slice.from (left.val ++ right.val) (by simpa using hcat)

theorem split_update (s : adler2.Adler32) (left right : Slice U8)
 (hcat : left.val.length + right.val.length ≤ Usize.max) :
 (do let mid ← adler2.Adler32.write_slice s left
     let sequential ← adler2.Adler32.write_slice mid right
     let whole ← adler2.Adler32.write_slice s (concatSlice left right hcat)
     ok (sequential,whole)) ⦃ sequential whole => sequential = whole ⦄ := by
 step as ⟨mid,hpartial⟩
 step as ⟨sequential,hsequential⟩
 step as ⟨whole,hwhole⟩
 have hpA := congrArg Prod.fst hpartial
 have hpB := congrArg Prod.snd hpartial
 change mid.a.val = (scalar s.a.val s.b.val left.val).1 at hpA
 change mid.b.val = (scalar s.a.val s.b.val left.val).2 at hpB
 have hval : (sequential.a.val,sequential.b.val) = (whole.a.val,whole.b.val) := by
   rw [hsequential,hwhole]
   simp only [concatSlice, Slice.from_val, scalar_append, hpA,hpB]
 apply state_ext
 · exact congrArg Prod.fst hval
 · exact congrArg Prod.snd hval

theorem save_restore_resume (s : adler2.Adler32) (left right : Slice U8)
 (hcat : left.val.length + right.val.length ≤ Usize.max) :
 (do let mid ← adler2.Adler32.write_slice s left
     let restored ← saveRestore mid
     let resumed ← adler2.Adler32.write_slice restored right
     let whole ← adler2.Adler32.write_slice s (concatSlice left right hcat)
     ok (resumed,whole)) ⦃ resumed whole => resumed = whole ⦄ := by
 step as ⟨mid,hpartial⟩
 step as ⟨restored,hrestore⟩
 subst restored
 step as ⟨resumed,hresumed⟩
 step as ⟨whole,hwhole⟩
 have hpA := congrArg Prod.fst hpartial
 have hpB := congrArg Prod.snd hpartial
 change mid.a.val = (scalar s.a.val s.b.val left.val).1 at hpA
 change mid.b.val = (scalar s.a.val s.b.val left.val).2 at hpB
 have hval : (resumed.a.val,resumed.b.val) = (whole.a.val,whole.b.val) := by
   rw [hresumed,hwhole]
   simp only [concatSlice, Slice.from_val, scalar_append, hpA,hpB]
 apply state_ext
 · exact congrArg Prod.fst hval
 · exact congrArg Prod.snd hval
