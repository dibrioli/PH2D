//! **O ALPHA DENTRO DA LEI DO TRAÇO** — os gates que decidem se o mapeamento 3D
//! era mesmo necessário.
//!
//! O `alpha_tests.rs` prova que cada padrão é um stencil. Aqui prova-se a coisa
//! que motivou o desenho inteiro: **um padrão lido na posição CONGELADA
//! sobrevive ao envelope, e um lido na posição VIVA não.**
//!
//! ⚠️ Os dois primeiros gates são a mesma frase em dois eixos — *o resultado é
//! função do CAMINHO, nunca de quão fino nem em que ORDEM o motor o amostrou* —
//! e é a frase que o cabeçalho do `stroke.rs` chama de a lei da casa. Foi ela
//! que a `line/Painter` pagou quatro vezes em 2D; se o alpha a quebrasse, ele
//! seria uma feature nova construída sobre o bug que este módulo existe para não
//! ter.

use super::*;
use crate::Alpha;
use ph2d_mesh::{Mesh, shapes};

/// ⚠️ **A esfera destes gates é a SUBDIVIDIDA, e a escolha é obrigatória.**
///
/// A lei das dez arestas (ver [`crate::DEFAULT_ALPHA_SCALE`]) diz que uma
/// feature precisa de ~10 arestas para ser amostrada como padrão em vez de como
/// chuvisco. A esfera 32×48 que as outras suítes usam tem aresta `0,098`, então
/// **nenhuma escala** a resolve: um gate escrito sobre ela mediria o aliasing e
/// concluiria coisas sobre o alpha que são sobre a malha. Esta tem aresta
/// `0,0245`, e com `ALPHA_SCALE = 0,20` a razão é 8,2 — resolvido, e ainda com
/// ~3,5 features atravessando a pegada, que é o que faz o padrão ser *visível
/// dentro de um dab* em vez de um multiplicador quase constante.
fn sphere() -> Mesh {
    shapes::uv_sphere(128, 192, 1.0)
}

const R: f32 = 0.35;
const ALPHA_SCALE: f32 = 0.20;

fn textured(verb: Verb) -> Brush {
    Brush {
        verb,
        strength: 0.8,
        radius: R,
        alpha: Some(Alpha::Pores),
        alpha_scale: ALPHA_SCALE,
        ..Brush::default()
    }
}

/// Uma passada reta pelo topo da esfera, com `n` dabs distribuídos no MESMO
/// caminho — é o `n` que muda entre as duas metades do gate da densidade.
fn sweep(mesh: &mut Mesh, brush: &Brush, dabs: usize, reverse: bool) {
    let mut stroke = SculptStroke::default();
    stroke.begin(mesh);
    let (from, to) = (-0.5_f32, 0.5_f32);
    let step = (to - from) / dabs as f32;
    let order: Vec<usize> = if reverse {
        (0..=dabs).rev().collect()
    } else {
        (0..=dabs).collect()
    };
    for k in order {
        let x = step.mul_add(k as f32, from);
        let z = (1.0 - x * x).max(0.0).sqrt();
        stroke.dab(
            mesh,
            brush,
            &Dab::at([x, 0.0, z], R, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
}

/// O deslocamento de cada vértice em relação à esfera de partida.
fn moves(before: &Mesh, after: &Mesh) -> Vec<f32> {
    before
        .positions()
        .iter()
        .zip(after.positions())
        .map(|(b, a)| {
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1])).sqrt()
        })
        .collect()
}

/// O maior desacordo entre dois campos de deslocamento, e o pico de referência.
fn worst(a: &[f32], b: &[f32]) -> (f32, f32) {
    let mut worst = 0.0f32;
    let mut peak = 0.0f32;
    for (&x, &y) in a.iter().zip(b) {
        worst = worst.max((x - y).abs());
        peak = peak.max(x.abs()).max(y.abs());
    }
    (worst, peak)
}

/// **O PADRÃO SOBREVIVE AO ENVELOPE: o mesmo caminho, amostrado três vezes mais
/// fino, dá o mesmo resultado.**
///
/// ⚠️ **É o gate que justifica o mapeamento 3D.** Com o alpha lido na posição
/// VIVA cada dab amostraria o padrão num lugar diferente — o próprio traço move
/// a superfície ao longo da normal, e o deslocamento (`raio × REACH_FRACTION`,
/// aqui 0,07) é da ordem de uma célula do padrão (0,05). O `max` do envelope
/// tomaria então o maior de dezenas de amostras independentes, e **quanto mais
/// dabs, maior esse máximo**: o padrão lavaria para a envoltória superior dele e
/// o pincel ficaria mais forte em vez de texturizado.
/// ⚠️ **O CONTROLE é o mesmo traço sem alpha, e ele não é opcional.** Doze e
/// trinta e seis dabs põem os centros em lugares diferentes, então o envelope do
/// FALLOFF já discorda um pouco entre as duas — é a discretização de varrer um
/// disco, e ela existe com ou sem padrão. Um gate que afirmasse um número
/// absoluto estaria medindo essa discretização e chamando-a de alpha.
fn density_disagreement(armed: bool) -> f32 {
    let base = sphere();
    let mut brush = textured(Verb::Draw);
    if !armed {
        brush.alpha = None;
    }
    let mut coarse = sphere();
    sweep(&mut coarse, &brush, 12, false);
    let mut fine = sphere();
    sweep(&mut fine, &brush, 36, false);
    let (a, b) = (moves(&base, &coarse), moves(&base, &fine));
    let (worst, peak) = worst(&a, &b);
    assert!(peak > 0.01, "a fixture não esculpiu nada (pico {peak})");
    worst / peak
}

#[test]
fn the_pattern_does_not_depend_on_how_finely_the_path_was_sampled() {
    let plain = density_disagreement(false);
    let armed = density_disagreement(true);
    assert!(
        armed < plain.max(0.005) * 2.0,
        "sem alpha, 12 e 36 dabs discordam em {plain:.4} do pico; com alpha, em \
         {armed:.4} — o padrão está sendo lavado pelo envelope"
    );
}

/// O desacordo entre percorrer o MESMO caminho de ida e de volta.
///
/// ⚠️ **O CONTROLE não é opcional aqui, e ele nasceu de uma medição que
/// derrubou a versão anterior deste gate.** Ele afirmava um número ABSOLUTO
/// (`< 2 % do pico`) e ficou vermelho em 3,68 % quando a família do carimbo
/// trocou de lei — e a causa **não era o padrão**: medido, com alpha 3,68 % e
/// **sem alpha 3,67 %**, indistinguíveis. Quem produz o desacordo é a
/// **SUPERFÍCIE MOVIDA** — a pegada (`verts_in_sphere`) sai das posições vivas,
/// então um depósito que vale uma fração grande do raio muda *quem está sob o
/// pincel* e o segundo dab de uma direção não vê o mesmo conjunto que o da
/// outra. Ela escala com a força, que é a assinatura: `0,8 → 3,68 %` ·
/// `0,08 → 0,01 %` · `0,008 → 0,01 %`.
///
/// ⚠️ **Isso é inerente a uma lei que COMPÕE, e a referência tem o mesmo** (o
/// `sculptStroke` dela também consulta a malha viva por dab). Sob o envelope
/// não aparecia porque o `max` é idempotente: um vértice que saía da esfera já
/// tinha o valor dele carimbado.
fn order_disagreement(armed: bool) -> f32 {
    let base = sphere();
    let mut brush = textured(Verb::Draw);
    if !armed {
        brush.alpha = None;
    }
    let mut forward = sphere();
    sweep(&mut forward, &brush, 24, false);
    let mut backward = sphere();
    sweep(&mut backward, &brush, 24, true);

    let (a, b) = (moves(&base, &forward), moves(&base, &backward));
    let (worst, peak) = worst(&a, &b);
    assert!(peak > 0.01, "a fixture não esculpiu nada (pico {peak})");
    worst / peak
}

/// **E o PADRÃO não depende da ORDEM em que o caminho foi percorrido.**
///
/// O irmão do gate acima no outro eixo. Ele sozinho ficaria verde num motor que
/// simplesmente ignorasse o alpha, e o de cima ficaria verde num motor que o
/// aplicasse por índice de dab — juntos, eles só passam se o padrão for função
/// da POSIÇÃO.
///
/// ⚠️ **O oráculo é o CONTROLE, não um limiar** — ver [`order_disagreement`]:
/// um número absoluto aqui mede a superfície movida e chama-a de alpha, que é
/// literalmente a crítica que o doc do gate da densidade já fazia ao irmão
/// dele. Um motor que aplicasse o padrão por ÍNDICE DE DAB faria a metade
/// armada disparar contra a desarmada, que é o que este gate existe para pegar.
#[test]
fn the_pattern_does_not_depend_on_which_way_the_stroke_was_walked() {
    let plain = order_disagreement(false);
    let armed = order_disagreement(true);
    assert!(
        armed < plain.max(0.005) * 2.0,
        "sem alpha, ida e volta discordam em {plain:.4} do pico; com alpha, em \
         {armed:.4} — o padrão está sendo aplicado por índice de dab, não por \
         posição"
    );
}

/// A dispersão do deslocamento dentro de um ANEL de raio quase constante — onde
/// o falloff é praticamente o mesmo, e portanto o que sobra é o alpha.
fn spread_in_the_annulus(before: &Mesh, after: &Mesh, centre: [f32; 3]) -> f32 {
    let d = moves(before, after);
    let mut ring: Vec<f32> = before
        .positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let v = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            let r = v[2].mul_add(v[2], v[0].mul_add(v[0], v[1] * v[1])).sqrt();
            // ⚠️ **A faixa é ESTREITA de propósito, e o gate confere o
            // controle.** Em `0,30 R .. 0,55 R` o Smooth já cai de 0,77 para
            // 0,64 — 0,29 de dispersão medida — e o gate estaria pesando o
            // falloff. Aqui ele cai de 0,73 para 0,62.
            (0.38 * R..0.46 * R).contains(&r)
        })
        .map(|(i, _)| d[i])
        .collect();
    assert!(
        ring.len() > 30,
        "o anel do fixture tem {} vértices",
        ring.len()
    );
    ring.sort_by(f32::total_cmp);
    let hi = ring[ring.len() * 9 / 10];
    let lo = ring[ring.len() / 10];
    if hi <= 0.0 { 0.0 } else { (hi - lo) / hi }
}

/// **O alpha MODULA dentro da pegada** — e o controle é o mesmo dab sem ele.
///
/// ⚠️ **O oráculo é medido num ANEL**, não na pegada inteira: sobre a pegada
/// toda o falloff sozinho já produz uma dispersão enorme (do pico à borda), e um
/// gate que a medisse ficaria verde com o alpha desligado. Dentro de um anel
/// estreito o falloff é quase constante, então o que sobra é o padrão.
#[test]
fn the_alpha_carves_variation_where_the_falloff_alone_is_flat() {
    let centre = [0.0, 0.0, 1.0];
    let dab = Dab::at(centre, R, [0.0, 0.0, -1.0]);

    let mut plain_spread = 0.0;
    let mut alpha_spread = 0.0;
    for (armed, out) in [(false, &mut plain_spread), (true, &mut alpha_spread)] {
        let base = sphere();
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let mut brush = textured(Verb::Draw);
        if !armed {
            brush.alpha = None;
        }
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
        *out = spread_in_the_annulus(&base, &mesh, centre);
    }
    assert!(
        plain_spread < 0.25,
        "o CONTROLE já é disperso ({plain_spread:.3}) — o anel está largo demais \
         e o gate mediria o falloff, não o alpha"
    );
    assert!(
        alpha_spread > 0.8,
        "com o alpha armado o anel varia só {alpha_spread:.3} — o padrão não recorta"
    );
}

/// **NENHUM verbo ignora o alpha** — os dezesseis, varridos.
///
/// ⚠️ Este é o gate da *capacidade sem porta*: o alpha entra por uma linha só, e
/// se algum grip contornasse aquela linha ele teria um chip que não faz nada
/// naquela ferramenta — o controle morto que esta casa varre a cada wave. A
/// varredura é por [`Verb::ALL`] e não por uma lista escrita aqui, então um
/// verbo novo nasce coberto.
///
/// ⚠️ **O verbo de máscara é comparado pela MÁSCARA**, porque ele não move
/// geometria: compará-lo por posição o daria como "inalterado" e o gate mentiria
/// exatamente sobre a única ferramenta cuja saída é outro canal.
#[test]
fn every_verb_reads_the_alpha() {
    for verb in Verb::ALL {
        let mut snap: [Vec<f32>; 2] = [Vec::new(), Vec::new()];
        for (k, armed) in [false, true].into_iter().enumerate() {
            let mut mesh = sphere();
            mesh.put_masks(vec![ph2d_mesh::DEFAULT_MASK; mesh.vert_count()]);
            let mut stroke = SculptStroke::default();
            stroke.begin(&mesh);
            let mut brush = textured(verb);
            if !armed {
                brush.alpha = None;
            }
            // Dois dabs, e o segundo deslocado: os verbos de âncora precisam de
            // um movimento para terem o que carregar.
            for x in [0.0_f32, 0.08] {
                let z = (1.0 - x * x).max(0.0).sqrt();
                stroke.dab(
                    &mut mesh,
                    &brush,
                    &Dab::pulling([x, 0.0, z], R, [0.0, 0.0, -1.0], [0.05, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            snap[k] = if verb.paints_mask() {
                mesh.masks().expect("a máscara existe").to_vec()
            } else {
                mesh.positions().iter().flat_map(|p| *p).collect()
            };
        }
        assert!(
            snap[0].iter().zip(&snap[1]).any(|(a, b)| a != b),
            "o verbo {verb:?} deu EXATAMENTE o mesmo resultado com e sem alpha — \
             ele não passa pela porta"
        );
    }
}

/// O deslocamento de UM dab, por vértice, com e sem o alpha armado.
///
/// Um dab só, e é isso que torna o oráculo exato: com vários, o `max` do
/// envelope pode eleger dabs diferentes nas duas corridas e a razão por vértice
/// deixaria de comparar a mesma expressão.
fn one_dab_with_and_without(verb: Verb) -> (Vec<f32>, Vec<f32>) {
    let dab = Dab::at([0.0, 0.0, 1.0], R, [0.0, 0.0, -1.0]);
    let mut out = [Vec::new(), Vec::new()];
    for (k, armed) in [false, true].into_iter().enumerate() {
        let base = sphere();
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let mut brush = textured(verb);
        // Sem o aperto lateral: o termo do pinch é LINEAR no peso enquanto o
        // termo normal leva o expoente, e misturá-los borraria a única coisa que
        // este gate mede.
        brush.pinch = 0.0;
        // ⚠️ **E o padrão é o DENSO.** Com o `Pores` (cobertura medida 0,14) só
        // 62 vértices da pegada tinham `α` alto o bastante para a razão dizer
        // alguma coisa — o gate morria por amostra, não por defeito. O `Noise`
        // cobre 0,50 e enche a faixa útil.
        brush.alpha = Some(Alpha::Noise);
        if !armed {
            brush.alpha = None;
        }
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
        out[k] = moves(&base, &mesh);
    }
    let [plain, armed] = out;
    (plain, armed)
}

/// **O CREASE AFIA O PADRÃO** — a prova de que o alpha vive no `shape`, e o
/// oráculo é a ÁLGEBRA e não um percentil.
///
/// O `shape` é lido por um verbo só, o Crease, que o eleva à quarta
/// (`Crease.js:68`, `pow(fallOff, 5)`, com o quinto expoente vindo do `accum`).
/// Com o pinch zerado o deslocamento de um dab é, por vértice:
///
/// ```text
/// Draw:    d = accum · reach                    = fall · α · intensidade · reach
/// Crease:  d = accum · reach · shape⁴           = fall · α · intensidade · reach · (fall · α)⁴
/// ```
///
/// então a razão *armado ÷ liso* do MESMO vértice vale `α` no Draw e `α⁵` no
/// Crease. **`r_crease = r_draw⁵`, exato** — e é isso que o gate afirma.
///
/// ⚠️ **Multiplicar o alpha no `w` já formado** — a alternativa que compila,
/// pinta igual e passa em todo o resto — deixaria o padrão **fora** daquele
/// expoente, e a mesma medição daria expoente **1**: o vinco afiaria a máscara e
/// o falloff e passaria por cima do padrão. O gate anterior a este comparava
/// percentis e mediu 2,16 contra 3,12 — o efeito estava lá e o oráculo o
/// achatava, porque `α⁵` empurra metade da pegada para baixo do piso de
/// "moveu-se" e a mediana sobe junto.
#[test]
fn the_crease_sharpens_the_alpha_because_it_lives_inside_the_shape() {
    let (draw_plain, draw_armed) = one_dab_with_and_without(Verb::Draw);
    let (cr_plain, cr_armed) = one_dab_with_and_without(Verb::Crease);

    // ⚠️ **O ORÁCULO É O EXPOENTE, e não a razão — a diferença é
    // CONDICIONAMENTO.** Afirmar `r_crease == r_draw⁵` compara dois números
    // pequenos: um deslocamento sai da diferença de duas posições `f32` de
    // magnitude ~1, carrega ~1e-3 de erro relativo perto do piso, e elevar
    // `r_draw` à quinta **multiplica esse erro por cinco**. Medido, a cauda
    // chegava a 2,79% com mediana 0,0023% — ruído de representação vestido de
    // desacordo de modelo. Em `ln(r_crease) / ln(r_draw)` o mesmo ruído entra
    // dividido pelo logaritmo e a grandeza afirmada passa a ser **o expoente**,
    // que é literalmente o que a álgebra do vinco diz: **CINCO**.
    //
    // ⚠️ E é a mesma frase que sangra sob a mutação: com o alpha multiplicado no
    // `w` já formado o padrão sai de dentro do `shape⁴` e o expoente vira **UM**.
    let mut expo: Vec<f32> = Vec::new();
    for i in 0..draw_plain.len() {
        // Só onde os dois traços depositam de verdade: perto da borda do pincel
        // as duas medidas são ruído de `f32` e a razão delas não diz nada.
        if draw_plain[i] < 1e-4 || cr_plain[i] < 1e-4 {
            continue;
        }
        let r_draw = draw_armed[i] / draw_plain[i];
        // A faixa útil de `α`: perto de 1 o logaritmo vai a zero e o quociente
        // explode; perto de 0 o `α⁵` cai sob a precisão da posição.
        if !(0.35..0.90).contains(&r_draw) {
            continue;
        }
        let r_crease = cr_armed[i] / cr_plain[i];
        if r_crease <= 0.0 {
            continue;
        }
        expo.push(r_crease.ln() / r_draw.ln());
    }
    assert!(
        expo.len() > 100,
        "só {} vértices entraram na medição",
        expo.len()
    );
    expo.sort_by(f32::total_cmp);
    let (lo, hi) = (expo[0], expo[expo.len() - 1]);
    assert!(
        (4.95..=5.05).contains(&lo) && (4.95..=5.05).contains(&hi),
        "o expoente com que o vinco lê o padrão ficou em [{lo:.3}, {hi:.3}] \
         ({} vértices) — esperado 5, que é `shape⁴` vezes o `accum`; em 1 o \
         padrão está FORA do expoente do vinco",
        expo.len()
    );
}
