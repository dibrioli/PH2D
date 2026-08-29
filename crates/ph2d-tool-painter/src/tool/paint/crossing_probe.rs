//! **Sonda do ENTALHE no CRUZAMENTO** — a reentrância branca que o Enio fotografou numa cruz de
//! aquarela (2026-08-12: *"em vários lugares deste app e principalmente na implementação do traço de
//! FLIP tivemos problemas com o Alpha que criava reentrâncias nos cruzamentos de traços. Parece que o
//! mesmo ocorre com watercolor"*).
//!
//! **A pergunta que ela responde é uma só, e é aritmética.** Duas faixas ortogonais de meia-largura
//! `R` se cruzam. Um ponto na bissetriz, a `s` px de CADA eixo, recebe de cada faixa a MESMA
//! cobertura `f(s)` que receberia dela sozinho. As duas leis possíveis dão números diferentes:
//!
//! * **UNIÃO** (`max`, o envelope): `cov = f(s)` ⇒ a axila fica **igual** ao ombro solitário.
//! * **COMPOSIÇÃO** (cobertura independente): `cov = 1 − (1−f)² = 2f − f²` ⇒ a axila fica **acima**,
//!   com o máximo em `f = 0,5` (0,50 → 0,75).
//!
//! Então a sonda mede `axila(s)` contra `ombro(s)` — o MESMO `s`, o mesmo pincel, longe do
//! cruzamento — e o predito pela composição ao lado. Não precisa de imagem de referência: o oráculo é
//! o próprio ombro da faixa.
//!
//! ⚠️ **`s` corre DENTRO da faixa, e a 1ª versão desta sonda media FORA — e leu zero em tudo.** Com
//! `hardness 0` + `Falloff::Smooth` o perfil chega a zero EM `R`: o ombro macio vive dentro do disco,
//! não além dele. A tabela de zeros passava como "sem diferença" e teria fechado a investigação no
//! lugar errado; o `controle` (quantos px têm tinta) é o que a denunciou.
//!
//! ⚠️ **É exatamente a lei que o FLIP curou em 2026-07-28** (`flip.wgsl`, §"UMA PASSAGEM, UMA
//! COBERTURA"): *"`min` de duas funções lisas tem VINCO na bissetriz do cruzamento — invisível em
//! hardness 1 (máscara binária) e uma costura com pincel macio"*. Lá o desvio entre as duas rotas
//! media 48/255 em hardness 0,4.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release crossing_probe -- --ignored --nocapture`

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::BrushSpec;

use super::accumulate_probe::cp;

const SIZE: u32 = 256;
/// O centro do cruzamento.
const C: f32 = 128.0;
/// Meia-largura da faixa = o raio do pincel.
const R: f32 = 24.0;
/// Onde o ombro SOLITÁRIO é medido — longe do cruzamento, no mesmo `t` perpendicular.
const LONE_X: f32 = 56.0;

pub(super) fn white(size: u32) -> Vec<u8> {
    vec![255u8; (size * size * 4) as usize]
}

/// O pincel do produto para esta cena: aquarela, macio, um traço largo.
fn wash_brush() -> BrushSpec {
    BrushSpec {
        radius_px: R,
        color: [0.85, 0.15, 0.15],
        space_attenuation: false,
        watercolor: true,
        ..Default::default()
    }
}

pub(super) fn arm(t: &mut PainterTool, spec: BrushSpec) {
    t.paint.brush = spec;
    t.paint.brush_by_mode.fill(spec);
}

fn stroke(t: &mut PainterTool, from: [f32; 2], to: [f32; 2]) {
    t.on_canvas_pointer(cp(from, PointerPhase::Down));
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let n = 24;
    for i in 1..=n {
        let f = i as f32 / n as f32;
        t.on_canvas_pointer(cp([from[0] + dx * f, from[1] + dy * f], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
}

/// Opacidade da tinta num ponto: vermelho sobre branco ⇒ o canal VERDE cai com a tinta.
fn alpha_at(t: &PainterTool, x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.round() as u32, y.round() as u32);
    let i = ((yi * SIZE) + xi) as usize * 4;
    (255.0 - f32::from(t.canvas_rgba[i + 1])) / 255.0
}

/// A cruz feita de DOIS traços (o gesto que a foto mostra).
fn cross_two_strokes(spec: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(white(SIZE), SIZE, SIZE);
    arm(&mut t, spec);
    stroke(&mut t, [40.0, C], [216.0, C]);
    stroke(&mut t, [C, 40.0], [C, 216.0]);
    t
}

/// UM traço que cruza a si mesmo — o caso que no FLIP se comportava DIFERENTE dos dois traços,
/// porque ali o depth é o mesmo e a lei caía na união em vez de compor.
fn cross_one_stroke(spec: BrushSpec) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(white(SIZE), SIZE, SIZE);
    arm(&mut t, spec);
    // Um "L" que volta e atravessa a própria perna: (40,C) → (216,C) → (216,56) → (C,56) → (C,216).
    t.on_canvas_pointer(cp([40.0, C], PointerPhase::Down));
    let legs = [
        ([40.0, C], [216.0, C]),
        ([216.0, C], [216.0, 56.0]),
        ([216.0, 56.0], [C, 56.0]),
        ([C, 56.0], [C, 216.0]),
    ];
    for (from, to) in legs {
        let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
        for i in 1..=24 {
            let f = i as f32 / 24.0;
            t.on_canvas_pointer(cp([from[0] + dx * f, from[1] + dy * f], PointerPhase::Move));
        }
    }
    t.on_canvas_pointer(cp([C, 216.0], PointerPhase::Up));
    t
}

fn report(label: &str, t: &PainterTool) {
    println!("\n  {label}");
    // CONTROLE: a cena tem de conter tinta, senao a tabela abaixo compara dois zeros.
    let painted = t
        .canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[1] != 255)
        .count();
    println!(
        "      (controle: {painted} px com tinta; centro do cruzamento alpha={:.3}; \
         miolo do braco alpha={:.3})",
        alpha_at(t, C, C),
        alpha_at(t, LONE_X, C)
    );
    println!("      s   ombro   axila   composto   axila-ombro");
    // ⚠️ O ombro macio vive DENTRO do disco: com `hardness 0` + `Falloff::Smooth` o perfil chega a
    // zero EM `R`, entao nao ha nada a medir fora da faixa. `s` e a distancia perpendicular ao eixo,
    // de 0 (o eixo) a R (a borda).
    for s in [0.0f32, 4.0, 8.0, 12.0, 16.0, 18.0, 20.0, 22.0] {
        // Ombro solitário: `s` px do eixo da faixa horizontal, longe do cruzamento.
        let lone = alpha_at(t, LONE_X, C + s);
        // Axila: na bissetriz, `s` px de CADA eixo — as duas faixas contribuem o mesmo `f(s)`.
        let pit = alpha_at(t, C + s, C + s);
        let composed = 2.0 * lone - lone * lone;
        println!(
            "   {s:5.1}   {lone:5.3}   {pit:5.3}      {composed:5.3}        {:+.3}",
            pit - lone
        );
    }
}

/// **RENDER-AND-LOOK headless** — o mapa do alfa em volta da quina côncava. A tabela acima diz
/// NÚMEROS; isto diz FORMA, que é o que a foto mostra: se há uma cunha clara entrando pela
/// bissetriz, ela aparece aqui como uma faixa de dígitos baixos entre dois aros escuros.
///
/// ⚠️ **A ESCALA é parte da sonda, e a primeira versão dela era CEGA:** com o `fill` de fábrica
/// (0,12) a lavagem inteira vive em alpha ≈ 0,28, então `alpha*9` colapsa a cena toda em `2` e um mapa
/// que só sabe dizer 1, 2 e 3 **não pode responder onde a tinta acaba**. Subir o `fill` para 1 resolve
/// o dígito e **muda o regime** (o interior satura e o aro deixa de dominar a aparência, que é o
/// oposto do produto) — então quem sobe é a ESCALA do dígito, nunca o pincel.
fn map_scaled(label: &str, t: &PainterTool, scale: f32) {
    println!(
        "\n  {label}  (x,y de {C:.0} a {:.0}; digito = alpha*{scale:.0})",
        C + 40.0
    );
    for y in 0..40u32 {
        let mut row = String::new();
        for x in 0..40u32 {
            let a = alpha_at(t, C + x as f32, C + y as f32);
            let d = (a * scale).round().clamp(0.0, 9.0) as u32;
            row.push(if d == 0 {
                '.'
            } else {
                char::from_digit(d, 10).unwrap()
            });
        }
        println!("   {row}");
    }
}

/// **O GATE da wave — o aro VIRA a quina côncava, e a tinta não some na axila.**
///
/// Não afirma um número de aparência: afirma a PROPRIEDADE que o teto de flanco reto estabelece —
/// na faixa que corre pela bissetriz saindo da quina, a lavagem **não pode ter um vão** onde os
/// dois lados têm tinta. É a cunha branca da foto, escrita como asserção.
///
/// ⚠️ **O oráculo é DERIVADO, não escolhido: uma cunha é um lugar mais CLARO que a tinta em volta.**
/// A 1ª versão deste gate levou uma barra tirada do mapa (`pit > 0.20`) e **a mutação sobreviveu** —
/// sem a cura o vão mede 0,247, que passava. Medidos os dois estados (sem cura **0,247**, com cura
/// **0,322**), o que os separa não é um número a escolher: é o **miolo do braço** (0,278), que está
/// entre eles. Então o gate afirma a propriedade que a foto mostra — *a axila não pode ser mais
/// clara que a lavagem lisa ao lado dela* — e o número vem da mesma corrida.
///
/// ⚠️ **Com CONTROLE**, senão ele passa por vácuo: a cena tem de conter os dois aros (é o contraste
/// entre eles e o vão que faz a cunha ser vista), e o vão é medido *entre* eles.
///
/// **Mutação que tem de sangrar:** devolver `1.0` de [`super::watercolor_rim::straight_edge_cap`]
/// (o teto inerte) ⇒ o vão reabre para 0,247, abaixo do miolo.
#[test]
fn the_rim_turns_the_concave_corner_instead_of_leaving_a_wedge() {
    let t = cross_two_strokes(wash_brush());
    // A linha y = C + 21 atravessa: miolo · aro vertical · a AXILA · aro horizontal.
    let y = C + 21.0;
    let rim_v = (14..=20)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(0.0f32, f32::max);
    let rim_h = (26..=34)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(0.0f32, f32::max);
    let pit = (21..=25)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(1.0f32, f32::min);
    // O miolo LISO de um braco, longe do cruzamento: a regua da propriedade, medida na mesma cena.
    let interior = alpha_at(&t, LONE_X, C);
    // CONTROLE: os dois aros existem e sao mais escuros que o miolo. Sem esta metade um `pit` alto
    // passaria numa cena sem aro nenhum, e o gate estaria verde sobre uma lavagem sem o fenomeno.
    assert!(
        rim_v > interior + 0.15 && rim_h > interior + 0.15,
        "a fixture tem de conter os DOIS aros (vertical {rim_v:.3}, horizontal {rim_h:.3}, \
         miolo {interior:.3})"
    );
    // A PROPRIEDADE: uma cunha e' um lugar mais CLARO que a tinta em volta.
    assert!(
        pit > interior,
        "a axila e' mais clara que a lavagem lisa: minimo {pit:.3} contra miolo {interior:.3} \
         (aros {rim_v:.3}/{rim_h:.3}) — o aro parou de virar a quina"
    );
}

/// **O MESMO, com a lavagem ainda VIVA** — um traço que cruza a si mesmo e que o artista **não
/// soltou**.
///
/// ⚠️ **Este gate existe porque a mutação do irmão SOBREVIVEU.** O campo de distância só é computado
/// se algum dono tem aro, e a pergunta tem dois lados: a **tabela** de estilos (donos já commitados)
/// e o pincel **VIVO**. Numa cruz de dois traços a tabela já responde `true`, então tirar o lado
/// vivo do gate não muda um pixel ali — é a defesa em camadas escondendo o buraco. O único caso que
/// isola o lado vivo é **a primeira lavagem da sessão, ainda aberta**, e é esta.
///
/// **Mutação que tem de sangrar:** tirar `|| self.paint.brush.edge_gain > 0.0` do `wants_rim`.
#[test]
fn the_first_stroke_of_the_session_turns_the_corner_too() {
    let t = cross_one_stroke(wash_brush());
    let y = C + 21.0;
    let interior = alpha_at(&t, LONE_X, C);
    let rim_v = (14..=20)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(0.0f32, f32::max);
    let rim_h = (26..=34)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(0.0f32, f32::max);
    let pit = (21..=25)
        .map(|x| alpha_at(&t, C + x as f32, y))
        .fold(1.0f32, f32::min);
    assert!(
        rim_v > interior + 0.15 && rim_h > interior + 0.15,
        "a fixture de UM traco tem de conter os DOIS aros ({rim_v:.3}/{rim_h:.3}, miolo {interior:.3})"
    );
    assert!(
        pit > interior,
        "a axila do PRIMEIRO traco e' mais clara que a lavagem lisa: {pit:.3} contra {interior:.3}"
    );
}

/// **QUANTO a cura move um traço RETO** — o número que o pino de fingerprint não dá.
///
/// O `smooth_edges_off_is_the_pre_aa_render_byte_for_byte` pina um hash de canvas inteiro, e um hash
/// move com UM byte: ele diz *que* mudou e nunca *quanto*. Esta sonda escreve o canvas da MESMA
/// fixture dele num arquivo, para o A/B ser um diff de bytes entre duas corridas (com a cura e com
/// [`super::watercolor_rim::straight_edge_cap`] devolvendo `1.0`).
///
/// `env PH2D_RIM_DUMP=<arquivo> cargo test -p ph2d-tool-painter --release rim_dump -- --ignored`
#[test]
#[ignore = "sonda; roda com --ignored e PH2D_RIM_DUMP"]
fn rim_dump_the_straight_stroke_the_pin_watches() {
    let Ok(path) = std::env::var("PH2D_RIM_DUMP") else {
        println!("defina PH2D_RIM_DUMP=<arquivo>");
        return;
    };
    // A fixture EXATA do pino: traço reto, raio 40, warp 6, Smooth Edges OFF.
    let mut t = PainterTool::default();
    t.set_source(white(256), 256, 256);
    let spec = BrushSpec {
        radius_px: 40.0,
        color: [0.85, 0.15, 0.15],
        space_attenuation: false,
        watercolor: true,
        warp: 6.0,
        smooth_edges: false,
        ..Default::default()
    };
    arm(&mut t, spec);
    stroke(&mut t, [70.0, 128.0], [186.0, 128.0]);
    std::fs::write(&path, &*t.canvas_rgba).expect("dump");
    println!("escrito: {path} ({} bytes)", t.canvas_rgba.len());
}

/// A medição. Imprime as duas cenas (dois traços · um traço que se cruza) para a aquarela e o
/// controle DIGITAL, que é a lei que ninguém reportou como quebrada.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_crossing_notch() {
    println!("\n=== O ENTALHE NO CRUZAMENTO ===");
    println!("  uniao  ⇒ axila == ombro   |   composicao ⇒ axila == composto (> ombro)");

    let wash = wash_brush();
    report("AQUARELA — dois tracos", &cross_two_strokes(wash));
    report(
        "AQUARELA — um traco cruzando a si mesmo",
        &cross_one_stroke(wash),
    );

    let digital = BrushSpec {
        watercolor: false,
        ..wash
    };
    report(
        "DIGITAL (controle) — dois tracos",
        &cross_two_strokes(digital),
    );
    report(
        "DIGITAL (controle) — um traco cruzando a si mesmo",
        &cross_one_stroke(digital),
    );

    println!("\n=== A FORMA (o quadrante inferior-direito da quina) ===");
    // ⚠️ ESCALA 30, não 9: no regime do PRODUTO (`fill` 0,12) o alfa da lavagem vive em ~0,28 e o do
    // aro em ~0,78 — `alpha*9` daria `2` para a lavagem inteira e a sonda ficaria cega. Ver `map_scaled`.
    map_scaled("AQUARELA — dois tracos", &cross_two_strokes(wash), 30.0);
    map_scaled(
        "DIGITAL (controle) — dois tracos",
        &cross_two_strokes(digital),
        9.0,
    );

    // ── ABLAÇÃO PELA ENTRADA, um knob por vez, no REGIME DO PRODUTO ────────────────────────────────
    //
    // Os quatro mapas abaixo são o MESMO pincel com um termo a menos cada, o `fill` **intocado** (subir
    // o fill resolveria o dígito e trocaria o regime: o interior satura e o aro deixa de dominar, que é
    // o oposto do que a foto mostra). A ordem responde a pergunta em degraus:
    //
    //   1. `granulacao 0 · warp 0`      — a cena LISA: sobra a silhueta + o unsharp.
    //   2. + `edge_gain 0`              — a SILHUETA sozinha: é a UNIÃO da cobertura.
    //   3. a lisa com warp de volta      — o warp desloca a AMOSTRA da cobertura.
    //   4. a lisa com granulacao de volta.
    //
    // A cunha aparece no 1º mapa em que ela existe, e o knob que a devolve é a causa.
    println!("\n=== ABLAÇÃO (regime do produto, fill 0.12): um knob por vez ===");
    let smooth = BrushSpec {
        granulation: 0.0,
        warp: 0.0,
        ..wash
    };
    map_scaled(
        "1. LISA (gran 0 · warp 0) — silhueta + aro",
        &cross_two_strokes(smooth),
        30.0,
    );
    let smooth_no_edge = BrushSpec {
        edge_gain: 0.0,
        ..smooth
    };
    map_scaled(
        "2. LISA sem aro — so a SILHUETA (a uniao da cobertura)",
        &cross_two_strokes(smooth_no_edge),
        30.0,
    );
    map_scaled(
        "3. LISA + warp 6 (a amostra da cobertura e' deslocada)",
        &cross_two_strokes(BrushSpec {
            warp: 6.0,
            ..smooth
        }),
        30.0,
    );
    map_scaled(
        "4. LISA + granulacao 0.3",
        &cross_two_strokes(BrushSpec {
            granulation: 0.3,
            ..smooth
        }),
        30.0,
    );
    // A célula que FECHA o 2×2: warp SEM aro. Se a cunha vive aqui, ela é da SILHUETA deslocada; se
    // ela some, é o unsharp (o lobo pálido) amplificado pela borda esfarrapada.
    map_scaled(
        "5. LISA + warp 6, SEM aro",
        &cross_two_strokes(BrushSpec {
            warp: 6.0,
            ..smooth_no_edge
        }),
        30.0,
    );

    corner_reach_table(&wash, &smooth);

    franja_sobre_tinta();
}

/// Limiar de "há tinta aqui" para as medidas de ALCANCE (o alfa de fundo é exatamente 0).
pub(super) const INK: f32 = 0.02;

/// O último `d` ao longo de um raio saindo do centro do cruzamento em que ainda há tinta.
/// `dir` é unitário; a varredura vai até bem além do raio do pincel.
fn ray_reach(t: &PainterTool, dir: [f32; 2]) -> f32 {
    let mut last = 0.0f32;
    let mut d = 0.0f32;
    while d < 60.0 {
        if alpha_at(t, C + dir[0] * d, C + dir[1] * d) > INK {
            last = d;
        }
        d += 0.5;
    }
    last
}

/// **A CUNHA, como número e nos QUATRO cantos.**
///
/// A união de duas faixas de meia-largura `w` tem quina RETA em `(w, w)`, então o alcance ao longo da
/// bissetriz vale `w·√2` — **exatamente**, e sem escolher constante nenhuma. Uma cunha é esse alcance
/// FALTANDO, e um mapa de um quadrante não distingue *ruído da borda esfarrapada* de *lei*: a foto
/// mostra a cunha nos QUATRO cantos, e é isso que esta tabela mede.
fn corner_reach_table(wash: &BrushSpec, smooth: &BrushSpec) {
    println!("\n=== O ALCANCE DA QUINA (uniao de duas faixas ⇒ bissetriz = w·raiz(2)) ===");
    println!("      cena                 w      esperado   ++     +-     -+     --");
    let s = std::f32::consts::FRAC_1_SQRT_2;
    for (label, spec) in [
        ("LISA (warp 0)", *smooth),
        (
            "LISA + warp 6",
            BrushSpec {
                warp: 6.0,
                ..*smooth
            },
        ),
        ("PRODUTO (warp 6 + gran)", *wash),
    ] {
        let t = cross_two_strokes(spec);
        // `w`: a meia-largura do braço, medida LONGE do cruzamento (o mesmo ombro solitário da tabela).
        let mut w = 0.0f32;
        let mut y = 0.0f32;
        while y < 60.0 {
            if alpha_at(&t, LONE_X, C + y) > INK {
                w = y;
            }
            y += 0.5;
        }
        let expected = w * std::f32::consts::SQRT_2;
        let r = [
            ray_reach(&t, [s, s]),
            ray_reach(&t, [s, -s]),
            ray_reach(&t, [-s, s]),
            ray_reach(&t, [-s, -s]),
        ];
        println!(
            "   {label:22} {w:4.1}    {expected:6.1}   {:5.1}  {:5.1}  {:5.1}  {:5.1}",
            r[0], r[1], r[2], r[3]
        );
    }

    // O PERFIL ao longo da bissetriz: `ray_reach` devolve o ÚLTIMO ponto com tinta e por isso é CEGO
    // a um buraco no meio — e um buraco cercado de tinta é exatamente a forma da cunha da foto.
    println!("\n   perfil ao longo da bissetriz (d = 20..40, alpha*100)");
    for (label, spec) in [
        ("LISA (warp 0)", *smooth),
        (
            "LISA + warp 6",
            BrushSpec {
                warp: 6.0,
                ..*smooth
            },
        ),
        ("PRODUTO", *wash),
    ] {
        let t = cross_two_strokes(spec);
        let mut line = String::new();
        let mut d = 20.0f32;
        while d <= 40.0 {
            let a = alpha_at(&t, C + d * s, C + d * s);
            line.push_str(&format!("{:4.0}", a * 100.0));
            d += 1.0;
        }
        println!("   {label:16}{line}");
    }
}

/// **O PREÇO do teto de dois lados, medido no lugar onde ele pode custar.**
///
/// O lobo pálido do unsharp vive FORA da lavagem. Sobre papel branco ele é invisível (branco sobre
/// branco), então limitá-lo não muda nada ali. Onde ele é visível é sobre TINTA QUE JÁ EXISTE — a
/// franja clara que uma lavagem nova abre na tinta vizinha. Esta sonda mede exatamente isso, num
/// flanco **RETO** (longe de qualquer quina), que é onde a lei antiga estava CERTA e onde o teto não
/// deveria agir.
fn franja_sobre_tinta() {
    println!("\n=== A FRANJA SOBRE TINTA EXISTENTE (flanco RETO — o preço do teto) ===");
    let size = SIZE;
    // Uma faixa de tinta no SOURCE (o vizinho que a franja pode empalidecer).
    let mut src = vec![255u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            if (100..156).contains(&y) {
                let i = ((y * size + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[40u8, 90, 200, 255]);
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    arm(&mut t, wash_brush());
    // Lavagem VERTICAL cruzando a faixa: o flanco reto dela fica em x = C ± R.
    stroke(&mut t, [C, 40.0], [C, 216.0]);
    println!("      x-C   R    G    B      (o flanco reto da lavagem, atravessando o ombro)");
    for dx in 12..28u32 {
        let x = C as u32 + dx;
        let i = ((128 * size + x) * 4) as usize;
        println!(
            "   {dx:5}   {:3}  {:3}  {:3}",
            t.canvas_rgba[i],
            t.canvas_rgba[i + 1],
            t.canvas_rgba[i + 2]
        );
    }
}
