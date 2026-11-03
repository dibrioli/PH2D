//! As medidas do CARTÃO e do campo Figma, em px LÓGICOS — as da maquete aprovada
//! (`docs/CartoesUI/maquete/bones.html`, `02_especificacao.md` §1). As cores são tokens
//! (`card-field`, `card-field-hover`, `card-label`, `kanban-*`); aqui só a geometria.

// ── A lista (o painel) ──
pub const LIST_RADIUS: f32 = 12.0;
/// D31: 5 px da régua e da borda da janela (ordem do dono, 02/11).
pub const LIST_MARGIN: f32 = 5.0;
pub const LIST_PAD_X: f32 = 8.0;
/// O vão entre a beirada direita dos cartões e a barra de rolagem da lista (a faixa dela).
pub const LIST_BAR_GAP: f32 = 4.0;
pub const LIST_PAD_BOTTOM: f32 = 8.0;
pub const LIST_HEAD_H: f32 = 44.0;
pub const LIST_HEAD_PAD_X: f32 = 6.0;
pub const LIST_TITLE_PX: f32 = 13.0;
pub const LIST_TITLE_TRACKING_EM: f32 = 0.06;
/// A sombra de pairar da lista: `(dy, desfoque)` de cada camada.
pub const LIST_SHADOW_FAR: (f64, f64) = (6.0, 18.0);
pub const LIST_SHADOW_NEAR: (f64, f64) = (1.0, 2.0);

// ── O cartão (a secção) ──
pub const CARD_RADIUS: f32 = 8.0;
pub const CARD_PAD_TOP: f32 = 8.0;
pub const CARD_PAD_X: f32 = 10.0;
pub const CARD_PAD_BOTTOM: f32 = 10.0;
pub const CARD_GAP: f32 = 8.0;
/// O pé da lista («+ Add component», como o «+ Add a card» do Kanban).
pub const LIST_FOOTER_H: f32 = 32.0;
pub const CARD_HEAD_H: f32 = 26.0;
pub const CARD_HEAD_BOTTOM: f32 = 4.0;
pub const CARD_TITLE_PX: f32 = 12.0;
/// A seta «›» antes do título de um cartão DOBRADO (maquete C: Ordering, Material & Blend).
pub const CARD_CHEVRON: f32 = 14.0;
pub const CARD_CHEVRON_GAP: f32 = 2.0;
/// O contorno decorativo da secção (D6) envolve o cartão e as notas a esta distância.
pub const CARD_OUTLINE_GAP: f32 = 4.0;
pub const HEAD_ICON_BOX: f32 = 24.0;
pub const HEAD_ICON_GLYPH: f32 = 16.0;
pub const HEAD_ICON_RADIUS: f32 = 6.0;

// ── O rótulo acima do campo ──
pub const LABEL_PX: f32 = 11.0;
pub const LABEL_TOP: f32 = 8.0;
pub const LABEL_BOTTOM: f32 = 4.0;
/// A marca de animação (ponto / losango) na linha do rótulo, à direita da caixa (D17c).
pub const KEY_MARK_PX: f32 = 6.0;
pub const KEY_MARK_BOX: f32 = 12.0;

// ── O campo ──
pub const FIELD_H: f32 = 28.0;
pub const FIELD_RADIUS: f32 = 6.0;
pub const FIELD_GAP: f32 = 6.0;
pub const FIELD_ROW_GAP: f32 = 6.0;
pub const FIELD_PAD_R: f32 = 8.0;
pub const FIELD_BORDER: f32 = 1.0;
/// T64: a caixa de marcar à Figma (dono 02/11) — o quadradinho e o espaço até ao nome.
pub const CHECK_BOX: f32 = 16.0;
pub const CHECK_GAP: f32 = 8.0;
/// A célula do ícone — a PEGA do scrub (o arrasto nasce só aqui).
pub const ICON_CELL: f32 = 26.0;
/// O desenho do ícone nas caixas e segmentos: `14` da maquete + 30 % (dono 01/11, «os ícones
/// devem ser maiores um pouco»). Cabe na célula (`26`) e no segmento (`FIELD_H − 2·SEG_PAD = 24`).
pub const ICON_GLYPH: f32 = 18.0;
pub const VALUE_PX: f32 = 12.0;
/// O mínimo de um campo de valor ao lado da caixa da cor: a célula do ícone (`26`) + a folga (`8`)
/// + «100» a `VALUE_PX` (`20,6 px` medidos, 02/11). A caixa da cor cresce até ao NOME do estilo
/// (maquete F: `145 : 70 px`) e pára aqui — no corpo de `257 px` da cena 3, «Brand orange» pede
/// `78,5 + 2·26 = 130,5 px` e a opacidade fica com `≥ 56`.
pub const FIELD_VALUE_MIN_W: f32 = 56.0;

// ── O segmentado ──
pub const SEG_PAD: f32 = 2.0;
pub const SEG_RADIUS: f32 = 4.0;
pub const SEG_TEXT_PX: f32 = 11.0;
pub const SEG_ICON_GAP: f32 = 5.0;
/// A folga de cada lado do conteúdo de um segmento (D32: a largura é a do conteúdo, não `1/n`).
pub const SEG_INSET: f32 = 6.0;

/// ⭐ **D32 — cada segmento tem a largura do que mostra** (dono 02/11: «Auto (…)» cortado ao lado
/// de dois ícones com a mesma largura). `need` é a largura do conteúdo com folga e `fixo` marca o
/// segmento só de ícone. A sobra reparte-se por igual; a falta corta só os de texto, na proporção.
#[must_use]
pub fn seg_widths(segs: &[(f32, bool)], avail: f32) -> Vec<f32> {
    let total: f32 = segs.iter().map(|s| s.0).sum();
    if total <= avail || segs.is_empty() {
        let extra = (avail - total).max(0.0) / segs.len().max(1) as f32;
        return segs.iter().map(|s| s.0 + extra).collect();
    }
    let fixos: f32 = segs.iter().filter(|s| s.1).map(|s| s.0).sum();
    let flex = total - fixos;
    if fixos >= avail || flex <= 0.0 {
        return segs.iter().map(|s| s.0 * avail / total).collect();
    }
    let k = (avail - fixos) / flex;
    segs.iter()
        .map(|s| if s.1 { s.0 } else { s.0 * k })
        .collect()
}

// ── O scrub (Figma) ──
/// Shift durante o arrasto ou nas setas: passo ×10.
pub const SCRUB_SHIFT_MUL: f64 = 10.0;
/// Alt durante o arrasto ou nas setas: passo ×0,1.
pub const SCRUB_ALT_MUL: f64 = 0.1;

// ── O cartão solto no canvas (C5, `02_especificacao.md` §8) ──
/// A linha-fantasma que fica na lista: «<Título> · on canvas».
pub const LOOSE_GHOST_H: f32 = 34.0;
/// O tracejado da linha-fantasma: traço e vão.
pub const LOOSE_DASH: (f64, f64) = (4.0, 3.0);
/// A sombra de pairar do cartão solto (dy, desfoque): longe e perto (8/20 + 1/3).
pub const LOOSE_SHADOW_FAR: (f64, f64) = (8.0, 20.0);
pub const LOOSE_SHADOW_NEAR: (f64, f64) = (1.0, 3.0);
/// Quanto do cartão solto fica SEMPRE dentro do canvas (a pega nunca se perde).
pub const LOOSE_KEEP_VISIBLE: f32 = 48.0;
/// O piscar de «trazer à vista» (clique na linha-fantasma), em segundos.
pub const LOOSE_FLASH_S: f64 = 0.6;
/// T39: a faixa das bordas esquerda/direita que muda a largura do solto (metade dentro, metade fora).
pub const LOOSE_EDGE_GRIP: f32 = 8.0;
/// O título mais curto que ainda se lê no cabeçalho («Angle Limit» a 12 px ≈ 70 px).
pub const LOOSE_MIN_TITLE_W: f32 = 72.0;
/// T39: a largura mínima do solto — o título curto mais três ícones do cabeçalho (alfinete, «–», «×»).
/// A máxima é a do canvas (o recurso é o ecrã).
pub const LOOSE_MIN_W: f32 = CARD_PAD_X * 2.0 + HEAD_ICON_BOX * 3.0 + LOOSE_MIN_TITLE_W;
/// T42: o nome do sujeito de um solto preso a OUTRO objecto pisca devagar com o rato por cima —
/// o período (s) e o mínimo da opacidade.
pub const LOOSE_PIN_BLINK_S: f64 = 1.6;
pub const LOOSE_PIN_BLINK_MIN: f32 = 0.3;
// ── T44: a corda do alfinete (a lei do nó `motion.verlet-rope`, em px de ecrã) ──
/// O traço da corda, o nó redondo na ponta do objecto e a solda na ponta do cartão.
pub const LOOSE_ROPE_W: f32 = 2.0;
pub const LOOSE_ROPE_KNOT_R: f32 = 4.0;
pub const LOOSE_ROPE_WELD_R: f32 = 5.0;
/// Os pontos da corda × os passes de relaxação — custo medido 02/11 (release, mínimo de 20 rodadas
/// intercaladas, 600 passos): 16 × 24 = 6,6 µs/quadro · **32 × 12 = 7,1 µs** · 32 × 24 = 14,1 µs.
/// O dobro dos pontos com metade dos passes custa o mesmo (≈ 0,04 % de um quadro a 60 Hz).
pub const LOOSE_ROPE_POINTS: usize = 32;
/// O comprimento = distância + o vão HORIZONTAL × (folga − 1) + o mínimo × (vão horizontal ÷
/// distância): de lado pende, na vertical é um fio de prumo (T44c).
pub const LOOSE_ROPE_SLACK: f32 = 1.12;
pub const LOOSE_ROPE_SLACK_MIN: f32 = 24.0;
/// Na vertical a corda fica esta fração MAIS CURTA que a distância — tensa, o peso não a encurva.
pub const LOOSE_ROPE_PLUMB_TENSION: f32 = 0.06;
/// A gravidade (px/s²), os passes de relaxação, o amortecimento e a flexão.
pub const LOOSE_ROPE_GRAVITY: f32 = 1800.0;
pub const LOOSE_ROPE_ITERATIONS: usize = 12;
pub const LOOSE_ROPE_DAMPING: f32 = 0.06;
pub const LOOSE_ROPE_BEND: f32 = 0.4;
/// T41: o recolher da corda que se solta (devolver o cartão, ciclar o alfinete): a ponta do cartão
/// corre até ao objecto em este tempo (s), com a folga a encolher — o mesmo tempo do piscar de «trazer
/// à vista», a escala das transições dos cartões.
pub const LOOSE_ROPE_REEL_S: f32 = 0.35;
/// T41/T42: o solto que some (outra classe escolhida) ou volta esmaece em este tempo (s) — o mesmo
/// do recolher da corda.
pub const LOOSE_FADE_S: f32 = LOOSE_ROPE_REEL_S;
/// O passo mais longo (s) — três quadros a 60 Hz: uma janela parada não lança a corda ao voltar.
pub const LOOSE_ROPE_DT_MAX: f32 = 0.05;
