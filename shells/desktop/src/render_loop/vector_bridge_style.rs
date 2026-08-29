//! **O estilo do traço**, do painel para o documento — módulo irmão do `vector_bridge`
//! (teto de 600 LOC por arquivo da shell).
//!
//! Aqui mora a metade "escrita" da ponte: o registro [`StrokeStyle`] que o painel edita, o
//! DETECTOR de mudança (`differs_from`) e o ESCRITOR (`onto`) — de propósito juntos.
//!
//! # Por que detector e escritor são o mesmo registro
//!
//! Um campo presente num e ausente no outro é um controle que mexe o número e **não muda nada
//! na tela** — e o compilador não diz uma palavra. O slider anda, o valor entra no snapshot, e
//! a linha continua igual; o usuário conclui que o parâmetro não faz nada. Foi o risco vivo do
//! Head Size e do Head Round, e é por isso que os dois lados leem a MESMA struct.

use ph2d_vec_edit::PenTool;
use ph2d_vec_render::GradHandle;
use ph2d_vec_scene::{LineCap, LineJoin, Paint, Rgba8, StrokeAlign, StrokeSpec, VecScene};
use std::cell::{Cell, RefCell};

pub(crate) fn rgba(c: [u8; 4]) -> Rgba8 {
    Rgba8::new(c[0], c[1], c[2], c[3])
}

/// **A ficha de traço que o painel manda** — tudo o que a tool possui do stroke, MENOS a
/// largura (que só muda enquanto o slider é arrastado, e por isso tem regra própria).
///
/// Uma struct, e não oito valores soltos, porque **dois** lugares precisam falar da mesma
/// coisa: quem DETECTA a mudança ([`Self::differs_from`]) e quem a GRAVA ([`Self::onto`]).
/// Um campo presente num e ausente no outro é um controle que mexe no número e não muda nada
/// na tela — e o compilador não diz nada. Foi exatamente o risco do Head Size / Head Round.
#[derive(Copy, Clone, Debug)]
pub(crate) struct StrokeStyle {
    pub color: Rgba8,
    pub cap: LineCap,
    pub join: LineJoin,
    /// De que lado da linha a faixa cai (Centre/Inner/Outer).
    pub align: StrokeAlign,
    pub dash: Option<(f64, f64)>,
    pub marker_start: ph2d_vec_scene::Marker,
    pub marker_end: ph2d_vec_scene::Marker,
    /// Tamanho da cabeça da seta (múltiplo) + arredondamento das quinas dela.
    pub marker_scale: f64,
    pub marker_round: f64,
}

impl StrokeStyle {
    /// O traço `s` já é esta ficha, ou reescrevê-lo mudaria alguma coisa? (A largura fica de
    /// fora: ela só acompanha a tool enquanto o slider é arrastado.)
    pub(crate) fn differs_from(&self, s: &StrokeSpec) -> bool {
        s.color() != self.color
            || s.cap != self.cap
            || s.join != self.join
            || s.align != self.align
            || s.dash != self.dash
            || s.marker_start != self.marker_start
            || s.marker_end != self.marker_end
            || (s.marker_scale - self.marker_scale).abs() > f64::EPSILON
            || (s.marker_round - self.marker_round).abs() > f64::EPSILON
    }

    /// O `StrokeSpec` desta ficha com a largura `width`, **escrito sobre `old`**.
    ///
    /// A ficha + a largura determinam todo o resto do traço — nada mais do spec antigo sobrevive, e
    /// é isso que garante que os dois lados (detectar / gravar) não possam divergir num campo
    /// esquecido.
    ///
    /// ⛔⛔ **A TINTA é a excepção, e ela custou um report** (Enio, 2026-08-28: *"se ajustar width
    /// sai de pattern e vai para solid"*).
    ///
    /// ⚠️⚠️ **O defeito estava escrito aqui como INVARIANTE**: *"a ficha + a largura determinam o
    /// traço INTEIRO — nada do spec antigo sobrevive"*. Era **verdade** enquanto o traço tinha uma
    /// tinta possível; a wave A do plano 35 deu-lhe duas, e a mesma frase passou a significar *"toda
    /// edição de geometria do traço destrói a tinta autorada"*. *Uma invariante é uma afirmação
    /// sobre o modelo do dia em que foi escrita — quem alarga o modelo tem de a reconferir.*
    ///
    /// ⭐ **A lei certa já existia nesta casa, no PREENCHIMENTO** — e também por um report do Enio
    /// (2026-07-08), escrita no `vector_bridge`: *"a fill pick only replaces a Solid / None fill; it
    /// must NEVER clobber a gradient (use Fill Type -> Solid for that)"*. O traço nunca a aprendeu
    /// porque, até dois dias atrás, ele não tinha o que houvesse a preservar.
    ///
    /// ⇒ **a ficha possui uma COR, nunca a TINTA**: ela escreve a cor *dentro* da tinta que já lá
    /// está, e a espécie da tinta só muda pela porta explícita (a fileira *Type*, plano 35 wave D).
    ///
    /// ⚠️ E *"nunca esmagar"* não pode virar *"não fazer nada"*: a swatch **mostra** a cor de
    /// recurso de um padrão (`StrokeSpec::color()` responde-a desde a wave A), então é ela que a
    /// cor escreve. Uma swatch que mostra um valor e não o muda é o controlo morto que esta linha
    /// caça há três waves — e é também o que faria o [`Self::differs_from`] disparar **todo
    /// quadro**, com cada um a virar um passo de undo.
    pub(crate) fn onto(&self, old: &StrokeSpec, width: f64) -> StrokeSpec {
        use ph2d_vec_scene::StrokePaint;
        let paint = match &old.paint {
            StrokePaint::Solid(_) => StrokePaint::Solid(self.color),
            // ⭐ **A MESMA lei do padrão** (plano 35 §7.2): a ficha possui uma COR, nunca a TINTA.
            // Um pincel guarda a cor de recurso, e é ela que a swatch mostra.
            //
            // ⚠️ **E aqui NÃO há opacidade a escrever**, ao contrário do padrão: as cópias do
            // pincel são `VecPath` com a tinta DELAS, e o desvanecimento delas mora em quem as
            // desenha. *A mesma pergunta tem respostas diferentes nos dois modelos, e é por isso
            // que o enum é fechado — ele trouxe-me a este braço.*
            StrokePaint::Brush(b) => {
                let mut b = b.clone();
                b.fallback = self.color;
                StrokePaint::Brush(b)
            }
            StrokePaint::Pattern(p) => {
                let mut p = p.clone();
                p.fallback = self.color;
                // ⚠️⚠️ **A OPACIDADE também**, e ela é a metade que se vê: com um ladrilho o desenho
                // lê o `alpha` do padrão, e a alfa da `fallback` só aparece enquanto a arte não
                // resolve (`ph2d-vec-render/src/stroke_draw.rs`). Escrever só a cor deixaria a
                // barra *Opacity* a andar sem mudar um pixel — um controlo morto.
                //
                // ⭐ A lei já estava escrita palavra por palavra no `paint_bind::fade`: *"um padrão
                // não tem cor para escalar — tem OPACIDADE, e as duas descem juntas"*. Ali para a
                // sobreposição derivada; aqui para a AUTORIA.
                //
                // ⚠️ **Uma opacidade, uma casa:** num traço sólido ela vive na alfa da cor, num de
                // padrão no `alpha` dele, com a `fallback` em sincronia. Duas casas divergiriam no
                // dia em que uma delas ganhasse um knob próprio.
                p.alpha = f32::from(self.color.a) / 255.0;
                StrokePaint::Pattern(p)
            }
        };
        StrokeSpec {
            paint,
            width,
            cap: self.cap,
            join: self.join,
            align: self.align,
            dash: self.dash,
            marker_start: self.marker_start,
            marker_end: self.marker_end,
            marker_scale: self.marker_scale,
            marker_round: self.marker_round,
        }
    }
}

/// **Reestiliza o traço de TODOS os caminhos selecionados.** É o ponto da feature: mudar o
/// Head Size (ou a cor, ou a ponta) com o diagrama inteiro selecionado calibra o diagrama
/// inteiro — não só o primeiro da seleção. Devolve quantos traços foram reescritos.
///
/// `new_width = Some(w)` **só** enquanto o slider de largura é arrastado; senão cada caminho
/// mantém a largura dele (uma escolha de cor nunca pode reengrossar a linha).
pub(crate) fn restyle_selected_strokes(
    scene: &mut VecScene,
    selection: &[ph2d_vec_scene::VecPathId],
    style: &StrokeStyle,
    new_width: Option<f64>,
) -> usize {
    let mut n = 0;
    for &id in selection {
        let Some(path) = scene.path_mut(id) else {
            continue;
        };
        // Um caminho SEM traço (só preenchimento) não tem o que reestilizar — e ganhar um
        // traço do nada seria a UI inventando geometria.
        let Some(old) = path.stroke.as_ref() else {
            continue;
        };
        path.stroke = Some(style.onto(old, new_width.unwrap_or(old.width)));
        n += 1;
    }
    n
}

/// The colour the selected gradient handle addresses on `fill` — a multi-point
/// point's colour, or the ramp stop at the START/END end the linear/radial handle
/// sits on. Drives the Fill swatch, the picker seed, and the recolour. `None` if the
/// handle doesn't match the fill kind (e.g. a stale selection after a kind switch).
pub(crate) fn selected_grad_color(fill: &Paint, handle: GradHandle) -> Option<Rgba8> {
    match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => points.get(i).map(|gp| gp.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => stops.first().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => stops.last().map(|s| s.color),
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => stops.get(i).map(|s| s.color),
        _ => None,
    }
}

/// Recolour the slot the selected gradient handle addresses to `c`; returns whether
/// it changed. Mirror of [`selected_grad_color`] on the mutable fill.
pub(crate) fn set_selected_grad_color(fill: &mut Paint, handle: GradHandle, c: Rgba8) -> bool {
    let slot = match (fill, handle) {
        (Paint::MultiPoint { points }, GradHandle::Point(i)) => {
            points.get_mut(i).map(|gp| &mut gp.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearStart)
        | (Paint::Radial { stops, .. }, GradHandle::RadialCenter) => {
            stops.first_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::LinearEnd)
        | (Paint::Radial { stops, .. }, GradHandle::RadialEdge) => {
            stops.last_mut().map(|s| &mut s.color)
        }
        (Paint::Linear { stops, .. }, GradHandle::Stop(i))
        | (Paint::Radial { stops, .. }, GradHandle::Stop(i)) => {
            stops.get_mut(i).map(|s| &mut s.color)
        }
        _ => None,
    };
    match slot {
        Some(slot) if *slot != c => {
            *slot = c;
            true
        }
        _ => false,
    }
}

/// Push `alpha` (0..255) onto an Opacity slider's stored value — unless the user
/// is dragging it — so a colour-picker alpha change reflects on the panel and the
/// drag baseline stays correct. The linked chip's display is driven from the
/// slider track in `paint`, so it follows without a separate push.
pub(crate) fn sync_opacity_slider(
    store: &mut ph2d_editor::interaction::WidgetStore,
    id: ph2d_editor::NodeId,
    alpha: u8,
) {
    use ph2d_editor::InteractiveState;
    use ph2d_editor::widget::SliderState;
    if let Some(InteractiveState::Slider { state, value, .. }) = store.get_mut(id)
        && !matches!(*state, SliderState::Dragging)
    {
        *value = f32::from(alpha) / 255.0;
    }
}

thread_local! {
    /// Pre-image of the scene captured at the START of a recolour gesture (the
    /// first frame the colour actually changes the selected path). Committed to
    /// `History` as ONE undo step when the gesture ends (the picker closes /
    /// the discrete pick's frame finishes). `None` between gestures.
    pub(crate) static RECOLOR_PRE: RefCell<Option<VecScene>> = const { RefCell::new(None) };
    /// O caminho cujas PONTAS a tool adotou por último — o "alvo" dos dois seletores de
    /// marker, no mesmo modelo do alvo dos campos de forma (`vec_shape_params`). Só a
    /// MUDANÇA de alvo semeia; semear todo frame brigaria com a escolha que o usuário
    /// acabou de fazer (o `SetValue` do popover chega DEPOIS deste passe, no frame
    /// seguinte, e seria imediatamente desfeito).
    static STYLE_TARGET: Cell<Option<ph2d_vec_scene::VecPathId>> = const { Cell::new(None) };
}

/// **Semeia o ESTILO a partir do caminho SELECIONADO** (quando a seleção muda) — cor, largura,
/// cap, join, alinhamento, tracejado e as pontas.
///
/// # A lei, numa frase: a ponte LÊ na seleção o que ESCREVE no apply
///
/// O `restyle_selected_strokes` escreve um `StrokeSpec` inteiro nos caminhos selecionados; esta
/// função lê um `StrokeSpec` inteiro do caminho selecionado. A simetria é literal e é o que
/// impede o painel de mentir.
///
/// ⚠️ **Ela era `seed_markers_from_selection` e adotava SÓ as pontas** — cor, largura, cap, join,
/// dash e o alinhamento novo ficavam com o último valor autorado. Não era só display: o
/// `take_apply_to_selected` REESCREVE a seleção com o que a tool tem, então tocar um único
/// controle empurrava o estilo velho inteiro para cima da forma recém-selecionada. Report do
/// Enio, 2026-08-01: *"as propriedades ainda não são atualizadas para o objeto selecionado"*.
///
/// ⚠️ **O STORE é re-semeado junto, e não é redundância:** as rows de slider (Width, as duas
/// Opacity, Dash, Gap) pintam do **store**, não do snapshot — `store.slider(...)` com o snapshot
/// só como *fallback*, e o fallback nunca dispara porque o `populate` registrou o widget. Adotar
/// na tool sem semear o store deixaria metade do painel a mostrar o valor velho.
///
/// Um caminho SEM traço (só preenchimento) não tem estilo a doar: o alvo passa a ser ele, mas o
/// Style da tool fica onde estava (o default do próximo traço).
pub(crate) fn seed_style_from_selection(
    tool: &mut ph2d_tool_vector::VectorTool,
    store: &mut ph2d_editor::interaction::WidgetStore,
    pen: &PenTool,
    scene: &VecScene,
    world_to_px: f64,
) {
    let target = pen.selected();
    if STYLE_TARGET.with(Cell::get) == target {
        return;
    }
    STYLE_TARGET.with(|c| c.set(target));
    let Some(path) = target.and_then(|id| scene.paths().iter().find(|p| p.id == id)) else {
        return;
    };
    // O preenchimento SÓLIDO vira a swatch de Fill; alfa 0 é o "sem preenchimento" que a
    // ponte já usa no caminho inverso. Um gradiente fica QUIETO — ele tem alça própria, e
    // esmagá-lo numa cor só seria a seleção destruindo o que o artista autorou.
    match &path.fill {
        Some(ph2d_vec_scene::Paint::Solid(c)) => tool.adopt_fill([c.r, c.g, c.b, c.a]),
        None => tool.adopt_fill([0, 0, 0, 0]),
        Some(_) => {}
    }
    let Some(stroke) = path.stroke.as_ref() else {
        return;
    };
    tool.adopt_stroke(stroke, stroke.width * world_to_px);
    reseed_style_sliders(store, tool);
}

/// As rows que pintam do STORE — sem isto o painel mostra o valor velho mesmo com a tool já
/// adotada. Espelho exato do `seed_shape_fields`, e pela mesma razão.
fn reseed_style_sliders(
    store: &mut ph2d_editor::interaction::WidgetStore,
    tool: &ph2d_tool_vector::VectorTool,
) {
    use ph2d_tool_vector::params;
    let set = |store: &mut ph2d_editor::interaction::WidgetStore, id, track: f32| {
        // ⚠️ Nunca sobre um slider em ARRASTO: a semente brigaria com o dedo do artista, a
        // mesma armadilha que o `seed_shape_fields` documenta.
        if let Some(ph2d_editor::InteractiveState::Slider { state, value, .. }) = store.get_mut(id)
            && !matches!(*state, ph2d_editor::widget::SliderState::Dragging)
        {
            *value = track.clamp(0.0, 1.0);
        }
    };
    set(
        store,
        ph2d_editor::ids::VECTOR_WIDTH,
        params::px_to_slider(tool.stroke_width_px()),
    );
    set(
        store,
        ph2d_editor::ids::VECTOR_DASH,
        params::dash_to_slider(tool.dash()),
    );
    set(
        store,
        ph2d_editor::ids::VECTOR_GAP,
        params::gap_to_slider(tool.gap()),
    );
    sync_opacity_slider(
        store,
        ph2d_editor::ids::VECTOR_STROKE_OPACITY,
        tool.stroke_rgba()[3],
    );
    sync_opacity_slider(
        store,
        ph2d_editor::ids::VECTOR_FILL_OPACITY,
        tool.fill_rgba()[3],
    );
}
