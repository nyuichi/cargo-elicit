import Finish
import Codec
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 3000000
set_option maxRecDepth 4096
namespace Verification
open adler2

@[step] theorem chunk_size_correct : algo.Adler32.compute.CHUNK_SIZE ⦃ r => r = 22208#usize ⦄ := by
  unfold algo.Adler32.compute.CHUNK_SIZE
  step* <;> scalar_tac

@[step] theorem compute_correct (self : Adler32) (bytes : Slice U8) :
    algo.Adler32.compute self bytes ⦃ s =>
      (s.a.val,s.b.val) = scalar (self.a.val,self.b.val) bytes.val ⦄ := by
  unfold algo.Adler32.compute
  simp only [lift,bind_ok]
  let a := core.convert.num.FromU32U16.from self.a
  let b := core.convert.num.FromU32U16.from self.b
  have ha : a.val ≤ 65535 := by dsimp [a]; scalar_tac
  have hb : b.val ≤ 65535 := by dsimp [b]; scalar_tac
  step as ⟨nmod,hnmod⟩
  step as ⟨aligned,haligned⟩
  step as ⟨whole,tail,hlenWhole,hlenTail,hwhole,htail⟩
  simp only [Slice.len_val,Slice.length] at hnmod haligned hlenWhole hlenTail
  step with chunk_size_correct as ⟨sz,hsz⟩
  subst sz
  step with chunks_spec whole 22208#usize (by decide) as ⟨chunks,hchunks,hflat,hrem⟩
  simp only [core.slice.iter.ChunksExact.getRemainder,bind_ok]
  let zero : Q := ⟨0#u32,0#u32,0#u32,0#u32⟩
  have hz : zero.bound 65520 := by simp [zero,Q.bound]
  have hezero : Std.Array.repeat 4#usize 0#u32 = zero.arr := by
    apply Std.Array.ext
    simp [zero,Q.arr,vec,Array.make]
  rw [hezero]
  step with outer_correct chunks.chunks chunks.remainder a b zero zero hchunks ha hb hz hz as
    ⟨b1,ra,rb,hout⟩
  rcases hout with ⟨bout,x,y,he,hb1,hx,hy,hm⟩
  rcases Prod.mk.inj he with ⟨rfl,he'⟩
  rcases Prod.mk.inj he' with ⟨rfl,rfl⟩
  have hlwhole : whole.val.length % 4 = 0 := by
    omega
  have hlchunks := congrArg List.length hflat
  rw [List.length_append, flat_length _ _ hchunks] at hlchunks
  have hlrem : chunks.remainder.val.length % 4 = 0 := by omega
  step with chunks_spec chunks.remainder 4#usize (by decide) as ⟨it,hcs,hf,hr⟩
  have hcomplete := four_complete chunks.remainder it hcs hf hr hlrem
  simp only [inner_duplicate]
  step with inner_aligned chunks.remainder it x y hcs hcomplete.1 (by omega) hx hy as
    ⟨ra,rb,u,v,hea,heb,hmrem⟩
  subst ra rb
  have hcast : (UScalar.cast .U32 chunks.remainder.len).val = chunks.remainder.val.length := by
    rw [cast32_val,Slice.len_val]
    change chunks.remainder.val.length % 4294967296 = chunks.remainder.val.length
    exact Nat.mod_eq_of_lt (by omega)
  step as ⟨p,hp⟩
  have hp' : p.val = chunks.remainder.val.length * a.val := by
    simpa only [hcast] using hp
  step as ⟨b2,hb2⟩
  simp only [algo.Adler32.compute.MOD]
  step with q_rem u as ⟨ru,u',heu,hu',hub⟩
  subst ru
  step with q_rem v as ⟨rv,v',hev,hv',hvb⟩
  subst rv
  step as ⟨b3,hb3⟩
  have hb3bound : b3.val ≤ 65520 := by change b3.val = b2.val % 65521 at hb3; omega
  have hmwhole : combineQ a.val b3.val u' v' = run ((a.val:F),(b.val:F)) whole.val := by
    rw [combineQ_mod _ _ _ _ _ _ hu' hv']
    have hb3' : b3.val = (b1.val+chunks.remainder.val.length*a.val)%65521 := by
      change b3.val = b2.val % 65521 at hb3
      rw [hb3,hb2,hp']
    rw [hb3']
    simp only [ZMod.natCast_mod,Nat.cast_add,Nat.cast_mul]
    rw [hmrem a.val b1.val,hm]
    rw [← run_append, hflat]
    congr 1
    simp [zero,combineQ,combine,a,b,core.convert.num.FromU32U16.from_val_eq]

  have htailLen : tail.val.length ≤ 3 := by omega
  change mix a b3 u' v' (finish tail) ⦃ _ ⦄
  apply mix_correct a b3 u' v' (finish tail) _ ha hb3bound hub hvb
  intro a' b' ha' hb' hmMix
  apply WP.spec_mono (finish_correct tail a' b' htailLen ha' hb')
  intro s hs
  rw [hs,scalar_eq]
  congr 1
  rw [hmMix,hmwhole,← run_append,hwhole,htail,List.take_append_drop]
  simp only [a,b,core.convert.num.FromU32U16.from_val_eq]

end Verification
