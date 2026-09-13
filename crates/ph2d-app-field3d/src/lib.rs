//! **`ph2d-app-field3d` — a família do modelador 3D, fora da shell** (W2 / L0, o piloto).
//!
//! # O que é uma «família», e por que ela era da shell
//!
//! O ADR-0075 põe cada feature numa drop-crate, e o módulo de modelagem honrou-o: `ph2d-field`,
//! `-field-eval`, `-field-ecs`, `-field-mesh`, `-field-render`, `ph2d-panel-model3d`. O que ficou
//! na shell foi **a metade que fala com a `App`** — o roteador de cenas, os ganchos de entrada, a
//! ponte com o `World`, a exportação, e as 17 cenas de smoke. Eram **15 207 linhas de produto e
//! 16 990 de teste** numa crate que toda edição de todo módulo recompila
//! ([auditoria][audit] §4-C2).
//!
//! Esta crate é essa metade. A shell continua a chamá-la **pelo nome** — e pode, porque a seta
//! aponta na direcção certa.
//!
//! # ⭐ Por que ESTA família foi o piloto: ela já tinha feito o trabalho
//!
//! O censo das seis famílias mediu o acoplamento à `App`:
//!
//! | família | `impl App` | membros de `self.` |
//! |---|---:|---:|
//! | sculpt3d | 6 | 180 |
//! | physics | 22 | 126 |
//! | vec | 3 | 89 |
//! | flip | 0 | 75 |
//! | motion | 0 | 59 |
//! | **field3d** | **1** | **33** |
//!
//! A razão está escrita no doc de [`smoke`], há meses: *«o estado vive neste arquivo, num
//! `thread_local`, em vez de num campo do `App`. Não é preguiça: `app_state.rs` é compartilhado e
//! a `line/sculpt3d` edita-o — um campo novo lá é uma colisão por conveniência.»* ⇒ **a família
//! que guardou o estado dela em casa foi a que conseguiu sair de casa.** É a lição nº 1 do
//! `HOWTO_partir_uma_familia_da_shell.md`, e ela é sobre as outras cinco, não sobre esta.
//!
//! # O que ficou na shell, e porquê
//!
//! - `main.rs` / `init.rs` / `App` / o esqueleto do laço / o `input_dispatch` (o roteador);
//! - **`field3d_snapshot_tests.rs`** — ele captura um `ProjectState`, que é a máquina de undo da
//!   shell. O doc dele já se chamava *«a metade de SHELL da ponte ECS»*;
//! - ~~**cinco alias de uma linha** (`field3d_views`, `_navball`, `_layout`, `_view_menu`,
//!   `_gizmo`), que existem só porque `sculpt3d_*` os consome e a `line/app-sculpt3d` está a mover
//!   os ficheiros dela **hoje**~~ — ⭐ **APAGADOS em 2026-09-11 pela W2/L3-B**, que era a condição
//!   escrita no topo de cada um deles (*«quando a `line/app-sculpt3d` fechar, este ficheiro
//!   some»*). Os chamadores escrevem hoje `ph2d_viewport3d::…` directamente.
//!   ⚠️ **A dívida está PAGA e o texto fica riscado, não apagado:** este item é a razão por que a
//!   piloto ficou a meio, e quem ler o §5.1 do HOWTO tem de poder emparelhar as duas pontas.
//!
//! [audit]: ../../../docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md

#![forbid(unsafe_code)]

use ph2d_app_host::{AppFamily, SmokeRouter};

// ── Os 19 módulos da família ────────────────────────────────────────────────────────────────
pub mod export;
pub mod export_job;
/// ADR-0161 W51 — a VIAGEM entre vistas: a câmera vai suavemente, com a lei de motion da casa.
pub mod flight;
pub mod gizmo_paint;
/// ADR-0161 W22 — a porta de ENTRADA: um arquivo de malha vira escultura dentro da peça.
pub mod import;
pub mod input;
/// ADR-0161 W25 — a VOZ do módulo: uma peça que não cozinha diz porquê, e diz uma vez.
pub mod mode;
pub mod notice;
pub mod pick;
/// ADR-0161 W24 — a resolução do preview é DERIVADA do relógio: grossa ao mexer, nítida ao assentar.
pub mod preview;
/// ADR-0161 W53 — o perfil DESENHADO vira peça: o fluxo do MoI, com a caneta que a casa já tem.
pub mod profile;
pub mod profile_live;
/// ADR-0161 W23 — o REGRESSO: um projeto carregado regenera cada escultura do arquivo que a nomeia.
pub mod reload;
pub mod scene;
/// ADR-0161 W100 — a PALETA de formas: o catálogo grande entra pelo modal genérico da casa.
pub mod shape_palette;
/// ADR-0161 W100 — o CATÁLOGO de formas: rótulo, família e construtor, uma linha por forma.
pub mod shapes;
/// ADR-0161 — o smoke do módulo (`PH2D_FIELD_SMOKE=1..32`): o **campo traçado** na tela, que é o
/// caminho pelo qual o artista vê a peça (a malha é só para exportar).
pub mod smoke;
/// ADR-0161 W26 — o NUMERO digitado no meio do gesto do gizmo (o `G X 0,5` do Blender).
pub mod typed;

// ── A moldura 3D partilhada, sob os nomes que esta família já usava ──────────────────────────
//
// ⚠️ **São ALIAS de uma linha para a [`ph2d_viewport3d`], não cópias.** Estes cinco módulos saíram
// para uma crate-folha porque `sculpt3d_*` também os consome (ver o doc do `lib.rs` de lá), e os
// alias existem por DOIS motivos concretos:
//
// 1. **Os ficheiros de teste deles usam `use super::*`** e leem os aliases do módulo pai — coisa
//    que um `pub use …::*` no sítio da chamada não dá. Cada módulo aqui reconstrói o escopo que o
//    teste espera, e o teste continua a exercitar a lei **através** do smoke do 3D, que é como ela
//    deve ser exercitada.
// 2. A reescrita `crate::field3d_X → crate::X` fica **uniforme sobre 901 sítios**. Um mapa com
//    excepções é onde nasce o `crate::gizmo_paint` que por acaso existe e aponta para o sítio
//    errado, em silêncio.
pub mod gizmo {
    pub use ph2d_viewport3d::gizmo::*;
}
pub mod navball_paint {
    pub use ph2d_viewport3d::navball_paint::*;
}

pub mod layout;
pub mod navball;
pub mod view_menu;
pub mod views;

/// **O que esta família declara à shell** (`ph2d-app-registry-init`).
///
/// ⚠️ **O `max_level` CONTA-SE no roteador**, nunca se escreve de memória — a fonte é o `match` de
/// [`smoke_scenes::scene`] e o gate que o mede é o `the_router_answers_for_every_level_it_claims`
/// em `smoke_scenes`. Esta nota já envelheceu once no `CLAUDE.md §5`, com cenas até `102` e a nota
/// parada em `97`.
///
/// [`smoke_scenes::scene`]: crate::smoke::scenes::scene
pub const FAMILY: AppFamily = AppFamily {
    key: "field3d",
    routers: &[SmokeRouter {
        env: "PH2D_FIELD_SMOKE",
        max_level: smoke::scenes::CENAS,
    }],
};

/// A porta dos gates desta crate que medem que o QUADRO da shell chama a família.
#[cfg(test)]
mod shell_frame_tests;
