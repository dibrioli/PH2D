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
//! # ⭐⭐ A 5.ª rodada (2026-09-13): as LEIS dos assets e a LEI do palco do prefab
//!
//! Vieram os ficheiros que o fecho de 12/09 deixara para trás por **ASSUNTO** (ADR-0165, o plano 07 e
//! o *Edit Prefab* de 07/09): o índice (`asset_index_build`), o que um cartão desenha
//! (`asset_card_art`/`_portrait`), os verbos do cartão e do catálogo, a lei da queda (`asset_drop`)
//! e a lei do palco (`prefab_stage`) — **14 ficheiros**, mais o censo da porta da textura. As quatro
//! âncoras que as prendiam, medidas pelo `scripts/fecho-da-familia.py`, curaram-se cada uma pela
//! espécie certa: o `init.rs` pelo `component_registry_for_tests::registo` que já existia · a
//! sub-UV (`render_loop::sim_extract`) é uma LEI com dois leitores e **desceu para o motor**
//! (`ph2d_render::sprite`) · o `canvas_area` era uma **fachada** e morreu · e o `undo.rs` só era
//! tocado pela PONTE das saídas, que se partiu da lei.
//!
//! ⛔ **Ficam na shell, por desenho** — e o fecho diz porquê em tipos:
//! - `asset_drag_wire` e `asset_drop_apply` — o arrasto precisa da câmara e do pick, e o braço da
//!   queda do `SpriteRenderer` e do funil `commit_edited_texture` (as âncoras `image_import` e
//!   `texture_edit` são DELES, não das leis);
//! - `prefab_exit` — o `Cancel` repõe o `ProjectState`, que é da fila de undo;
//! - os smokes dirigidos pelo PONTEIRO (`asset_menu_smoke`, `variant_*_smoke`, níveis do
//!   `PH2D_BUILD_SMOKE`) — conduzem a `App` real, e o `AppHost` não tem porta para isso.
//!
//! ⚠️ O `nest_smoke`, que o handoff de 12/09 contava nesta família, é da **Timeline** (ADR-0133):
//! *o prefixo não é a família* (HOWTO §2.15).
//!
//! [howto]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

#![forbid(unsafe_code)]

pub mod action_trigger_inspector;
pub mod asset_card_art;
pub mod asset_card_portrait;
pub mod asset_card_verbs;
pub mod asset_catalog_verbs;
pub mod asset_drop;
pub mod asset_index_build;
// ⛔⛔ **A ponte do SOM DE CENA FICOU na shell, e a razão é um GATE:** ela precisa do
// `ph2d-app-audio`, que é uma FAMÍLIA — e o `architecture_no_dependency_climbs_a_layer` recusa
// família → família (ADR-0075). A cura que ele prescreve é *«uma tabela injectada pela
// composição»*, e é o que o [`signal_actions_bridge::apply`] faz: ele pede `Som::Toca` a um fecho,
// e quem responde é a shell, dona dos dois lados. *A aresta foi tentada e o portão apanhou-a.*
pub mod audio_2d_smoke;
pub mod camera_2d_smoke;
pub mod component_attach;
pub mod component_palette;
pub mod component_seed;
pub mod component_smoke;
/// ⭐⭐⭐ **A PONTE da FÁBRICA e da MORTE** (TOP-20 #11 e #12) — os factos que a lei pura devolve
/// viram objectos no mundo.
pub mod counter_watch_bridge;
pub mod counter_watch_inspector;
pub mod counter_watch_smoke;
/// ⭐⭐⭐ **O smoke do GOLPE** (suplente #24) — duas fileiras de alvos e uma caixa de diferença.
pub mod dano_smoke;
pub mod factory_bridge;
/// ⭐⭐⭐ **A FÁBRICA e o CICLO DE VIDA** (TOP-20 #11 e #12) — as duas cenas do dono.
pub mod factory_smoke;
/// ⭐ TOP-20 #16 — a ponte dos scripts do artista (o quadro, o ledger e o rebobinar).
pub mod hud_bridge;
pub mod hud_inspector;
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
/// ⭐⭐⭐ **A ponte dos emissores de partículas** (TOP-20 #18) — ver o cabeçalho do módulo.
pub mod particles_bridge;
/// ⭐⭐⭐ TOP-20 #18 — a secção PARTICLES do Inspector: o instantâneo e o dreno.
pub mod particles_inspector;
/// ⭐⭐⭐ O smoke do EMISSOR DE PARTÍCULAS (TOP-20 #18) — ver o cabeçalho.
pub mod particles_smoke;
pub mod prefab_stage;
/// ⭐⭐⭐ **O PROJÉCTIL** (TOP-20 #14) — as duas cenas do dono.
/// ⭐ O instantâneo e o dreno da secção PROJECTILE MOTION (TOP-20 #14).
pub mod projectile_inspector;
pub mod projectile_smoke;
/// ⭐⭐⭐ O instantâneo e o dreno da secção RAY SENSOR (suplente #21).
pub mod ray_inspector;
/// ⭐⭐⭐ **O smoke do OLHO** (suplente #21, W6) — um poste com dois olhos e uma caixa que chega.
pub mod ray_smoke;
pub mod scene_ctx;
pub mod script_bridge;
/// ⭐ TOP-20 #16 — a secção SCRIPT do Inspector: o instantâneo e o dreno.
pub mod script_inspector;
/// ⭐⭐⭐ O smoke do SCRIPT DO ARTISTA (TOP-20 #16) — ver o cabeçalho.
pub mod script_smoke;
/// ⭐⭐⭐ **O instantâneo da CUTSCENE** (TOP-20 #19) — ver o cabeçalho.
pub mod sequence_inspector;
pub mod signal_action_smoke;
/// ⭐⭐⭐ **A PONTE da tabela nome→acção** (TOP-20 #5) — onde um sinal deixa de ser um toast e vira
/// jogo. ⚠️ Desceu da shell em 2026-09-19: ela era a **única** das oito pontes desta família que
/// ainda lá vivia, e a catraca `the_shell_only_shrinks` foi quem o disse.
pub mod signal_actions_bridge;
/// ⭐⭐⭐ O smoke do CÉREBRO AUTORÁVEL (TOP-20 #15) — ver o cabeçalho.
pub mod statemachine_smoke;
pub mod tags_doc;
/// ⭐⭐⭐ As duas cenas do TOP-20 #9 (`PH2D_TAGS_SMOKE=1|2`) — ver o cabeçalho do módulo.
pub mod tags_smoke;
pub mod timer_smoke;
/// ⭐⭐⭐ **O mover de VISTA DE CIMA** (TOP-20 #13) — as duas cenas do dono.
pub mod topdown_smoke;
/// ⭐⭐⭐ **O GATILHO** (suplente #24) — a ponte PURA do teclado; ver o cabeçalho.
pub mod trigger_bridge;
/// ⭐⭐⭐ **O GATILHO** (suplente #24) — a arma que aponta e a que não aponta; ver o cabeçalho.
pub mod trigger_smoke;
pub mod tween_bridge;
pub mod tween_inspector;
/// ⭐⭐⭐ **O TWEEN** (suplente #22) — a galeria dos canais e a cópia que nasce a meio da
/// corrida; ver o cabeçalho.
pub mod tween_smoke;

/// ⚠️ **`#[cfg(any(test, feature = "test-support"))]` e não `#[cfg(test)]`** (HOWTO §2.5): daqui a
/// shell é um **consumidor**, e um `cfg(test)` desta crate é falso quando ela a compila. As duas
/// costuras que ficaram lá precisam deste arnês.
#[cfg(any(test, feature = "test-support"))]
pub mod component_registry_for_tests;

/// ⛔ **O arnês que ATRAVESSA a fronteira, e nada mais** — as duas costuras de teste que ficaram
/// na shell porque o sujeito delas é meio chrome. Ver o cabeçalho dele.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

/// ⛔⛔⛔ **O censo «que textura usa esta peça?» tem UMA porta** — veio de
/// `shells/desktop/tests/it/the_index_asks_the_texture_door.rs` com os três ficheiros que ele vigia
/// (W2 5.ª rodada, 2026-09-13). *Um censo mora com o sujeito que mede* (HOWTO §2.6).
#[cfg(test)]
mod asset_texture_door_census_tests;
/// ⭐ **Veio da shell na integração de 2026-09-16** — a catraca `the_shell_only_shrinks` ficou
/// vermelha por ACUMULAÇÃO das seis linhas da rodada, e a cura dela é MOVER, nunca subir o
/// número. A §8 *Visibility* do Inspector não tocava em nada da shell (zero `crate::`): ela é
/// código desta família, e o sítio dela é aqui.
pub mod inspector_visibility;
#[cfg(test)]
mod instance_tags_tests;

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
        // ⭐⭐⭐ As TAGS (TOP-20 #9, W4): `=1` o sinal que fala com a família · `=2` a armadilha
        // filtrada. ⚠️ O `max_level` é **contado** no `match` do `tags_smoke::tags_smoke`.
        // ⭐⭐⭐ A FÁBRICA (TOP-20 #11 e #12, W4): `=1` a chuva que não cresce · `=2` os pontos
        // marcados com limite e colheita. ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_FACTORY_SMOKE", factory_smoke::CENAS),
        r("PH2D_TAGS_SMOKE", tags_smoke::CENAS),
        r("PH2D_TIMER_SMOKE", 1),
        // ⭐⭐⭐ O mover de VISTA DE CIMA (TOP-20 #13): `=1` o corredor · `=2` a isometria com o
        // controlo ao lado. ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_TOPDOWN_SMOKE", topdown_smoke::CENAS),
        r("PH2D_PROJECTILE_SMOKE", projectile_smoke::CENAS),
        // ⭐⭐⭐ O CÉREBRO (TOP-20 #15): `=1` a porta com o CONTROLO ao lado.
        // ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_STATEMACHINE_SMOKE", statemachine_smoke::CENAS),
        // ⭐⭐⭐ O SCRIPT DO ARTISTA (TOP-20 #16): `=1` três bonecos, um ficheiro.
        r("PH2D_SCRIPT_SMOKE", script_smoke::CENAS),
        // ⭐⭐⭐ O EMISSOR DE PARTÍCULAS (TOP-20 #18): `=1` a galeria das quatro fontes · `=2` o
        // rasto e a tocha. ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_PARTICLES_SMOKE", particles_smoke::CENAS),
        // ⭐⭐⭐ O GATILHO (suplente #24, 18/09) e o GOLPE (19/09) — ⚠️ **o do gatilho nasceu FORA
        // desta lista** e entra agora: ela é a declaração que a família faz à shell, e uma cena que
        // não está aqui é uma cena que a família não declara possuir.
        r("PH2D_TRIGGER_SMOKE", trigger_smoke::CENAS),
        r("PH2D_DANO_SMOKE", dano_smoke::CENAS),
        // ⭐⭐⭐ O OLHO (suplente #21, W6): `=1` o poste com dois olhos e a caixa que chega.
        r("PH2D_RAY_SMOKE", ray_smoke::CENAS),
        // ⭐⭐⭐ O TWEEN (suplente #22): `=1` a galeria dos canais · `=2` a cópia que nasce a
        // meio da corrida. ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_TWEEN_SMOKE", tween_smoke::CENAS),
    ],
};

const fn r(env: &'static str, max_level: u32) -> ph2d_app_host::SmokeRouter {
    ph2d_app_host::SmokeRouter { env, max_level }
}
