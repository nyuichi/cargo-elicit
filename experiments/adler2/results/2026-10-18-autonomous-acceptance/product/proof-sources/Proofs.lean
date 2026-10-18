import ElicitContracts
import Lemmas
open Aeneas Aeneas.Std Result ElicitGenerated
namespace AdlerProof
theorem new_correct : ElicitContracts.c_8055bec449b87b3d := by
  intro _
  refine ⟨{ a := 1#u16, b := 0#u16 }, rfl, ?_⟩
  decide
elicit_spec new_spec from new_correct


theorem checksum_correct : ElicitContracts.c_0a059b8c0885cf07 := by
  intro self _
  apply ElicitAdapter.spec_to_contract
  unfold Adler32.checksum
  simp only [lift]
  step* +scalarTac
  change (i1.bv ||| (core.convert.num.FromU32U16.from self.a).bv).toNat = _
  rw [BitVec.toNat_or]
  change i1.val ||| (core.convert.num.FromU32U16.from self.a).val = _
  simp only [core.convert.num.FromU32U16.from_val_eq] at *
  rw [i1_post]
  have hb : self.b.val < 65536 := by scalar_tac
  have ha : self.a.val < 65536 := by scalar_tac
  have hmod : self.b.val <<< 16 < U32.size := by
    simp only [Nat.shiftLeft_eq]
    scalar_tac
  rw [Nat.mod_eq_of_lt hmod, ← Nat.shiftLeft_add_eq_or_of_lt (by omega : self.a.val < 2^16)]
  simp [Nat.shiftLeft_eq, Nat.add_comm, Nat.mul_comm]

elicit_spec checksum_spec from checksum_correct
end AdlerProof

namespace AdlerProof
/-- The original convenience function follows from the complete arbitrary-input
update contract. This explicit implication is not a proof of that prerequisite. -/
theorem slice_of_update (update_contract : ElicitContracts.c_98f3436459f22f5d) :
    ElicitContracts.c_8484fdd92cfcd3e7 := by
  intro data _
  obtain ⟨seed, hseed, ha, hb⟩ := new_correct trivial
  obtain ⟨updated, hupdate, hstate⟩ := update_contract seed data trivial
  obtain ⟨out, hout, hpack⟩ := checksum_correct updated trivial
  refine ⟨out, ?_, ?_⟩
  · simp [adler32_slice, hseed, hupdate, hout]
  · have hstate' : (updated.a.val, updated.b.val) =
        data.val.foldl (fun (s : Nat × Nat) x =>
          let a := (s.1 + x.val) % 65521
          (a, (s.2 + a) % 65521)) (1, 0) := by
      simpa [AdlerSpec.run, ha, hb] using hstate
    have hfirst := congrArg Prod.fst hstate'
    have hsecond := congrArg Prod.snd hstate'
    simp only [Prod.fst, Prod.snd] at hfirst hsecond
    simpa only [hfirst, hsecond] using hpack
end AdlerProof

namespace AdlerProof
theorem from_correct : ElicitContracts.c_0dfc1ce11e75ed00 := by
  intro bytes h
  exact ElicitAdapter.spec_to_contract (from_wp bytes h)
elicit_spec from_spec from from_correct

theorem add_correct : ElicitContracts.c_20b5198a3591154d := by
  intro self other h
  exact ElicitAdapter.spec_to_contract (add_wp self other h)
elicit_spec add_spec from add_correct
end AdlerProof

set_option maxHeartbeats 10000000
namespace AdlerProof
attribute [local step] rem_mod_wp
@[local simp] theorem mod_value : algo.Adler32.compute.MOD.val = 65521 := by simp [algo.Adler32.compute.MOD]
theorem write_slice_wp (self : Adler32) (bytes : Slice U8) :
  Adler32.write_slice self bytes ⦃ fun out => (out.a.val, out.b.val) = AdlerSpec.run self.a.val self.b.val bytes.val ⦄ := by
  unfold Adler32.write_slice algo.Adler32.compute algo.Adler32.compute.CHUNK_SIZE
  simp only [lift, core.slice.iter.ChunksExact.getRemainder, core.slice.Slice.iter]
  step* +scalarTac
  case hc =>
    intro c hc
    rw [iter_post1 c hc]
  case hn =>
    exact chunk_count_bound iter.chunks iter.remainder chunk_iter.remainder iter_post iter_post1 (by omega)
  case hchunks =>
    intro c hc
    have h := chunk_iter_post1 c hc
    omega
  case hav =>
    intro i hi
    have cases : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp [Array.repeat]
  case hbv =>
    intro i hi
    have cases : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
    rcases cases with rfl | rfl | rfl | rfl <;> simp [Array.repeat]


  all_goals try (norm_num [algo.Adler32.compute.MOD])
  all_goals try (apply Nat.le_of_lt; simp only [UScalar.cast_val_eq, core.convert.num.FromU32U16.from, lift] at *; scalar_tac)
  all_goals try (
    apply Nat.le_of_lt
    have h0 := (a_vec2_post 0 (by decide)).2
    have h1 := (a_vec2_post 1 (by decide)).2
    have h2 := (a_vec2_post 2 (by decide)).2
    have h3 := (a_vec2_post 3 (by decide)).2
    simp_all [List.getElem!_eq_getElem?_getD, algo.Adler32.compute.MOD, Array.set_val_eq]
    scalar_tac)


  all_goals try (
    apply Nat.le_of_lt
    have hlen : chunk_iter.remainder.val.length ≤ 22208 := by omega
    have hs : self.a.val ≤ 65535 := by scalar_tac
    have hc : chunk_iter.remainder.val.length % 4294967296 ≤ 22208 :=
      Nat.le_trans (Nat.mod_le _ _) hlen
    have hm := Nat.mul_le_mul hc hs
    scalar_tac)
  all_goals try (simp only [vector_iter_sum] at *)
  all_goals try (
    apply Nat.le_of_lt
    have h0 := (a_vec2_post 0 (by decide)).2
    have h1 := (a_vec2_post 1 (by decide)).2
    have h2 := (a_vec2_post 2 (by decide)).2
    have h3 := (a_vec2_post 3 (by decide)).2
    have k0 := (b_vec2_post 0 (by decide)).2
    have k1 := (b_vec2_post 1 (by decide)).2
    have k2 := (b_vec2_post 2 (by decide)).2
    have k3 := (b_vec2_post 3 (by decide)).2
    have t0 := b_vec3_post 0 (by decide)
    have t1 := b_vec3_post 1 (by decide)
    have t2 := b_vec3_post 2 (by decide)
    have t3 := b_vec3_post 3 (by decide)
    simp_all [List.getElem!_eq_getElem?_getD, algo.Adler32.compute.MOD, Array.set_val_eq]
    scalar_tac)


  have hseed : ∀ i : Nat, i < 4 →
      a_vec.val[i]!.val = (laneRun qs 0 0 i).1 % 65521 ∧
      b_vec.val[i]!.val = (laneRun qs 0 0 i).2 % 65521 := by
    intro i hi
    have h := b1_post4 i hi
    have hc : i = 0 ∨ i = 1 ∨ i = 2 ∨ i = 3 := by omega
    rcases hc with rfl | rfl | rfl | rfl <;> simpa [Array.repeat] using And.intro h.1 h.2.1
  have hlane := combine_lane_residues qs iter.chunks a_vec b_vec a_vec1 b_vec1 hseed a_vec1_post
  have hq : ∀ q ∈ qs ++ iter.chunks, q.val.length = 4 := by
    intro q hmem
    rcases List.mem_append.mp hmem with hmem | hmem
    · exact b1_post1 q hmem
    · exact iter_post1 q hmem
  have houter : ∀ c ∈ chunk_iter.chunks, c.val.length = 22208 := by
    intro c hc; have h := chunk_iter_post1 c hc; omega
  have hclen := fixed_chunks_length chunk_iter.chunks 22208 houter
  have hilen := fixed_chunks_length iter.chunks 4 iter_post1
  have hqlen := fixed_chunks_length qs 4 b1_post1
  have hp1 := congrArg List.length chunk_iter_post
  have hp2 := congrArg List.length iter_post
  have hql := congrArg List.length b1_post
  simp only [List.length_append] at hp1 hp2
  have hprefixlen : bytes1.val.length = bytes.val.length - bytes.val.length % 4 := by
    simpa only [Slice.length, Slice.len_val, i3_post, i2_post] using bytes1_post
  have hemptylen : iter.remainder.val.length = 0 := by omega
  have hempty : iter.remainder.val = [] := List.eq_nil_of_length_eq_zero hemptylen
  have hflat : ((qs ++ iter.chunks).map Slice.val).flatten = bytes1.val := by
    simp only [List.map_append, List.flatten_append]
    rw [b1_post]
    have hi := iter_post
    rw [hempty, List.append_nil] at hi
    rw [hi]
    exact chunk_iter_post
  have hcount : 22208 * chunk_iter.chunks.length + chunk_iter.remainder.val.length =
      4 * (qs ++ iter.chunks).length := by simp only [List.length_append]; omega
  have hwhole : bytes1.val ++ remainder.val = bytes.val := by
    rw [bytes1_post2, bytes1_post3]; exact List.take_append_drop _ _
  have hbase : b3.val = (self.b.val + 4 * (qs ++ iter.chunks).length * self.a.val) % 65521 := by
    have hcast : (UScalar.cast UScalarTy.U32 chunk_iter.remainder.len).val = chunk_iter.remainder.val.length := by
      simp [UScalar.cast_val_eq, Nat.mod_eq_of_lt (show chunk_iter.remainder.val.length < 4294967296 by omega)]
    simp only [hcast, core.convert.num.FromU32U16.from_val_eq] at i7_post b1_post2
    rw [b3_post, b2_post, i7_post]
    simp only [mod_value]
    rw [Nat.add_mod, b1_post2, ← Nat.add_mod]
    congr 1
    rw [← hcount]
    ring
  have hav (i : Nat) (hi : i < 4) :
      a_vec2.val[i]!.val = (laneRun (qs ++ iter.chunks) 0 0 i).1 % 65521 :=
    (a_vec2_post i hi).1.trans (hlane i hi).1
  have hbv (i : Nat) (hi : i < 4) :
      b_vec2.val[i]!.val = (laneRun (qs ++ iter.chunks) 0 0 i).2 % 65521 :=
    (b_vec2_post i hi).1.trans (hlane i hi).2
  have hquart := quartet_mod_recombine (qs ++ iter.chunks) hq self.a.val self.b.val
  have hpre : (a5.val % 65521, b4.val % 65521) = AdlerSpec.run self.a.val self.b.val bytes1.val := by
    rw [← hflat]
    apply Eq.trans ?_ hquart
    apply Prod.ext
    · dsimp only
      rw [a5_post]
      simp only [core.convert.num.FromU32U16.from_val_eq]
      rw [hav 0 (by decide), hav 1 (by decide), hav 2 (by decide), hav 3 (by decide)]
      congr 1; omega
    · dsimp only
      rw [b4_post, hbase]
      have t0 := b_vec3_post 0 (by decide)
      have t1 := b_vec3_post 1 (by decide)
      have t2 := b_vec3_post 2 (by decide)
      have t3 := b_vec3_post 3 (by decide)
      have v0 := hbv 0 (by decide)
      have v1 := hbv 1 (by decide)
      have v2 := hbv 2 (by decide)
      have v3 := hbv 3 (by decide)
      have w1 := hav 1 (by decide)
      have w2 := hav 2 (by decide)
      have w3 := hav 3 (by decide)
      simp at t0 t1 t2 t3 v0 v1 v2 v3 w1 w2 w3
      simp [a4_post, a3_post, a2_post, Array.set_val_eq,
        i21_post, i20_post, i19_post, i18_post, i17_post, i16_post, i15_post,
        i14_post, i13_post, i12_post, i11_post, i10_post, i9_post, i8_post,
        algo.Adler32.compute.MOD, t0, t1, t2, t3, v0, v1, v2, v3, w1, w2, w3]
      simp [Nat.add_mod, Nat.mul_mod, Nat.mul_add, Nat.add_mul,
        Nat.add_assoc, Nat.add_comm, Nat.add_left_comm, Nat.mul_assoc,
        Nat.mul_comm, Nat.mul_left_comm] 
  have htail := rawRun_mod remainder.val a5.val b4.val
  have hresult : (a6.val % 65521, b5.val % 65521) = AdlerSpec.run self.a.val self.b.val bytes.val := by
    have hnorm : AdlerSpec.run a5.val b4.val remainder.val =
        AdlerSpec.run (AdlerSpec.run self.a.val self.b.val bytes1.val).1
          (AdlerSpec.run self.a.val self.b.val bytes1.val).2 remainder.val := by
      apply run_congr
      · have h := congrArg Prod.fst hpre; simpa using congrArg (fun x : Nat => x % 65521) h
      · have h := congrArg Prod.snd hpre; simpa using congrArg (fun x : Nat => x % 65521) h
    rw [← run_append, hwhole] at hnorm
    have hp : (a6.val, b5.val) = rawRun remainder.val a5.val b4.val := by
      simpa only [List.drop_zero] using a6_post
    rw [← hp] at htail
    exact htail.trans hnorm
  have ha22 : i22.val < 65536 := by rw [i22_post, mod_value]; have h := Nat.mod_lt a6.val (show 0 < 65521 by decide); omega
  have hb24 : i24.val < 65536 := by rw [i24_post, mod_value]; have h := Nat.mod_lt b5.val (show 0 < 65521 by decide); omega
  rw [Nat.mod_eq_of_lt ha22, Nat.mod_eq_of_lt hb24, i22_post, i24_post]
  simpa only [mod_value] using hresult
end AdlerProof

namespace AdlerProof
theorem write_slice_correct : ElicitContracts.c_98f3436459f22f5d := by
  intro self bytes _
  exact ElicitAdapter.spec_to_contract (write_slice_wp self bytes)
elicit_spec write_slice_spec from write_slice_correct

theorem adler32_slice_correct : ElicitContracts.c_8484fdd92cfcd3e7 :=
  slice_of_update write_slice_correct
end AdlerProof

namespace AdlerProof
theorem from_checksum_correct : ElicitContracts.c_9a31b7b9cb8492b0 := by
  intro sum _
  exact ElicitAdapter.spec_to_contract (from_checksum_wp sum)
elicit_spec from_checksum_spec from from_checksum_correct
end AdlerProof

namespace AdlerProof
set_option maxHeartbeats 1000000

theorem state_eq_of_values (s t : Adler32)
    (h : (s.a.val, s.b.val) = (t.a.val, t.b.val)) : s = t := by
  have ha : s.a = t.a := UScalar.eq_of_val_eq (congrArg Prod.fst h)
  have hb : s.b = t.b := UScalar.eq_of_val_eq (congrArg Prod.snd h)
  cases s; cases t
  simp_all

theorem checksum_from_checksum_roundtrip (sum : U32) :
    ∃ (s : Adler32) (out : U32), Adler32.from_checksum sum = .ok s ∧
      Adler32.checksum s = .ok out ∧ out = sum := by
  obtain ⟨s, hs, ha, hb⟩ := from_checksum_correct sum trivial
  obtain ⟨out, hout, hp⟩ := checksum_correct s trivial
  refine ⟨s, out, hs, hout, UScalar.eq_of_val_eq ?_⟩
  rw [hp, ha, hb]
  omega

theorem from_checksum_checksum_roundtrip (s : Adler32) :
    ∃ (sum : U32) (restored : Adler32), Adler32.checksum s = .ok sum ∧
      Adler32.from_checksum sum = .ok restored ∧ restored = s := by
  obtain ⟨sum, hsum, hp⟩ := checksum_correct s trivial
  obtain ⟨restored, hr, ha, hb⟩ := from_checksum_correct sum trivial
  refine ⟨sum, restored, hsum, hr, state_eq_of_values restored s ?_⟩
  have hsmall : s.a.val < 65536 := by scalar_tac
  apply Prod.ext
  · dsimp only
    rw [ha, hp]
    simp [Nat.add_mod, Nat.mul_mod, Nat.mod_eq_of_lt hsmall]
  · dsimp only
    rw [hb, hp]
    simp [Nat.add_div, Nat.div_eq_of_lt hsmall]
    exact hsmall

theorem split_update (s : Adler32) (x y xy : Slice U8)
    (hrep : x.val.length + y.val.length ≤ Usize.max)
    (hjoin : xy.val = x.val ++ y.val) :
    ∃ (sx sxy whole : Adler32), Adler32.write_slice s x = .ok sx ∧
      Adler32.write_slice sx y = .ok sxy ∧
      Adler32.write_slice s xy = .ok whole ∧ sxy = whole := by
  obtain ⟨sx, hx, hstate⟩ := write_slice_correct s x trivial
  obtain ⟨sxy, hy, hstate'⟩ := write_slice_correct sx y trivial
  obtain ⟨whole, hw, hwhole⟩ := write_slice_correct s xy trivial
  refine ⟨sx, sxy, whole, hx, hy, hw, state_eq_of_values sxy whole ?_⟩
  have ha := congrArg Prod.fst hstate
  have hb := congrArg Prod.snd hstate
  dsimp only at ha hb
  rw [hstate', ha, hb, ← run_append, ← hjoin]
  exact hwhole.symm

theorem save_restore_resume (s : Adler32) (x y xy : Slice U8)
    (hrep : x.val.length + y.val.length ≤ Usize.max)
    (hjoin : xy.val = x.val ++ y.val) :
    ∃ (sx : Adler32) (saved : U32) (restored resumed whole : Adler32),
      Adler32.write_slice s x = .ok sx ∧
      Adler32.checksum sx = .ok saved ∧
      Adler32.from_checksum saved = .ok restored ∧
      Adler32.write_slice restored y = .ok resumed ∧
      Adler32.write_slice s xy = .ok whole ∧ resumed = whole := by
  obtain ⟨sx, sxy, whole, hx, hy, hw, heq⟩ := split_update s x y xy hrep hjoin
  obtain ⟨saved, restored, hs, hr, heq'⟩ := from_checksum_checksum_roundtrip sx
  refine ⟨sx, saved, restored, sxy, whole, hx, hs, hr, ?_, hw, heq⟩
  simpa only [heq'] using hy
end AdlerProof
