import ElicitContracts

namespace AdlerSpec

/-- Normalized outputs fit the original storage fields. -/
theorem normalized_representable (a b : Nat) (h : Normalized a b) :
    Representable a b := by
  constructor
  · exact Nat.lt_trans h.1 (by decide)
  · exact Nat.lt_trans h.2 (by decide)

/-- No normalized-seed assumption is imposed by the invariant. -/
theorem max_halves_representable : Representable 65535 65535 := by
  unfold Representable
  decide

end AdlerSpec

namespace AdlerSpec

/-- Mathematical preservation; the original add_assign body needs its own proof. -/
theorem lane_add_preserves (v w : Lanes) (h : AddSafe v w) :
    LanesRepresentable (fun i => v i + w i) := h

/-- Mathematical preservation; the original mul_assign body needs its own proof. -/
theorem lane_mul_preserves (v : Lanes) (k : Nat) (h : MulSafe v k) :
    LanesRepresentable (fun i => v i * k) := h

/-- A nonzero representable divisor bounds all remainder lanes. -/
theorem lane_rem_preserves (v : Lanes) (k : Nat)
    (hk : 0 < k) (hbound : k < 4294967296) :
    LanesRepresentable (fun i => v i % k) := by
  intro i
  exact Nat.lt_trans (Nat.mod_lt _ hk) hbound

end AdlerSpec

open Aeneas Aeneas.Std Result ElicitGenerated

namespace AdlerSpec

/-- Successful lane-wise remainder of the original translated Rust method. -/
theorem rem_assign_correct : ElicitContracts.c_f321f3834d6c15c0 := by
  intro self quotient hq
  have hlen : self.val.length = 4 := self.property
  obtain ⟨a,b,c,d,hs⟩ : ∃ a b c d, self.val = [a,b,c,d] := by
    generalize self.val = xs at *
    rcases xs with _ | ⟨a,xs⟩ <;> simp_all
    rcases xs with _ | ⟨b,xs⟩ <;> simp_all
    rcases xs with _ | ⟨c,xs⟩ <;> simp_all
    rcases xs with _ | ⟨d,xs⟩ <;> simp_all
  simp [algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign,
    Array.index_usize, Array.update, hs, HMod.hMod, UScalar.rem, hq]
  simp only [UScalar.val]
  exact ⟨rfl, rfl, rfl, rfl⟩

end AdlerSpec

namespace AdlerSpec

/-- Successful exact products of the original checked four-lane method. -/
theorem mul_assign_correct : ElicitContracts.c_6c7496278e307fe7 := by
  intro self rhs hsafe
  have hlen : self.val.length = 4 := self.property
  obtain ⟨a,b,c,d,hs⟩ : ∃ a b c d, self.val = [a,b,c,d] := by
    generalize self.val = xs at *
    rcases xs with _ | ⟨a,xs⟩ <;> simp_all
    rcases xs with _ | ⟨b,xs⟩ <;> simp_all
    rcases xs with _ | ⟨c,xs⟩ <;> simp_all
    rcases xs with _ | ⟨d,xs⟩ <;> simp_all
  simp only [hs, List.mem_cons, List.not_mem_nil, or_false, forall_eq_or_imp,
    forall_eq] at hsafe
  rcases hsafe with ⟨ha, hb, hc, hd⟩
  simp only [HMul.hMul] at ha hb hc hd
  simp [algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign,
    Array.index_usize, Array.update, hs, HMul.hMul, UScalar.mul,
    UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds,
    ha, hb, hc, hd]

end AdlerSpec

namespace AdlerSpec

/-- Successful exact sums of the original checked four-lane method. -/
theorem add_assign_correct : ElicitContracts.c_20b5198a3591154d := by
  intro self other hsafe
  have hlen : self.val.length = 4 := self.property
  obtain ⟨a,b,c,d,hs⟩ : ∃ a b c d, self.val = [a,b,c,d] := by
    generalize self.val = xs at *
    rcases xs with _ | ⟨a,xs⟩ <;> simp_all
    rcases xs with _ | ⟨b,xs⟩ <;> simp_all
    rcases xs with _ | ⟨c,xs⟩ <;> simp_all
    rcases xs with _ | ⟨d,xs⟩ <;> simp_all
  have holen : other.val.length = 4 := other.property
  obtain ⟨e,f,g,h,ho⟩ : ∃ e f g h, other.val = [e,f,g,h] := by
    generalize other.val = xs at *
    rcases xs with _ | ⟨e,xs⟩ <;> simp_all
    rcases xs with _ | ⟨f,xs⟩ <;> simp_all
    rcases xs with _ | ⟨g,xs⟩ <;> simp_all
    rcases xs with _ | ⟨h,xs⟩ <;> simp_all
  simp [hs, ho] at hsafe
  rcases hsafe with ⟨ha, hb, hc, hd⟩
  simp only [HAdd.hAdd] at ha hb hc hd
  simp [algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign,
    Array.index_usize, Array.update, hs, ho, HAdd.hAdd, UScalar.add,
    UScalar.tryMk, UScalar.tryMkOpt, UScalar.check_bounds,
    ha, hb, hc, hd]

end AdlerSpec

namespace AdlerSpec

/-- The original default constructor succeeds with the exact Adler seed. -/
theorem default_correct : ElicitContracts.c_2fa15a7d1ba5a684 := by
  intro _
  exact ⟨{ a := 1#u16, b := 0#u16 }, rfl, rfl, rfl⟩

end AdlerSpec

namespace AdlerSpec

/-- The original checksum succeeds for every representable pair of halves. -/
theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  simp [Adler32.checksum, HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar,
    UScalar.shiftLeft, lift, -BitVec.shiftLeft_eq]
  have ha : self.a.val < 65536 := self.a.bv.isLt
  have hb : self.b.val < 65536 := self.b.bv.isLt
  simp only [UScalar.val]
  rw [BitVec.shiftLeft_eq, BitVec.toNat_shiftLeft]
  have hw : (self.b.bv.setWidth 32).toNat = self.b.bv.toNat :=
    BitVec.toNat_setWidth_of_le (by decide)
  rw [hw]
  have hfit : self.b.bv.toNat <<< 16 < 4294967296 := by
    simp only [Nat.shiftLeft_eq]
    omega
  change (self.b.bv.toNat <<< 16) % 4294967296 ||| self.a.bv.toNat = 65536 * self.b.bv.toNat + self.a.bv.toNat
  rw [Nat.mod_eq_of_lt hfit]
  change self.a.bv.toNat < 2^16 at ha
  rw [← Nat.shiftLeft_add_eq_or_of_lt (i := 16) ha]
  simp [Nat.shiftLeft_eq, Nat.mul_comm]

end AdlerSpec

namespace AdlerSpec

/-- Decode every u32, including values with unnormalized halves. -/
theorem from_checksum_correct : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  simp [Adler32.from_checksum, lift, HShiftRight.hShiftRight,
    UScalar.shiftRight_IScalar, UScalar.shiftRight, -BitVec.ushiftRight_eq]
  have hsum : sum.val < 4294967296 := sum.bv.isLt
  simp only [UScalar.val]
  rw [BitVec.ushiftRight_eq, BitVec.toNat_ushiftRight]
  simp only [Nat.shiftRight_eq_div_pow]
  exact Nat.mod_eq_of_lt (by change sum.val / 65536 < 65536; omega)

end AdlerSpec

/- Codec laws over the original translated methods. -/
namespace AdlerSpec

theorem restore_checksum (self : Adler32) :
    ∃ sum, Adler32.checksum self = ok sum ∧ Adler32.from_checksum sum = ok self := by
  obtain ⟨sum, hc, hv⟩ := checksum_correct self trivial
  obtain ⟨decoded, hd, ha, hb⟩ := from_checksum_correct sum trivial
  have hsa : self.a.val < 65536 := self.a.bv.isLt
  have hsb : self.b.val < 65536 := self.b.bv.isLt
  have hea : decoded.a = self.a := UScalar.val_eq_imp _ _ (by omega)
  have heb : decoded.b = self.b := UScalar.val_eq_imp _ _ (by omega)
  have he : decoded = self := by cases decoded; cases self; simp_all
  exact ⟨sum, hc, he ▸ hd⟩

theorem checksum_restore (sum : U32) :
    ∃ state, Adler32.from_checksum sum = ok state ∧ Adler32.checksum state = ok sum := by
  obtain ⟨state, hd, ha, hb⟩ := from_checksum_correct sum trivial
  obtain ⟨result, hc, hv⟩ := checksum_correct state trivial
  have he : result = sum := UScalar.val_eq_imp _ _ (by omega)
  exact ⟨state, hd, he ▸ hc⟩

end AdlerSpec

namespace AdlerSpec

/-- The original constructor widens exactly the first four bytes. -/
theorem lanes_from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes hlen
  obtain ⟨a,b,c,d,tail,hs⟩ : ∃ a b c d tail, bytes.val = a::b::c::d::tail := by
    generalize bytes.val = xs at *
    rcases xs with _ | ⟨a,xs⟩ <;> simp_all
    rcases xs with _ | ⟨b,xs⟩ <;> simp_all
    rcases xs with _ | ⟨c,xs⟩ <;> simp_all
    rcases xs with _ | ⟨d,xs⟩ <;> simp_all
  simp [algo.U32X4.from, Slice.index_usize, hs, lift]

end AdlerSpec

namespace AdlerSpec

/-- The original finish method widens the exact checksum without changing it. -/
theorem finish_correct : ElicitContracts.c_375e1cdaf1f944d2 := by
  intro self _
  obtain ⟨sum, hc, hv⟩ := checksum_correct self trivial
  refine ⟨core.convert.num.FromU64U32.from sum, ?_, ?_⟩
  · simp [Adler32.Insts.CoreHashHasher.finish, hc]
  · simpa using hv

end AdlerSpec

namespace AdlerSpec

/-- The original new method succeeds through its original default call. -/
theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  simpa only [ElicitContracts.c_8055bec449b87b3d,
    ElicitContracts.c_2fa15a7d1ba5a684, Adler32.new] using default_correct

end AdlerSpec

namespace AdlerSpec

theorem scalar_fold_normalized (bytes : List Nat) (seed : Nat × Nat)
    (hseed : Normalized seed.1 seed.2) :
    Normalized
      (bytes.foldl (fun s byte => let a' := (s.1 + byte) % 65521
        (a', (s.2 + a') % 65521)) seed).1
      (bytes.foldl (fun s byte => let a' := (s.1 + byte) % 65521
        (a', (s.2 + a') % 65521)) seed).2 := by
  induction bytes generalizing seed with
  | nil => exact hseed
  | cons byte bytes ih =>
    apply ih
    exact ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩

theorem update_normalized (a b : Nat) (bytes : List Nat) :
    Normalized (Update a b bytes).1 (Update a b bytes).2 := by
  apply scalar_fold_normalized
  exact ⟨Nat.mod_lt _ (by decide), Nat.mod_lt _ (by decide)⟩

/-- Model split law; concatenation representability is an explicit condition.
    Original optimized compute correspondence remains a separate obligation. -/
theorem update_append (a b : Nat) (xs ys : Slice U8)
    (_hconcat : xs.val.length + ys.val.length ≤ Usize.max) :
    Update a b ((xs.val ++ ys.val).map (fun byte => byte.val)) =
      Update (Update a b (xs.val.map (fun byte => byte.val))).1
        (Update a b (xs.val.map (fun byte => byte.val))).2
        (ys.val.map (fun byte => byte.val)) := by
  have hn := update_normalized a b (xs.val.map (fun byte => byte.val))
  have ha := Nat.mod_eq_of_lt hn.1
  have hb := Nat.mod_eq_of_lt hn.2
  simp only [Update, List.map_append, List.foldl_append] at *
  rw [ha, hb]

end AdlerSpec

namespace AdlerSpec

/-- Original wrapper equality, with no assumed successful execution. -/
theorem write_slice_eq_compute (self : Adler32) (bytes : Slice U8) :
    Adler32.write_slice self bytes = algo.Adler32.compute self bytes := rfl

/-- This equivalence does not prove either unconditional contract. -/
theorem write_slice_contract_iff_compute :
    ElicitContracts.c_98f3436459f22f5d ↔ ElicitContracts.c_a058c96663922dc4 := by
  rfl

end AdlerSpec

namespace AdlerSpec

/-- Save/restore commutes with the actual update computation. This proves equality
    of Results, not that the update succeeds or meets its recurrence contract. -/
theorem resume_saved_checksum (self : Adler32) (bytes : Slice U8) :
    ∃ sum, Adler32.checksum self = ok sum ∧
      (do let restored ← Adler32.from_checksum sum
          Adler32.write_slice restored bytes) = Adler32.write_slice self bytes := by
  obtain ⟨sum, hc, hd⟩ := restore_checksum self
  exact ⟨sum, hc, by simp [hd]⟩

end AdlerSpec

namespace AdlerSpec

/-- Conditional composition of the original methods. The hypothesis is the
    outstanding full update contract, not a trusted replacement for its proof. -/
theorem adler32_slice_of_write_slice
    (hwrite : ElicitContracts.c_98f3436459f22f5d) :
    ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro data _
  obtain ⟨seed, hnew, ha, hb⟩ := new_correct trivial
  obtain ⟨state, hw, hu, _⟩ := hwrite seed data trivial
  simp only [ha, hb] at hu
  obtain ⟨sum, hc, hv⟩ := checksum_correct state trivial
  refine ⟨sum, ?_, ?_⟩
  · simp [adler32_slice, hnew, hw, hc]
  · change sum.val = Checksum
      (Update 1 0 (data.val.map (fun byte => byte.val))).1
      (Update 1 0 (data.val.map (fun byte => byte.val))).2
    rw [← hu]
    exact hv

end AdlerSpec

namespace AdlerSpec

/-- Equality of original method computations, including their failure behavior. -/
theorem hasher_write_eq_write_slice (self : Adler32) (bytes : Slice U8) :
    Adler32.Insts.CoreHashHasher.write self bytes =
      Adler32.write_slice self bytes := rfl

/-- Neither full correctness contract is established by this equivalence. -/
theorem hasher_write_contract_iff_write_slice :
    ElicitContracts.c_91f5cdb130e53bd4 ↔ ElicitContracts.c_98f3436459f22f5d := by
  rfl

end AdlerSpec
