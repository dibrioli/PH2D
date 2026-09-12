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
//! # ⚠️ Porque ela tem TRINTA E TRÊS ficheiros e não cento e vinte e sete
//!
//! O conjunto que pode sair **tem de ser fechado sob toda aresta de compilação**. A Fase A mediu-o
//! quatro vezes (71 → 29 → 15 → **8** ficheiros), cada resposta menor que a anterior e **as três
//! primeiras a favor de mover demasiado**; a Fase B mediu-o outras quatro, e o padrão repetiu-se:
//!
//! | a régua da Fase B corrigiu-se assim | efeito |
//! |---|---|
//! | `^impl App` perdia os **10** `impl crate::App` — são **13**, não 3 (HOWTO §2.1) | subestimava |
//! | `\bApp\b` no ficheiro CRU acusa 23 e **7** mencionam-no só em doc-comment (§2.12) | 69 → **85** livres |
//! | `crate::render_loop::vector_bridge` é ficheiro **desta** família na pasta do laço | +6 |
//! | um movido não pode referir um da família que **FICOU** — `vec_entities` prendia **30 de 50** | 50 → **21** |
//!
//! ⛔⛔ **E o bloqueador de fundo NÃO é a `App`.** Medido: curar todo o acoplamento a `App` move
//! **+4 ficheiros / 615 LOC** e nada mais, porque os 9 que ela prende batem imediatamente no
//! `vec_entities`. O grafo da família tem **uma raiz só**:
//!
//! ```text
//! vec_entities  ──blocked by──▶  name_unique::unique_name
//!      │                         morph_set::is_set_member
//!      │                         render_loop::off_canvas::is_off_canvas
//!      ├──#[path]──▶ vec_zorder ──▶ vec_zorder_fixpoint_tests ──▶ undo · hero_intents ·
//!      │                                                          project_library · preview_drive
//!      └──referido por 42 ficheiros da família
//! ```
//!
//! Os três predicados são **puros sobre o ECS** e pertencem a outras famílias ⇒ tomá-los violaria o
//! [HOWTO §1.2] (*duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra*).
//!
//! ⭐⭐⭐ **RESOLVIDO em 2026-09-12 pela `line/shell-folhas`, e exactamente como este diagrama
//! prescrevia:** os três predicados viraram folhas ([`ph2d_unique_name`],
//! [`ph2d_entity_visibility`]) e o `vec_entities` inteiro — com o `vec_transform` e o `morph_set`,
//! que fazem ciclo com ele — mudou-se para a [`ph2d_vec_entities`]. ⚠️ **O diagrama acima fica
//! como está de propósito:** ele é a medição que abriu aquela linha, e apagá-lo apagaria a razão.
//! ⚠️ **O `name_unique` e o `preview_drive` são DOIS dos três que a `line/app-physics` nomeou** como
//! folhas partilhadas de linha própria — esta família confirma-os **independentemente**.
//!
//! ⛔ **E a cura barata é pior que a doença, com número:** dar a `vec_entities::sync` um parâmetro
//! para o nome único custa **168 sítios de chamada**, espalhados por `bool_live`, `blend_live`,
//! `connector_live`, `envelope_live`, `instance_*`, `label_live` — as famílias da 3.ª rodada. Um
//! parâmetro que atravessa 168 sítios de cinco famílias não é uma assinatura: é uma wave.
//!
//! ⇒ **o que falta desta família não é trabalho desta linha.** A lista, com o preço de cada item,
//! está no handoff da Fase B.
//!
//! # O molde que estes ficheiros provam
//!
//! Um `Cargo.toml` medido dependência a dependência (**quatro** eram invisíveis até a crate
//! existir — `ph2d-core`, `ph2d-tool-vector`, `ph2d-vec-blend`, `ph2d-vec-fill`; o piloto achou
//! quatro também), a **re-exportação com alias** que mantém os ~190 `crate::vec_x` da shell byte a
//! byte iguais, e as duas armadilhas de caminho que só a corrida revela (§2.6): o `include_str!`
//! que falha alto e o `CARGO_MANIFEST_DIR` que **não** falha.
//!
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

pub mod align_live;
// ⛔⛔ **O censo de ÓRFÃOS desta crate** — um ficheiro que nenhum `mod` declara não é compilado, e
// nenhum `check`, `clippy` ou suíte o acusa. Ver o cabeçalho do módulo: ele nasceu de um defeito
// real, apanhado só pelo `ONLY-A` da prova.
#[cfg(test)]
#[path = "no_orphan_module_tests.rs"]
mod no_orphan_module_tests;

pub mod appearance;
pub mod bindings;
pub mod blend;
pub mod blend_live;
pub mod bool_gesture;
pub mod bool_live;
pub mod bool_shape;
pub mod bucket;
pub mod bucket_claim;
pub mod bucket_repro;
pub mod component_edit;
pub mod connector_live;
pub mod connector_panel;
pub mod contour_live;
pub mod cut_line;
pub mod driven_style;
pub mod expand;
pub mod frame_labels;
pub mod frame_spans;
pub mod fx_atlas;
pub mod fx_bridge;
pub mod fx_bridge_dispatch;
pub mod fx_dump;
pub mod fx_live;
pub mod fx_live_hit;
pub mod fx_live_memo;
pub mod fx_live_resolve;
pub mod fx_silhouette;
pub mod guide;
pub mod marquee;
pub mod morph_edit;
pub mod morph_live;
pub mod morph_machine_drive;
pub mod offset_live;
pub mod paint_dilate;
pub mod pattern_live;
pub mod pencil_input;
pub mod profile_live;
pub mod resize_box_edit;
pub mod shape_params;
pub mod svg_export;
pub mod svg_import;
pub mod symmetry_live;
pub mod texture_pattern_edit;
pub mod ui_panel_spec;
pub mod ui_state_bridge;
pub mod ui_state_edit;
pub mod vector_bridge;
pub mod widget_drive;
pub mod widget_edit;
pub mod widget_icon;
pub mod widget_live;
pub mod widget_value;
// ⭐ **O mapa mudou-se para a folha** (`line/shell-folhas`, 12/09): ele é a MESMA peça
// partilhada um degrau acima — a `motion` e a `flip` consomem a ponte inteira, e uma peça que
// três famílias usam não pode viver na crate de uma delas (ADR-0075). O re-export mantém os 18
// ficheiros desta família a escrever `ph2d_app_vec::entity_map::…`, byte a byte como antes.
pub use ph2d_vec_entities::entity_map;
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
pub mod smoke_appearance;
pub mod smoke_bone;
pub mod smoke_fade;
pub mod smoke_stack;
pub mod smoke_svg;
pub mod snap;
pub mod snap_labels;
pub mod snap_sprites;
pub mod state;
pub mod stroke_paint;
pub mod stroke_present;
pub mod weld;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⭐⭐ **Os CINCO roteadores estão aqui desde 2026-09-12** (W2 Fase B, 2.ª volta), e a chave
/// `"vec"` saiu da catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` no mesmo commit — *uma dívida
/// cumprida e não apagada lê-se como dívida aberta*.
///
/// # ⚠️ São CINCO, e o quinto quase não foi contado
///
/// O briefing desta linha listava **quatro** (`BONE` · `STACK` · `APPEARANCE` · `FADE`) e mandava
/// **contar, não copiar**. Contados, são cinco: o `PH2D_VEC_SVG_SMOKE` é lido por um ficheiro que
/// se chamava `svg_import_smoke.rs` — **sem** o prefixo `vec_` —, e o censo desta família era
/// definido por *prefixo de nome de ficheiro*. ⛔ *A unidade da posse é o ASSUNTO, nunca o nome do
/// ficheiro* (é a §2.7 do HOWTO um nível acima: ali um censo por prefixo varre **zero** e fica
/// verde; aqui varreu **menos** e a diferença leu-se como «não há mais nada»).
///
/// # ⛔ O que NÃO é declarado, e porquê
///
/// O `PH2D_BUILD_SMOKE` e o `PH2D_UI_MOTION_SMOKE` aparecem como smokes do módulo Vector no
/// `CLAUDE.md` §5, e **continuam a ser lidos pela SHELL** (`build_smoke.rs`, `ui_motion_smoke.rs`).
/// Declará-los aqui diria que esta crate responde por cenas que ela não encaminha — e **nenhum
/// gate do registo o apanharia**, porque eles medem a FORMA do nome e o `max_level`, nunca se a
/// env é lida deste lado. *Um registo que se pode mentir sem reprovar só vale o cuidado de quem o
/// escreve.*
///
/// ⚠️ `PH2D_BLEND_LOG`, `PH2D_TEXT_LOG` e `PH2D_VEC_OVERLAY_DIAG` são **diagnóstico** e não entram;
/// `PH2D_VEC_PEN` e `PH2D_VEC_DEMO_N` são configuração de arranque, não roteadores de cena.
///
/// # O `max_level` é CONTADO, e para os cinco ele é `1`
///
/// Cada um é um **interruptor**: o corpo pergunta `std::env::var_os(..).is_some()` e monta UMA
/// cena — não há `match` de níveis em nenhum deles (ao contrário do `PH2D_PHYSICS_SMOKE`, que
/// declara 117). O número sai da forma do roteador, não de memória.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "vec",
    routers: &[
        r("PH2D_VEC_APPEARANCE_SMOKE"),
        r("PH2D_VEC_BONE_SMOKE"),
        r("PH2D_VEC_FADE_SMOKE"),
        r("PH2D_VEC_STACK_SMOKE"),
        r("PH2D_VEC_SVG_SMOKE"),
    ],
};

/// Um roteador-interruptor desta família — ver a nota do [`FAMILY`] sobre o `max_level`.
const fn r(env: &'static str) -> ph2d_app_host::SmokeRouter {
    ph2d_app_host::SmokeRouter { env, max_level: 1 }
}
