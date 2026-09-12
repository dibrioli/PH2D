//! ⭐ **O estado de editor da família Flip, num dono só** (W2/L5, 2026-09-11).
//!
//! # Por que este ficheiro existe
//!
//! Até 11/09 a família Flip vivia em **21 campos soltos de [`crate::App`]** — `flip_active`,
//! `flip_draw`, `flip_strip`, `flip_edit_gesture`, … — no meio dos 311 campos que todas as
//! famílias da shell partilham. Isso tem dois preços medidos: a `App` é o agregado que TODA
//! família recompila, e uma família não podia **sair** da shell porque o estado dela não era um
//! objecto — era uma dispersão.
//!
//! Juntá-los num tipo só é o que torna o corte possível: quando a `ph2d-app-flip` for dona deste
//! tipo, a shell guarda **um** campo e a família viaja inteira (ADR-0075 — o desacoplamento é por
//! estado + eventos, nunca por plugin em runtime).
//!
//! # ⚠️ Isto NÃO é o documento
//!
//! O **documento** do Flip é [`ph2d_flip::FlipDoc`], que vive em [`crate::AppGfx`] no campo `flip`
//! e é **partilhado** (a F8 dos Componentes fechou-o em 10/09). Ele é o que se grava, o que o undo
//! fotografa, o que o `.ph2dproj` carrega.
//!
//! O que mora AQUI é o estado de **interacção**: gestos em voo, arrastos, hover, a tira de
//! quadros, os espelhos por-quadro do estilo da tool. Ele morre com a sessão, e **nada** dele
//! entra em serialização — é por isso que mover estes campos não toca `PROJECT_SCHEMA` nem
//! `FLIP_SCHEMA_VERSION`.

/// Todo o estado de editor da família Flip.
///
/// Os 21 campos abaixo são exactamente os que estavam em [`crate::App`], com os mesmos tipos e a
/// mesma ordem; o prefixo `flip_` saiu porque o dono agora o diz (`app.flip_state.draw`, não
/// `app.flip_draw`).
///
/// `Default` é derivado porque os 21 nasciam de `false` / `None` / `::default()` no construtor da
/// `App` — a derivação **é** o construtor de antes, sem uma linha escrita à mão que pudesse
/// divergir dele.
#[derive(Default)]
pub struct FlipState {
    /// ADR-0114 W2: a tool Flip está ativa? Cacheado do registry pelo `flip_bridge`
    /// (o `input_dispatch` não pode fazer downcast — vive no bridge allowlistado).
    pub active: bool,
    /// ADR-0114 W2: o estilo de brush + modo espelhados do `FlipTool` a cada frame
    /// pelo `flip_bridge`. `None` quando a tool está inativa. O `input_dispatch` lê
    /// isto (sem downcast) pra decidir desenhar + assar o traço.
    pub style: Option<ph2d_tool_flip::FlipStyleSnapshot>,
    /// ADR-0114 W2: o traço do Flip em curso (amostras mundo+pressão); assado no
    /// `FlipDoc` no pen-up. Vazio quando não há gesto.
    pub draw: crate::draw::FlipDraw,
    /// ADR-0114 C2 (Colorize): os rabiscos coloridos acumulados + o rabisco em curso
    /// (transientes — sementes do corte LazyBrush, não arte). Apply os transforma em
    /// regiões preenchidas; Clear os descarta. Ver `flip_colorize`.
    pub colorize: crate::colorize::FlipColorize,
    /// Doc 06 §8: os helpers ao vivo do Gap Closure — os vãos que o alcance atual
    /// fecha, computados num worker (o custo é 5-339 ms, medido) e desenhados pelo
    /// overlay em modo Fill. Display-only: nunca toca o documento.
    pub gap: crate::gap_live::GapHelpers,
    /// ADR-0114 C2: o botão Apply/Clear do Colorize foi clicado neste frame? O drain de
    /// painel roda com `self.gfx` preso; o gesto real (que precisa de `self` livre) roda no
    /// topo do frame seguinte. Falso fora do clique.
    pub pending_colorize_apply: bool,
    pub pending_colorize_clear: bool,
    /// ADR-0114 W2: a camada ATIVA do Flip (alvo do traço/borracha + destaque no
    /// painel). Setada pela seleção de linha no painel (drain do shell); `None`
    /// ⇒ o bake usa a camada de topo (o `flip_bridge` também destaca a de topo).
    pub active_layer: Option<ph2d_flip::LayerId>,
    /// ADR-0114 W2 T2.9: uma borracha do Flip está em curso (Down..Up no modo
    /// Erase). Enquanto `true`, cada move apaga sob o cursor; o pen-up faz o
    /// cleanup do Soft. `false` quando não há gesto.
    pub erasing: bool,
    /// ADR-0114 W3: o estado de autoria da TIRA de frames (autokey/additive/quantos
    /// inbetweens/seleção de chaves). O documento (frames, desenhos, ciclos) vive no
    /// `FlipDoc`; aqui só o que não é documento.
    pub strip: crate::strip::FlipStrip,
    /// ADR-0114 W5: o gesto de ESCULTURA em curso (Down..Up no modo Reshape). Carrega
    /// a máscara congelada no pen-down — e, no Grab, os pesos. `None` quando não há
    /// gesto. Ver `flip_reshape`.
    pub reshape: Option<crate::reshape::FlipReshape>,
    /// ADR-0114 W6.1: o GESTO em curso no modo Edit — a caixa do marquee, ou a translação
    /// da seleção. `None` fora de um arrasto. Ver `flip_edit_gesture`.
    pub edit_gesture: Option<crate::edit_gesture::EditGesture>,
    /// Shift & Trace: o arrasto de DESLOCAR um fantasma em curso (modo Trace). O mapa
    /// de deslocamentos mora em `flip_strip.trace`; aqui é só o gesto.
    pub trace_drag: Option<crate::trace::TraceDrag>,
    /// O PEEK do Shift & Trace (fatia 2): a folha que F1/F2/F3 estão segurando —
    /// `None` fora do aperto. Press arma (só com a tool Flip), release desarma sempre.
    pub peek: Option<crate::peek::PeekDir>,
    /// ADR-0114 W7.5: o arrasto do gizmo de POSE em curso (modo Edit, quadro
    /// instanciado) — rotate/scale escrevendo a pose da chave, nunca o `Transform`.
    /// `None` fora de um arrasto. Ver `flip_pose_gizmo`.
    pub pose_drag: Option<crate::pose_gizmo::FlipPoseDrag>,
    /// ADR-0114 §4.A: o arrasto do gizmo de SELEÇÃO em curso (modo Edit, arte
    /// exclusiva) — rotate/scale assando o delta na geometria dos pontos selecionados.
    /// `None` fora de um arrasto. Ver `flip_selection_gizmo`.
    pub selection_drag: Option<crate::selection_gizmo::FlipSelectionDrag>,
    /// ADR-0114 W8: o DOMÍNIO da seleção do frame ANTERIOR — a memória que deixa a
    /// troca do toggle (Stroke↔Point) converter a seleção no documento UMA vez
    /// (broadcast/promoção, `flip_select::flip_edit_domain_refresh`). `None` = tool
    /// inativa.
    pub edit_domain: Option<ph2d_tool_flip::EditDomain>,
    /// ADR-0114 W6: o estilo do painel no frame ANTERIOR, enquanto há seleção no modo
    /// Edit. É o que deixa **só a MUDANÇA** agir sobre os traços selecionados: sem esta
    /// memória, o passe reaplicaria o estilo a cada frame e selecionar um traço vermelho
    /// com o painel em azul o pintaria de azul no ato do clique. Ver `flip_select`.
    pub edit_style: Option<ph2d_tool_flip::FlipStyleSnapshot>,
    /// ADR-0114 §4.C: o PEDAÇO sob o cursor no modo Segment — `(traço, pontos do pedaço)`,
    /// a promessa do que o clique vai pegar. Recomputado pelo passe
    /// `flip_segment_hover_refresh` **só quando o cursor MOVE** (e nunca durante um gesto),
    /// e lido pelo overlay. `None` = sem hover (fora do Segment, ou o cursor no vazio).
    pub segment_hover: Option<(usize, Vec<usize>)>,
    /// A posição de cursor com que o `flip_segment_hover` foi computado — a guarda que
    /// evita refazer o pick (hit-test + cortes) a cada frame com o mouse parado. Ver
    /// `flip_select_segment::hover_refresh`.
    pub segment_hover_at: Option<(f32, f32)>,
    /// `FlipObjectId` → entidade ECS que o representa na Hierarquia (ADR-0114). O
    /// invariante "um objeto ⟺ uma entidade" é mantido por `flip_entities::sync`.
    pub entities: ph2d_flip_entities::entities::FlipEntityMap,
}
