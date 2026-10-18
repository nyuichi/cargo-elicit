import ElicitGenerated
open Aeneas Aeneas.Std
namespace AdlerSpec
 def step (s : Nat × Nat) (x : Nat) : Nat × Nat :=
   let a := (s.1 + x) % 65521
   (a, (s.2 + a) % 65521)
 def run (a b : Nat) (xs : List U8) : Nat × Nat :=
   xs.foldl (fun s x => step s x.val) (a % 65521, b % 65521)
 def encode (s : Nat × Nat) : Nat := 65536 * s.2 + s.1
end AdlerSpec

namespace AdlerSpec
 def lane (v : ElicitGenerated.algo.U32X4) (i : Fin 4) : Nat :=
   (v.val[i.val]'(by have hp := v.property; simp_all)).val
end AdlerSpec

namespace AdlerSpec
 def rawLanes : List (Slice U8) → (Fin 4 → Nat) → (Fin 4 → Nat) → ((Fin 4 → Nat) × (Fin 4 → Nat))
   | [], a, b => (a, b)
   | c :: cs, a, b =>
       rawLanes cs (fun i => a i + c.val[i.val]!.val)
         (fun i => b i + (a i + c.val[i.val]!.val))
 def safeLanes : List (Slice U8) → (Fin 4 → Nat) → (Fin 4 → Nat) → Prop
   | [], _, _ => True
   | c :: cs, a, b =>
       4 ≤ c.length ∧
       (∀ i, a i + c.val[i.val]!.val ≤ U32.max) ∧
       (∀ i, b i + (a i + c.val[i.val]!.val) ≤ U32.max) ∧
       safeLanes cs (fun i => a i + c.val[i.val]!.val)
         (fun i => b i + (a i + c.val[i.val]!.val))
end AdlerSpec
