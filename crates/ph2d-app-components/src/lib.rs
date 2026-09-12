//! **`ph2d-app-components` — a família das INSTÂNCIAS** (W2 Fase D; ADR-0164 / 0165 / 0166).
//!
//! Identidade de objecto (`StableId`), receitas e cópias vivas, variantes, excepções, o prefab que
//! se abre, e os quatro componentes do TOP-20 que a linha trouxe (`Timer` · `SignalActions` ·
//! `AudioSource2D` · `GameCamera`). Ela saiu de `shells/desktop` em 2026-09-12 pelo molde do
//! [HOWTO][howto]: **código de família vive em `crates/ph2d-app-<família>`; a shell é COMPOSIÇÃO.**
//!
//! # ⚠️ Por que os ficheiros MANTIVERAM o prefixo, ao contrário das seis irmãs
//!
//! O HOWTO §1.3 diz *«o prefixo `<fam>_` sai: dentro da crate tudo é a família»*, e as seis famílias
//! anteriores fizeram-no (`field3d_gizmo.rs` → `gizmo.rs`). ⛔ **Aqui a regra não se aplica, e não é
//! preguiça:** ali o prefixo **era o nome da família**; aqui a família é `components` e os prefixos
//! são `instance_` e `component_`, que são **dois assuntos DENTRO dela** — `instance_verbs` («os
//! verbos de uma instância») e `component_attach` («anexar um componente») não são redundantes com o
//! nome da crate, são o que distingue as duas metades.
//!
//! ⚠️ E a régua que decide não é estética: **despi-los COLIDE.** `instance_smoke.rs` e
//! `component_smoke.rs` reduzem os dois a `smoke.rs`.
//!
//! ⭐ O efeito colateral é que a reescrita interna foi **zero**: `crate::instance_verbs::X` continua
//! a resolver, porque o módulo tem o mesmo nome deste lado. As duas armadilhas mudas do HOWTO §2.2
//! (`crate::foo` é prefixo de `crate::foo_bar`) e §2.3 (o `use` liga o nome NU) simplesmente não
//! têm onde morder dentro da crate — só na shell, onde a reescrita é `crate::X` →
//! `ph2d_app_components::X` e a âncora é o nome inteiro.
//!
//! # ⛔ O que NÃO veio, e porquê
//!
//! - **`init.rs::build_component_registry`** — regista componentes de cinco crates irmãs, logo é
//!   **composição** e fica na shell por desenho (ESTADO §3; a `line/app-physics` deixou quatro
//!   ficheiros para trás pela mesma razão). Era a âncora que valia **91 %** do fecho desta família,
//!   e a cura é o [`test_support::registo`], que monta o **mesmo** catálogo com um gate a exigir
//!   igualdade exacta.
//! - **`project*` e `undo*`** — a persistência e a fila de undo, uma por desenho. ⚠️ O bloco de
//!   reabertura desta linha avisava que *«o `ProjectState::capture` e o `deep_copy_subtree` são o
//!   coração do módulo»* e que a fronteira aqui seria *lei pura ↔ ponte*. **Medido, essa fronteira
//!   não existe:** o `capture` nunca apareceu como âncora do fecho — o corte já estava feito, e o
//!   `deep_copy_subtree` vive no `ph2d-ecs` (crate irmã) com o [`instantiate`] a ser a porta única
//!   que o compõe com o remap.
//! - **Duas COSTURAS de teste** ficam na shell porque o sujeito delas é meio chrome de famílias que
//!   ainda não saíram — ver [`test_support`].
//!
//! [howto]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

#![forbid(unsafe_code)]

pub mod audio_2d_smoke;
pub mod camera_2d_smoke;
pub mod component_attach;
pub mod component_palette;
pub mod component_seed;
pub mod component_smoke;
pub mod instance_added;
pub mod instance_added_smoke;
pub mod instance_apply_deep;
pub mod instance_diag;
pub mod instance_docs;
pub mod instance_move_smoke;
pub mod instance_nested_smoke;
pub mod instance_open;
pub mod instance_refs;
pub mod instance_removed_smoke;
pub mod instance_replace_smoke;
pub mod instance_revert;
pub mod instance_smoke;
pub mod instance_structure;
pub mod instance_swap_match;
pub mod instance_sync;
pub mod instance_sync_docs;
pub mod instance_unmake;
pub mod instance_variant;
pub mod instance_verbs;
pub mod instance_verbs_walk;
pub mod instantiate;
pub mod master_editing;
pub mod scene_ctx;
pub mod signal_action_smoke;
pub mod timer_smoke;

/// ⚠️ **`#[cfg(any(test, feature = "test-support"))]` e não `#[cfg(test)]`** (HOWTO §2.5): daqui a
/// shell é um **consumidor**, e um `cfg(test)` desta crate é falso quando ela a compila. As duas
/// costuras que ficaram lá precisam deste arnês.
#[cfg(any(test, feature = "test-support"))]
pub mod component_registry_for_tests;

/// ⛔ **O arnês que ATRAVESSA a fronteira, e nada mais** — as duas costuras de teste que ficaram
/// na shell porque o sujeito delas é meio chrome. Ver o cabeçalho dele.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

/// **A declaração da família** — a chave e os roteadores de smoke que ela POSSUI.
///
/// # ⚠️ São CINCO, e o bloco de reabertura listava QUATRO — as três correcções são a mesma lei
///
/// O bloco nomeava *«~7 ficheiros de roteador»* e citava `PH2D_SIGNAL_TABLE_SMOKE`. Contados no
/// código — `grep` por `var(_os)?("PH2D_…")`, que é o que uma env **LIDA** parece — são **cinco**,
/// e cada correcção é *a unidade da posse é o ASSUNTO, nunca o prefixo do nome* (regra 5 da Fase D):
///
/// - ⛔ **`PH2D_SIGNAL_TABLE_SMOKE` não existe.** Nenhum ficheiro do repo o lê; o
///   `signal_table_smoke.rs` é um nível do `PH2D_BUILD_SMOKE` (`=68`), que fica na shell.
/// - ⛔ **`signal_smoke.rs` não é desta família** — é a cena do R0/timeline (ADR-0143), e o próprio
///   ficheiro abre a avisar: *«Não confundir com o `crate::signal_smoke`»*.
/// - ⭐ **`camera_2d_smoke.rs` lê `PH2D_GAME_CAMERA_SMOKE`** — derivar a env do nome do ficheiro
///   teria declarado uma variável que ninguém lê.
/// - ⭐⭐ **E o quinto NÃO ESTAVA EM LISTA NENHUMA:** o `PH2D_INSTANCE_SMOKE`, que é o roteador das
///   sete cenas de instância — o mais antigo da família e o único com `match`. Ele não aparece nos
///   `Smokes:` do `CLAUDE.md` §5 nem no handoff de 10/09, porque aquelas listas foram escritas
///   *por wave* e ele é anterior a todas elas. *Uma lista de roteadores mantida a cada jornada
///   descreve as jornadas, não a família.*
///
/// ⚠️ `PH2D_INSTANCE_LOG` é **diagnóstico** e não entra: um roteador declarado diz ao dono que ele
/// tem uma cena para ver.
///
/// # Cada `max_level` é CONTADO no corpo do roteador
///
/// Quatro são **interruptores** (`std::env::var_os(..).is_none()` ⇒ UMA cena ⇒ `1`); o
/// `PH2D_INSTANCE_SMOKE` tem um `match` com os braços `"1"`..`"7"`, contados ⇒ **`7`**.
/// ⛔ Nunca de memória (CLAUDE.md §5.0).
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "components",
    routers: &[
        r("PH2D_AUDIO_2D_SMOKE", 1),
        r("PH2D_GAME_CAMERA_SMOKE", 1),
        r("PH2D_INSTANCE_SMOKE", 7),
        r("PH2D_SIGNAL_ACTION_SMOKE", 1),
        r("PH2D_TIMER_SMOKE", 1),
    ],
};

const fn r(env: &'static str, max_level: u32) -> ph2d_app_host::SmokeRouter {
    ph2d_app_host::SmokeRouter { env, max_level }
}
