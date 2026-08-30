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
            // ⭐⭐⭐ **E a OPACIDADE vai junto, dentro da própria cor** (plano 36, W6). Esta linha
            // não mudou; o que mudou foi quem lê a `fallback`.
            //
            // ⚠️ **A redacção anterior estava errada no ponto que decidia o produto:** ela dizia
            // *"aqui NÃO há opacidade a escrever ... o desvanecimento das cópias mora em quem as
            // desenha"*. A 1.ª metade descrevia o modelo (uma casa só) e a 2.ª uma AUSÊNCIA — e o
            // que se lia era que a barra *Opacity* estava tratada. Não estava: ela escrevia a
            // alfa aqui e **ninguém a consumia**, então o artista arrastava-a de ponta a ponta sem
            // mudar um pixel. Hoje `ph2d_vec_scene::brush_copies` desvanece a arte por
            // `fallback.a`. *Uma nota que descreve o modelo certo e uma ausência ao lado lê-se
            // como se a ausência estivesse coberta.*
            //
            // ⇒ um pincel **não** tem `alpha` próprio, ao contrário do padrão: ali o campo existe
            // porque o amostrador quer um `f32`; aqui a opacidade é a alfa desta cor e mais nada.
            StrokePaint::Brush(b) => {
                let mut b = b.clone();
                b.fallback = self.color;
                StrokePaint::Brush(b)
            }
            StrokePaint::Pattern(p) => {
                let mut p = p.clone();
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
                //
                // ⭐ **E a conta mora numa PORTA desde a W6** ([`recolour_pattern`]): o
                // preenchimento faz exactamente isto do outro lado, e duas cópias do `a/255`
                // divergiriam no dia em que uma ganhasse um arredondamento.
                recolour_pattern(&mut p, self.color);
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

/// ⭐⭐⭐ **A COR — e a OPACIDADE que a alfa dela carrega — escritas num PADRÃO** (plano 36, W6).
///
/// A porta única do `alpha` de um [`ph2d_vec_scene::PatternFill`]: o traço chega aqui pelo
/// [`StrokeStyle::onto`], o preenchimento pelo [`apply_fill_colour`]. Uma segunda cópia da conta
/// `a/255` divergiria no dia em que uma delas ganhasse um arredondamento.
///
/// Devolve **se mudou alguma coisa**, o que a torna detector e escritor de uma vez — quem só quer
/// perguntar chama-a num clone. É a disciplina do [`StrokeStyle`] (um campo num lado e não no outro
/// é um controlo que mexe o número e não muda a tela) reduzida a uma função só: não há dois lados
/// que possam divergir.
///
/// ⚠️ **Uma opacidade, uma casa:** o `alpha` é o que se DESENHA e a `fallback.a` é o instante
/// pré-resolução; as duas descem juntas para que a forma não salte quando o ladrilho carrega.
fn recolour_pattern(pat: &mut ph2d_vec_scene::PatternFill, c: Rgba8) -> bool {
    let alpha = f32::from(c.a) / 255.0;
    let mudou = pat.fallback != c || (pat.alpha - alpha).abs() > f32::EPSILON;
    pat.fallback = c;
    pat.alpha = alpha;
    mudou
}

/// ⭐⭐⭐ **O QUE UM PICK DE PREENCHIMENTO FAZ A ESTA TINTA** — a porta única do `differs` e do
/// escritor (plano 36, W6).
///
/// Devolve se mudou alguma coisa; chamada num clone, responde *"isto difere?"*. **Um lugar só**,
/// porque a versão anterior tinha a decisão escrita duas vezes — a condição do `differs` e o `else
/// if` do escritor — e as duas tinham de ser mantidas iguais à mão.
///
/// # As três leis, e de onde vem cada uma
///
/// - ⛔ **Um gradiente é INTOCÁVEL** (Enio, 2026-07-08): a alça dele endereça uma parada, e um pick
///   sem alça esmagaria a rampa inteira numa cor. A porta de sair de um gradiente é a fileira
///   *Fill Type*, e só ela.
/// - ⭐ **Um PADRÃO recebe a cor de recurso E a opacidade, e SOBREVIVE** (W6). ⚠️ A guarda antiga —
///   *"só substitui um `Solid`/vazio"* — mantinha-o a salvo de ser destruído e, na mesma linha,
///   deixava-o **sem nenhum caminho de escrita**: a barra *Fill Opacity* andava de ponta a ponta
///   sem mudar um pixel. *Proteger uma tinta de ser destruída não é a mesma coisa que ela ser
///   editável.*
/// - **Um `Solid`/vazio** é substituído, e **alfa zero apaga o preenchimento** — a convenção que a
///   ponte já usa nos dois sentidos. ⚠️ Ela **não** se estende ao padrão: ali destruiria a grade, o
///   ladrilho, a rotação e a fonte por um arrasto acidental até ao fundo da barra, e a leitura útil
///   é *invisível, mas lá*. *Uma convenção herdada aplica-se onde não custa nada; onde custa, ela é
///   a pergunta.*
pub(crate) fn apply_fill_colour(fill: &mut Option<Paint>, closed: bool, c: Rgba8) -> bool {
    if !closed {
        return false;
    }
    match fill {
        // ⭐ O PADRÃO: cor + opacidade, e a tinta sobrevive.
        Some(Paint::Pattern(p)) => recolour_pattern(p, c),
        // ⛔ Gradientes (linear, radial, multi-ponto): quietos.
        Some(_) if !matches!(fill, Some(Paint::Solid(_))) => false,
        // Sólido ou vazio: substituído, com alfa zero a significar "sem preenchimento".
        _ => {
            let novo = (c.a != 0).then_some(c);
            if fill.as_ref().map(Paint::primary_color) == novo {
                return false;
            }
            *fill = novo.map(Paint::solid);
            true
        }
    }
}

/// ⭐⭐ **O que a ferramenta ADOTA do preenchimento de uma forma recém-selecionada** — `None` = não
/// adota nada (plano 36, W6).
///
/// Espelho exacto do [`apply_fill_colour`]: o que aquele ESCREVE, este LÊ. ⚠️ **As duas metades são
/// independentes e uma sem a outra é pior que o buraco:** escrever sem semear faz a primeira mexida
/// em qualquer controlo jogar na forma a alfa da forma **anterior**; semear sem escrever deixa a
/// barra a mostrar o valor certo e a não o mudar.
///
/// ⚠️ **Num padrão a semente lê o `alpha`, e não a `fallback.a`.** Esta ponte mantém os dois em
/// sincronia, mas um documento gravado antes da W6 tem a `fallback` opaca e o `alpha` autorado
/// noutro sítio — e o que se DESENHA é o `alpha`. *Semear pelo campo que não se vê poria a barra a
/// mentir sobre o ficheiro do artista.*
///
/// ⛔ **Um gradiente fica QUIETO** (`None`): ele tem alça própria, e esmagá-lo numa cor só seria a
/// selecção a destruir o que o artista autorou.
pub(crate) fn seed_fill_from_paint(fill: Option<&Paint>) -> Option<[u8; 4]> {
    match fill {
        Some(Paint::Solid(c)) => Some([c.r, c.g, c.b, c.a]),
        Some(Paint::Pattern(p)) => {
            let a = (p.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
            Some([p.fallback.r, p.fallback.g, p.fallback.b, a])
        }
        None => Some([0, 0, 0, 0]),
        Some(_) => None,
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
    // O preenchimento vira a swatch de Fill — SÓLIDO pela cor, PADRÃO pela cor de recurso mais a
    // opacidade que ele DESENHA (W6); alfa 0 é o "sem preenchimento" que a ponte já usa no caminho
    // inverso. Um gradiente fica QUIETO — ele tem alça própria, e esmagá-lo numa cor só seria a
    // seleção destruindo o que o artista autorou. A lei inteira vive no [`seed_fill_from_paint`],
    // que é o espelho do [`apply_fill_colour`].
    if let Some(c) = seed_fill_from_paint(path.fill.as_ref()) {
        tool.adopt_fill(c);
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
