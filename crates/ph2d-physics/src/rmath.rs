//! **O VOCABULÁRIO MATEMÁTICO DA RAPIER, num sítio só** — e o aviso que a migração de 2026-08-29
//! deixou.
//!
//! Até à `rapier2d` 0.31 a matemática dela era `nalgebra`, e esta crate importava
//! `Vector2`/`Point2`/`Isometry2`/`UnitComplex` de lá, ficheiro a ficheiro. Na 0.32 a dimforge
//! trocou para o `glam` (através do invólucro `glamx`), e o `parry` deixou de exportar `Point`,
//! `Isometry` e `Translation`. Os nomes novos são os **aliases do próprio `parry`**, que é o
//! vocabulário que a documentação dela usa:
//!
//! | era (nalgebra) | é (glam/glamx) | nota |
//! |---|---|---|
//! | `Vector2<f32>` | [`Vector`] (`glam::Vec2`) | `Vector::new(x, y)` é a mesma chamada |
//! | `Point2<f32>` | [`Vector`] | ⚠️ **o MESMO tipo** — ver abaixo |
//! | `Isometry2<f32>` | [`Pose`] (`glamx::Pose2`) | `pose.translation` já é um `Vector` |
//! | `UnitComplex<f32>` | [`Rotation`] (`glamx::Rot2`) | `rot.angle()` é idêntico |
//!
//! # ⛔⛔ O PONTO E O VETOR VIRARAM O MESMO TIPO — e isso é o risco nº 1 desta migração
//!
//! No `nalgebra`, `Point2` e `Vector2` são tipos **distintos** de propósito: um ponto é um lugar,
//! um vetor é um deslocamento. Somar dois pontos não tem significado geométrico, e o compilador
//! recusava-o. No `glam` os dois são `Vec2`.
//!
//! ⇒ **Essa rede desapareceu.** Um erro de ponto-vs-vetor — somar duas posições, subtrair um
//! deslocamento de onde devia entrar um lugar — passa a **compilar em silêncio** e a produzir um
//! número errado. Não há aviso, não há lint, e o resultado é plausível: uma força que aponta para
//! o sítio errado parece afinação má, não defeito.
//!
//! ⚠️ **A única defesa que temos é que os gates desta crate medem NÚMEROS, não tipos** — a
//! trajetória de um corpo, a razão de uma talha, o hash `physics_ecs_c9`. É por isso que a
//! migração dos ficheiros densos em matemática se faz com a suíte a correr, e não a olho.
//!
//! # Por que uma porta e não um `use` por ficheiro
//!
//! Porque o aviso acima tem de estar escrito **uma vez** e ser alcançável de todos os 25 ficheiros
//! que o assunto toca. Um `use rapier2d::math::…` espalhado não tem onde levar o texto, e a próxima
//! pessoa a mexer em `form_drag.rs` não tem como saber o que o compilador deixou de lhe garantir.

pub use rapier2d::math::{Pose, Real, Rotation, Vector};
