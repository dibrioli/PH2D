//! Gates do **RELEVO DO DEPÓSITO** — o pigmento que a silhueta pousa, visto como relevo (Enio
//! 2026-08-10).
//!
//! A lei em uma frase: **o pigmento que o pincel pousa tem espessura própria, ela é da SILHUETA (não
//! do impasto), e ela não engrossa quando o pincel cresce.**
//!
//! Cada uma das três metades pode falhar sozinha, e cada uma tem gate próprio — mais o gate que a
//! medição obrigou a escrever, o [`the_film_does_not_touch_the_pigment`]: ligar o filme mudou a
//! SILHUETA da tinta na primeira versão, e o número (61 níveis) não tinha nada a ver com relevo.
use super::*;
use crate::Region;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::TextureKind;

const N: u32 = 96;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma tela BRANCA opaca com um pincel de cerdas — o Digital, e o documento do smoke.
///
/// ⚠️ **A Shape é parte da FIXTURE, não decoração:** um pincel redondo macio deposita um domo cuja
/// inclinação a luz mal vê (medido: *pior 0,21* nível contra *14,46* com listras), então um gate do
/// filme escrito sobre um pincel liso mediria zero e passaria por vácuo. O pedido do Enio diz
/// literalmente *"a deposição do pigmento com Shape"*.
fn bristle(size: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_brush_size_px(size);
    t.set_brush_color_srgb8([200, 30, 30]);
    t.set_brush_shape_kind(TextureKind::Stripes as u8);
    t
}

fn drag(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Down));
    let mut x = 24.0f32;
    while x < 72.0 {
        x += 1.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([72.0, 48.0], PointerPhase::Up));
}

fn lit(t: &PainterTool) -> Vec<u8> {
    let mut rgba = t.canvas_rgba.as_ref().clone();
    t.apply_impasto_light(
        &mut rgba,
        Region {
            x: 0,
            y: 0,
            w: N,
            h: N,
        },
    );
    rgba
}

/// O que o RELEVO acrescentou, em níveis de luminância: o pior texel e a média, dentro do traço.
///
/// ⚠️ **A DIFERENÇA, nunca a luminância absoluta** — um traço com Shape listrada já tem 53,80 de
/// excursão de PIGMENTO, e medir isso mediria o desenho (a lição do `emboss_probe`).
fn relief_delta(with: &[u8], without: &[u8]) -> (f64, f64) {
    let lum = |px: &[u8], i: usize| {
        0.2126 * f64::from(px[i]) + 0.7152 * f64::from(px[i + 1]) + 0.0722 * f64::from(px[i + 2])
    };
    let d: Vec<f64> = (24..72)
        .map(|x| {
            let i = ((48 * N + x) * 4) as usize;
            (lum(with, i) - lum(without, i)).abs()
        })
        .collect();
    (
        d.iter().cloned().fold(0.0, f64::max),
        d.iter().sum::<f64>() / d.len() as f64,
    )
}

/// Pinta o MESMO traço com e sem o filme, e devolve o que o filme acrescentou.
fn film(size: f32, paint: f32) -> (f64, f64) {
    let mut with = bristle(size);
    with.set_shape_relief(paint);
    drag(&mut with);
    let mut without = bristle(size);
    drag(&mut without);
    relief_delta(&lit(&with), &lit(&without))
}

/// ⚠️ **O NEUTRO É BYTE-IDÊNTICO** — `Relief = 0` é o default, e todo documento que ninguém tocou tem
/// de sair exatamente como saía. Não "quase": ao BYTE, e no PIXEL, não só na luz.
#[test]
fn the_film_is_off_by_default_and_off_is_byte_identical() {
    let mut t = bristle(20.0);
    assert_eq!(t.shape_relief(), 0.0, "o default tem de ser DESLIGADO");
    drag(&mut t);
    let baseline = lit(&t);

    let mut t2 = bristle(20.0);
    t2.set_shape_relief(0.0); // o gesto explícito de desligar tem de ser igualmente inerte
    drag(&mut t2);
    assert_eq!(lit(&t2), baseline, "Relief 0 nao pode mover um byte");
}

/// ⚠️ **A ENTREGA: o depósito de pigmento é visto como relevo** (Enio 2026-08-10, com a foto do Wet
/// Paint ao lado). Sem isto a wave nao existe.
#[test]
fn the_pigment_deposit_reads_as_relief() {
    let (worst, mean) = film(20.0, 1.0);
    assert!(
        worst >= 6.0 && mean >= 1.5,
        "o filme tem de se ver no traco; pior {worst:.2} media {mean:.2} niveis"
    );
}

/// ⚠️ **O SLIDER ORDENA** — um controle que não ordena não é um controle.
#[test]
fn a_thicker_film_reads_thicker() {
    let mut prev = 0.0f64;
    for step in [0.25f32, 0.5, 1.0] {
        let (worst, _) = film(20.0, step);
        assert!(
            worst > prev + 1.0,
            "Relief {step} tem de ler mais fundo que o anterior: {worst:.2} contra {prev:.2}"
        );
        prev = worst;
    }
}

/// ⚠️ **O FILME NÃO ENGROSSA COM O PINCEL** — e é a razão inteira de ele não ser um impasto fino.
///
/// O `derive_height` escala o corpo por `radius / IMPASTO_REFERENCE_RADIUS_PX` **de propósito** (um
/// domo de altura fixa sobre 60 px tem `n_z ≈ 1` e a luz o desenha chato). Um filme de pigmento não
/// tem essa propriedade: ele é a espessura da tinta, que não sabe o tamanho do pincel que a pousou.
///
/// **Mutação que sangra:** somar o filme ANTES do `size_scale` (`(depth + film) * a * size_scale`) —
/// medido, o mesmo número rende *pior 9,72* num raio de 10 e *66,92* num de 40, **7×**.
#[test]
fn the_film_does_not_thicken_with_the_brush() {
    let small = film(10.0, 1.0).0;
    let large = film(80.0, 1.0).0;
    let ratio = large / small.max(1e-6);
    assert!(
        (0.5..=2.0).contains(&ratio),
        "o filme escalou com o pincel: raio 5 -> {small:.2}, raio 40 -> {large:.2} ({ratio:.2}x)"
    );
}

/// ⚠️ **O FILME NÃO TOCA O PIGMENTO** — o gate que a medição obrigou a escrever.
///
/// A primeira versão pôs o filme dentro do [`BrushSpec::deposits_height`], que parecia o predicado
/// natural (*"este pincel escreve altura"*). Ele não é: **quatro sítios do caminho de COR o leem** para
/// cortar o pigmento na borda do corpo (`height::film_coverage`), então subir o Relief mudava a
/// SILHUETA da tinta — *pior 61 níveis* de diferença, insensível ao próprio slider, que é a assinatura
/// de estar medindo outra coisa.
///
/// Aqui a asserção é sobre o CANVAS, não sobre a luz: o filme é aparência, e o que ele não pode fazer
/// é mudar a tinta que o artista depositou.
///
/// **Mutação que sangra:** devolver a cláusula do filme ao `deposits_height`.
#[test]
fn the_film_does_not_touch_the_pigment() {
    let mut with = bristle(20.0);
    with.set_shape_relief(1.0);
    drag(&mut with);
    let mut without = bristle(20.0);
    drag(&mut without);
    let diff = with
        .canvas_rgba
        .iter()
        .zip(without.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diff, 0,
        "ligar o filme mudou o PIGMENTO em {diff} bytes — ele e aparencia, nao deposito"
    );
}

/// ⚠️ **O FILME ACENDE SEM PAPEL** — as duas metades do substrato são independentes.
///
/// Com Relief `0` não há dente nenhum, e o `substrate()` devolve `None`; o que faz a luz correr é o
/// próprio relevo que o filme depositou. Sem esta metade, o artista que quisesse relevo de tinta sobre
/// papel liso não veria nada.
#[test]
fn the_film_lights_with_no_paper_tooth_at_all() {
    let mut t = bristle(20.0);
    t.set_shape_relief(1.0);
    assert_eq!(t.substrate_depth(), 0.0, "a fixture nao pode ter dente");
    drag(&mut t);
    assert!(
        t.impasto_visible(),
        "o passe de luz tem de correr para um documento que so tem FILME"
    );
    let (worst, _) = film(20.0, 1.0);
    assert!(worst >= 6.0, "o filme sozinho tem de se ver: {worst:.2}");
}

/// ⚠️ **O SLIDER É VIVO NO ÚLTIMO TRAÇO — sob *Adjust Last Stroke*, e SÓ sob ele.**
///
/// ⚠️ **A minha expectativa estava errada e o código certo, e as duas metades ficam gateadas por isso.**
/// Eu escrevi este gate esperando o contrato de 2026-07-12 (*"todos os parâmetros vivos em tempo real
/// para ajustes depois do traço"*) e ele nasceu VERMELHO com `0` texels movidos — porque em 2026-07-19
/// o Enio inverteu o default: *tinta pronta fica pronta*, e o `Adjust Last Stroke` nasce DESMARCADO
/// (`b29cfabb`). O filme herda esse contrato porque é um knob de depósito como o Depth, e um knob de
/// depósito que fosse vivo sem o toggle seria a exceção que ninguém pediu.
///
/// **Mutação que sangra:** tirar o `refresh_live_relief()` do `set_shape_relief` (a metade viva) ·
/// tirar o gate `impasto_live_edit()` do `refresh_live_relief` (a metade quieta).
#[test]
fn the_paint_slider_is_live_on_the_last_stroke_only_under_adjust_last_stroke() {
    let moved_with = |live: bool| {
        let mut t = bristle(20.0);
        if live {
            t.toggle_impasto_live_edit();
        }
        t.set_shape_relief(0.25);
        drag(&mut t);
        let thin = lit(&t);
        t.set_shape_relief(1.0); // sem pintar de novo
        let thick = lit(&t);
        thin.as_chunks::<4>()
            .0
            .iter()
            .zip(thick.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a[0] != b[0])
            .count()
    };
    assert!(
        moved_with(true) > 50,
        "com Adjust Last Stroke, subir o Relief tem de re-derivar o ultimo traco; movidos: {}",
        moved_with(true)
    );
    assert_eq!(
        moved_with(false),
        0,
        "sem Adjust Last Stroke, tinta pronta fica pronta"
    );
}

/// ⚠️ **O CAMPO É O ENVELOPE DE CARGA, e a COBERTURA não serve** — pinado para ninguém reconstruir o
/// desenho que a medição rejeitou.
///
/// A primeira versão derivou o filme da cobertura que a luz já dobra (`ReliefFields::cover_at`), o que
/// teria deixado o slider vivo sobre a tela INTEIRA em vez de só no último traço — uma propriedade
/// melhor. A medição a matou: dentro de um traço a cobertura é um platô, e um gradiente sobre um platô
/// é zero. Este gate guarda o número.
#[test]
fn the_coverage_saturates_inside_a_stroke_and_this_is_its_number() {
    let mut t = bristle(20.0);
    t.set_shape_relief(1.0);
    drag(&mut t);
    let id = t.layers.active().expect("camada ativa");
    let cov = t.covers.get(&id).cloned().unwrap_or_default();
    let row: Vec<f64> = (30..66)
        .map(|x| f64::from(cov.get((48 * N + x) as usize).copied().unwrap_or(0)) / 255.0)
        .collect();
    let lo = row.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        lo >= 0.95,
        "a cobertura dentro do traco tem de SATURAR (o motivo de o filme nao sair dela); min {lo:.3}"
    );
}

// ── O MATERIAL do depósito: alto brilho ou fosco (Enio, 2026-08-10, no smoke do relevo) ────────────

/// ⚠️ **UMA PORTA, e é a que já existia.** A row **Shine** da seção Shape e a row Shine do card
/// **Material** editam o MESMO `BrushSpec::impasto_shine`, pelo MESMO `set_impasto_shine` — que faz o
/// fan-out pelos slots de relevo e re-assa o material do último traço. Duas VISTAS de um valor.
///
/// **Mutação que tem de sangrar:** dar ao Shine da Shape um campo próprio (ou escrever só no pincel
/// vivo, sem o fan-out) — o segundo braço deste gate morre.
#[test]
fn the_deposits_shine_is_the_paints_own_shine() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::PanelEvent;
    let mut t = bristle(20.0);
    // A porta do PRODUTO (o que o painel chama), não o roteador interno: se ninguém consumir o evento,
    // o campo não se move e o primeiro braço sangra.
    t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_SHAPE_SHINE, 0.31));
    assert!(
        (t.brush_settings().impasto_shine - 0.31).abs() < 1e-4,
        "a row da Shape tem de escrever o MESMO campo que o card Material; leu {}",
        t.brush_settings().impasto_shine
    );
    // O fan-out do `set_material_field`: sem ele, pegar a Faca devolveria o Shine ao valor do slot dela
    // e o material do depósito mudaria por um gesto que não fala de material nenhum.
    for slot in [PaintMode::Paint, PaintMode::Knife, PaintMode::Sculpt] {
        let v = t.paint.brush_by_mode[slot.slot()].impasto_shine;
        assert!(
            (v - 0.31).abs() < 1e-4,
            "o slot {slot:?} ficou fora do fan-out do material; leu {v}"
        );
    }
}

/// ⚠️ **O Shine MOVE o depósito e não move o papel nu — e as duas metades são o gate.**
///
/// A segunda é o motivo de a row existir só na Shape: o `⛔` do [`super::substrate_relief`] já mediu
/// que um realce sobre o dente do papel não move **um texel** (num relevo de ~1 px a normal quase não
/// sai do plano, e o realce tem a resposta PLANA subtraída). Se o filme sofresse do mesmo mecanismo, a
/// row seria knob morto — e a leitura barata dizia que sofreria, porque a espessura é a MESMA. O que
/// difere é a INCLINAÇÃO: o dente é uma onda larga, o depósito com cerdas cai de 1 px a zero em ~1 px.
///
/// **Mutação que tem de sangrar:** ignorar `mat.shine` no realce — o primeiro braço vai a zero.
#[test]
fn the_shine_moves_the_deposit_and_leaves_the_bare_paper_alone() {
    let deposit = |shine: f32| {
        let mut t = bristle(20.0);
        t.set_shape_relief(1.0);
        t.set_impasto_shine(shine);
        drag(&mut t);
        lit(&t)
    };
    let (worst, _) = relief_delta(&deposit(1.0), &deposit(0.0));
    assert!(
        worst > 1.5,
        "o Shine tem de mudar como o deposito le; pior {worst:.2} nivel"
    );

    // O CONTROLE — a mesma pergunta sobre o papel, que o ⛔ reprovou. Sem esta metade o gate acima
    // passaria por qualquer relevo e a row poderia migrar de volta para a seção Paper sem ninguém ver.
    let paper = |shine: f32| {
        let mut t = bristle(20.0);
        t.set_substrate_depth(1.0);
        t.set_substrate_roughness(0.5);
        t.set_impasto_shine(shine);
        lit(&t) // sem uma pincelada: é o papel NU
    };
    assert_eq!(
        paper(1.0),
        paper(0.0),
        "sobre o papel nu o Shine nao pode mover um byte (o ⛔ do substrate_relief)"
    );
}
