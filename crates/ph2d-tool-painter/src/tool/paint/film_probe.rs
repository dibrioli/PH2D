//! SONDA — **quanto relevo o DEPÓSITO de pigmento tem de deixar?** (estudo, Enio 2026-08-10)
//!
//! O pedido: *"criar o Relief para a deposição do pigmento com Shape exatamente como faz Wet Paint:
//! o depósito de pigmento com pouca água é visto como relevo"*.
//!
//! O irmão [`super::emboss_probe`] mediu por que a lei CRUA do Wet Paint (a diferença central da massa
//! impressa na cor) não transplanta: o Digital não tem campo de massa, e tinta opaca satura. Esta sonda
//! mede a outra metade — **se o depósito escrever um relevo, que amplitude lê como a foto?**
//!
//! Ela mede pela porta do PRODUTO (`on_canvas_pointer` → o depósito → `apply_impasto_light`), nunca por
//! um laço próprio: o que interessa é o número que o artista vê.
//!
//! Rodar: `cargo test -p ph2d-tool-painter probe_pigment_film -- --ignored --nocapture`
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

/// Uma tela BRANCA opaca — o documento do Digital, e o da cena de smoke.
fn blank() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    // Raio 10 = `IMPASTO_REFERENCE_RADIUS_PX`, então o `size_scale` do `derive_height` é exatamente 1
    // e o número medido é o do modelo, não o da escala do pincel.
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([200, 30, 30]);
    t
}

fn drag(t: &mut PainterTool, y: f32, x0: f32, x1: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 1.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

/// A tela composta e ILUMINADA — o que o artista vê.
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

/// Luminância de uma linha, dentro da faixa do traço.
fn row(px: &[u8], y: u32, x0: u32, x1: u32) -> Vec<f64> {
    (x0..x1)
        .map(|x| {
            let i = ((y * N + x) * 4) as usize;
            0.2126 * f64::from(px[i])
                + 0.7152 * f64::from(px[i + 1])
                + 0.0722 * f64::from(px[i + 2])
        })
        .collect()
}

/// **O que o RELEVO acrescentou** — `(pior |Δ|, média |Δ|)` entre a tela iluminada e a MESMA tela sem
/// relevo nenhum, dentro da faixa do traço.
///
/// ⚠️ **A primeira versão desta sonda media a luminância ABSOLUTA e não mediu o relevo.** Um traço com
/// Shape listrada já tem excursão **53,80** de PIGMENTO — o desenho, não o relevo —, e é exatamente a
/// armadilha que o [`super::emboss_probe`] nomeia: *a luminância não distingue tinta de desenho*. O
/// oráculo de um relevo é a DIFERENÇA que ele faz.
fn relief_delta(with: &[u8], without: &[u8], y: u32) -> (f64, f64) {
    let a = row(with, y, 24, 72);
    let b = row(without, y, 24, 72);
    let d: Vec<f64> = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).collect();
    let worst = d.iter().cloned().fold(0.0, f64::max);
    let mean = d.iter().sum::<f64>() / d.len() as f64;
    (worst, mean)
}

/// Pinta o mesmo traço duas vezes — com e sem o relevo — e devolve o que o relevo acrescentou.
fn film(shape: TextureKind, size: f32, depth: f32, paper: bool) -> (f64, f64) {
    let arm = |t: &mut PainterTool, relief: bool| {
        t.set_brush_size_px(size);
        t.set_brush_shape_kind(shape as u8);
        // ⚠️ **O CONTROLE tem de depositar o MESMO pigmento**, e a 1ª versão desta sonda não o fazia:
        // `set_brush_impasto(true)` troca o falloff de fábrica (Smooth → Sphere, Enio 2026-07-17), então
        // os dois braços pintavam tinta diferente e o "delta do relevo" media a mudança de SILHUETA —
        // 64 níveis para um relevo de 0,32 px, que é o número denunciando a fixture.
        t.paint.brush.falloff = ph2d_painter_brush::Falloff::Sphere;
        if paper {
            t.set_substrate_depth(1.0);
            t.set_substrate_roughness(0.5);
        }
        if relief {
            t.set_brush_impasto(true);
            t.paint.brush.impasto_depth = depth;
            t.paint.brush.impasto_body = 0.0; // o perfil da PRÓPRIA tinta: a Shape esculpe o relevo
        }
    };
    let mut with = blank();
    arm(&mut with, true);
    drag(&mut with, 48.0, 24.0, 72.0);
    let mut without = blank();
    arm(&mut without, false);
    drag(&mut without, 48.0, 24.0, 72.0);
    relief_delta(&lit(&with), &lit(&without), 48)
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn probe_pigment_film() {
    println!("\n=== O DEPÓSITO COMO RELEVO — quanto? (canvas {N}², niveis de luminancia) ===");
    println!("(o numero e o que o RELEVO acrescentou: a mesma cena com e sem ele)\n");

    println!("-- o filme sozinho, sem papel --");
    for shape in [TextureKind::None, TextureKind::Stripes] {
        for depth in [0.02f32, 0.05, 0.1, 0.25, 0.5, 1.0] {
            let (w, m) = film(shape, 20.0, depth, false);
            let px = depth * super::impasto_light::DEPTH_UNIT_PX;
            println!(
                "Shape {shape:<8?} depth {depth:5.2} (~{px:5.2} px)  pior {w:7.2}  media {m:6.2}"
            );
        }
    }

    println!("\n-- o filme SOBRE o papel (Relief 1, Rough 0,5) --");
    for depth in [0.02f32, 0.05, 0.1, 0.25] {
        let (w, m) = film(TextureKind::Stripes, 20.0, depth, true);
        println!("papel + filme depth {depth:5.2}          pior {w:7.2}  media {m:6.2}");
    }

    // ⚠️ **O PREÇO do `size_scale` do `derive_height`** — a razão pela qual um filme não pode ser um
    // impasto fino: a altura do impasto escala com o RAIO (para a razão de aspecto do domo ficar
    // constante), e um filme de pigmento não engrossa porque o pincel é maior.
    println!("\n-- o MESMO depth em pinceis diferentes (o `size_scale`) --");
    for size in [10.0f32, 20.0, 40.0, 80.0, 160.0] {
        let (w, m) = film(TextureKind::Stripes, size, 0.05, false);
        println!(
            "size {size:6.1} px (raio {:5.1})           pior {w:7.2}  media {m:6.2}",
            size / 2.0
        );
    }
    println!();
}

/// ⚠️ **O CAMPO, antes de qualquer lei sobre ele — e é esta sonda que decidiu o desenho.**
///
/// A primeira versão do filme derivava a espessura da COBERTURA que a luz já dobra (`cover_at`), o que
/// teria deixado o slider vivo sobre a tela INTEIRA em vez de só no último traço — uma propriedade
/// melhor. Ela media **0,21** nível onde a rota do impasto, com a MESMA Shape, media 9,72; e a
/// diferença tinha de estar no campo, não na amplitude.
///
/// Está: **a cobertura SATURA** (`0,992..1,000` dentro do traço) e um gradiente sobre um platô é zero.
/// Quem carrega a estrutura é o envelope de CARGA — e é por isso que o filme é uma altura.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn probe_what_the_deposit_records() {
    println!("\n=== O QUE O DEPOSITO ANOTA (canvas {N}², r=10, traco horizontal em y=48) ===\n");
    for shape in [TextureKind::None, TextureKind::Stripes, TextureKind::Noise] {
        let mut t = blank();
        t.set_brush_shape_kind(shape as u8);
        t.set_shape_relief(1.0);
        drag(&mut t, 48.0, 24.0, 72.0);
        let id = t.layers.active().expect("camada ativa");
        let cov = t.covers.get(&id).cloned().unwrap_or_default();
        let row: Vec<f64> = (24..72)
            .map(|x| f64::from(cov.get((48 * N + x) as usize).copied().unwrap_or(0)) / 255.0)
            .collect();
        let hi = row.iter().cloned().fold(f64::MIN, f64::max);
        let lo = row.iter().cloned().fold(f64::MAX, f64::min);
        let d: f64 =
            row.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (row.len() - 1) as f64;
        // O envelope de CARGA que o impasto usa, para comparar no mesmo traço.
        let lp = &t.paint.relief.live_paint;
        let prow: Vec<f64> = (24..72)
            .map(|x| f64::from(lp.get((48 * N + x) as usize).copied().unwrap_or(0.0)))
            .collect();
        let phi = prow.iter().cloned().fold(f64::MIN, f64::max);
        let plo = prow.iter().cloned().fold(f64::MAX, f64::min);
        let pd: f64 =
            prow.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (prow.len() - 1) as f64;
        println!(
            "Shape {shape:<8?}  covers  min {lo:5.3} max {hi:5.3} |d| {d:6.4}   \
             carga  min {plo:5.3} max {phi:5.3} |d| {pd:6.4}"
        );
    }
    println!();
}

/// O filme pela PORTA DO PRODUTO — `set_shape_relief`, que é o que o artista arrasta.
fn product_film(shape: TextureKind, size: f32, paint: f32, relief: f32) -> (f64, f64) {
    let arm = |t: &mut PainterTool, on: bool| {
        t.set_brush_size_px(size);
        t.set_brush_shape_kind(shape as u8);
        t.set_substrate_depth(relief);
        t.set_substrate_roughness(0.5);
        if on {
            t.set_shape_relief(paint);
        }
    };
    let mut with = blank();
    arm(&mut with, true);
    drag(&mut with, 48.0, 24.0, 72.0);
    let mut without = blank();
    arm(&mut without, false);
    drag(&mut without, 48.0, 24.0, 72.0);
    relief_delta(&lit(&with), &lit(&without), 48)
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn probe_film_through_the_product_door() {
    println!("\n=== O FILME PELA PORTA DO ARTISTA (`set_shape_relief`) ===");
    println!("(o numero e o que o FILME acrescentou sobre a mesma cena sem ele)\n");

    for shape in [TextureKind::None, TextureKind::Stripes, TextureKind::Noise] {
        for paint in [0.25f32, 0.5, 1.0] {
            let (w, m) = product_film(shape, 20.0, paint, 0.0);
            println!(
                "sem papel   Shape {shape:<8?} Paint {paint:4.2}   pior {w:7.2}  media {m:6.2}"
            );
        }
    }
    println!();
    for shape in [TextureKind::None, TextureKind::Stripes, TextureKind::Noise] {
        let (w, m) = product_film(shape, 20.0, 1.0, 1.0);
        println!("papel 1     Shape {shape:<8?} Paint 1.00   pior {w:7.2}  media {m:6.2}");
    }

    // ⚠️ **A textura pode vir do GRAIN também** — o filme não inventa estrutura, ele revela a que o
    // pincel já deposita, e o envelope de carga inclui a máscara inteira (silhueta × grain).
    {
        let arm = |t: &mut PainterTool, on: bool| {
            t.set_brush_size_px(20.0);
            t.set_brush_texture_kind(TextureKind::Noise as u8);
            if on {
                t.set_shape_relief(1.0);
            }
        };
        let mut with = blank();
        arm(&mut with, true);
        drag(&mut with, 48.0, 24.0, 72.0);
        let mut without = blank();
        arm(&mut without, false);
        drag(&mut without, 48.0, 24.0, 72.0);
        let (w, m) = relief_delta(&lit(&with), &lit(&without), 48);
        println!("\nsem Shape, com GRAIN Noise    Paint 1.00   pior {w:7.2}  media {m:6.2}");
    }

    println!(
        "\n-- o filme NAO escala com o pincel (o que o impasto faz e um filme nao pode fazer) --"
    );
    for size in [10.0f32, 20.0, 40.0, 80.0, 160.0] {
        let (w, m) = product_film(TextureKind::Stripes, size, 1.0, 0.0);
        println!(
            "size {size:6.1} px (raio {:5.1})           pior {w:7.2}  media {m:6.2}",
            size / 2.0
        );
    }
    println!();
}

/// ⚠️ **A TERCEIRA metade do pedido — *"a tinta é de alto brilho ou fosca"* — e a sonda existe porque o
/// irmão dela JÁ FOI REPROVADO uma vez.**
///
/// O `⛔` do [`super::substrate_relief`] mediu que um realce especular no PAPEL não move um texel: num
/// dente de ~1 px a normal quase não sai do plano, e o realce de cada lâmpada tem a resposta PLANA
/// subtraída (`the_glint_only_ever_adds_light`), logo `spec − flat_spec` é nulo em qualquer expoente.
/// **O filme tem a MESMA espessura** (`MAX_FILM_PX == MAX_TOOTH_PX == 1,0`), então a leitura barata é
/// que um slider de brilho aqui nasceria morto pelo mesmo mecanismo.
///
/// A diferença que a sonda tem de decidir é a INCLINAÇÃO, não a espessura: o dente do papel é uma onda
/// larga (a normal mal se inclina) e o filme com Shape listrada cai de 1 px a zero em ~1 px — 45°.
///
/// Rodar: `cargo test -p ph2d-tool-painter probe_the_films_gloss -- --ignored --nocapture`
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn probe_the_films_gloss() {
    /// O MESMO filme com dois materiais — o número é o que o material acrescentou.
    fn gloss(shape: TextureKind, paint: f32, a: (f32, f32), b: (f32, f32)) -> (f64, f64) {
        let arm = |t: &mut PainterTool, (shine, rough): (f32, f32)| {
            t.set_brush_size_px(20.0);
            t.set_brush_shape_kind(shape as u8);
            t.set_shape_relief(paint);
            t.set_impasto_shine(shine);
            t.set_impasto_roughness(rough);
        };
        let mut x = blank();
        arm(&mut x, a);
        drag(&mut x, 48.0, 24.0, 72.0);
        let mut y = blank();
        arm(&mut y, b);
        drag(&mut y, 48.0, 24.0, 72.0);
        relief_delta(&lit(&x), &lit(&y), 48)
    }

    println!("\n=== O MATERIAL DO FILME — brilhante ou fosco? (niveis de luminancia) ===");
    println!("(o numero e o que o MATERIAL acrescentou sobre o MESMO filme fosco)\n");

    println!("-- Shine, na Roughness de fabrica --");
    let neutral = ph2d_painter_brush::material::Material::NEUTRAL.roughness;
    for shine in [0.25f32, 0.5, 0.7, 1.0] {
        let (w, m) = gloss(
            TextureKind::Stripes,
            1.0,
            (shine, neutral),
            (0.0, neutral),
        );
        println!("Shine {shine:4.2} (rough {neutral:4.2})            pior {w:7.2}  media {m:6.2}");
    }

    println!("\n-- Roughness, no Shine de fabrica (0,7): o realce APERTA ou ESPALHA? --");
    for rough in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let (w, m) = gloss(TextureKind::Stripes, 1.0, (0.7, rough), (0.0, rough));
        println!("Rough {rough:4.2} contra fosco             pior {w:7.2}  media {m:6.2}");
    }
    for rough in [0.0f32, 1.0] {
        let (w, m) = gloss(TextureKind::Stripes, 1.0, (0.7, rough), (0.7, neutral));
        println!("Rough {rough:4.2} contra a de fabrica       pior {w:7.2}  media {m:6.2}");
    }

    println!("\n-- e o CONTROLE: a mesma pergunta no PAPEL, que o ⛔ ja reprovou --");
    {
        let arm = |t: &mut PainterTool, shine: f32| {
            t.set_brush_size_px(20.0);
            t.set_substrate_depth(1.0);
            t.set_substrate_roughness(0.5);
            t.set_impasto_shine(shine);
        };
        let mut x = blank();
        arm(&mut x, 1.0);
        let mut y = blank();
        arm(&mut y, 0.0);
        let (w, m) = relief_delta(&lit(&x), &lit(&y), 48);
        println!("papel nu, Shine 1 contra 0        pior {w:7.2}  media {m:6.2}");
    }

    println!("\n-- e por espessura: onde o realce nasce --");
    for paint in [0.1f32, 0.25, 0.5, 1.0] {
        let (w, m) = gloss(
            TextureKind::Stripes,
            paint,
            (1.0, neutral),
            (0.0, neutral),
        );
        println!("Paint {paint:4.2}, Shine 1 contra 0        pior {w:7.2}  media {m:6.2}");
    }
    println!();
}

/// ⚠️ **O filme age nos QUATRO meios?** — a pergunta que decide se a row do Relief pode ficar sob o
/// portão "Automatic" da aquarela (que esconde o corpo inteiro da seção Shape, doc 13 #1).
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn probe_the_film_across_the_media() {
    println!("\n=== O FILME EM CADA MEIO (niveis de luminancia que o relevo acrescenta) ===\n");
    for media in [
        crate::PaintMedia::Digital,
        crate::PaintMedia::Watercolor,
        crate::PaintMedia::Impasto,
        crate::PaintMedia::WetPaint,
    ] {
        let arm = |t: &mut PainterTool, on: bool| {
            t.set_paint_media(media);
            t.set_brush_size_px(20.0);
            t.set_brush_shape_kind(TextureKind::Stripes as u8);
            if on {
                t.set_shape_relief(1.0);
            }
        };
        let mut with = blank();
        arm(&mut with, true);
        drag(&mut with, 48.0, 24.0, 72.0);
        let mut without = blank();
        arm(&mut without, false);
        drag(&mut without, 48.0, 24.0, 72.0);
        let (w, m) = relief_delta(&lit(&with), &lit(&without), 48);
        println!("{:<12}  pior {w:7.2}  media {m:6.2}", format!("{media:?}"));
    }
    println!();
}
