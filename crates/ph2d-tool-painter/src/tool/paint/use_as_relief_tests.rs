//! **Os quatro "Use as …" leem a APARÊNCIA, não o pigmento.**
//!
//! Report do Enio (2026-08-09): *"Use as Brush Shape não transfere para o brush os relevos criados
//! por Impasto"*, com o pedido de conferir os outros modos "Use as …" do menu. Conferidos: os
//! quatro passavam por DUAS portas do tool (`capture_layers_as_brush_shape` para o Shape,
//! `composite_to_lum` para Grain / Paper / Granulation) e **nenhuma das duas iluminava**.
//!
//! ⚠️ **O enquadramento que decide o conserto:** o BAKE (`RasterEditTool::run_full`) responde à
//! MESMA pergunta — *com o que este documento se parece?* — e o doc-comment dele já dizia por que
//! ilumina: *"o campo de altura não sobrevive ao Apply, então a sombra tem de ser assada, senão o
//! Apply jogaria o relevo fora em silêncio e devolveria tinta chapada"*. Duas portas, uma pergunta,
//! respostas diferentes — e o sintoma era assimétrico: um sprite JÁ assado da hierarquia levava a
//! sombra do relevo (o ramo `read_sprite_source` do shell lê a textura assada) e o documento ATIVO
//! não. Medido antes do conserto: **523 de 3600 texels diferem, pior delta 68**.
//!
//! ⚠️ **E a luz não podia entrar no Shape pela mesma porta**, o que é fato e não escolha: a silhueta
//! que o documento ativo captura é COBERTURA — alpha —, e o shade escreve COR e nunca alpha. Por
//! isso o relevo chega ali como um **GANHO** ([`super::impasto_gain`]), que multiplica a cobertura
//! em vez de a substituir: um desenho a tinta PRETA continua imprimindo a própria cobertura, e é o
//! relevo que a modula.

use crate::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

/// `paper_alpha` = a opacidade do papel: **255** é o caso do sprite importado e **0** o da tela
/// transparente em que o artista desenha do zero. A rota de Shape lê COBERTURA, então os dois
/// respondem coisas diferentes e os gates precisam dos dois.
///
/// `impasto` liga o depósito de corpo; com ele desligado o documento é o CONTROLE — sem relevo, tudo
/// aqui tem de ser byte-idêntico ao que já shipava.
fn ridge(paper_alpha: u8, impasto: bool, color: [f32; 3]) -> PainterTool {
    let size = 60u32;
    let mut t = PainterTool::default();
    let mut px = vec![255u8; (size * size * 4) as usize];
    for p in px.chunks_exact_mut(4) {
        p[3] = paper_alpha;
    }
    t.set_source(px, size, size);
    // Falloff MACIO de propósito: um disco duro deixa um platô de paredes verticais, cujo `h` é o
    // mesmo no centro e nos dois flancos — não há gradiente para a luz ler, e o gate estaria
    // afirmando sobre nada (a armadilha que o `impasto_light_reads_as_raised_not_engraved` documenta).
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 0.0,
        falloff: Falloff::Smooth,
        color,
        space_attenuation: false,
        impasto,
        impasto_depth: 1.0,
        impasto_smoothing: 0.0,
        impasto_body: 1.0,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.paint.impasto_show = true;
    let at = |x: f32, y: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(at(30.0, 10.0, PointerPhase::Down));
    t.on_canvas_pointer(at(30.0, 50.0, PointerPhase::Move));
    t.on_canvas_pointer(at(30.0, 50.0, PointerPhase::Up));
    t
}

fn lum(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect()
}

/// `(quantos texels diferem, pior delta)` — um resumo honesto de quão longe dois campos estão.
fn spread(a: &[u8], b: &[u8]) -> (usize, i32) {
    let n = a.len().min(b.len());
    let mut diff = 0usize;
    let mut worst = 0i32;
    for i in 0..n {
        let d = i32::from(a[i]) - i32::from(b[i]);
        if d != 0 {
            diff += 1;
        }
        worst = worst.max(d.abs());
    }
    (diff, worst)
}

fn silhouette(t: &mut PainterTool) -> Vec<u8> {
    t.capture_layers_as_brush_shape();
    t.brush_shape_image()
        .map(|(sil, _, _)| sil.to_vec())
        .expect("a captura produz uma silhueta")
}

/// **Uma pergunta, uma resposta.** O que o Grain / Paper / Granulation leem é, ao byte, o que o BAKE
/// produz — as duas portas para *"com o que este documento se parece?"* passam pela MESMA função.
///
/// **Mutação que deve sangrar:** remover o `apply_impasto_light` do `composite_to_lum`.
#[test]
fn the_use_as_paths_read_what_the_bake_bakes() {
    let (grain, _, _) = ridge(255, true, [0.1, 0.2, 0.3])
        .composite_to_lum()
        .expect("composite");
    let (baked, _, _) = ridge(255, true, [0.1, 0.2, 0.3]).run_full();
    let (diff, worst) = spread(&grain, &lum(&baked));
    assert_eq!(
        (diff, worst),
        (0, 0),
        "o Grain / Paper / Granulation e o BAKE respondem a mesma pergunta e discordam — e discordam \
         exatamente pela luz do relevo, que era o report do Enio"
    );
}

/// **O CONTROLE, e é ele que torna as duas metades seguras:** num documento sem relevo o passe
/// multiplica por 1 e soma 0, então nada aqui se move — nem a luminância que o Grain lê, nem a
/// silhueta que o Shape captura.
///
/// ⚠️ Sem este gate as duas mudanças seriam afirmações sobre o caso novo apenas; com ele, todo
/// documento que ninguém esculpiu está pinado.
#[test]
fn a_document_without_relief_is_untouched_to_the_byte() {
    let mut lit = ridge(255, false, [0.1, 0.2, 0.3]);
    lit.paint.impasto_show = true;
    let mut dark = ridge(255, false, [0.1, 0.2, 0.3]);
    dark.paint.impasto_show = false;

    let (a, _, _) = lit.composite_to_lum().expect("composite");
    let (b, _, _) = dark.composite_to_lum().expect("composite");
    assert_eq!(spread(&a, &b), (0, 0), "sem relevo, a luz nao muda um byte");
    assert_eq!(
        spread(&silhouette(&mut lit), &silhouette(&mut dark)),
        (0, 0),
        "sem relevo, o ganho e a identidade e a silhueta nao se move"
    );
}

/// **O relevo CHEGA à silhueta do pincel** — o report, medido no lugar onde ele foi feito.
///
/// A fixture é o papel OPACO de propósito: é o caso do sprite importado, onde a cobertura vale 255
/// em todo texel e a silhueta capturada era um quadrado sem feição nenhuma (medido antes:
/// `255..255`, um carimbo quadrado). Se o relevo não entra, não há nada aqui que o artista possa
/// usar como ponta de pincel.
///
/// **Mutação que deve sangrar:** remover a multiplicação pelo ganho no `reflatten_shape_image`.
#[test]
fn the_relief_reaches_the_brush_silhouette() {
    let sil = silhouette(&mut ridge(255, true, [0.1, 0.2, 0.3]));
    let (min, max) = sil
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(
        max - min > 32,
        "a silhueta capturada e chata ({min}..{max}) — o relevo esculpido nao alcancou o pincel, e \
         um documento opaco vira um carimbo QUADRADO"
    );
    // E o CONTROLE ao lado: o mesmo gesto sem impasto continua sendo o quadrado, que é o que uma
    // cobertura opaca honestamente é. Sem esta metade o gate passaria por qualquer silhueta variada.
    let flat = silhouette(&mut ridge(255, false, [0.1, 0.2, 0.3]));
    let (fmin, fmax) = flat
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    assert_eq!(
        (fmin, fmax),
        (255, 255),
        "controle: sem relevo a cobertura de um papel opaco E constante — se ela variar, o gate \
         acima nao esta medindo o relevo"
    );
}

/// **O ganho é a IDENTIDADE onde não há relevo**, e o número é exato porque os pesos Rec.601 deste
/// repo somam 256: um cinza neutro lumina para si mesmo. É disso que a byte-identidade do controle
/// acima depende.
///
/// **Mutação que deve sangrar:** `FLAT_PROBE` em qualquer valor que não 128.
#[test]
fn the_gain_is_the_identity_away_from_the_relief() {
    let t = ridge(255, true, [0.1, 0.2, 0.3]);
    let gain = t.relief_shade_gain().expect("ha relevo, logo ha ganho");
    // O canto superior esquerdo: o traço desce pela coluna 30 com raio 10, então (0,0) nunca foi
    // tocado. Um ganho diferente de `FLAT_PROBE` ali significa que a superfície plana deixou de
    // devolver a si mesma.
    assert_eq!(
        gain[0],
        super::impasto_gain::FLAT_PROBE,
        "longe do relevo o ganho tem de ser exatamente a resposta plana"
    );
    // E que o ganho de fato VARIA onde o relevo está — senão o gate acima é verdade por vácuo.
    let (min, max) = gain
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    assert!(
        max > min,
        "o ganho e constante ({min}..{max}) — nao ha relevo nesta fixture"
    );
}

/// **O ganho resolve os DOIS lados do relevo** — o flanco que escurece e a crista que brilha.
///
/// ⚠️ Este gate existe porque o irmão acima **não podia falhar**: ele compara o produto contra a
/// própria constante que o produto usa, então `FLAT_PROBE = 127` sobreviveu a tudo. A identidade é
/// grátis (os pesos Rec.601 somam 256, logo *qualquer* cinza lumina para si mesmo); o que a escolha
/// do valor de fato compra é **faixa**, e é a faixa que se afirma.
///
/// Medido na crista de teste: `45..206` contra um plano de 128 — 0,35× de um lado, 1,61× do outro.
///
/// **Mutação que deve sangrar:** `FLAT_PROBE` num albedo alto (250), onde o especular da crista bate
/// no teto de 255 e o lado claro colapsa para ~1,02×.
#[test]
fn the_gain_resolves_both_sides_of_the_relief() {
    let t = ridge(255, true, [0.1, 0.2, 0.3]);
    let gain = t.relief_shade_gain().expect("ha relevo, logo ha ganho");
    let flat = f32::from(super::impasto_gain::FLAT_PROBE);
    let (min, max) = gain
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    let (dark, bright) = (f32::from(min) / flat, f32::from(max) / flat);
    assert!(
        dark < 0.7,
        "o flanco escurece so ate {dark:.2}x — o albedo da sonda quantizou o lado escuro"
    );
    assert!(
        bright > 1.3,
        "a crista brilha so ate {bright:.2}x — o albedo da sonda bateu no teto de 255 e o especular \
         do relevo foi ceifado"
    );
}

/// **A silhueta é COR-INDEPENDENTE, e é por isso que ela continua sendo cobertura.**
///
/// Trocar a captura do documento ativo pela LUMINÂNCIA — o que as outras portas do slot Shape leem
/// (o file-load e o sprite chapado) — teria "resolvido" o report e transformado todo desenho a tinta
/// PRETA sobre transparência num carimbo invisível. O ganho multiplica o que já está lá; ele não
/// decide o que está lá.
///
/// **Mutação que deve sangrar:** a silhueta passar a ser a luminância do composite.
/// ⚠️ **A primeira versão deste gate tomava o MÁXIMO sobre a tela e passava sob a mutação.** O papel
/// transparente tem alpha 0 e RGB 255, então a luminância dele é 255 — o máximo media o PAPEL, não a
/// pincelada. Um oráculo global sobre uma tela cujo fundo domina não fala sobre o traço; a asserção é
/// no texel onde a tinta está.
#[test]
fn a_black_drawing_still_prints() {
    let sil = silhouette(&mut ridge(0, true, [0.0, 0.0, 0.0]));
    let at_stroke = sil[30 * 60 + 30]; // o centro do traço, que desce pela coluna 30
    assert!(
        at_stroke > 200,
        "uma pincelada PRETA sobre tela transparente imprime {at_stroke} no proprio traco — a \
         silhueta virou luminancia e o carimbo ficou invisivel"
    );
}

/// Sonda: imprime o retrato inteiro (as quatro rotas contra o que o artista vê).
#[test]
#[ignore = "sonda de medicao; roda com -- --ignored --nocapture"]
fn measure_what_the_use_as_paths_read() {
    let mut t = ridge(255, true, [0.1, 0.2, 0.3]);
    let (seen, w, h) = t.take_preview_arc().expect("preview");
    let seen_lum = lum(&seen);
    println!("canvas {w}x{h}   relevo visivel = {}", t.impasto_visible());
    let (grain, _, _) = t.composite_to_lum().expect("composite");
    println!(
        "GRAIN/PAPER/GRANUL. vs o que se VE : {:?}",
        spread(&grain, &seen_lum)
    );
    let sil = silhouette(&mut t);
    let (min, max) = sil
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    println!("SHAPE  faixa de valores: {min}..{max}");
    let (baked, _, _) = ridge(255, true, [0.1, 0.2, 0.3]).run_full();
    println!(
        "BAKE   vs GRAIN                    : {:?}",
        spread(&lum(&baked), &grain)
    );
    let g = ridge(255, true, [0.1, 0.2, 0.3])
        .relief_shade_gain()
        .expect("ganho");
    let (gmin, gmax) = g
        .iter()
        .fold((255u8, 0u8), |(a, b), &v| (a.min(v), b.max(v)));
    println!(
        "GANHO  faixa: {gmin}..{gmax}   (plano = {})",
        super::impasto_gain::FLAT_PROBE
    );
}
