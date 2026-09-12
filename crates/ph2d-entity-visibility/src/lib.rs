//! ⭐⭐ **«Esta entidade DESENHA — e PROCESSA — neste quadro?»**
//!
//! As duas metades da mesma pergunta, que viviam em `shells/desktop/src/render_loop/` e que
//! **três consumidores de famílias diferentes** já faziam: o extract de sprites
//! ([`off_canvas::draws_this_frame`]), o tique da §11 Animation
//! ([`on_screen_gate::processing_paused`]) e a cadeia de visibilidade do vetor
//! ([`off_canvas::is_off_canvas`], de `vec_entities`).
//!
//! ⚠️ **Elas saíram JUNTAS porque uma chama a outra** — o
//! [`off_canvas::draws_this_frame`] pergunta ao [`on_screen_gate::hides`], e essa aresta é a
//! razão pela qual o censo de 2026-09-12 que lia o `off_canvas` como *«zero arestas à shell»*
//! estava errado: ela existe, em código, na linha 42 do ficheiro.
//!
//! ⛔ **Nada aqui sabe o que é um renderer.** É o que permite a esta crate ser uma FOLHA e não
//! pertencer a nenhuma família (ADR-0075): quem pergunta são o raster, a animação e o vetor.

pub mod off_canvas;
pub mod on_screen_gate;
