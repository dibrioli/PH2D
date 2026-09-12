//! ⭐⭐⭐ **A família `vec` da shell, fora da shell** — W2/L4 (auditoria de velocidade de
//! 2026-09-10 §4-C2, DIRETRIZ §6.7 item 5).
//!
//! # O que esta crate é
//!
//! A `shells/desktop` é um `bin` de **493 k linhas** que dobrou em quatro semanas e é a **última
//! unidade de todo build grande** (34–45 s sozinha no fim do gate de fecho). Dentro dela vivem
//! famílias inteiras que só ali estão por inércia — e a metade-shell do módulo vectorial é uma
//! delas: **129 ficheiros, 17 527 LOC de produto e 17 933 de teste**.
//!
//! Esta crate é o destino dessa família. O sentido é **sempre shell → crate**: a shell é um
//! binário, logo nada aqui pode referir `crate::` da shell. Quem precisa do que só a shell tem —
//! janela, `gfx`, painéis, a captura do undo — **fica lá**, e isso é o ADR-0075 outra vez (estado
//! de família é recurso/componente do ECS; o resto é composição da raiz).
//!
//! # ⚠️ Porque ela começa com OITO ficheiros e não com cento e vinte e nove
//!
//! O conjunto que pode sair **tem de ser fechado sob toda aresta de compilação**, e medi-lo deu
//! quatro respostas, cada uma menor que a anterior — todas as três primeiras **a favor de mover
//! demasiado**:
//!
//! | régua | move | o que ela esquecia |
//! |---|---:|---|
//! | «não toca `App` nem `crate::<mod>` de fora» | 71 ficheiros / 18 585 LOC | tudo o resto abaixo |
//! | + fecho sob `crate::vec_x` (quem refere quem fica, fica) | 29 / 7 621 | o hub `vec_entities` |
//! | + um `_tests.rs` não sai sem quem o **declara** | 15 / 3 055 | o sentido filho → pai |
//! | **+ `#[path]` é aresta DURA nos DOIS sentidos** | **8 / 1 843** | — |
//!
//! A última régua é a que importa e foi a que mordeu: `vec_gizmo_view.rs` **declara**
//! `#[path = "vec_gizmo_pick.rs"]`, e esse toca `App` ⇒ o pai não pode sair. *Um `mod` declarado
//! por `#[path]` é parte da árvore de módulos do pai, não uma referência que se re-aponta.*
//!
//! ⇒ **A família `vec` não sai da shell por incrementos**, e o número é 8 de 129. Os três
//! bloqueadores estão medidos e nomeados no handoff de integração: `App` (23 ficheiros),
//! `crate::<mod>` de fora da família (34) e o efeito de cascata dos dois (34 + 23). A cura de
//! qualquer um deles cascateia — é isso que a Fase B herda.
//!
//! # ⛔ O que NÃO se conclui daqui
//!
//! Que a extracção não vale a pena. O que estes 8 ficheiros compram é o **molde provado**: um
//! `Cargo.toml` medido dependência a dependência, a re-exportação que poupa a churn de
//! `crate::vec_x` na shell, e os **dois gates que nomeiam um ficheiro da família por caminho de
//! string** — que é a armadilha que a W1 pagou e que nenhuma leitura de `crate::` vê.

pub mod appearance;
pub mod blend;
pub mod bucket_claim;
pub mod bucket_repro;
pub mod component_edit;
pub mod driven_style;
pub mod entity_map;
pub mod font;
/// ⚠️ **Atrás da mesma feature que a guardava na shell** — a pré-visualização de fonte
/// usa `ph2d_panel_vector::FontPreview`, logo ela não existe num build sem painel vectorial.
#[cfg(feature = "panel-vector")]
pub mod font_preview;
pub mod glyph;
pub mod glyph_build;
pub mod overlay;
pub mod overlay_diag;
pub mod paint_stack;
pub mod pick;
pub mod shape_live;
pub mod snap;
pub mod snap_labels;
pub mod snap_sprites;
pub mod state;
pub mod stroke_present;
pub mod weld;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⚠️⚠️ **`routers: &[]` aqui é a verdade MEDIDA da Fase A, não um esquecimento** — e por isso a
/// chave `"vec"` está escrita na catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` do registo, que
/// é o único sítio onde *«ainda não saiu»* se distingue de *«alguém esqueceu»*.
///
/// Medido em 2026-09-11: dos nomes `PH2D_VEC_*_SMOKE` que esta crate menciona, **nenhum é lido
/// aqui** — eles aparecem só em doc-comments dos campos de [`state::VecState`], que são a memória
/// *«esta cena já montou?»*. Quem **lê** a variável e escolhe a cena é o roteador da shell, porque
/// ele toca a `App`. ⛔ *Uma varredura textual pelo nome da variável mede MENÇÃO, não posse* — foi
/// exactamente essa leitura que, na integração, quase escreveu quatro roteadores que esta crate
/// não responde.
///
/// ⇒ a entrada sai da catraca no dia em que a Fase B trouxer o roteador de cenas para cá.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "vec",
    routers: &[],
};
