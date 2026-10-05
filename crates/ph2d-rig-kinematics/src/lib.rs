//! A cinemática partilhada da família `rig.*` dos Motion Nodes: o contrato das colunas e o
//! `resolve` de forward kinematics ([`fk`]), as ajudas de pose dos solvers ([`pose`]) e o seno e
//! cosseno por voltas ([`trig`]). Bug #11 (`docs/Motion Nodes/BUGS_motion_nodes.md`): viviam
//! copiados byte a byte em seis crates de nó.

pub mod fk;
pub mod pose;
pub mod trig;
