//! O vocabulário partilhado dos nós do Motion, numa porta só: o sorteio sem estado
//! ([`hash`]), o seno e o cosseno por voltas ([`trig`]) e a acumulação das forças na
//! coluna `accel` ([`forca`]).
//!
//! Bug #11 (`docs/Motion Nodes/BUGS_motion_nodes.md`): viviam copiados em trinta e oito
//! crates (nós, a `ph2d-rig-kinematics`, a `ph2d-contact`, a `ph2d-bloom`), cada cópia a
//! dizer que «o vocabulário partilhado é o comportamento». O custo medido foi a cura do `rot` digitada seis vezes. Uma lei que
//! tem de ser a MESMA em todos os nós mora numa porta, não em N cópias que se prometem
//! iguais.
//!
//! ⚠️ Os kernels WGSL que espelham estas leis continuam inline em cada nó (e o gate de
//! paridade GPU↔CPU de cada um é que os prende a esta porta).
//!
//! ⚠️ [`forca`] está atrás da feature `forca` (é a única parte que depende do
//! `ph2d-nodegraph`): `hash` e `trig` não puxam nada, e por isso servem também folhas fora
//! do grafo de nós (`ph2d-contact`, `ph2d-bloom`). O portão
//! `architecture_a_lei_partilhada_dos_nos_vive_numa_porta` (ph2d-editor-core) reprova uma
//! cópia nova.

#[cfg(feature = "forca")]
pub mod forca;
pub mod hash;
pub mod trig;
