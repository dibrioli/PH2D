//! SONDA — **o PREÇO de um re-carimbo com a pilha do Composite** (report do dono, 2026-09-21:
//! *"Performance ruim"*).
//!
//! ⚠️ Cortada da irmã [`super::diag_composite_e_as_formas`] em 2026-09-21, quando as cinco sondas
//! do preço a levaram a `933` linhas contra o tecto de `700`. O corte é por ASSUNTO: lá mede-se o
//! que o re-carimbo DESENHA, aqui o que ele CUSTA.
//!
//! ⚠️⚠️ **Todas correm em `--release` e imprimem a CPU ociosa ao lado** — nenhuma leitura de
//! relógio desta máquina vale nada acima de `load ~5`, e o `loadavg` mente a decair.
//!
//! A ordem em que elas fecharam o diagnóstico, e que é a ordem de as ler:
//! **(15)** onde o custo está (`COMPOR` é `90`–`99 %`, e uma camada `Blur` custa `8×` uma
//! `Brush`) · **(16)** o que dentro do borrão custa (nem o núcleo nem o avental: `O(área)` puro) ·
//! **(17)** ⛔ a alocação está ILIBADA (`vec![[f32; 4]]` é `alloc_zeroed`, `0,00 ms`) ·
//! **(18)** ⛔ o cover por BLOCOS está REFUTADO, a família inteira · **(19)** o tecto de um borrão
//! paralelo, que é a largura de banda do soquete (`4,05×` a 8 fios).

use super::*;
// ⚠️ O ponteiro vem da irmã por uma PORTA e não por uma cópia: duas fixturas do mesmo gesto
// divergem no dia em que uma delas ganhar um campo.
use super::diag_composite_e_as_formas::cp2;
use ph2d_editor_core::tool::RasterEditTool;

/// (15) O RELÓGIO de um re-carimbo com a pilha — o report *«performance ruim»* de 2026-09-21.
/// ⚠️ Corre em `--release`; em debug o número é ~20× e não descreve o produto.
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_relogio_do_recarimbo -- --ignored
/// --nocapture`
#[test]
#[ignore = "sonda de relógio"]
fn diag_o_relogio_do_recarimbo() {
    use composite::CompositeOp::{Blur, Brush, Erase, Smear};
    for (boolean, lado) in [(false, 1024u32), (false, 2048), (true, 1024), (true, 2048)] {
        for (nome, camadas) in [
            ("sem pilha          ", &[][..]),
            ("1: Brush           ", &[(Brush, 1.0f32)][..]),
            ("2: Blur / Brush    ", &[(Blur, 1.0), (Brush, 1.0)][..]),
            ("2: Brush / Brush   ", &[(Brush, 1.0), (Brush, 1.0)][..]),
            (
                "7: a pilha CHEIA   ",
                &[
                    (Blur, 1.0),
                    (Smear, 1.0),
                    (Erase, 0.3),
                    (Erase, 0.3),
                    (Brush, 1.0),
                    (Brush, 1.0),
                    (Brush, 1.0),
                ][..],
            ),
        ] {
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; (lado * lado * 4) as usize], lado, lado);
            t.paint.brush.radius_px = 10.0;
            t.paint.brush.color = [0.75, 0.12, 0.12];
            t.paint.brush.space_attenuation = false;
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
            t.paint.composite_enabled = !camadas.is_empty();
            for (op, s) in camadas {
                t.acrescenta_camada(op.to_u8());
                let pos = t.composite_len() - 1;
                t.set_composite_layer_strength(pos, *s);
            }
            let c = [lado as f32 * 0.5, lado as f32 * 0.5];
            let r = lado as f32 * 0.35;
            if boolean {
                t.set_stroke_op_mode(1); // Add: a configuração da foto do dono
            }
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Move));
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
            if boolean {
                // a 2.ª circunferência, que cruza a 1.ª — é ela que arma o composite booleano
                let c2 = [c[0] + r * 0.8, c[1] + r * 0.6];
                t.on_canvas_pointer(cp2(c2, PointerPhase::Down));
                t.on_canvas_pointer(cp2([c2[0] + r, c2[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp2([c2[0] + r, c2[1]], PointerPhase::Up));
            }
            // Agora o gesto que o artista repete: agarrar e arrastar a figura. Cada pen-up é UM
            // re-carimbo da figura inteira pela pilha.
            // ⚠️ **As QUATRO FASES em vez de um relógio de parede:** elas descrevem o MESMO evento
            // e a máquina está partilhada com outra linha — *uma proporção medida dentro de um
            // evento sobrevive à carga; um número absoluto não*.
            let _ = super::super::stamp_banded::diag::take();
            let n = 8;
            for i in 0..n {
                let d = (i % 2) as f32 * 4.0 - 2.0;
                t.on_canvas_pointer(cp2(c, PointerPhase::Down));
                t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Move));
                t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Up));
            }
            let g = super::super::stamp_banded::diag::take();
            let (f, ev_p, area) = super::super::composite_acumulado::fases::take();
            let ev = f64::from(g.deliveries.max(1));
            let us = |v: u64| v as f64 / ev / 1000.0;
            let p = |i: usize| f[i] as f64 / ev / 1000.0;
            let media_area = if ev_p > 0 {
                area / ev_p as f64 * 100.0
            } else {
                0.0
            };
            eprintln!(
                "{}{lado}² | {nome} | CARIMBAR {:>7.2} = pre {:>5.2} + acumular {:>6.2} + COMPOR {:>7.2} + copias {:>5.2} | alvo {media_area:>5.1}% da tela",
                if boolean { "bool " } else { "     " },
                us(g.stamp_us),
                p(0),
                p(1),
                p(2),
                p(3),
            );
        }
        eprintln!();
    }
}

/// (16) **O QUE, DENTRO DO BORRÃO, CUSTA** — o discriminador do report *«performance ruim»*.
///
/// A medição (15) diz que uma camada `Blur` custa `8×` uma `Brush` e é `87 %` do COMPOR. Este
/// varre o **tamanho da camada**, que escala `k` linearmente e **mais nada** — o núcleo de caixa
/// é `O(1)` em `k` (três somas correntes) e o único termo que cresce com `k` é o **AVENTAL**
/// `(bw + 2k)(bh + 2k)`.
///
/// ⇒ *se o COMPOR ficar plano enquanto `k` anda `16×`, o custo não é o avental nem a convolução:
/// são os SETE buffers de `[f32; 4]` que a passagem aloca de fresco e o tráfego deles.*
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_que_custa_no_borrao -- --ignored
/// --nocapture`
#[test]
#[ignore = "sonda de relógio"]
fn diag_o_que_custa_no_borrao() {
    use composite::CompositeOp::{Blur, Brush};
    const LADO: u32 = 2048;
    for tam in [0.25f32, 0.5, 1.0, 2.0, 4.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (LADO * LADO * 4) as usize], LADO, LADO);
        t.paint.brush.radius_px = 10.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        t.acrescenta_camada(Blur.to_u8());
        t.set_composite_layer_strength(0, 1.0);
        t.set_composite_layer_size(0, tam);
        t.acrescenta_camada(Brush.to_u8());
        t.set_composite_layer_strength(1, 1.0);
        let c = [LADO as f32 * 0.5, LADO as f32 * 0.5];
        let r = LADO as f32 * 0.35;
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        let _ = super::super::stamp_banded::diag::take();
        let _ = super::super::composite_acumulado::fases::take();
        for i in 0..8 {
            let d = (i % 2) as f32 * 4.0 - 2.0;
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Move));
            t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Up));
        }
        let g = super::super::stamp_banded::diag::take();
        let (f, ev_p, area) = super::super::composite_acumulado::fases::take();
        let ev = f64::from(g.deliveries.max(1));
        let k = ph2d_painter_brush::kernel_radius(10.0 * tam)
            * super::super::composite_acumulado::passagens_do_borrao(t.paint.brush.spacing)
                as usize;
        let media_area = if ev_p > 0 {
            area / ev_p as f64 * 100.0
        } else {
            0.0
        };
        eprintln!(
            "tamanho {tam:>4} | k = {k:>4} | COMPOR {:>7.2} ms | alvo {media_area:>5.1}% da tela",
            f[2] as f64 / ev / 1000.0,
        );
    }
}

/// (17) **QUANTO DO BORRÃO É ALOCAR** — a sonda (16) diz que o custo é `O(área)` e cego a `k`; a
/// cadeia de caixa aloca **SETE** `Vec<[f32; 4]>` do tamanho da região (avental + 3 horizontais +
/// 3 verticais) ⇒ `~250 MB` de `vec![…]` zerado por composição a `2048²`.
///
/// Aqui mede-se, ao tamanho do PRODUTO: (a) a passagem inteira · (b) só as sete alocações.
/// `cargo test -p ph2d-tool-painter --release --lib diag_quanto_do_borrao_e_alocar -- --ignored
/// --nocapture`
#[test]
#[ignore = "sonda de relógio"]
fn diag_quanto_do_borrao_e_alocar() {
    const W: u32 = 2048;
    const LADO: usize = 1484; // a caixa que o produto compõe (54,1 % da tela)
    let k = 24usize;
    let n = LADO * LADO;
    let mut buf = vec![128u8; (W * W * 4) as usize];
    let peso = vec![200u8; n];
    let x0 = i64::from((W as usize - LADO) as u32 / 2);

    let mut melhor_tudo = f64::MAX;
    for _ in 0..5 {
        let mut b = buf.clone();
        let t = std::time::Instant::now();
        ph2d_painter_brush::blur_region_por_peso(
            &mut b,
            W,
            W,
            x0,
            x0,
            LADO,
            LADO,
            k,
            &peso,
            [false, false],
            ph2d_painter_brush::BlurKernel::Caixa,
        );
        melhor_tudo = melhor_tudo.min(t.elapsed().as_secs_f64() * 1000.0);
        buf = b;
    }

    let mut melhor_aloc = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let mut soma = 0.0f32;
        for _ in 0..7 {
            let v = vec![[0f32; 4]; n];
            soma += v[n / 2][0]; // impede que o alocador seja optimizado para fora
        }
        melhor_aloc = melhor_aloc.min(t.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(soma, 0.0);
    }

    eprintln!(
        "borrão inteiro {melhor_tudo:>7.2} ms  |  só as 7 alocações {melhor_aloc:>7.2} ms  \
         ({:.0} % do total)  |  {:.1} ns/px",
        melhor_aloc / melhor_tudo * 100.0,
        melhor_tudo * 1e6 / n as f64,
    );
}

/// (18) **O TECTO DE UM COVER QUE SEGUE A FIGURA** — a sonda (17) diz que o borrão é `O(área)`
/// puro (`31,1 ns/px`, e alocar é GRÁTIS: `vec![[f32; 4]]` é `alloc_zeroed` e as páginas chegam
/// preguiçosas). ⇒ *a única alavanca é a ÁREA COMPOSTA*, e um anel ocupa `~5 %` da caixa dele.
///
/// Mas um cover por blocos paga o **AVENTAL** em cada bloco (`(b + 2·pad)²` por bloco tocado), e
/// com `pad = 25` um bloco de `16` é `77 %` avental. ⇒ *o tamanho do bloco é uma medição.*
/// Esta sonda conta, sobre a LISTA DE DABS REAL do produto, o que cada tamanho pagaria.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_tecto_de_um_cover -- --ignored
/// --nocapture`
#[test]
#[ignore = "sonda de geometria"]
fn diag_o_tecto_de_um_cover() {
    use super::super::composite_acumulado::fases;
    use composite::CompositeOp::{Blur, Brush};
    for lado in [1024u32, 2048] {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (lado * lado * 4) as usize], lado, lado);
        t.paint.brush.radius_px = 10.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        t.acrescenta_camada(Blur.to_u8());
        t.set_composite_layer_strength(0, 1.0);
        t.acrescenta_camada(Brush.to_u8());
        t.set_composite_layer_strength(1, 1.0);
        let c = [lado as f32 * 0.5, lado as f32 * 0.5];
        let r = lado as f32 * 0.35;
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Move));
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        let _ = fases::take();
        let _ = fases::take_cobertura();
        for i in 0..8 {
            let d = (i % 2) as f32 * 4.0 - 2.0;
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Move));
            t.on_canvas_pointer(cp2([c[0] + d, c[1]], PointerPhase::Up));
        }
        let (_, ev, _) = fases::take();
        let cob = fases::take_cobertura();
        let ev = ev.max(1) as f64;
        let col = fases::BLOCOS
            .iter()
            .zip(cob)
            .map(|(b, v)| format!("{b:>4}px {:>5.2}×", v / ev))
            .collect::<Vec<_>>()
            .join("  |  ");
        eprintln!("{lado}²  o cover custaria (1,00× = a caixa envolvente):  {col}");
    }
}

/// (19) **O TECTO DE UM BORRÃO PARALELO** — a sonda (18) refutou o cover por blocos (o melhor é
/// `0,89×` a `2048²` e **pior que a caixa** em toda a escada a `1024²`: o avental come a
/// esparsidade). ⇒ a área é forçada, e o que sobra é o custo POR PIXEL.
///
/// O borrão move `~224 B/px` (sete passagens de `[f32; 4]`, lidas e escritas) em `31,1 ns/px` =
/// **`7,2 GB/s`** — que é da ordem de UM núcleo, não do soquete. Esta sonda mede se o soquete tem
/// folga: `N` cópias do MESMO trabalho em `N` threads, e o débito agregado contra o de uma.
///
/// ⚠️ Ela **não** implementa nada: ela diz se vale a pena o ADR que a `ph2d-painter-brush` exige
/// para todo uso novo de `rayon` (a cerca do ADR-0109/0158).
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_tecto_de_um_borrao_paralelo --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de relógio"]
fn diag_o_tecto_de_um_borrao_paralelo() {
    const W: u32 = 2048;
    const LADO: usize = 1484;
    let k = 24usize;
    let n = LADO * LADO;
    let peso = vec![200u8; n];
    let x0 = i64::from((W as usize - LADO) as u32 / 2);
    let um = |b: &mut Vec<u8>| {
        ph2d_painter_brush::blur_region_por_peso(
            b,
            W,
            W,
            x0,
            x0,
            LADO,
            LADO,
            k,
            &peso,
            [false, false],
            ph2d_painter_brush::BlurKernel::Caixa,
        );
    };
    let mut base = f64::MAX;
    for fios in [1usize, 2, 4, 8, 16] {
        let mut melhor = f64::MAX;
        for _ in 0..3 {
            let mut bufs: Vec<Vec<u8>> = (0..fios)
                .map(|_| vec![128u8; (W * W * 4) as usize])
                .collect();
            let t = std::time::Instant::now();
            std::thread::scope(|s| {
                for b in &mut bufs {
                    s.spawn(|| um(b));
                }
            });
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
        }
        if fios == 1 {
            base = melhor;
        }
        let por_borrao = melhor / fios as f64;
        eprintln!(
            "{fios:>2} fios | parede {melhor:>7.2} ms | por borrão {por_borrao:>7.2} ms | débito \
             agregado {:>5.2}× | {:>5.1} GB/s",
            base / por_borrao,
            fios as f64 * n as f64 * 224.0 / (melhor / 1000.0) / 1e9,
        );
    }
}
