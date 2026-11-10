//! **`ph2d-app-components` — a família das INSTÂNCIAS** (W2 Fase D; ADR-0164 / 0165 / 0166).
//!
//! Identidade de objecto (`StableId`), receitas e cópias vivas, variantes, excepções, o prefab que
//! se abre, e os quatro componentes do TOP-20 que a linha trouxe (`Timer` · `SignalActions` ·
//! `AudioSource2D` · `GameCamera`). Ela saiu de `shells/desktop` em 2026-09-12 pelo molde do
//! [HOWTO][howto]: **código de família vive em `crates/ph2d-app-<família>`; a shell é COMPOSIÇÃO.**
//!
//! O porquê das rodadas de saída (prefixos mantidos, o que não veio, a 5.ª rodada) está em
//! `docs/archive/integracao-jornadas/COMENTARIOS_CORTADOS_line_TimelinePrincipal_2026-11-10.md`.
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
/// ⭐ A arte de fábrica como imagem da biblioteca (plano GEF 12).
pub mod factory_images;
pub mod object_drag;
pub mod property_drag;
// ⛔⛔ **A ponte do SOM DE CENA FICOU na shell, e a razão é um GATE:** ela precisa do
// `ph2d-app-audio`, que é uma FAMÍLIA — e o `architecture_no_dependency_climbs_a_layer` recusa
// família → família (ADR-0075). A cura que ele prescreve é *«uma tabela injectada pela
// composição»*, e é o que o [`signal_actions_bridge::apply`] faz: ele pede `Som::Toca` a um fecho,
// e quem responde é a shell, dona dos dois lados. *A aresta foi tentada e o portão apanhou-a.*
pub mod audio_2d_smoke;
/// A lei dos números da fonte de som — o cartão (na shell) e o script passam por ela.
pub mod audio_inspector;
mod autokey_smoke_cena;
pub mod autokey_smoke_componentes;
/// ⭐⭐⭐ **A PONTE DA CÂMERA DE JOGO** (TOP-20 #7) — saiu da shell em 19/09; ver o cabeçalho.
pub mod camera_2d;
pub mod camera_2d_smoke;
/// ⭐⭐ **A secção CAMERA** (TOP-20 #7) — o instantâneo e o commit; veio da shell na W7 da paralaxe.
pub mod camera_inspector;
/// ⭐⭐ A família OBJETO nos canais: o skew e a aparência da sprite (plano 35 §3.0).
pub mod canais_componente;
pub mod canais_lista;
pub mod canais_objeto;
pub mod canais_script;
pub mod code_smoke;
pub mod code_smoke_inspector;
pub mod code_smoke_props;
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
/// ⭐⭐⭐ **A ponte da BARRA DE VIDA** (plano 28, W4) — o rasto e as três faixas de cada barra.
pub mod health_bar_bridge;
/// ⭐ TOP-20 #16 — a ponte dos scripts do artista (o quadro, o ledger e o rebobinar).
/// ⭐ Largar uma ou VÁRIAS linhas da Hierarquia sobre outra (F30 §F).
pub mod hierarchy_reparent;
pub mod hud_anchors;
pub mod hud_arranjo;
pub mod hud_bridge;
pub mod hud_inspector;
pub mod hud_smoke_anchors;
/// ⭐ O IMPACTO (plano 28, W5): a pausa no golpe e os números de dano a subir.
pub mod impacto;
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
/// ADR-0186: a secção MACRO do Inspector.
pub mod macro_inspector;
/// ⭐⭐ O losango de cada campo do Inspector (plano 35 §5).
pub mod marcas_inspector;
pub mod master_editing;
/// Ciclo 19 do Motion: o `Motion Input` e os sinais publicam as entradas do grafo.
pub mod motion_input_bridge;
/// A secção MOTION INPUT do Inspector — o instantâneo e o dreno (ciclo 19 do Motion).
pub mod motion_input_inspector;
/// Os motores da janela dos cérebros (scripts, partículas, vigias, gatilho, armas, barras) — a
/// shell chama-os pela ordem do quadro (`fase_signal_outbox`).
pub mod motores;
/// ⭐⭐⭐ As secções NAV REGION e NAV AGENT do Inspector — o instantâneo e o dreno (plano 30, W4).
pub mod nav_inspector;
pub mod nav_rota;
/// ⭐⭐⭐ **O mover de VISTA DE CIMA** (TOP-20 #13) — as duas cenas do dono.
/// ⭐⭐⭐ A NAVEGAÇÃO (plano 30, W3) — `PH2D_NAV_SMOKE=1`.
pub mod nav_smoke;
pub mod nav_smoke_estreita;
pub mod nav_smoke_guarda;
pub mod nav_smoke_lama;
pub mod nav_smoke_lava;
pub mod nav_smoke_porta;
pub mod nav_smoke_usos;
/// O vazio e os objectos de JOGO no menu Add de objectos (spec/06 F1).
pub mod object_add;
mod object_add_dicas;
/// O modo do objecto activo, com o tipo pelo marcador (spec/06 F2).
pub mod object_mode;
/// ⭐⭐⭐ **A PARALAXE** (plano 24, W1) — um objecto guarda uma fracção do movimento do mundo; a
/// lei, a referência e o caso do arrasto estão no cabeçalho.
pub mod parallax_bridge;
/// ⭐⭐⭐ **A PARALAXE** (plano 24, W7) — o instantâneo e o dreno da secção.
pub mod parallax_inspector;
/// ⭐⭐⭐ **A PARALAXE** (plano 24, W7) — `=1` os quatro planos e as duas leis lado a lado · `=2` o
/// dolly, que é o que nenhum outro motor deste género dá.
pub mod parallax_smoke;
/// ⭐⭐⭐ **A ponte dos emissores de partículas** (TOP-20 #18) — ver o cabeçalho do módulo.
pub mod particles_bridge;
/// ⭐⭐⭐ TOP-20 #18 — a secção PARTICLES do Inspector: o instantâneo e o dreno.
pub mod particles_inspector;
/// ⭐⭐⭐ O smoke do EMISSOR DE PARTÍCULAS (TOP-20 #18) — ver o cabeçalho.
pub mod particles_smoke;
/// ⭐⭐⭐ **O SEGUIDOR DE CAMINHO** (suplente #23) — onde o nome vira curva; ver o cabeçalho.
pub mod path_follow_bridge;
/// ⭐⭐⭐ **O SEGUIDOR DE CAMINHO** (suplente #23) — o instantâneo e o dreno da secção.
pub mod path_follow_inspector;
/// ⭐⭐⭐ **O SEGUIDOR DE CAMINHO** (suplente #23) — a pista desenhada e quem a percorre.
pub mod path_follow_smoke;
pub mod posse_do_jogo;
pub mod posse_do_jogo_registo;
pub mod prefab_stage;
/// ⭐⭐⭐ **O PROJÉCTIL** (TOP-20 #14) — as duas cenas do dono.
/// ⭐ O instantâneo e o dreno da secção PROJECTILE MOTION (TOP-20 #14).
pub mod projectile_inspector;
pub mod projectile_smoke;
/// ⭐⭐⭐ O instantâneo e o dreno da secção RAY SENSOR (suplente #21).
pub mod ray_inspector;
/// ⭐⭐⭐ **O smoke do OLHO** (suplente #21, W6) — um poste com dois olhos e uma caixa que chega.
pub mod ray_smoke;
/// ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25) — a cena do dono; ver o cabeçalho.
pub mod restart_smoke;
pub mod row_components;
pub mod scene_ctx;
pub mod script_bridge;
/// O comando de jogo, o teclado e o rato que os scripts leem (CodeEditor item 6).
pub mod script_input;
/// ⭐ TOP-20 #16 — a secção SCRIPT do Inspector: o instantâneo e o dreno.
pub mod script_inspector;
/// ADR-0188 — os scripts do projeto no `ProjectFile.scripts`: gravar, abrir (importar caminhos antigos).
pub mod script_project;
/// ⭐⭐⭐ O smoke do SCRIPT DO ARTISTA (TOP-20 #16) — ver o cabeçalho.
pub mod script_smoke;
/// ⭐⭐⭐ **O instantâneo da CUTSCENE** (TOP-20 #19) — ver o cabeçalho.
pub mod sequence_inspector;
/// ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25) — o sinal vira trauma; ver o cabeçalho.
pub mod shake_bridge;
/// ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25) — o instantâneo e o dreno do Inspector.
pub mod shake_inspector;
pub mod shake_smoke;
pub mod signal_action_smoke;
/// ⭐⭐⭐ **A PONTE da tabela nome→acção** (TOP-20 #5) — onde um sinal deixa de ser um toast e vira
/// jogo. ⚠️ Desceu da shell em 2026-09-19: ela era a **única** das oito pontes desta família que
/// ainda lá vivia, e a catraca `the_shell_only_shrinks` foi quem o disse.
pub mod signal_actions_bridge;
/// ⭐ **A secção SIGNAL ACTIONS do Inspector** (TOP-20 #5, W3) — o snapshot e o commit dela.
/// ⚠️ Desceu da shell em 2026-09-24 (plano 28, W4): é pura sobre o mundo e a árvore de tags, e a
/// catraca `the_shell_only_shrinks` estava a `16` linhas do tecto.
pub mod signal_actions_inspector;
/// ⭐ A cena 42 do smoke do modo (F42): um objecto de cada tipo na Hierarquia.
pub mod smoke_tipos;
/// GEF 21 E2 — os sons do projeto no `ProjectFile.sounds`: importar, gravar, abrir (caminhos antigos).
pub mod sound_project;
/// ⭐⭐⭐ O smoke do CÉREBRO AUTORÁVEL (TOP-20 #15) — ver o cabeçalho.
pub mod state_machine_tick;
pub mod statemachine_inspector;
pub mod statemachine_smoke;
pub mod tags_doc;
/// ⭐⭐⭐ As duas cenas do TOP-20 #9 (`PH2D_TAGS_SMOKE=1|2`) — ver o cabeçalho do módulo.
pub mod tags_smoke;
/// A secção TIMERS do Inspector: o snapshot e o commit (desceu da shell).
pub mod timer_inspector;
pub mod timer_smoke;
/// ⭐⭐⭐ **A ponte do `Timer`** (TOP-20 #2) — o tique no passo fixo e o sinal que sai dele.
/// Desceu de `shells/desktop/src/render_loop/` em 2026-10-25.
pub mod timer_tick;
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
/// ⭐⭐⭐ A ARENA (plano 28, W7) — um jogo pequeno com vida, do princípio ao fim;
/// `PH2D_VIDA_SMOKE=4`.
pub mod vida_arena_smoke;
/// ⭐ A cena do IMPACTO (plano 28, W5) — `PH2D_VIDA_SMOKE=2`.
pub mod vida_impacto_smoke;
/// ⭐⭐⭐ O instantâneo e o dreno das secções HEALTH e DAMAGE (plano 28, W3).
pub mod vida_inspector;
/// ⭐⭐⭐ **O smoke da VIDA** (plano 28, W2) — três alvos, três vidas, e o aliado que não morre.
pub mod vida_smoke;
/// ⭐ A cena dos TIPOS de dano e do dano que DURA (plano 28, W6) — `PH2D_VIDA_SMOKE=3`.
pub mod vida_tipos_smoke;
/// ⭐⭐⭐ **A ARMA** — a ponte que lê o pente, escreve a munição e publica o tiro; ver o cabeçalho.
pub mod weapon_bridge;
/// ⭐⭐⭐ **A ARMA** — o instantâneo e o dreno da secção dela; ver o cabeçalho.
pub mod weapon_inspector;
/// ⭐⭐⭐ **A ARMA** — três torretas, uma tecla e uma diferença cada; ver o cabeçalho.
pub mod weapon_smoke;

/// ⚠️ **`#[cfg(any(test, feature = "test-support"))]` e não `#[cfg(test)]`** (HOWTO §2.5): daqui a
/// shell é um **consumidor**, e um `cfg(test)` desta crate é falso quando ela a compila. As duas
/// costuras que ficaram lá precisam deste arnês.
#[cfg(any(test, feature = "test-support"))]
pub mod component_registry_for_tests;
/// ⛔⛔ A cópia que a fábrica faria de um molde — o arnês partilhado pelos gates das cenas da vida.
#[cfg(test)]
mod smoke_copia;
/// O desenho encaixado no corpo (disco + «Rumo») das cenas de smoke.
pub(crate) mod smoke_desenho;

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
/// O nome ÚNICO de uma imagem da biblioteca — o do painel Assets e o do script.
/// W11 do Kanban — a cena publica-se aos cartões e serve os pedidos «abrir» deles.
pub mod kanban_scene;
pub mod library_names;
#[cfg(test)]
mod property_coverage_tests;
pub(crate) mod property_laws;
#[cfg(test)]
mod property_names_tests;
#[cfg(test)]
mod property_refs_tests;
/// ⭐ A imagem de uma sprite trocada por um script a correr (`sprite.image`, doc CodeEditor 08 d4).
pub mod script_image;
/// ⭐ O nome de um objecto trocado por um script a correr (`name`, doc CodeEditor 11).
pub mod script_name;
pub(crate) mod script_properties;
#[cfg(test)]
mod smoke_desenho_e_corpo_tests;
/// ⭐ A lei do valor dos cartões do núcleo e da imagem — a que o Inspector e o script partilham.
pub mod sprite_inspector;

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
        // ⭐⭐⭐ O ABANÃO DA VISTA (suplente #25): `=1` o pátio onde a mesma bomba abana mais de
        // perto. ⚠️ O `max_level` é **contado** no corpo do `montar`.
        r("PH2D_RESTART_SMOKE", restart_smoke::CENAS),
        r("PH2D_SHAKE_SMOKE", shake_smoke::CENAS),
        r("PH2D_TIMER_SMOKE", 1),
        // ⭐⭐⭐ O mover de VISTA DE CIMA (TOP-20 #13): `=1` o corredor · `=2` a isometria com o
        // controlo ao lado. ⚠️ O `max_level` é **contado** no `match` do `montar`.
        r("PH2D_TOPDOWN_SMOKE", topdown_smoke::CENAS),
        // ⭐⭐⭐ A NAVEGAÇÃO (plano 30, W3): `=1` o labirinto em S com o controlo ao lado.
        r("PH2D_NAV_SMOKE", nav_smoke::CENAS),
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
        // ⭐⭐⭐ O SEGUIDOR DE CAMINHO (suplente #23): `=1` a pista desenhada, os três que a percorrem
        // e o CONTROLO que corta pelo meio.
        r("PH2D_PATHFOLLOW_SMOKE", path_follow_smoke::CENAS),
        // ⭐⭐⭐ A PARALAXE (plano 24): `=1` o fundo de quatro planos, com a repetição e a cerca
        // lado a lado · `=2` o dolly. ⚠️ O `max_level` é **contado** no `montar`.
        r("PH2D_PARALLAX_SMOKE", parallax_smoke::CENAS),
        // ⭐⭐⭐ A VIDA (plano 28, W2): `=1` três alvos que morrem ao 1.º, 2.º e 3.º tiro, e o
        // aliado que as balas não ferem (o CONTROLO).
        r("PH2D_VIDA_SMOKE", vida_smoke::CENAS),
        // ⭐⭐⭐ A ARMA DO JOGADOR: `=1` as três colunas. ⚠️ O `max_level` é **contado**: a shell
        // chama `weapon_smoke::montar(world, 1)` e a `montar` ignora o nível (`CENAS = 1`).
        r("PH2D_WEAPON_SMOKE", weapon_smoke::CENAS),
    ],
};

const fn r(env: &'static str, max_level: u32) -> ph2d_app_host::SmokeRouter {
    ph2d_app_host::SmokeRouter { env, max_level }
}
