//! Os gates de PRODUTO da razão da grade — a wave inteira medida pela porta do
//! artista (`on_canvas_pointer`), não pelas funções de [`super::grid_map`].
//!
//! A ordem é a das perguntas que decidem se a wave pode shipar:
//!
//! 1. **razão 1 pinta o que sempre pintou** — e o oráculo é a expressão ANTIGA
//!    do composite, congelada aqui. Sem isto a wave seria uma mudança de
//!    aparência disfarçada de otimização.
//! 2. **a grade encolhe e a tinta pousa no MESMO lugar** — as duas metades: o
//!    ganho existe, e a conversão não desloca nada.
//! 3. **a tinta alcança a última coluna de pixels** — o `div_ceil`.
//! 4. **trocar a razão encerra a água viva; não trocar, não.**
//! 5. **um passo fica mais barato** — o número que a wave entrega, como RAZÃO
//!    entre duas grades na MESMA tela (imune à deriva de máquina).

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::Falloff;

const W: usize = 400;
const H: usize = 240;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool de Wet Paint com a razão de grade pedida, canvas branco.
fn wet_tool(ratio: u8) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    let b = BrushSpec {
        radius_px: 48.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.8, 0.1, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("wetpaint");
    // Pela PORTA (o slider), não pelo campo — é o caminho do artista.
    t.set_wet_grid_ratio(f64::from(ratio));
    t
}

/// Um traço curto e horizontal no meio da tela.
fn stroke(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([120.0, 120.0], PointerPhase::Down));
    for k in 1..=8 {
        t.on_canvas_pointer(cp([120.0 + 20.0 * k as f32, 120.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([280.0, 120.0], PointerPhase::Up));
}

/// Força um composite da tela INTEIRA, para o canvas ser função só do estado
/// atual do pigmento (sem regiões de composites anteriores).
fn composite_all(t: &mut PainterTool) {
    if let Some(sess) = t.paint.wetpaint.session.as_mut() {
        sess.bring_home();
        sess.engine.mark_dirty_full();
    }
    crate::tool::paint::wetpaint::composite_for_measure(t);
}

/// **O OVER QUE SHIPAVA, congelado** — a expressão exata do composite antes da
/// wave, quando `pigment` era canvas-shaped e um pixel era uma célula.
///
/// ⚠️ Ele mora sob `cfg(test)` e não tem chamador de produção **de propósito**:
/// um `pub(super)` sem chamador não é código morto silencioso, é uma SEGUNDA
/// resposta esperando alguém chamá-la (a lição do `warp_axis` e do
/// `serial_side`). O que ele é, é um oráculo.
fn over_as_it_shipped(pigment: &[u8], base: &[u8], px: usize, py: usize) -> [u8; 4] {
    let o = (py * W + px) * 4;
    let pa = pigment[o + 3] as f32 / 255.0;
    if pa <= 0.0 {
        return [base[o], base[o + 1], base[o + 2], base[o + 3]];
    }
    let ba = base[o + 3] as f32 / 255.0;
    let oa = pa + ba * (1.0 - pa);
    let mut out = [0u8; 4];
    for ch in 0..3 {
        let pc = pigment[o + ch] as f32;
        let bc = base[o + ch] as f32;
        out[ch] = ((pc * pa + bc * ba * (1.0 - pa)) / oa).round() as u8;
    }
    out[3] = (oa * 255.0).round() as u8;
    out
}

/// **A razão 1 compõe EXACTAMENTE o que a versão pré-wave compunha** — byte a
/// byte, contra o oráculo congelado acima.
///
/// ⚠️ **A mutação que eu prometi aqui NÃO sangra, e o defeito era da minha
/// afirmação.** Eu escrevi que forçar a razão 1 pela rota bilinear divergiria
/// (*"a bilinear computa `(c·a)/255` em `f64` e a identidade `c·(a/255)` em
/// `f32`"*): medido, **0 bytes divergem** nos 96.000 pixels desta fixture. O
/// motivo é aritmético e vale a pena estar escrito — em razão 1 as frações
/// bilineares são `tx = ty = 0` **exatos**, os três cantos vizinhos caem no
/// `continue` de peso zero, e o único termo sobrevivente atravessa um
/// `.round() as u8` que absorve os ~1e-7 de diferença entre as duas ordens.
///
/// Então a **rota de identidade existe pelo CUSTO** (quatro bytes contra quatro
/// taps de `f64` por pixel, no caminho que o default do produto percorre 40×/s),
/// não pela identidade — e este gate prova o que importa: o produto compõe o que
/// compunha. Segunda vez nesta linha que uma mutação minha não sangra porque a
/// premissa aritmética estava errada (a primeira foi a comutatividade da adição
/// IEEE-754 no warp da aquarela, doc 28 §5.11).
#[test]
fn ratio_one_composites_exactly_what_the_old_over_produced() {
    let mut t = wet_tool(1);
    stroke(&mut t);
    composite_all(&mut t);
    let sess = t.paint.wetpaint.session.as_ref().expect("uma sessao viva");
    assert_eq!(sess.ratio, 1);
    assert_eq!(sess.grid, (W, H), "razao 1 e uma celula por pixel");
    let (pig, base) = (&sess.pigment, &sess.base);
    let mut diff = 0usize;
    let mut worst = 0i32;
    for py in 0..H {
        for px in 0..W {
            let want = over_as_it_shipped(pig, base, px, py);
            let o = (py * W + px) * 4;
            let got = &t.canvas_rgba[o..o + 4];
            for c in 0..4 {
                let d = i32::from(got[c]) - i32::from(want[c]);
                if d != 0 {
                    diff += 1;
                    worst = worst.max(d.abs());
                }
            }
        }
    }
    assert_eq!(diff, 0, "{diff} bytes divergem, pior delta {worst}");
}

/// Centroide (em pixels de canvas) e massa da tinta que o traço deixou no
/// canvas — o oráculo de APARÊNCIA, lido dos pixels e não da grade.
fn ink_centroid(t: &PainterTool) -> ([f64; 2], f64) {
    let mut m = 0.0f64;
    let mut sx = 0.0f64;
    let mut sy = 0.0f64;
    for py in 0..H {
        for px in 0..W {
            let o = (py * W + px) * 4;
            // Quanto o pixel escureceu em relacao ao branco = quanta tinta ha.
            let ink = 255.0 - f64::from(t.canvas_rgba[o]);
            if ink > 1.0 {
                m += ink;
                sx += ink * px as f64;
                sy += ink * py as f64;
            }
        }
    }
    if m <= 0.0 {
        return ([0.0, 0.0], 0.0);
    }
    ([sx / m, sy / m], m)
}

/// **A grade encolhe pela razão, e a tinta continua onde o artista a pôs.**
///
/// As duas metades são um gate só de propósito: um ganho que desloca a pintura
/// não é ganho, e uma conversão correta que não corta trabalho não é a wave.
///
/// Mutação: tirar o `/ r` de `px_to_cell` (voltar a `px + 1.0`) mantém a
/// contagem de células — o ganho — e joga o traço para o canto superior
/// esquerdo, o que este gate nomeia em pixels.
#[test]
fn a_coarser_grid_has_fewer_cells_and_the_paint_lands_in_the_same_place() {
    let mut fine = wet_tool(1);
    stroke(&mut fine);
    composite_all(&mut fine);
    let (c1, m1) = ink_centroid(&fine);

    let mut coarse = wet_tool(4);
    stroke(&mut coarse);
    composite_all(&mut coarse);
    let (c4, m4) = ink_centroid(&coarse);

    let g1 = fine.paint.wetpaint.session.as_ref().unwrap().grid;
    let g4 = coarse.paint.wetpaint.session.as_ref().unwrap().grid;
    assert_eq!(g1, (W, H));
    assert_eq!(g4, (W.div_ceil(4), H.div_ceil(4)));
    let cells1 = g1.0 * g1.1;
    let cells4 = g4.0 * g4.1;
    assert!(
        cells1 as f64 / cells4 as f64 > 14.0,
        "razao 4 deveria cortar ~16x as celulas: {cells1} -> {cells4}"
    );

    assert!(m1 > 0.0 && m4 > 0.0, "as duas pintaram: {m1} / {m4}");
    let dx = (c1[0] - c4[0]).abs();
    let dy = (c1[1] - c4[1]).abs();
    // Tolerancia = uma celula grossa (4 px): a agua e resolvida mais grosso,
    // mas o CENTRO do que ela pinta e o mesmo lugar.
    assert!(
        dx < 4.0 && dy < 4.0,
        "a tinta se deslocou: fino {c1:?} vs grosso {c4:?} (dx {dx:.2}, dy {dy:.2})"
    );
}

/// **A tinta chega à borda do documento em qualquer razão** — o `div_ceil` de
/// [`super::grid_map::grid_dims`] e o `min(w)` de `cell_rect_to_px`, medidos
/// nos PIXELS.
///
/// ⚠️ **O oráculo mede a FAIXA da borda, não a última célula.** A 1ª versão
/// deste gate exigia massa na última COLUNA de células e nasceu vermelha na
/// razão 1 — isto é, sobre o mundo que já shipava: o stamp do motor recorta a
/// janela em `min(grid_w − 1)`, então a última coluna viva nunca recebe, em
/// nenhuma razão. Um gate que falha no controle está medindo a coisa errada.
///
/// ⚠️ **Ele NÃO pega o `floor` no `grid_dims`, e afirmar que pegava seria
/// over-claim:** medido, a mutação `w / r` tira ≤ `r − 1` pixels do fim, e a
/// faixa de 20 px que este gate soma continua com tinta ⇒ **verde sobre o
/// defeito**. Quem pina o `div_ceil` no PRODUTO é
/// `changing_the_ratio_ends_the_live_session_and_keeping_it_does_not` (ele
/// afirma `sess.grid == (W.div_ceil(6), H.div_ceil(6))` e sangra), e na unidade
/// são `every_canvas_pixel_has_a_cell` + `a_cell_window_covers_its_pixels…`.
///
/// O que ele PROVA é a propriedade que sobra e que nenhum outro cobre: uma
/// conversão que perdesse a borda inteira sangra aqui — e sangra de fato com a
/// mutação `px_to_cell` sem o `/ r`.
#[test]
fn paint_reaches_the_document_edge_at_every_ratio() {
    for ratio in [1u8, 3, 7] {
        let mut t = wet_tool(ratio);
        // Um traço vertical colado na borda DIREITA — o pincel de 48 px mede
        // ainda 6,9 células a 7:1, acima do limiar do banco de cerdas.
        let x = W as f32 - 6.0;
        t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Down));
        for k in 1..=6 {
            t.on_canvas_pointer(cp([x, 80.0 + 12.0 * k as f32], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 152.0], PointerPhase::Up));
        composite_all(&mut t);
        // A faixa dos ultimos 20 px de canvas.
        let mut edge = 0.0f64;
        for py in 0..H {
            for px in (W - 20)..W {
                let o = (py * W + px) * 4;
                edge += 255.0 - f64::from(t.canvas_rgba[o]);
            }
        }
        assert!(
            edge > 0.0,
            "ratio {ratio}: a faixa dos ultimos 20 px do canvas ficou seca — \
             a fatia final nao tem celula"
        );
    }
}

/// **O NÚMERO que a razão custa: a célula não pode ser maior que o pincel.**
///
/// A tile de cerdas do modelo (128×128) é indexada em unidades de CÉLULA, com
/// pontas de ~1-2 células — então o que decide se o banco resolve o pincel é o
/// **raio em células**, `raio_px / ratio`. Medido pela porta do motor, com a
/// silhueta do host (o caminho do produto):
///
/// ```text
///   raio (células)   massa      cobertura
///        < 1,5        0,0          0 %     <- NADA e depositado
///          1,5       16,6         14 %     (uma celula)
///          3,0      204,2          7 %
///          6,0     1375,9        6,2 %
///         12,5     4990,6        5,5 %     <- o regime normal do modelo
///         25,0    16258,3        5,2 %
/// ```
///
/// A cobertura converge em ~5 % (a densidade do banco), então **acima de ~6
/// células o depósito é auto-similar**: é o depósito de sempre, resolvido mais
/// grosso. O que a razão custa é RESOLUÇÃO, e o limite duro é o cliff em 1,5
/// células: **`ratio` acima de `raio_px / 1,5` não pinta nada.**
///
/// ⚠️ **A massa de tinta NÃO é o oráculo, e a 1ª versão deste gate errou por
/// isso** — ela exigia que a razão 30 depositasse menos da metade da razão 4 e
/// mediu **74 %**: cada célula pinta `ratio²` pixels de canvas, então a massa
/// visível se preserva enquanto a forma granula. *Número no lugar errado diz o
/// contrário do que a foto diz* — a mesma armadilha que o doc 25 §13.10
/// registrou no eixo do traço de máscara.
///
/// ⚠️ **Não há piso nem cap, de propósito.** Um piso no raio em células faria o
/// pincel pintar MAIOR do que o artista pediu (mentira silenciosa); um cap na
/// razão faria o pincel decidir a resolução do fluido. O comportamento honesto
/// é o que este gate pina: com a célula maior que o pincel, nada sai — e o
/// slider diz "Grid Size (px)" ao lado do tamanho do pincel, então a leitura é
/// direta. A cura possível (ler a tile em escala de CANVAS, o que faria a
/// granulação convergir para a da razão 1) muda a APARÊNCIA do depósito e é
/// wave própria com smoke próprio; fica nomeada no doc 28 §5.41.
#[test]
fn a_cell_larger_than_the_brush_deposits_nothing() {
    // Um pincel PEQUENO — e o fenômeno vive aqui, não no pincel grande: a 30:1
    // um raio de 48 px ainda mede 1,6 células e pinta.
    let mass_with = |radius_px: f32, ratio: u8| -> f64 {
        let mut t = wet_tool(ratio);
        let b = BrushSpec {
            radius_px,
            ..t.paint.brush
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        stroke(&mut t);
        composite_all(&mut t);
        ink_centroid(&t).1
    };
    // 12 px de raio: a 1:1 sao 12 celulas (regime normal) e a 30:1 sao 0,4.
    let fine = mass_with(12.0, 1);
    assert!(fine > 0.0, "12 celulas de raio pintam: {fine}");
    let starved = mass_with(12.0, 30);
    assert_eq!(
        starved, 0.0,
        "uma celula de 30 px sob um pincel de 12 px nao tem como depositar \
         (raio = 0,4 celula, abaixo do cliff de 1,5) — se isto passar a pintar, \
         o banco de cerdas mudou de escala e a nota do doc 28 §5.41 esta obsoleta"
    );
    // E o pincel GRANDE atravessa a faixa inteira do slider — o caso de uso.
    let big = mass_with(48.0, 30);
    assert!(
        big > 0.0,
        "48 px de raio a 30:1 sao 1,6 celulas, acima do cliff: {big}"
    );
}

/// **Trocar a razão encerra a água viva; re-emitir o mesmo valor não.**
///
/// A segunda metade é o que torna o chip numérico seguro: um arrasto de slider
/// re-emite o valor a cada frame, e uma porta sem o guard de igualdade mataria
/// a sessão em todos eles.
///
/// Mutação: tirar o guard `== want` faz a segunda asserção sangrar (a sessão
/// morre num no-op); tirar o `wetpaint_end_session` faz a primeira sangrar (o
/// motor continua com a grade antiga sob uma razão nova, e o dab passaria a
/// pousar `r` vezes fora do lugar).
#[test]
fn changing_the_ratio_ends_the_live_session_and_keeping_it_does_not() {
    let mut t = wet_tool(1);
    stroke(&mut t);
    assert!(t.paint.wetpaint.session.is_some(), "ha agua viva");

    // Re-emitir o MESMO valor: nada acontece.
    t.set_wet_grid_ratio(1.0);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "re-emitir o mesmo valor nao pode matar a sessao"
    );

    // Trocar: a sessao morre (o bake), e a proxima nasce na razao nova.
    t.set_wet_grid_ratio(6.0);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "trocar a razao tem de encerrar a sessao"
    );
    assert_eq!(t.paint.wetpaint.grid_ratio, 6);
    stroke(&mut t);
    let sess = t.paint.wetpaint.session.as_ref().expect("uma sessao nova");
    assert_eq!(sess.ratio, 6, "a sessao nova nasce na razao autorada");
    assert_eq!(sess.grid, (W.div_ceil(6), H.div_ceil(6)));
}

/// A faixa do slider é honrada pela porta (o clamp).
#[test]
fn the_ratio_door_clamps_to_the_sliders_range() {
    let mut t = wet_tool(1);
    t.set_wet_grid_ratio(0.0);
    assert_eq!(t.paint.wetpaint.grid_ratio, grid_map::MIN_RATIO);
    t.set_wet_grid_ratio(999.0);
    assert_eq!(t.paint.wetpaint.grid_ratio, grid_map::MAX_RATIO);
}
// sonda temporária — colada em grid_ratio_tests.rs

/// **O slider chega ao tool pelo BARRAMENTO** (`PanelEvent::SetValue`), não por
/// uma chamada direta — a costura que o painel de fato usa.
///
/// Mutação: tirar o braço `PAINTER_WETPAINT_GRID` do `set_wet_knob_value` faz o
/// evento não ser consumido e este gate nomeia o id.
#[test]
fn the_grid_slider_reaches_the_tool_over_the_panel_bus() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = wet_tool(1);
    stroke(&mut t);
    assert!(t.paint.wetpaint.session.is_some());
    assert!(
        t.route_brush_wetpaint_event(&PanelEvent::SetValue(core_ids::PAINTER_WETPAINT_GRID, 5.0)),
        "o SetValue da grade nao foi consumido pela rota do wet"
    );
    assert_eq!(t.paint.wetpaint.grid_ratio, 5);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "a rota tem de passar pela PORTA (que encerra a sessao), nao escrever o campo"
    );
    // E o valor sobrevive no snapshot que o painel lê de volta (senão o chip
    // mostraria o valor antigo depois de commitado).
    assert_eq!(t.brush_settings().wet_grid_ratio, 5);
}

#[test]
#[ignore]
fn probe_cell_alpha_at_the_edge() {
    // O alpha do PIGMENTO por celula atravessando a borda lateral de um traco
    // vertical: se for degrau duro (255,255,0,0) a silhueta SATUROU e o AA de
    // entrada e inerte; se houver rampa, ha informacao a interpolar.
    for ratio in [1u8, 8] {
        for (label, moves) in [("traco LENTO (muitos dabs)", 40u32), ("um DAB so", 0)] {
            let mut t = wet_tool(ratio);
            let x = 200.0f32;
            t.on_canvas_pointer(cp([x, 60.0], PointerPhase::Down));
            for k in 1..=moves {
                t.on_canvas_pointer(cp([x, 60.0 + 4.0 * k as f32], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x, 60.0 + 4.0 * moves as f32], PointerPhase::Up));
            composite_all(&mut t);
            let sess = t.paint.wetpaint.session.as_ref().expect("sessao");
            let (gw, _) = sess.grid;
            // A celula do centro do traco e a linha de celula do meio.
            let ccx = (x as usize) / usize::from(ratio) + 1;
            let ccy = (100usize) / usize::from(ratio) + 1;
            let mut row = Vec::new();
            for cx in ccx..(ccx + 8).min(gw + 1) {
                let o = ((ccy - 1) * gw + (cx - 1)) * 4;
                row.push(sess.pigment[o + 3]);
            }
            eprintln!("  ratio {ratio}, {label}: alpha do pigmento saindo do centro = {row:?}");
        }
    }
}

/// **O AA de entrada dá PENUMBRA onde a saturação não a come** — o número.
///
/// Medido: num dab ISOLADO a borda ganha uma célula de transição
/// (`[143, 153, 113, 33, 0, …]` com AA contra `[143, 153, 120, 0, …]` sem), e num
/// traço LENTO ele é quase inerte (212 → 204) porque centenas de dabs saturam a
/// célula — a silhueta é uma TAXA, e com muitas passadas o teto é atingido de
/// qualquer jeito (a mesma aritmética do doc 25 §13.9).
///
/// ⚠️ Por isso o AA de entrada **não é a cura da foto** — a cura é o smoothstep do
/// upsample (`the_upsample_weights_are_smooth_at_the_cell_seams`). Ele fica porque
/// **se paga**: 16 taps/célula em 8:1 custam **0,280 ms/move contra 2,081 da razão
/// 1** (7,4× mais barato), já que o carimbo é `O(área/ratio²)`.
///
/// Mutação: `n = 1` sempre (o AA de entrada desligado) faz a célula de penumbra
/// desaparecer e este gate a nomeia.
#[test]
fn the_deposit_aa_adds_penumbra_where_saturation_does_not_eat_it() {
    let mut t = wet_tool(8);
    // Um dab ISOLADO (Down+Up no mesmo ponto) — sem saturacao.
    t.on_canvas_pointer(cp([200.0, 120.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([200.0, 120.0], PointerPhase::Up));
    composite_all(&mut t);
    let sess = t.paint.wetpaint.session.as_ref().expect("sessao");
    let (gw, _) = sess.grid;
    let ccx = 200usize / 8 + 1;
    let ccy = 120usize / 8 + 1;
    let alpha = |cx: usize| -> u8 { sess.pigment[((ccy - 1) * gw + (cx - 1)) * 4 + 3] };
    // Saindo do centro, conta as celulas com cobertura PARCIAL antes do zero —
    // sao elas a penumbra que o supersampling cria.
    let mut partial = 0usize;
    for cx in ccx..ccx + 8 {
        let a = alpha(cx);
        if a == 0 {
            break;
        }
        if a < 200 {
            partial += 1;
        }
    }
    assert!(
        partial >= 2,
        "so {partial} celula(s) de cobertura parcial na borda de um dab isolado — \
         o AA de entrada esta inerte (esperado >= 2 com MAX_AA = 4)"
    );
}
