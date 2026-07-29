//! **O que um ID DE PAINEL endereça na pilha de FX** — o dicionário entre a UI e o modelo.
//!
//! Irmão de [`super::fx_live`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo é
//! *o que uma pilha É e o que a shell FAZ com ela* (o componente, a resolução em pixels, o cozimento
//! e a edição); este é *o que um id QUER DIZER*. Os ids da seção são hashes de nome derivados por
//! LINHA, então não há aritmética que os inverta — decodifica-se varrendo o teto, e é por isso que
//! a tradução merece uma casa própria em vez de ficar espalhada pelos três sítios que perguntam.

/// **Que controle da pilha um id de painel endereça.** Os ids da seção são derivados por LINHA
/// (hashes de nome), então não há aritmética que os inverta: decodifica-se varrendo o teto.
///
/// Porta única de propósito — a ponte tem TRÊS sítios que perguntam *"este id é da pilha?"* (o
/// comando, o valor e o alvo do picker), e três varreduras escritas à mão divergiriam na primeira
/// linha nova.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FilterHit {
    /// "Add \<tipo\>" — põe um degrau novo no fim da pilha.
    Add(u8),
    /// ✕ — apaga a linha (a última apaga o componente).
    Remove(usize),
    /// ↑ / ↓ — a ORDEM é a feature.
    Up(usize),
    Down(usize),
    /// 👁 — desarma sem apagar.
    Hide(usize),
    /// A swatch de cor da linha (abre o picker OKLCH partilhado).
    Color(usize),
    /// A SEGUNDA swatch — a ponta CLARA da rampa do Duotone. Mesmo picker, outro campo.
    ColorB(usize),
    /// **A swatch do stop SELECIONADO** da rampa de N stops. Mesmo picker; o campo é escolhido pela
    /// SELEÇÃO do painel, não pelo id (há uma swatch por linha, não uma por stop).
    StopColor(usize),
    /// `+` / `−` do trilho da rampa — acrescenta um stop no maior vão, remove o selecionado.
    StopAdd(usize),
    StopRemove(usize),
    /// O chip de MODO da linha (a LEI do degrau).
    Mode(usize, u8),
    /// Uma opção do popover de MISTURA (a lei de como a cor do degrau encosta na de baixo).
    Blend(usize, u8),
    /// Os sliders.
    Radius(usize),
    OffX(usize),
    OffY(usize),
    Opacity(usize),
    /// Os três knobs do RUÍDO (só a turbulência os oferece).
    Scale(usize),
    Detail(usize),
    Seed(usize),
    /// O Amount do Grow / Shrink (bipolar).
    Grow(usize),
    /// Os três knobs do AJUSTE DE COR (bipolares). ⚠️ O `Hue` chega em GRAUS — a conversão para
    /// voltas mora no `apply`, ao lado da que o publica.
    Hue(usize),
    Sat(usize),
    Bright(usize),
}

/// Decodifica um id de painel para o controle da pilha que ele endereça.
pub(crate) fn hit_of(id: ph2d_editor::NodeId) -> Option<FilterHit> {
    use ph2d_editor::ids as vid;
    for k in 0..vid::MAX_FILTER_KINDS {
        if id == vid::filter_add_id(k) {
            #[allow(clippy::cast_possible_truncation)]
            return Some(FilterHit::Add(k as u8));
        }
    }
    for r in 0..vid::MAX_FILTER_ROWS {
        for m in 0..vid::MAX_FILTER_MODES {
            if id == vid::filter_mode_id(r, m) {
                #[allow(clippy::cast_possible_truncation)]
                return Some(FilterHit::Mode(r, m as u8));
            }
        }
        for m in 0..vid::MAX_FILTER_BLENDS {
            if id == vid::filter_blend_option_id(r, m) {
                #[allow(clippy::cast_possible_truncation)]
                return Some(FilterHit::Blend(r, m as u8));
            }
        }
        let hit = if id == vid::filter_remove_id(r) {
            FilterHit::Remove(r)
        } else if id == vid::filter_up_id(r) {
            FilterHit::Up(r)
        } else if id == vid::filter_down_id(r) {
            FilterHit::Down(r)
        } else if id == vid::filter_hide_id(r) {
            FilterHit::Hide(r)
        } else if id == vid::filter_stop_add_id(r) {
            FilterHit::StopAdd(r)
        } else if id == vid::filter_stop_remove_id(r) {
            FilterHit::StopRemove(r)
        } else if id == vid::filter_stop_color_id(r) {
            FilterHit::StopColor(r)
        } else if id == vid::filter_color_id(r) {
            FilterHit::Color(r)
        } else if id == vid::filter_color_b_id(r) {
            FilterHit::ColorB(r)
        } else if id == vid::filter_radius_id(r) {
            FilterHit::Radius(r)
        } else if id == vid::filter_offx_id(r) {
            FilterHit::OffX(r)
        } else if id == vid::filter_offy_id(r) {
            FilterHit::OffY(r)
        } else if id == vid::filter_opacity_id(r) {
            FilterHit::Opacity(r)
        } else if id == vid::filter_scale_id(r) {
            FilterHit::Scale(r)
        } else if id == vid::filter_detail_id(r) {
            FilterHit::Detail(r)
        } else if id == vid::filter_seed_id(r) {
            FilterHit::Seed(r)
        } else if id == vid::filter_grow_id(r) {
            FilterHit::Grow(r)
        } else if id == vid::filter_hue_id(r) {
            FilterHit::Hue(r)
        } else if id == vid::filter_sat_id(r) {
            FilterHit::Sat(r)
        } else if id == vid::filter_bright_id(r) {
            FilterHit::Bright(r)
        } else {
            continue;
        };
        return Some(hit);
    }
    None
}

/// **A cor de um degrau em BYTES**, como a swatch do painel a mostra.
///
/// Porta única, e ela nasceu com o Duotone: a partir de duas pontas a conversão passou a ter dois
/// chamadores, e duas cópias de um arredondamento divergem exactamente onde ninguém lê um número —
/// numa swatch.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn colour_bytes(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

/// **Qual PONTA de cor este id nomeia** — `(linha, é_a_segunda)`.
///
/// Porta única do readback do picker, e ela existe porque o picker é o ÚNICO consumidor que precisa
/// distinguir as duas swatches: para todo o resto (o dispatch, a varredura de seam) as duas são o
/// mesmo tipo de controle. Escrita aqui, a shell não repete a enumeração dos dois variants.
pub(crate) fn colour_target(id: ph2d_editor::NodeId) -> Option<(usize, ColourSlot)> {
    match hit_of(id)? {
        FilterHit::Color(r) => Some((r, ColourSlot::First)),
        FilterHit::ColorB(r) => Some((r, ColourSlot::Second)),
        FilterHit::StopColor(r) => Some((r, ColourSlot::SelectedStop)),
        _ => None,
    }
}

/// **QUAL cor de um degrau uma swatch nomeia.**
///
/// ⚠️ **Era um `bool` (*"é a segunda?"*), e o comentário do readback já previa esta wave:** *"derivar
/// a ponta do `kind` faria a segunda escrever na primeira em qualquer tipo que ganhasse uma rampa
/// depois"*. Com três alvos o booleano deixa de ser expressivo — e um terceiro caso dobrado num
/// `else` escreveria na ponta escura toda vez que o artista pintasse um stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColourSlot {
    /// A cor do halo / a ponta ESCURA.
    First,
    /// A ponta CLARA da rampa do Duotone.
    Second,
    /// O stop que o trilho da rampa tem em foco.
    SelectedStop,
}

/// **A cor escolhida no picker pousa no slot que o artista abriu.**
///
/// ⚠️ **Ela é uma função PURA de propósito, e a razão é um gate que falhou.** A decisão morava dentro
/// do `render_frame` — a função que exige janela e dispositivo, que nenhum teste de unidade alcança
/// — então o que a cobria era um arch-gate sobre o FONTE. E um arch-gate só vê FORMA: a mutação que
/// dobrava o stop na ponta escura **manteve o nome `SelectedStop` num braço inalcançável** e passou
/// verde. Aqui a rota é observável, e a mutação sangra num `assert_eq!`.
///
/// O `selected` é clampado à contagem VIVA: a rampa pode ter encolhido desde o último clique.
pub(crate) fn apply_picked_colour(
    op: &mut ph2d_ecs::FxOp,
    slot: ColourSlot,
    selected: usize,
    col: [f32; 4],
) {
    match slot {
        ColourSlot::First => op.color = col,
        ColourSlot::Second => op.color_b = col,
        ColourSlot::SelectedStop => {
            let n = usize::from(op.stop_count).min(ph2d_ecs::FxOp::MAX_GRADIENT_STOPS);
            if let Some(stop) = op.stops.get_mut(selected.min(n.saturating_sub(1))) {
                *stop = col;
            }
        }
    }
}

/// **Acrescenta um stop — e a lei é que isso NÃO muda o desenho.**
///
/// ⚠️ **No maior VÃO, com a cor que a rampa já tem ali.** As duas metades são a mesma decisão: um
/// stop novo que caísse em cima de outro seria inalcançável pelo ponteiro (a caixa de agarre é o
/// recurso de que o teto é), e um stop novo de cor arbitrária faria o `+` **editar a arte** — o
/// artista clica para ganhar um ponto de controle, não para mudar a cor. É o que Photoshop e
/// Illustrator fazem, e é o análogo exacto do *"um degrau novo não muda o desenho antes de o artista
/// tocar nele"* que o resto desta pilha honra.
///
/// ⚠️ **Apenda na ordem de AUTORIA** (índice `stop_count`), nunca na posição ordenada: o índice de
/// cada punho tem de ficar estável, senão o `+` re-liga os gestos abertos a outros stops.
///
/// No-op no teto.
pub(crate) fn add_stop(op: &mut ph2d_ecs::FxOp) {
    let n = usize::from(op.stop_count).min(ph2d_ecs::FxOp::MAX_GRADIENT_STOPS);
    if n >= ph2d_ecs::FxOp::MAX_GRADIENT_STOPS {
        return;
    }
    // O maior vão, medido sobre a rampa ORDENADA (a que se vê), com as bordas 0 e 1 contando: uma
    // rampa que começa em 0,3 tem o vão de entrada como candidato legítimo.
    let (_, pos, count) = op.ramp_for_device();
    let at = |i: usize| pos[i / 4][i % 4];
    let mut best = (0.5_f32, 0.0_f32);
    let mut prev = 0.0_f32;
    for i in 0..count as usize {
        let p = at(i);
        if p - prev > best.1 {
            best = ((prev + p) * 0.5, p - prev);
        }
        prev = p;
    }
    if 1.0 - prev > best.1 {
        best = ((prev + 1.0) * 0.5, 1.0 - prev);
    }
    let offset = best.0.clamp(0.0, 1.0);
    // A cor que a rampa JÁ tem ali — amostrada pela mesma lei que o device honra.
    let preview = ramp_preview(op);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let slot = (offset * (ph2d_panel_vector::RAMP_PREVIEW_N - 1) as f32).round() as usize;
    let rgb = preview[slot.min(ph2d_panel_vector::RAMP_PREVIEW_N - 1)];
    op.stops[n] = [
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
        // Força cheia: um stop novo que nascesse transparente seria um ponto de controle que não
        // controla nada, e o artista concluiria que o `+` está quebrado.
        1.0,
    ];
    op.stop_pos[n] = offset;
    op.stop_count = (n + 1) as u8;
}

/// **Remove o stop `sel`** — com PISO em dois.
///
/// ⚠️ **O piso não é cautela: é a definição.** Uma rampa com um stop é uma cor sólida (e o Color
/// Overlay já é isso), e com zero stops cai numa lei DIFERENTE — o ramo vazio do `gradient_sample`,
/// que difere do default de dois stops em 73 níveis de byte (gate
/// `no_stops_is_the_painters_empty_ramp_which_is_not_the_two_stop_default`). Deixar o `−` chegar lá
/// faria o artista atravessar uma descontinuidade que nada na tela explica.
pub(crate) fn remove_stop(op: &mut ph2d_ecs::FxOp, sel: usize) {
    let n = usize::from(op.stop_count).min(ph2d_ecs::FxOp::MAX_GRADIENT_STOPS);
    if n <= 2 || sel >= n {
        return;
    }
    for i in sel..n - 1 {
        op.stops[i] = op.stops[i + 1];
        op.stop_pos[i] = op.stop_pos[i + 1];
    }
    op.stop_count = (n - 1) as u8;
}

/// **A rampa AMOSTRADA**, para o trilho do painel pintar.
///
/// ⚠️ **Pela `gradient_map_lut` do `ph2d-painter-effects` — a MESMA função que é o oráculo dos gates
/// de paridade da GPU** (medido: device vs esta lei, 1 nível de byte). O bar é o que o artista lê
/// para prever o render; amostrá-lo por um lerp de conveniência seria uma SEGUNDA resposta a *"que
/// cor vive em `t`"*, divergindo em gama nos meios-tons — e o único lugar onde a divergência
/// apareceria é uma screenshot.
///
/// ⚠️ **Os stops são ordenados pela porta única do componente** (`FxOp::ramp_for_device`), porque a
/// ordem de autoria é livre e o `gradient_sample` do Painter assume ASCENDENTE.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn ramp_preview(op: &ph2d_ecs::FxOp) -> [[u8; 3]; ph2d_panel_vector::RAMP_PREVIEW_N] {
    use ph2d_painter_effects::adjustments::{ColorStop, GradientInterp, GradientMapParams};
    let (stops, pos, n) = op.ramp_for_device();
    let params = GradientMapParams {
        stops: (0..n as usize)
            .map(|i| ColorStop {
                offset: pos[i / 4][i % 4],
                color: colour_bytes(stops[i]),
            })
            .collect(),
        interpolation: if op.mode == 1 {
            GradientInterp::Smooth
        } else {
            GradientInterp::Linear
        },
    };
    let lut = ph2d_painter_effects::adjustments::gradient_map_lut(&params);
    core::array::from_fn(|i| {
        let t = i as f32 / (ph2d_panel_vector::RAMP_PREVIEW_N - 1).max(1) as f32;
        let c = lut[(t * 255.0).round() as usize];
        core::array::from_fn(|ch| ph2d_color::srgb::linear_to_srgb_byte(c[ch]))
    })
}
