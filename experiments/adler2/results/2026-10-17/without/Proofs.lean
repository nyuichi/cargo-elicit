import Adler2
open Aeneas Aeneas.Std Result
namespace Verification

theorem new_correct : adler2.Adler32.new = ok ⟨1#u16, 0#u16⟩ := by rfl

def encode (s : adler2.Adler32) : U32 :=
  ⟨(s.b.bv.setWidth 32 <<< 16) ||| s.a.bv.setWidth 32⟩

def decode (x : U32) : adler2.Adler32 :=
  ⟨⟨x.bv.setWidth 16⟩, ⟨(x.bv >>> 16).setWidth 16⟩⟩

private theorem shl16 (x : U32) : x <<< 16#i32 = ok (⟨x.bv <<< 16⟩ : U32) := by
  change UScalar.shiftLeft_IScalar x 16#i32 = _
  simp [UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat, UScalarTy.numBits]

private theorem shr16 (x : U32) : x >>> 16#i32 = ok (⟨x.bv >>> 16⟩ : U32) := by
  change UScalar.shiftRight_IScalar x 16#i32 = _
  simp [UScalar.shiftRight_IScalar, UScalar.shiftRight, IScalar.toNat, UScalarTy.numBits]

theorem checksum_exec (s : adler2.Adler32) :
    adler2.Adler32.checksum s = ok (encode s) := by
  simp [adler2.Adler32.checksum, lift, shl16]
  rfl

theorem from_checksum_exec (x : U32) :
    adler2.Adler32.from_checksum x = ok (decode x) := by
  simp [adler2.Adler32.from_checksum, lift, shr16]
  rfl

theorem encode_value (s : adler2.Adler32) :
    (encode s).val = 65536 * s.b.val + s.a.val := by
  have ha : s.a.val < 65536 := s.a.bv.isLt
  have hb : s.b.val < 65536 := s.b.bv.isLt
  have ha32 : s.a.val < 4294967296 := by omega
  have hb32 : s.b.val < 4294967296 := by omega
  have hm : s.b.val * 65536 < 4294967296 := by omega
  change (s.b.bv.setWidth 32 <<< 16 ||| s.a.bv.setWidth 32).toNat = _
  simp only [BitVec.toNat_or, BitVec.toNat_shiftLeft, BitVec.toNat_setWidth,
    Nat.shiftLeft_eq]
  change ((s.b.val % 4294967296 * 65536) % 4294967296 |||
    s.a.val % 4294967296) = _
  rw [Nat.mod_eq_of_lt hb32, Nat.mod_eq_of_lt hm, Nat.mod_eq_of_lt ha32]
  simpa [Nat.shiftLeft_eq, Nat.mul_comm] using
    (Nat.shiftLeft_add_eq_or_of_lt (i := 16) ha s.b.val).symm

theorem decode_value (x : U32) :
    (decode x).a.val = x.val % 65536 ∧
    (decode x).b.val = x.val / 65536 := by
  have hx := x.bv.isLt
  change (x.bv.setWidth 16).toNat = x.bv.toNat % 65536 ∧
    ((x.bv >>> 16).setWidth 16).toNat = x.bv.toNat / 65536
  simp only [BitVec.toNat_setWidth,
    BitVec.toNat_ushiftRight, Nat.shiftRight_eq_div_pow]
  norm_num at hx ⊢
  omega

theorem checksum_correct (s : adler2.Adler32) :
    ∃ x, adler2.Adler32.checksum s = ok x ∧ x.val = 65536 * s.b.val + s.a.val :=
  ⟨encode s, checksum_exec s, encode_value s⟩

theorem from_checksum_correct (x : U32) :
    ∃ s, adler2.Adler32.from_checksum x = ok s ∧
      s.a.val = x.val % 65536 ∧ s.b.val = x.val / 65536 :=
  ⟨decode x, from_checksum_exec x, decode_value x⟩

theorem decode_encode (s : adler2.Adler32) : decode (encode s) = s := by
  have h := decode_value (encode s)
  rw [encode_value] at h
  have ha : s.a.val < 65536 := s.a.bv.isLt
  have hb : s.b.val < 65536 := s.b.bv.isLt
  have heqa : (decode (encode s)).a.val = s.a.val := by omega
  have heqb : (decode (encode s)).b.val = s.b.val := by omega
  have hsa := UScalar.eq_of_val_eq heqa
  have hsb := UScalar.eq_of_val_eq heqb
  calc decode (encode s) = ⟨(decode (encode s)).a, (decode (encode s)).b⟩ := rfl
       _ = ⟨s.a, s.b⟩ := congrArg₂ adler2.Adler32.mk hsa hsb
       _ = s := by cases s; rfl

theorem encode_decode (x : U32) : encode (decode x) = x := by
  apply UScalar.eq_of_val_eq
  rw [encode_value]
  have h := decode_value x
  omega

theorem codec_state_roundtrip (s : adler2.Adler32) :
    (do let x ← adler2.Adler32.checksum s; adler2.Adler32.from_checksum x) = ok s := by
  rw [checksum_exec]
  simp only [bind_ok]
  rw [from_checksum_exec, decode_encode]

theorem codec_checksum_roundtrip (x : U32) :
    (do let s ← adler2.Adler32.from_checksum x; adler2.Adler32.checksum s) = ok x := by
  rw [from_checksum_exec]
  simp only [bind_ok]
  rw [checksum_exec, encode_decode]

/-- Save/restore preserves the exact subsequent computation, including its effects.
    This theorem alone does not establish successful execution of write_slice. -/
theorem resume_equivalence (s : adler2.Adler32) (bytes : Slice U8) :
    (do let x ← adler2.Adler32.checksum s
        let restored ← adler2.Adler32.from_checksum x
        adler2.Adler32.write_slice restored bytes) =
      adler2.Adler32.write_slice s bytes := by
  simp only [checksum_exec, bind_ok, from_checksum_exec, decode_encode]

def normalize (s : Nat × Nat) : Nat × Nat := (s.1 % 65521, s.2 % 65521)

def step (s : Nat × Nat) (x : U8) : Nat × Nat :=
  let a := (s.1 + x.val) % 65521
  (a, (s.2 + a) % 65521)

/-- Empty inputs normalize both halves. There is no normalized-seed assumption. -/
def scalar (s : Nat × Nat) (xs : List U8) : Nat × Nat :=
  xs.foldl step (normalize s)

theorem normalize_idem (s : Nat × Nat) : normalize (normalize s) = normalize s := by
  simp [normalize]

theorem normalize_step (s : Nat × Nat) (x : U8) : normalize (step s x) = step s x := by
  simp [normalize, step]

theorem step_normalize (s : Nat × Nat) (x : U8) :
    step (normalize s) x = step s x := by
  simp [step, normalize, Nat.mod_add_mod]

theorem normalize_fold (xs : List U8) (s : Nat × Nat)
    (hs : normalize s = s) : normalize (xs.foldl step s) = xs.foldl step s := by
  induction xs generalizing s with
  | nil => exact hs
  | cons x xs ih => exact ih (step s x) (normalize_step s x)

theorem scalar_empty (s : Nat × Nat) : scalar s [] = normalize s := rfl

theorem scalar_cons (s : Nat × Nat) (x : U8) (xs : List U8) :
    scalar s (x :: xs) = scalar (step s x) xs := by
  simp [scalar, step_normalize, normalize_step]

theorem scalar_split (s : Nat × Nat) (xs ys : List U8) :
    scalar s (xs ++ ys) = scalar (scalar s xs) ys := by
  unfold scalar
  rw [List.foldl_append, normalize_fold xs (normalize s) (normalize_idem s)]

/-- Slice split laws must require the concatenation to be representable. -/
theorem scalar_split_representable (s : Nat × Nat) (xs ys : Slice U8)
    (_hconcat : xs.val.length + ys.val.length ≤ Usize.max) :
    scalar s (xs.val ++ ys.val) = scalar (scalar s xs.val) ys.val :=
  scalar_split s xs.val ys.val

/-- The outstanding contract for the original optimized update. -/
def UpdateContract : Prop := ∀ (s : adler2.Adler32) (xs : Slice U8),
  ∃ t, adler2.Adler32.write_slice s xs = ok t ∧
    (t.a.val, t.b.val) = scalar (s.a.val, s.b.val) xs.val

/-- Explicitly conditional: the optimized UpdateContract has not been proved. -/
theorem adler32_slice_of_update (h : UpdateContract) (xs : Slice U8) :
    ∃ x, adler2.adler32_slice xs = ok x ∧
      x.val = 65536 * (scalar (1, 0) xs.val).2 + (scalar (1, 0) xs.val).1 := by
  obtain ⟨t, ht, hv⟩ := h ⟨1#u16, 0#u16⟩ xs
  refine ⟨encode t, ?_, ?_⟩
  · simp only [adler2.adler32_slice, new_correct, bind_ok, ht, checksum_exec]
  · rw [encode_value]
    have ha := congrArg Prod.fst hv
    have hb := congrArg Prod.snd hv
    simpa using congrArg (fun p : Nat × Nat => 65536 * p.2 + p.1) hv

/-- The actual indexed loader succeeds for every four-byte slice. -/
theorem four_byte_loader (xs : Slice U8) (x0 x1 x2 x3 : U8)
    (hx : xs.val = [x0, x1, x2, x3]) :
    adler2.algo.U32X4.from xs = ok (Array.make 4#usize
      [core.convert.num.FromU32U8.from x0, core.convert.num.FromU32U8.from x1,
       core.convert.num.FromU32U8.from x2, core.convert.num.FromU32U8.from x3]) := by
  simp [adler2.algo.U32X4.from, Slice.index_usize, Slice.getElem?_Usize_eq, hx, lift]

/-- Numerical bound required for each SIMD lane in a full chunk.
    The loop still needs to be related to this closed-form bound. -/
theorem lane_chunk_bound (n a b : Nat) (hn : n ≤ 5552)
    (ha : a ≤ 65520) (hb : b ≤ 65520) :
    b + n * a + 255 * (n * (n + 1) / 2) ≤ 4294967295 := by
  have hmul : n * a ≤ 5552 * 65520 := Nat.mul_le_mul hn ha
  have htri : n * (n + 1) ≤ 5552 * 5553 := Nat.mul_le_mul hn (by omega)
  omega

/-- The original seed may use all sixteen bits, including values above MOD. -/
theorem seed_chunk_bound (a b : Nat) (ha : a ≤ 65535) (hb : b ≤ 65535) :
    b + 22208 * a ≤ 4294967295 := by omega

#print axioms four_byte_loader
#print axioms lane_chunk_bound
#print axioms seed_chunk_bound
#print axioms new_correct
#print axioms checksum_correct
#print axioms from_checksum_correct
#print axioms codec_state_roundtrip
#print axioms codec_checksum_roundtrip
#print axioms resume_equivalence
#print axioms scalar_cons
#print axioms scalar_split_representable
#print axioms adler32_slice_of_update
end Verification
