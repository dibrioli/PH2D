//! Os gates do lote em bandas — a identidade primeiro, o relógio depois.
//!
//! ⚠️ **O oráculo é o PRODUTO, não uma segunda implementação:** com o piso em `usize::MAX` a própria
//! `stamp_plain_dabs_banded_with` cai no laço `for d in dabs` que o `stamp_dabs_per_pixel` rodava antes
//! desta wave, chamando o mesmo kernel com os mesmos argumentos. As duas rotas comparadas aqui são,
//! literalmente, *a de antes* e *a de agora*.

use super::stamp_banded::{BATCH_MIN_AREA, batch_work, stamp_plain_dabs_banded_with, wants_bands};
use ph2d_painter_brush::{BrushSpec, Dab};

pub(super) const W: u32 = 512;
pub(super) const H: u32 = 512;

/// As caixas do lote pela porta do produto, com o pincel da fixture.
pub(super) fn caixas_de(dabs: &[Dab]) -> Vec<Option<ph2d_painter_brush::DirtyRect>> {
    super::stamp_banded::caixas(dabs, &brush(), W, H)
}

pub(super) fn canvas() -> Vec<u8> {
    vec![255u8; (W as usize) * (H as usize) * 4]
}

pub(super) fn brush() -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        color: [0.1, 0.2, 0.8],
        ..BrushSpec::default()
    }
}

/// Um arco de `n` dabs — a forma que um editor de figura re-carimba, com sobreposição de verdade
/// (espaçamento bem menor que o diâmetro) para que a ORDEM entre dabs seja observável.
#[allow(clippy::cast_precision_loss)]
pub(super) fn arc(n: usize, radius: f32) -> Vec<Dab> {
    (0..n)
        .map(|i| {
            let t = (i as f32) / (n as f32) * std::f32::consts::TAU;
            let (s, c) = (t.sin(), t.cos());
            Dab {
                center: [256.0 + c * radius, 256.0 + s * radius],
                radius_px: 12.0,
                coverage: 0.6,
                // A cor varia por dab: trocar a ordem de composição muda a cor da sobreposição.
                color: [(i % 7) as f32 / 7.0, 0.3, 0.9],
                color_shift: [0.0; 3],
                rotation: [1.0, 0.0],
                dir: [c, s],
                arc_len: 0.0,
                stroke_radius_px: 12.0,
                step: [0.0, 0.0],
                ante: None,
                tilt: None,
            }
        })
        .collect()
}

/// Todo pixel de `buf` que difere de `pristine` cabe em `r`? — e quantos mudaram.
fn covers_every_change(
    r: Option<ph2d_painter_brush::DirtyRect>,
    buf: &[u8],
    pristine: &[u8],
) -> (bool, usize) {
    let mut ok = true;
    let mut n = 0usize;
    for (i, (a, b)) in buf
        .as_chunks::<4>()
        .0
        .iter()
        .zip(pristine.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if a == b {
            continue;
        }
        n += 1;
        let (x, y) = ((i as u32) % W, (i as u32) / W);
        let inside = r.is_some_and(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h);
        ok &= inside;
    }
    (ok, n)
}

/// **A wave inteira se apoia nisto:** dividir as linhas entre os núcleos não pode mover um byte.
///
/// Bandas são linhas disjuntas e cada uma percorre TODOS os dabs na ordem da lista, então um pixel é
/// composto pelos mesmos dabs, na mesma ordem — muda quem AVALIA a linha, nunca o que ela diz.
///
/// ⚠️ **O retângulo NÃO é comparado por igualdade, e a primeira versão deste gate estava errada nisso.**
/// O laço serial devolve o *span* de todo dab que tocou alguma coisa; a rota em banda devolve só as
/// linhas em que de fato escreveu, então ela é **mais APERTADA** — as linhas do aro, onde o falloff já
/// zerou, entram no span e não são escritas. Apertado é melhor (menos upload, janela de undo menor) e
/// continua correto, então a propriedade honesta não é *"os dois retângulos são o mesmo"* e sim
/// **"o retângulo cobre tudo que mudou, e não inventa área"** — que é o que os dois consumidores
/// (`declare_wrote` e `mark_dirty`) de fato pedem.
#[test]
fn the_banded_batch_paints_exactly_what_the_serial_loop_painted() {
    for n in [2usize, 17, 200] {
        for radius in [40.0f32, 160.0] {
            identity_of(&arc(n, radius), &format!("denso n={n} r={radius}"));
        }
    }
}

/// A metade que compara as duas rotas, para o laço acima varrer denso E esparso.
pub(super) fn identity_of(dabs: &[Dab], case: &str) {
    let pristine = canvas();
    let mut serial = canvas();
    let mut banded = canvas();
    let rs =
        stamp_plain_dabs_banded_with(&mut serial, W, H, dabs, &brush(), false, None, usize::MAX);
    let rb = stamp_plain_dabs_banded_with(&mut banded, W, H, dabs, &brush(), false, None, 0);
    // A metade que importa: os PIXELS.
    let diff = serial.iter().zip(&banded).filter(|(a, b)| a != b).count();
    assert_eq!(diff, 0, "{diff} bytes divergem ({case})");
    // E o retângulo cobre tudo que mudou — nas DUAS rotas.
    let (ok_s, changed) = covers_every_change(rs, &serial, &pristine);
    let (ok_b, _) = covers_every_change(rb, &banded, &pristine);
    assert!(changed > 0, "a fixture não pintou nada ({case})");
    assert!(ok_s, "o retângulo serial não cobre tudo ({case})");
    assert!(ok_b, "o retângulo da banda não cobre tudo ({case})");
    // …e não inventa área: o da banda cabe dentro do serial.
    if let (Some(a), Some(b)) = (rb, rs) {
        assert!(
            a.x >= b.x && a.y >= b.y && a.x + a.w <= b.x + b.w && a.y + a.h <= b.y + b.h,
            "o retângulo da banda escapa do serial ({case}): {a:?} vs {b:?}"
        );
    }
}

/// **O alpha-lock lê o pixel de baixo**, então ele é a fixture que separa "cada banda escreve o seu"
/// de "cada banda LÊ o seu" — um kernel que espiasse o vizinho quebraria aqui e não no gate acima.
#[test]
fn the_banded_batch_is_identical_under_alpha_lock_too() {
    let dabs = arc(120, 130.0);
    let mut serial = canvas();
    let mut banded = canvas();
    // Alpha variado por linha: o `preserve_alpha` multiplica pelo alpha ANTERIOR do pixel.
    for (i, px) in serial.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        px[3] = u8::try_from((i / W as usize) % 256).unwrap_or(255);
    }
    banded.copy_from_slice(&serial);
    let rs =
        stamp_plain_dabs_banded_with(&mut serial, W, H, &dabs, &brush(), true, None, usize::MAX);
    let rb = stamp_plain_dabs_banded_with(&mut banded, W, H, &dabs, &brush(), true, None, 0);
    assert!(
        rs.is_some() && rb.is_some(),
        "as duas rotas têm de pintar sob alpha-lock"
    );
    assert_eq!(
        serial.iter().zip(&banded).filter(|(a, b)| a != b).count(),
        0,
        "bytes divergem sob alpha-lock"
    );
}

/// **O piso protege o lote que não enche duas bandas** — e a frase que estava aqui antes era outra.
///
/// ⚠️ **Este gate dizia *"o piso protege o traço à mão livre, que é a razão de o piso existir"*, e a
/// medição de 2026-08-04 derrubou a premissa.** Ela nunca tinha sido medida: um lote de mão livre de 6
/// dabs (≈10 k visitas) **paga** a divisão, `1,5× a 2×` pela tabela (B) da
/// `measure_route_cost::what_the_banded_batch_buys_when_the_cap_is_on`. O que perdia não era o lote
/// pequeno — era abrir **32 threads** para ele, e isso deixou de acontecer quando a contagem de bandas
/// passou a sair do TRABALHO ([`ph2d_painter_brush::band_count`]).
///
/// O que o piso protege, medido, é o lote que não enche **duas** bandas: abaixo de
/// `SPAWN_EQUIV_VISITS × 4` a divisão é `1,00×` (raio 20 da tabela (C)) e não há o que colher.
///
/// A régua segue sendo a SOMA DAS PEGADAS (o trabalho real), não a caixa da união — os dabs de um traço
/// se sobrepõem fortemente, e a caixa mentiria para baixo.
#[test]
fn a_freehand_sized_batch_stays_serial_and_a_figure_sized_one_does_not() {
    // ⚠️ Pergunta ao PRODUTO (`wants_bands`), não à aritmética do teste: a 1ª versão deste gate
    // recomputava a regra por conta própria e teria ficado verde com o produto decidindo outra coisa.
    assert!(
        !wants_bands(
            batch_work(&caixas_de(&arc(2, 3.0))),
            &caixas_de(&arc(2, 3.0)),
            BATCH_MIN_AREA
        ),
        "um lote que não enche duas bandas tem de ficar SERIAL"
    );
    assert!(
        !wants_bands(
            batch_work(&caixas_de(&arc(1, 20.0))),
            &caixas_de(&arc(1, 20.0)),
            BATCH_MIN_AREA
        ),
        "um dab só nunca vale uma divisão"
    );
    // ⚠️ E a metade que a medição ACRESCENTOU: o lote de mão livre, que este gate pinava como serial,
    // agora TEM de dividir — sem ela, restaurar o piso antigo passaria aqui em silêncio.
    assert!(
        wants_bands(
            batch_work(&caixas_de(&arc(6, 20.0))),
            &caixas_de(&arc(6, 20.0)),
            BATCH_MIN_AREA
        ),
        "um lote de mão livre de 6 dabs paga a divisão (medido 1,5-2,0x) e tem de DIVIDIR"
    );
    assert!(
        wants_bands(
            batch_work(&caixas_de(&arc(525, 200.0))),
            &caixas_de(&arc(525, 200.0)),
            BATCH_MIN_AREA
        ),
        "a figura do report tem de DIVIDIR"
    );
    // E a régua é a soma das pegadas, não a caixa: esta figura tem caixa PEQUENA e trabalho GRANDE.
    // ⚠️ **A caixa é MEDIDA pela porta do produto, não escrita como literal.** A versão anterior
    // cravava `62 × 62 × 4` como *"a ordem da caixa desta figura"*, e quando o piso desceu de 131 072
    // para 3 232 o literal passou a ficar ACIMA dele — a asserção virou falsa sem que a propriedade
    // que ela afirma tivesse mudado. Perguntar ao `batch_bounds` mantém as duas metades honestas.
    let tight = arc(400, 14.0);
    let bbox = super::stamp_banded::batch_bounds(&caixas_de(&tight))
        .map_or(0, |b| (b.w as usize) * (b.h as usize));
    assert!(
        bbox < BATCH_MIN_AREA,
        "a fixture precisa de caixa ABAIXO do piso para separar as duas réguas \
         ({bbox} vs {BATCH_MIN_AREA})"
    );
    assert!(
        wants_bands(
            batch_work(&caixas_de(&tight)),
            &caixas_de(&tight),
            BATCH_MIN_AREA
        ),
        "uma figura de caixa pequena e muito trabalho tem de DIVIDIR"
    );
}

/// **A consequência**: o lote dividido é materialmente mais rápido que o serial.
///
/// ⚠️ É uma RAZÃO entre as duas rotas medidas **costas-com-costas no mesmo processo e sobre o mesmo
/// estado**, e não um bar de relógio: a máquina desta linha é compartilhada, e ao longo de uma sessão
/// o MESMO trabalho já variou 2× sem uma linha mudar (doc 28 §5.46). Uma razão torna a carga um fator
/// comum. A barra é folgada de propósito — o que ela vigia é *a divisão acontecer*, e o número honesto
/// medido é ~10×.
///
/// ⚠️ **`#[ignore]`, e não por preguiça:** ele mede PARALELISMO, e a suíte roda os testes em paralelo —
/// as outras threads disputam os mesmos 32 núcleos e a razão desaba. Reprovou exatamente assim na 1ª
/// corrida da suíte cheia, verde isolado. Rodar:
/// `cargo test -p ph2d-tool-painter --profile smoke the_banded_batch_is_materially -- --ignored --test-threads=1`
#[test]
#[ignore = "mede PARALELISMO: precisa da máquina; --ignored --test-threads=1"]
fn the_banded_batch_is_materially_faster_than_the_serial_one() {
    // ⚠️ Raio 200 num canvas de 512: a figura do report tem raio 400, mas numa tela de 512 ela cairia
    // quase toda FORA — a fixture mediria o recorte, não o trabalho.
    let dabs = arc(525, 200.0);
    let b = brush();
    let mut buf = canvas();
    // Aquece as duas rotas antes de cronometrar (first-touch da tela).
    let _ = stamp_plain_dabs_banded_with(&mut buf, W, H, &dabs, &b, false, None, usize::MAX);
    let _ = stamp_plain_dabs_banded_with(&mut buf, W, H, &dabs, &b, false, None, 0);
    let mut ser = f64::MAX;
    let mut par = f64::MAX;
    for _ in 0..5 {
        let mut a = canvas();
        let t0 = std::time::Instant::now();
        let _ = stamp_plain_dabs_banded_with(&mut a, W, H, &dabs, &b, false, None, usize::MAX);
        ser = ser.min(t0.elapsed().as_secs_f64());
        let mut c = canvas();
        let t0 = std::time::Instant::now();
        let _ = stamp_plain_dabs_banded_with(&mut c, W, H, &dabs, &b, false, None, 0);
        par = par.min(t0.elapsed().as_secs_f64());
    }
    let ratio = ser / par.max(1e-12);
    assert!(
        ratio > 2.0,
        "o lote dividido tem de ser materialmente mais rápido: {ratio:.2}x (serial {:.3} ms, banda {:.3} ms)",
        ser * 1e3,
        par * 1e3
    );
}

fn cpx(
    pos: [f32; 2],
    phase: ph2d_editor_core::tool::PointerPhase,
) -> ph2d_editor_core::tool::CanvasPointer {
    ph2d_editor_core::tool::CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}
use ph2d_editor_core::tool::{CanvasPaintTool as _, PointerPhase, RasterEditTool as _};

/// **O pincel do ARTISTA toma a rota nova?** — a metade do report de 2026-08-03 (*"não houve
/// melhora real nem no modo digital simples"*) que **não precisa de relógio**.
///
/// A pergunta admitia duas leituras opostas — *o ramo em banda não dispara neste pincel* contra
/// *dispara e o tempo está noutro lugar* — e eu pedi um log ao Enio para separá-las. Metade dela é
/// **estrutural**: qual rota o despacho toma é função da configuração, não da máquina, e responde-se
/// em processo. O `band_diag` é por-thread (ver o doc dele), então este gate é imune ao paralelismo
/// da suíte — poluição aqui produziria **falso VERDE**, porque a afirmação é positiva.
///
/// A fixture é a do smoke: **Digital**, pincel de fábrica em `set_brush_size_px(40)` (o que o
/// `PH2D_IMPASTO_SMOKE` arma), elipse grande, entregue pela porta do artista.
#[test]
fn the_artists_default_brush_takes_the_banded_road_on_a_live_figure() {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    assert_eq!(
        t.paint_media(),
        crate::tool::paint::media::PaintMedia::Digital,
        "a fixture tem de ser o Digital de fábrica — é o meio do report"
    );
    t.set_brush_size_px(40.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;

    let _ = super::stamp_banded::diag::take(); // zera o que esta thread trouxe
    t.on_canvas_pointer(cpx([100.0, 412.0], PointerPhase::Down));
    t.on_canvas_pointer(cpx([900.0, 612.0], PointerPhase::Move));
    // ⚠️ **O carimbo agora acontece em REPOUSO** (`super::shape_draft`): sob a mão o gizmo é o
    // preview e nenhum lote é produzido, então a fixture tem de SOLTAR — senão ela mede o silêncio.
    t.on_canvas_pointer(cpx([900.0, 612.0], PointerPhase::Up));
    let d = super::stamp_banded::diag::take();
    let (banded, serial, dabs) = (d.banded, d.serial, d.dabs);

    // ⚠️ **O controle positivo NÃO passa pelo instrumento sob teste.** A 1ª versão perguntava
    // `dabs > 0` para provar *"a fixture pintou"*, e sob a mutação (o ramo desligado) ela falhava
    // dizendo **"a fixture não carimbou nada"** — falso: a fixture carimbou, pela rota antiga, e
    // `dabs` só conta quem chega a ESTE módulo. Um contador que zera nas duas hipóteses não as
    // separa. Quem prova que a fixture contém o fenômeno é o CANVAS.
    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "a fixture não pintou um pixel — ela não contém o fenômeno"
    );
    assert!(
        banded > 0,
        "o pincel DEFAULT do Digital não alcança o depósito em banda: {banded} em banda x \
         {serial} serial(is), {dabs} dabs no módulo. A wave está INERTE no produto."
    );

    // E o CONTROLE pela mesma porta: uma figura pequena com pincel pequeno segue serial — sem esta
    // metade, uma mutação que ligasse a banda SEMPRE passaria por aqui.
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
    t.set_brush_size_px(3.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    let _ = super::stamp_banded::diag::take();
    // ⚠️ **A elipse encolheu em 2026-08-04, e o gate mandou.** Com o piso medido (3 232 visitas) a
    // figura de 40 × 20 px passou a PAGAR a divisão — o controle tem de descer até um lote que
    // genuinamente não enche duas bandas, senão ele afirma o lado errado da cerca.
    t.on_canvas_pointer(cpx([100.0, 118.0], PointerPhase::Down));
    t.on_canvas_pointer(cpx([102.0, 119.0], PointerPhase::Move));
    // ⚠️ O carimbo acontece em REPOUSO (`super::shape_draft`) — a fixture tem de SOLTAR.
    t.on_canvas_pointer(cpx([102.0, 119.0], PointerPhase::Up));
    let d = super::stamp_banded::diag::take();
    let (banded, serial, dabs) = (d.banded, d.serial, d.dabs);
    assert!(dabs > 0, "o controle não carimbou nada");
    // ⚠️ **O controle DECLARA a premissa que assume**, e é isso que o faz falhar alto quando alguém
    // move o piso em vez de continuar verde medindo o outro lado da cerca: a versão anterior tinha
    // 4 290 visitas contra um piso que desceu para 3 232. *Um controle de piso que não afirma de que
    // lado do piso ele está não é um controle.*
    assert!(
        (d.visits as usize) < BATCH_MIN_AREA,
        "o controle TEM de ficar abaixo do piso ({} visitas vs {BATCH_MIN_AREA})",
        d.visits
    );
    assert_eq!(
        banded, 0,
        "uma figura pequena não paga o custo de dividir: {banded} em banda x {serial} serial(is), \
         {dabs} dabs / {} visitas contra o piso de {BATCH_MIN_AREA}",
        d.visits
    );
}
/// **O modo de pintura SOBREVIVE a um stroke vivo?** — o report do Enio de 2026-08-03
/// (*"Wet Paint regride para digital ao usar os strokes vivos"*).
#[test]
fn the_paint_media_survives_every_live_shape_method() {
    use ph2d_painter_brush::StrokeMethod;
    for method in [
        StrokeMethod::Line,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::Arc,
        StrokeMethod::FreeHand,
    ] {
        let mut t = crate::tool::PainterTool::default();
        t.set_source(vec![255u8; 256 * 256 * 4], 256, 256);
        t.set_paint_media(crate::tool::paint::media::PaintMedia::WetPaint);
        assert_eq!(
            t.paint_media(),
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio nem chegou a armar"
        );
        t.paint.brush.stroke_method = method;
        t.on_canvas_pointer(cpx([80.0, 128.0], PointerPhase::Down));
        t.on_canvas_pointer(cpx([170.0, 128.0], PointerPhase::Move));
        // ⚠️ **O carimbo agora acontece em REPOUSO** (`super::shape_draft`): sob a mão o gizmo é o
        // preview e nenhum lote é produzido, então a fixture tem de SOLTAR — senão ela mede o silêncio.
        t.on_canvas_pointer(cpx([170.0, 128.0], PointerPhase::Up));
        let after_move = t.paint_media();
        t.on_canvas_pointer(cpx([170.0, 128.0], PointerPhase::Up));
        let after_up = t.paint_media();
        assert_eq!(
            after_move,
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio REGREDIU durante o arrasto -> {after_move:?}"
        );
        assert_eq!(
            after_up,
            crate::tool::paint::media::PaintMedia::WetPaint,
            "{method:?}: o meio REGREDIU no pen-up -> {after_up:?}"
        );
    }
}

/// **A razão do log divide o próprio tempo pelo próprio trabalho.**
///
/// ⚠️ O `ns/visita` do `[frame]` nasceu dividindo os µs das quatro fases do **RE-STAMP** pelas visitas
/// do **DEPÓSITO** — dois eventos diferentes. Numa sessão de mão livre o re-stamp não roda, então o
/// numerador é zero por construção e o log de 2026-08-04 imprimiu `0.0 ns/visita` **ao lado de 99 M
/// visitas**: um zero que se lê como *"o carimbo é de graça"*, exatamente a falha que o
/// [`crate::wet_diag`] documenta — um instrumento mudo TRANQUILIZA.
///
/// A fixture é a que continha o fenômeno: mão livre (`Space`, o método de fábrica), que carimba muito
/// e **não re-carimba nada**. É por isso que ela pede `deliveries == 0` — sem essa metade, um
/// numerador emprestado do re-stamp passaria despercebido.
#[test]
fn the_deposit_ratio_divides_its_own_time_by_its_own_work() {
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_brush_size_px(80.0);

    let _ = super::stamp_banded::diag::take(); // zera o que esta thread trouxe
    t.on_canvas_pointer(cpx([120.0, 500.0], PointerPhase::Down));
    for x in 1..=8u8 {
        let px = 120.0 + f32::from(x) * 100.0;
        t.on_canvas_pointer(cpx([px, 500.0], PointerPhase::Move));
        // ⚠️ **O carimbo agora acontece em REPOUSO** (`super::shape_draft`): sob a mão o gizmo é o
        // preview e nenhum lote é produzido, então a fixture tem de SOLTAR — senão ela mede o silêncio.
        t.on_canvas_pointer(cpx([px, 500.0], PointerPhase::Up));
    }
    let d = super::stamp_banded::diag::take();

    // O CONTROLE não passa pelo instrumento sob teste: quem prova que a fixture pintou é o CANVAS.
    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "a fixture não pintou um pixel — ela não contém o fenômeno"
    );
    assert_eq!(
        d.deliveries, 0,
        "a fixture tem de ser mão livre PURA: com re-stamp no meio, um numerador emprestado dele \
         ficaria indistinguível do numerador próprio"
    );
    assert!(
        d.visits > 0,
        "o depósito não contou trabalho nenhum: {} dabs, {} visitas",
        d.dabs,
        d.visits
    );
    assert!(
        d.cpu_us > 0,
        "o depósito da CPU contou {} visitas e ZERO tempo — a razão do log volta a ser 0.0 \
         ns/visita sobre trabalho real",
        d.visits
    );
}

/// **O que o DISPOSITIVO carimba entra na conta.** — a outra metade do mesmo defeito.
///
/// Quando o device aceita o lote, `stamp_plain_dabs_banded` nem é chamado, então até 2026-08-04 os
/// `dabs` e as `visitas` do log descreviam **só a metade que ficou na CPU** — com a linha dizendo
/// `775 dabs` para um traço que carimbou mais. O contador é exercitado direto porque a ROTA precisa de
/// um dispositivo real (`#[ignore]`, na suíte de GPU): o que este gate prova é que o balde existe e
/// soma, não que a rota disparou.
#[test]
fn what_the_device_stamps_is_counted_too() {
    let _ = super::stamp_banded::diag::take();
    super::stamp_banded::diag::note_device(7, 4096, 250);
    super::stamp_banded::diag::note_device(3, 1024, 50);
    let d = super::stamp_banded::diag::take();
    assert_eq!(
        (d.device, d.dev_dabs, d.dev_visits, d.dev_us),
        (2, 10, 5120, 300)
    );
    assert_eq!(
        (d.dabs, d.visits, d.cpu_us),
        (0, 0, 0),
        "o balde do device vazou para o da CPU — as duas razões deixam de ser comparáveis"
    );
}

#[path = "stamp_banded_capped_tests.rs"]
mod capped; // o cap de accumulate atravessa a divisão
