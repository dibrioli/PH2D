//! **O que uma CENA de smoke do Motion pede à shell** — no vocabulário da família, não no da `App`.
//!
//! # Porque existe (W2 Fase C, 2026-09-12)
//!
//! As 14 cenas desta família assinavam `app: &mut crate::App`, e uma crate **não pode nomear um
//! tipo que vive no binário da shell**. ⛔ A saída NÃO é um sexto método no
//! [`ph2d_app_host::AppHost`] — o trait tem cinco e nenhum devolve um handle
//! (`HOWTO §1.5`). A saída é a regra 4 do bloco: *antes de pedir porta, escreva em TIPOS o que a
//! função precisa*.
//!
//! Medido: as 14 cenas tocam **dez** campos, e **todos** são nomeáveis de fora da shell —
//! `MotionState` e `MotionShellState` viajam com a família, os outros oito são crates irmãs.
//! ⭐ **Zero portas novas.** É o mesmo molde do [`ph2d_app_physics::SceneCtx`], que a batedora
//! provou em 117 cenas.
//!
//! # ⚠️ A guarda do `gfx` subiu para o CHAMADOR, e isso é a decisão
//!
//! Cada cena abria com `if app.gfx.is_none() { return; }` — catorze cópias da mesma pergunta, e
//! catorze sítios onde a resposta podia divergir. Quem sabe se há `gfx` é quem o segura: o laço.
//! ⇒ o `render_loop` constrói o contexto **dentro** do `if let Some(gfx)`, e uma cena que corre
//! já tem tudo. *Uma pergunta feita no sítio onde a resposta existe não precisa de ser repetida.*
//!
//! # ⛔ Porque são dez campos e não um `&mut AppGfx`
//!
//! Devolver o `AppGfx` desfaria a fronteira inteira (é a proibição escrita no topo do
//! `ph2d-app-host`): a família voltaria a poder tudo, e a crate voltaria a recompilar quando a
//! shell mudasse. Dez campos **nomeados** são a lista do que esta família de facto alcança — e ela
//! é auditável: acrescentar um obriga a dizer de quem é o tipo.

use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_editor::ToolRegistry;

/// **A fatia do mundo que uma cena de Motion pode tocar.**
///
/// Os cinco primeiros vêm do `AppGfx`; os cinco seguintes da `App`. ⚠️ Eles são **campos
/// disjuntos** dos dois, e é isso que deixa o chamador emprestar os dez ao mesmo tempo.
pub struct MotionSceneCtx<'a> {
    /// O documento e os sinks da família — o que 69 dos 130 acessos medidos pedem.
    pub motion: &'a mut crate::motion_state::MotionState,
    /// Para uma cena poder pôr a ferramenta «motion» na mão.
    pub tools: &'a mut ToolRegistry,
    /// O mundo ECS que as cenas povoam.
    pub sim: &'a mut SimWorld,
    /// A cena vetorial, para quem carimba sobre formas.
    pub vec_scene: &'a mut ph2d_vec_scene::VecScene,
    /// O documento de Flip, para as cenas de mídia mista.
    pub flip: &'a mut ph2d_flip::FlipDoc,
    /// O mapa objecto-Flip ⟷ entidade (folha desde 12/09).
    pub vec_entities: &'a mut ph2d_vec_entities::entities::VecEntityMap,
    /// O estado de shell da própria família.
    pub motion_shell: &'a mut crate::motion_shell_state::MotionShellState,
    /// ⚠️ Só LEITURA: a cena pergunta o estado do Flip, não o escreve.
    pub flip_state: &'a ph2d_app_flip::state::FlipState,
    /// ⚠️⚠️ **`&mut`, e a 1.ª redacção deste ficheiro dizia o contrário.** Eu escrevi *«só
    /// LEITURA: o relógio é do transporte, e uma cena que o escrevesse lutaria com ele»* — e a
    /// cena `=7` do `motion_object_smoke` chama `playhead.play()` **de propósito**, com a razão
    /// escrita ao lado: *«um offset de tempo só é visível numa animação que CORRE; uma cena
    /// parada mostraria dois desenhos diferentes e não diria se a diferença é de FASE»*.
    /// *Uma cena de smoke não é um observador — ela ENCENA, e encenar inclui carregar no play.*
    pub playhead: &'a mut Playhead,
    /// Para as cenas que AUTORAM uma track — a mesma porta que a física abriu.
    pub timeline: &'a mut ph2d_timeline::TimelineState,
    /// ⚠️ **O 11.º campo, e ele não estava na 1.ª redacção deste ficheiro.** A régua do fecho
    /// listou-o (`.hero_screen 1 f`) e eu li a linha como *«uma coisa que o `AppGfx` segura»*
    /// em vez de *«uma coisa que uma CENA toca»* — foi o compilador que o apanhou, em dois
    /// sítios do `motion_path_smoke`, os dois a fazer `hero.gizmo.replace_selection(..)`.
    /// *Uma tabela de campos ordenada por frequência põe o caso raro no fim, e o fim é onde
    /// se deixa de ler.*
    ///
    /// `Option` porque um smoke pode correr antes de haver ecrã.
    pub hero: Option<&'a mut ph2d_editor::HeroScreen>,
}
