//! **A ARTE VIVA do vetor** — o que uma forma revela como TINTA, assado e memoizado por quadro.
//!
//! # Porque esta crate existe (W2 Fase C, 2026-09-12)
//!
//! Os dois módulos aqui dentro declaravam-se **irmãos um do outro**, cada um no cabeçalho do
//! outro (*«irmão do `crate::texture_pattern_live`, e pela mesma razão que ele existe»* — os nomes
//! antigos, citados como estavam), e
//! viviam na `shells/desktop` porque a `ph2d-vec-render` **não alcança a cena**: resolver a
//! forma-fonte lá dentro poria o guarda de ciclo, a geometria viva e o cozimento num sítio que
//! não os pode medir.
//!
//! ⛔⛔ **Mas a shell não é o único sítio que alcança a cena — é só o sítio onde eles estavam.**
//! Medido em 12/09: estas `632` linhas de produto prendiam **`99 845`** da família `motion`, pela
//! cadeia `motion_object_bake → brush::resolve` ⇒ `motion_state.rs` (que 106 ficheiros nomeiam e
//! de que 42 são filhos `#[path]`). *Uma chamada dentro de um laço é uma aresta tão dura como um
//! `#[path]`.*
//!
//! # ⚠️ O que é uma FOLHA aqui, e o que NÃO é
//!
//! O **tipo** vive nesta crate; o **campo** fica na `App`. O `BrushLive` e o `TexturePatternLive`
//! são caches com estado (`self.brush_live`, `self.texture_pattern_live`, 13 consumidores na
//! shell) — e o dono do estado do quadro continua a ser a shell. ⛔ Esta crate **não** guarda um
//! `static`: uma cache global partilhada entre documentos é a mesma classe de defeito que o
//! `thread_local` de uma família resolve *por ser da família*.
//!
//! # ⛔ Uma folha por ASSUNTO, nunca um saco (ESTADO_W2 §2)
//!
//! O assunto é **a arte que uma forma revela**: o ladrilho do padrão (*a TINTA*) e a arte que
//! percorre o traço (*o PINCEL*) — os dois modelos que o
//! [plano 33](../../../docs/Vector%20Module/33_plano_texture_pattern.md) e o
//! [36](../../../docs/Vector%20Module/36_plano_pincel_de_contorno.md) nomeiam. Descreve-se sem
//! dizer *«comum»*, *«shared»* ou *«utils»*, que é o teste do nome.
//!
//! ⛔ **Ela NÃO entrou em `ph2d-app-vec`**, e a razão é de calendário e não de desenho: a
//! `line/app-vec` está a reestruturar aquela crate. *Uma folha de assunto sobrevive à
//! reestruturação de qualquer família.*

#![forbid(unsafe_code)]

pub mod brush;
pub mod pattern;

/// SONDA `--ignored`: quanto custa o [`brush::resolve`] por quadro.
///
/// ⛔⛔ **Ela esteve FORA DO BUILD desde que chegou aqui (`fe7d9b93a`, 12/09) até à integração da
/// Fase D:** na shell chamava-se `brush_live_cost_probe` e era declarada no `main.rs`; o ficheiro
/// mudou de casa e a declaração não veio. 583 linhas no disco, no git e no diff — e fora do
/// compilador. Achada pelo censo de órfãos que lê os `.d` do compilador.
#[cfg(test)]
mod brush_cost_probe;
