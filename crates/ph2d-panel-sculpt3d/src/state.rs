//! **O que o artista autora, e o que ele mandou fazer.**
//!
//! ## Por que intents, e não `ToolPanelEvent`
//!
//! Todo painel docado deste app encaminha para uma **tool**
//! (`EditorAction::ToolPanelEvent` → `tool.handle_panel_event`). A cena 3D não é
//! uma `Tool` e não pode ser — a navegação orbital mora no shell justamente para
//! manter o contrato congelado intacto (ADR-0150) —, então este painel segue o
//! precedente do de física: empurra intents numa fila e a ponte do shell as
//! drena. O shell continua a única coisa que toca a `Sculpt3dScene`.
//!
//! ## Por que o estado autorado é UMA struct
//!
//! [`Sculpt3dUi`] junta tudo o que o artista **ajusta** — o pincel, o raio, o
//! espelho, a cavidade, a luz, o detalhe. O painel recebe uma cópia por frame,
//! edita UM campo e devolve a struct INTEIRA ([`Sculpt3dIntent::SetUi`]). Um
//! intent por knob seriam quinze maneiras de dizer a mesma coisa e quinze
//! lugares para o shell esquecer um; é a mesma lei do `PhysicsIntent::SetSettings`.
//!
//! ⚠️ **O que NÃO está nela são os gestos com consequência** — subdividir,
//! remalhar, apagar uma peça. Esses não são um valor que se ajusta, são uma
//! coisa que ACONTECE, e enfiá-los num `SetUi` faria toda mexida de slider ter
//! de decidir se o remesh já rodou.

use ph2d_sculpt3d::{Brush, Symmetry};
use std::cell::{Cell, RefCell};

thread_local! {
    /// O retrato vivo que o host publica antes de cada `paint`. `None` até a
    /// cena 3D existir — e é isso que faz o painel se recusar a pintar.
    static CURRENT: RefCell<Option<Sculpt3dSnapshot>> = const { RefCell::new(None) };
    /// O que o artista fez, esperando o shell drenar.
    static INTENTS: RefCell<Vec<Sculpt3dIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// **O estado AUTORADO da cena 3D** — tudo o que um controle contínuo ou um
/// rádio deste painel escreve.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sculpt3dUi {
    /// O verbo, a curva, a força e os dois knobs condicionais.
    ///
    /// ⚠️ O `Brush::radius` é de MUNDO e **derivado por dab** (contra a câmera e
    /// o ponto de acerto), então ele viaja aqui e ninguém o edita: quem o artista
    /// ajusta é o [`Sculpt3dUi::radius_px`] logo abaixo. Guardar os dois num
    /// campo só seria a segunda resposta a *"que tamanho tem o pincel?"*.
    pub brush: Brush,
    /// O raio autorado, em **pixels de tela**.
    pub radius_px: f32,
    pub symmetry: Symmetry,
    /// Quanto a curvatura escurece a fresta e clareia a crista.
    pub cavity: f32,
    /// Azimute da lâmpada selecionada, em graus.
    pub light_az_deg: f32,
    /// Elevação da lâmpada selecionada, em graus.
    pub light_elev_deg: f32,
    /// **COM QUE LUZ** — `None` é o rig do artista, `Some(i)` é o matcap `i`.
    ///
    /// ⚠️ Ele mora no estado AUTORADO e não nos fatos porque o artista o escolhe;
    /// mas ele **não é do documento** (o shell não o salva) — escolher com que
    /// luz olhar não muda a escultura.
    pub matcap: Option<u8>,
    /// A malha de arestas por cima da forma.
    pub wireframe: bool,
    /// Qual degrau de detalhe a topologia dinâmica usa (índice em `DETAIL_STEPS`).
    pub detail: u8,
}

impl Default for Sculpt3dUi {
    fn default() -> Self {
        Self {
            brush: Brush::default(),
            radius_px: 50.0, // LITERAL-PX-OK: espelha o DEFAULT_RADIUS_PX do shell (raio de pincel, medido)
            symmetry: Symmetry::default(),
            cavity: 0.0,
            light_az_deg: 0.0,
            light_elev_deg: 45.0, // LITERAL-PX-OK: graus de elevacao, nao metrica de design
            matcap: None,
            wireframe: false,
            detail: 1,
        }
    }
}

/// O que o painel precisa saber da cena neste frame: o estado autorado mais os
/// **fatos** que ele só mostra.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Sculpt3dSnapshot {
    pub ui: Sculpt3dUi,
    /// A topologia dinâmica está armada? **Lido, nunca escrito por `SetUi`** —
    /// ligá-la TRIANGULA a malha, e uma consequência dessas não pode viajar
    /// dentro de um struct de valores que todo arrasto de slider reenvia.
    pub dyntopo: bool,
    /// O nível de multiresolução vivo, e quantos existem.
    pub level: usize,
    pub level_count: usize,
    /// Quantas peças a cena tem, e se uma delas está isolada.
    pub pieces: usize,
    pub isolated: bool,
    /// Quantos vértices a malha viva tem. Zero é digno de ver: é a diferença
    /// entre *"o pincel não funciona"* e *"esta peça está vazia"*.
    pub verts: usize,
    /// **OS NOMES DOS MATERIAIS de matcap**, na ordem em que o renderizador os
    /// numera.
    ///
    /// ⚠️ **Eles chegam no retrato em vez de o painel os importar**, e a razão é
    /// uma aresta de dependência: quem os conhece é a `ph2d-mesh-render`, que
    /// carrega o `wgpu` inteiro. Um painel que a importasse passaria a compilar
    /// um backend gráfico para escrever seis palavras — e o `ph2d-panel-*` deste
    /// repo não fala com device nenhum. É o mesmo caminho de `dyntopo` e
    /// `level`: fatos que o painel MOSTRA e não possui.
    ///
    /// Vazio ⇒ só a opção do rig é pintada.
    pub matcaps: &'static [&'static str],
}

/// Um gesto do artista, para o shell aplicar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Sculpt3dIntent {
    /// Substitui o estado autorado inteiro — ver [`Sculpt3dUi`].
    SetUi(Sculpt3dUi),
    /// Liga/desliga a topologia dinâmica (e triangula, se ligar).
    ToggleDyntopo,
    /// Desce (`false`) ou sobe (`true`) um nível de multiresolução.
    ChangeLevel(bool),
    Subdivide,
    ReverseLevel,
    Remesh,
    CloseHoles,
    /// As quatro primitivas, na ordem em que o painel as lista.
    ///
    /// ⚠️ **Um comando por forma, e não um enum espelho do `Primitive` do
    /// shell.** Um enum duplicado aqui concordaria com o de lá exatamente até
    /// alguém acrescentar a quinta forma num só dos dois; um comando novo não
    /// compila sem que o painel também ganhe o botão dela, que é a ordem certa.
    AddSphere,
    AddCube,
    AddCylinder,
    AddTorus,
    Duplicate,
    Delete,
    ToggleIsolate,
    Merge,
    MaskClear,
    MaskInvert,
    MaskBlur,
    MaskSharpen,
}

/// Estado retido por-instância. Vazio de propósito: a autoridade é a
/// `Sculpt3dScene` do shell, e o painel renderiza o retrato do frame.
#[derive(Clone, Debug, Default)]
pub struct Sculpt3dPanelState;

/// Host → painel, uma vez por frame antes do `paint`.
pub fn set_current_sculpt3d(snapshot: Option<Sculpt3dSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// O que `paint` e `event` leem. `None` quando não há cena 3D — e aí o painel
/// **não pinta**: um painel de escultura sem escultura seria seis seções de
/// controles que não alcançam nada.
pub(crate) fn current() -> Option<Sculpt3dSnapshot> {
    CURRENT.with(|c| *c.borrow())
}

/// Painel → host. Enfileirado pelo `event`, drenado pela ponte do shell.
pub(crate) fn push_intent(intent: Sculpt3dIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Leva tudo o que o artista fez desde o último frame.
pub fn drain_intents() -> Vec<Sculpt3dIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Contabilidade de rolagem (o dock do shell precisa das alturas medidas).
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Ver [`last_content_h`].
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
