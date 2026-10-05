//! O vocabulário partilhado dos nós do Motion, numa porta só: o sorteio sem estado
//! ([`hash`]), o seno e o cosseno por voltas ([`trig`]) e a acumulação das forças na
//! coluna `accel` ([`forca`]).
//!
//! Bug #11 (`docs/Motion Nodes/BUGS_motion_nodes.md`): viviam copiados byte a byte em
//! vinte crates de nó (e na `ph2d-rig-kinematics`), cada cópia a dizer que «o vocabulário partilhado é o
//! comportamento». O custo medido foi a cura do `rot` digitada seis vezes. Uma lei que
//! tem de ser a MESMA em todos os nós mora numa porta, não em N cópias que se prometem
//! iguais.
//!
//! ⚠️ Os kernels WGSL que espelham estas leis continuam inline em cada nó (e o gate de
//! paridade GPU↔CPU de cada um é que os prende a esta porta).

pub mod forca;
pub mod hash;
pub mod trig;
