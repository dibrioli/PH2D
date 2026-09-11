//! **A JANELA DO TRAIL SEGUE O PINCEL** — o cap que o modelo de referência
//! impunha ao produto.
//!
//! ⚠️ **Report do Enio (2026-08-01):** *"todos esses testes tenho feito com raio
//! 300, mas na prática o app limita o tamanho para aproximadamente 200"*. Medido
//! pela porta do artista antes de tocar em código: o slider promete
//! `BRUSH_SIZE_MAX_PX = 512` e o traço **saturava em 119 px de largura** a
//! partir do raio 100 — raio efetivo **59,5**, **0,15×** do pedido em 400, e em
//! SILÊNCIO.
//!
//! A causa é o `TRAIL_HALF = 61 // ceil(35 + 4*6) + 2`: **35 é o teto de raio do
//! modelo JS de referência**, e ele era o teto deste produto. É a forma exata
//! que o CLAUDE.md §0 nomeia — *nunca deixe o fallback definir o produto* — e o
//! §0 também diz o que fazer: **medir**, e escrever o número que a medição deu.

use ph2d_wet_paint::brush::BrushShape;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, TRAIL_HALF, TRAIL_SIZE, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

fn dab(x: f64, r: f64) -> Dab {
    dab_at(x, 200.0, r)
}

fn dab_at(x: f64, y: f64, r: f64) -> Dab {
    Dab {
        x,
        y,
        r,
        hardness: 0.5,
        intensity: 1.0,
        water_amount: 0.5,
        dry_gate: 0.0,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// **A janela cresce com o raio** — e o oráculo é o RETÂNGULO TOCADO, nunca uma
/// constante (o gate não pode espelhar a regra que ele julga).
///
/// Mutação que sangra: devolver cedo de `Trail::fit_to` (o cap de volta) — a
/// largura tocada satura e as duas asserções de crescimento morrem.
#[test]
fn the_touched_rect_grows_with_the_brush_instead_of_saturating() {
    let mut e = Engine::new(1400, 400);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    let mut widths = Vec::new();
    for r in [30.0f64, 120.0, 300.0] {
        let mut t = Trail::default();
        t.start_stroke(700.0, 200.0, [0.2, 0.3, 0.4], TrailMode::Paint);
        t.on_segment(4.0, 0.05 * 2.0 * r);
        let mut ge = Engine::new(1400, 400);
        let g = ge.active_grid_mut();
        let _ = t.accumulate_paint(g, &p, &tex, &dab(700.0, r), false);
        let rect = t.touched_extent_for_measure();
        widths.push(rect.map_or(0, |(x0, _, x1, _)| (x1 - x0 + 1).max(0)));
    }
    assert!(
        widths[1] > widths[0] * 2,
        "raio 120 tocou {} contra {} do raio 30: a janela parou de crescer com o \
         pincel, e o teto do MODELO de referencia voltou a ser o teto do PRODUTO",
        widths[1],
        widths[0]
    );
    assert!(
        widths[2] > widths[1] * 2,
        "raio 300 tocou {} contra {} do raio 120: idem — e e no raio grande que o \
         artista bate no cap (medido: o traco saturava em 119 px de largura)",
        widths[2],
        widths[1]
    );
}

/// **E o PISO fica**, que é o que mantém o fingerprint byte-idêntico.
///
/// ⚠️ Um pincel dentro do teto do modelo JS tem de produzir a janela EXATA de
/// antes — é isto que torna a wave uma extensão e não uma mudança de modelo, e
/// é por isso que o `TRAIL_HALF` continua existindo como PISO.
#[test]
fn a_brush_within_the_reference_models_ceiling_keeps_the_old_window() {
    let mut e = Engine::new(400, 400);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(200.0, 200.0, [0.2, 0.3, 0.4], TrailMode::Paint);
    t.on_segment(4.0, 6.0);
    let g = e.active_grid_mut();
    let _ = t.accumulate_paint(g, &p, &tex, &dab(200.0, 20.0), false);
    assert_eq!(
        t.window_half_for_measure(),
        TRAIL_HALF,
        "um pincel dentro do teto do modelo (raio 20 <= 35) moveu a janela: o \
         fingerprint do ADR-0134 deixa de ser byte-identico POR CONSTRUCAO"
    );
    assert_eq!(TRAIL_SIZE, TRAIL_HALF * 2 + 1);
}

/// Suja o bico pelo PICKUP (a única forma como ele suja de verdade), roda dois
/// transfers com a auto-limpeza no valor `clean`, e devolve o azul do texel no
/// CENTRO da janela mais a meia-largura viva.
///
/// ⚠️ **O oráculo é o A/B do próprio knob, e a 1ª versão deste gate estava
/// errada:** ela lia o azul ANTES e DEPOIS de um transfer, e o número ANDAVA
/// PARA BAIXO nos dois mundos (189,66 → 165,44) porque o PICKUP e a LIMPEZA
/// puxam em direções opostas dentro do mesmo transfer e o pickup puxa ~16×
/// mais forte. Medir o líquido de dois efeitos que competem não distingue
/// "limpou" de "não limpou" — a pergunta é *o knob ALCANÇA este texel?*, e
/// quem a responde é a diferença entre `clean` armado e `clean` zerado.
///
/// ⚠️ O centro é lido da janela VIVA (`window_half_for_measure`), nunca de um
/// número que espelhe a fórmula do `fit_to` — e é o centro do DAB, então está
/// sujo por construção nos dois tamanhos de pincel.
///
/// A tinta da folha é VERMELHA e a do traço é AZUL: limpar é o azul SUBIR.
fn tip_blue_at_centre(r: f64, clean: f64) -> (f32, i32) {
    const CX: f64 = 700.0;
    const CY: f64 = 450.0;
    let mut e = Engine::new(1400, 900);
    // Sem pickup não há sujeira, e um gate sobre um bico limpo é vácuo.
    e.set_knob(Knob::Pickup, 0.2);
    e.set_knob(Knob::TipClean, clean);
    let p = e.sim.gather_params(&e.tuning);
    let tex: Vec<f32> = e.bristle_texture_for_measure();
    {
        let g = e.active_grid_mut();
        for i in 0..g.susp.len() {
            g.susp[i] = 800.0;
            g.susp_rgb[i] = [200.0, 30.0, 30.0];
        }
    }
    let mut t = Trail::default();
    t.start_stroke(CX, CY, [10.0, 10.0, 220.0], TrailMode::Paint);
    t.on_segment(4.0, 0.05 * 2.0 * r);
    let g = e.active_grid_mut();
    // 1º transfer: a limpeza é no-op (bico == base) e o PICKUP suja.
    // 2º transfer: agora a limpeza tem o que limpar.
    for _ in 0..2 {
        let _ = t.accumulate_paint(g, &p, &tex, &dab_at(CX, CY, r), false);
        let _ = t.transfer_paint(g, &p);
    }
    let half = t.window_half_for_measure();
    let blue = t.tip_rgb_for_measure(half, half).expect("centro da janela")[2];
    (blue, half)
}

/// **A AUTO-LIMPEZA DO BICO COBRE A JANELA VIVA, NÃO A DO PISO.**
///
/// ⚠️ Defeito REAL desta wave, achado relendo o `transfer_paint` para decompor
/// o custo do depósito: o laço de auto-limpeza (SPEC §10, passo 1) ia `0..N`, e
/// `N` é a área da janela do **PISO** (`TRAIL_SIZE²` = 15129). Com o cap
/// removido os buffers passaram a medir `size²`, então num pincel maior que o
/// piso a limpeza cobria os primeiros 15129 índices LINEARES — que não são uma
/// região, são as ~18 primeiras LINHAS de uma janela de 845 de largura. O resto
/// do bico não limpava nunca.
///
/// É a classe que este repo já nomeou: **uma constante que era igual ao valor
/// vivo**, deixada para trás no dia em que o vivo virou variável.
///
/// ⚠️ **Alcançável:** `Knob::TipClean` é knob do grupo PAINT do painel Tuning
/// (boot 0.0, faixa até 0.05) — o artista o levanta e o pincel grande fica com
/// a cabeça limpa e o corpo sujo para sempre.
///
/// **O par é o gate.** O CONTROLE é um pincel dentro do teto do modelo, cuja
/// janela inteira cabe em `N`: ele limpa nas duas versões, e é ele que prova
/// que a fixture de fato sujou o bico (sem essa metade, o teste de baixo
/// passaria num bico que ninguém sujou). O TESTE é o pincel grande, no centro
/// da janela, índice muito além de `N`.
///
/// Mutação que sangra: `0..self.tip_r.len()` de volta para `0..N` — o controle
/// segue VERDE e só o pincel grande morre, que é a assinatura exata do defeito.
#[test]
fn the_tip_cleaning_covers_the_live_window_not_the_floor_sized_one() {
    // CONTROLE: raio 20 <= o teto do modelo ⇒ janela do piso, tudo dentro de N.
    let (c_off, c_half) = tip_blue_at_centre(20.0, 0.0);
    let (c_on, _) = tip_blue_at_centre(20.0, 0.05);
    assert_eq!(
        c_half, TRAIL_HALF,
        "o controle tem de usar a janela do PISO"
    );
    assert!(
        c_on > c_off + 0.1,
        "o CONTROLE nao limpou (azul {c_off:.2} sem knob -> {c_on:.2} com knob): a \
         fixture nao sujou o bico, entao o teste do pincel grande nao significa nada"
    );

    // TESTE: um pincel bem maior que o teto do modelo.
    let (b_off, b_half) = tip_blue_at_centre(300.0, 0.0);
    let (b_on, _) = tip_blue_at_centre(300.0, 0.05);
    // A fixture CONTÉM o fenômeno: o centro desta janela vive depois do fim da
    // janela do piso. Sem esta linha o teste abaixo poderia passar por vácuo.
    let centre_index = i64::from(b_half) + i64::from(b_half) * i64::from(b_half * 2 + 1);
    let floor_area = i64::from(TRAIL_SIZE) * i64::from(TRAIL_SIZE);
    assert!(
        centre_index > floor_area,
        "o centro da janela grande caiu em {centre_index}, dentro da area do piso \
         ({floor_area}): a fixture nao contem o fenomeno"
    );
    assert!(
        b_on > b_off + 0.1,
        "no centro da janela de um pincel grande o knob de auto-limpeza NAO faz \
         nada (azul {b_off:.2} sem knob -> {b_on:.2} com knob) enquanto no controle \
         ele move {c_off:.2} -> {c_on:.2}: a limpeza esta presa na area da janela \
         do PISO, e o corpo do pincel fica sujo para sempre"
    );
}
