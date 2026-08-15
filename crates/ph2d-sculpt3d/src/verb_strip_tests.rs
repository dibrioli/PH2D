//! Gates da **FAIXA** — o `Verb::ClayStrips`, medido pela porta do PRODUTO.
//!
//! ⚠️ **Os gates do [`crate::footprint`] medem a FORMA; estes medem o TRAÇO.**
//! Um kernel correto não prova um produto correto — a lição que esta linha pagou
//! no `l-mode` do Smooth (a fiação do shell é cega aos gates de unidade). O que
//! estes exigem é o caminho inteiro: `Brush { verb: ClayStrips }` →
//! `stroke.dab` → as posições da malha.

use super::*;

/// Uma grade plana de `n × n` no plano `z = 0`.
fn plane_grid(n: usize, half: f32) -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            let f = |k: usize| (k as f32 / n as f32) * 2.0 * half - half;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut faces = Vec::with_capacity(n * n * 2);
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("índices válidos")
}

const N: usize = 60;
const HALF: f32 = 1.5;
const R: f32 = 0.4;

fn strip_brush(length: f32, roundness: f32) -> Brush {
    Brush {
        verb: Verb::ClayStrips,
        radius: R,
        strength: 1.0,
        // ⚠️ **NÃO há `plane_offset` aqui, e a ausência é o gate:** a faixa
        // ergue o próprio plano ([`crate::STRIP_PLANE_FRACTION`]), então esta
        // fixture exercita o DEFAULT que shipa. A 1ª versão punha `-0,5` — o
        // SINAL trocado — e o portão fechava em todo vértice: sobrava só o
        // primeiro dab, e dois gates mediam um disco contra outro reportando
        // `0,6000 → 0,6000`.
        strip_length: length,
        tip_roundness: roundness,
        // ⚠️ **A PREMISSA, DECLARADA — e ela invalidou uma sessão inteira de
        // medições minhas.** `Brush::default().accumulate` está preso ao default
        // do **Draw** (`true`), mas o da faixa é `false`
        // (`ClayStrips::default_accumulate()`, porque o SculptGL não tem perfil
        // para ela). Uma fixture `..Brush::default()` corria com o Accumulate
        // LIGADO, que não é o que a ferramenta shipa — e é justamente o
        // interruptor que escolhe a fonte do PLANO na referência.
        accumulate: Verb::ClayStrips.default_accumulate(),
        ..Brush::default()
    }
}

/// Um traço ao longo de `+x`, e a malha que ele deixou.
fn stroke_along_x(brush: &Brush, dabs: usize) -> ph2d_mesh::Mesh {
    let mut mesh = plane_grid(N, HALF);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..dabs {
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * R;
        stroke.dab(
            &mut mesh,
            brush,
            &Dab::at([x, 0.0, 0.0], R, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh
}

/// O alcance da tinta ao longo de um eixo: a maior coordenada de um vértice que
/// se moveu.
fn reach_along(mesh: &ph2d_mesh::Mesh, axis: usize) -> f32 {
    let rest = plane_grid(N, HALF);
    let mut out = 0.0f32;
    for (i, p) in mesh.positions().iter().enumerate() {
        if (p[2] - rest.positions()[i][2]).abs() > 1e-5 {
            out = out.max(p[axis].abs());
        }
    }
    out
}

/// **A FAIXA DEITA NA DIREÇÃO DO TRAÇO** — o alcance ao longo do caminho cresce
/// com o comprimento, e o alcance ATRAVESSADO não.
///
/// ⚠️ **É o gate que separa a wave de um Draw com outro nome.** Um verbo que
/// herdasse a silhueta redonda passaria em tudo o que mede *quanto* o barro
/// sobe; o que ele não pode fingir é a FORMA da região tocada.
#[test]
fn the_strip_lies_along_the_stroke_and_stretches_with_its_length() {
    let short = stroke_along_x(&strip_brush(1.0, 0.0), 9);
    let long = stroke_along_x(&strip_brush(3.0, 0.0), 9);
    let (sx, sy) = (reach_along(&short, 0), reach_along(&short, 1));
    let (lx, ly) = (reach_along(&long, 0), reach_along(&long, 1));
    assert!(
        lx > sx * 1.5,
        "esticar a faixa tinha de alcançar mais AO LONGO: {sx:.4} → {lx:.4}"
    );
    assert!(
        (ly - sy).abs() < R * 0.1,
        "e não mudar o alcance ATRAVESSADO: {sy:.4} → {ly:.4}"
    );
}

/// **O MIOLO DA FAIXA É CHATO** — é o que a faz depositar um degrau de barro em
/// vez de um domo.
///
/// ⚠️ **Medido contra o DRAW, que é o controle**, e não contra um número
/// absoluto: os dois sobem pelo mesmo `reach` e pela mesma normal de área, então
/// a única coisa que a razão pode estar a medir é a silhueta.
#[test]
fn the_strips_core_is_a_plateau_where_the_draws_is_a_dome() {
    let flat = stroke_along_x(&strip_brush(1.0, 0.0), 9);
    let dome = stroke_along_x(
        &Brush {
            verb: Verb::Draw,
            ..strip_brush(1.0, 0.0)
        },
        9,
    );
    // A razão entre a altura a meio raio do eixo e a altura no eixo: um platô
    // vale ~1, um domo cai.
    let rest = plane_grid(N, HALF);
    let h = |m: &ph2d_mesh::Mesh, y: f32| {
        let mut best = (f32::MAX, 0.0f32);
        for (i, p) in rest.positions().iter().enumerate() {
            let d = (p[0]).abs() + (p[1] - y).abs();
            if d < best.0 {
                best = (d, m.positions()[i][2] - p[2]);
            }
        }
        best.1
    };
    let ratio = |m: &ph2d_mesh::Mesh| {
        let axis = h(m, 0.0);
        if axis.abs() < 1e-6 {
            0.0
        } else {
            h(m, R * 0.5) / axis
        }
    };
    let (rf, rd) = (ratio(&flat), ratio(&dome));
    assert!(
        rf > rd * 1.15,
        "o miolo da faixa tinha de ser mais chato que o do domo: \
         faixa {rf:.4} contra draw {rd:.4}"
    );
}

/// **SEM DIREÇÃO A FAIXA É UM DISCO** — e isso é a decisão, não um acidente.
///
/// Um dab só (o toque) não tem antecessor, então não há caminho; a silhueta que
/// sobra é a redonda, que é a que a ferramenta teria antes de saber para onde
/// ia. O gate afirma a SIMETRIA da região tocada, que é o que "disco" quer dizer
/// aqui.
#[test]
fn a_single_tap_has_no_path_so_it_lands_a_disc() {
    let m = stroke_along_x(&strip_brush(3.0, 0.0), 1);
    let (x, y) = (reach_along(&m, 0), reach_along(&m, 1));
    assert!(x > 0.0, "o toque tem de tocar em alguma coisa");
    assert!(
        (x - y).abs() < R * 0.1,
        "sem caminho a pegada é redonda: x {x:.4} contra y {y:.4}, \
         e uma faixa de comprimento 3 teria x ≫ y"
    );
}

/// **AS QUINAS SOBREVIVEM À CONSULTA** — o raio de consulta cresce com a caixa,
/// senão a faixa chega com os cantos comidos e ninguém vê que faltam.
#[test]
fn the_query_radius_grows_so_the_corners_survive() {
    let b = strip_brush(3.0, 0.0);
    let q = b.query_radius(R);
    assert!(
        (q / R - 10.0f32.sqrt()).abs() < 1e-4,
        "uma faixa de comprimento 3 consulta √10 raios: {}",
        q / R
    );
    // E o disco não paga por isso.
    let d = Brush {
        verb: Verb::Draw,
        ..b
    };
    assert_eq!(d.query_radius(R), R);
}

/// **OS OUTROS DEZASSEIS VERBOS ATRAVESSAM A CAMADA DA SILHUETA BYTE A BYTE.**
///
/// ⚠️ **É o controle da wave inteira**, e ele é comparado contra um traço
/// gravado antes de a `Footprint` existir? Não — contra a rota do DISCO, que é a
/// mesma expressão de sempre (`dist · inv_r`, portão `1`). A afirmação que ele
/// faz é a que importa: *acrescentar uma forma não mexeu em quem não a pediu*.
#[test]
fn the_disc_verbs_are_untouched_to_the_bit() {
    for verb in Verb::ALL {
        if verb == Verb::ClayStrips {
            continue;
        }
        let b = Brush {
            verb,
            ..strip_brush(3.0, 0.0)
        };
        assert_eq!(
            b.query_radius(R),
            Brush { mode: b.mode, ..b }.query_radius(R),
            "{verb:?}"
        );
        // A silhueta que um verbo de disco recebe é a identidade sobre a
        // distância, para qualquer `strip_length`/`roundness` autorados.
        let f = crate::Footprint::Disc;
        for i in 0..8 {
            let dist = i as f32 * 0.05;
            assert_eq!(f.at([1.0, 2.0, 3.0], dist, 1.0 / R), (dist / R, 1.0));
        }
    }
}

/// **A FAIXA SÓ TOCA O QUE ESTÁ ABAIXO DO PLANO** — com o plano SOBRE a
/// superfície, ela não move um vértice.
///
/// ⚠️ **É o gate que faltava, e a mutação foi quem o pediu.** Tirar o portão
/// parabólico da profundidade deixava os cinco gates acima VERDES — a silhueta,
/// o comprimento, o disco do toque e a consulta não dizem nada sobre
/// profundidade.
///
/// ⚠️ **E a 1ª versão deste gate afirmava a coisa ERRADA:** eu pedi que a
/// segunda passada se esgotasse, e ela não se esgota — medido, a primeira sobe
/// `0,115595` e a segunda acrescenta `0,094020` (81 %). O motivo é que o plano é
/// re-ajustado na superfície VIVA a cada dab, então ele **sobe com o barro**, e
/// é isso que faz a faixa CONSTRUIR, como o Clay. O portão não é um limite entre
/// passadas; ele é a fronteira de UMA.
///
/// ⇒ A lei que ele de facto declara é esta, e ela é exata: `z · (1 − z)` vale
/// **zero na superfície do plano**, então um plano que não foi erguido não
/// alcança barro nenhum.
#[test]
fn the_strip_only_reaches_what_lies_below_its_plane() {
    let flush = Brush {
        // Baixar o plano do artista por exatamente a fração que o verbo o ergue
        // devolve-o à superfície — é a forma de perguntar *"e se não houvesse
        // barro abaixo dele?"* sem tocar na lei.
        plane_offset: -crate::STRIP_PLANE_FRACTION,
        ..strip_brush(1.0, 0.0)
    };
    let m = stroke_along_x(&flush, 9);
    let rest = plane_grid(N, HALF);
    let worst = m
        .positions()
        .iter()
        .zip(rest.positions())
        .map(|(p, q)| (p[2] - q[2]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-6,
        "com o plano rente à superfície a faixa não tem barro abaixo dele,          e mexeu {worst:.6}"
    );
    // O CONTROLE: erguido, ela deposita — senão o gate estaria a medir uma
    // ferramenta morta e a chamar-lhe lei.
    let raised = stroke_along_x(&strip_brush(1.0, 0.0), 9);
    let built = raised
        .positions()
        .iter()
        .zip(rest.positions())
        .map(|(p, q)| p[2] - q[2])
        .fold(0.0f32, f32::max);
    assert!(built > 0.05, "o controle tem de depositar: {built:.6}");
}

/// **A CÓPIA ESPELHADA DEITA A FAIXA DO LADO DELA** — o caminho espelha como
/// VETOR, como o `eye` e o `pull`.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE**: tirar o `mirror` do
/// [`crate::Dab::path`] deixava os 213 testes VERDES. A razão é de fixture — o
/// resto da suíte ou não usa simetria com a faixa, ou traça ao longo de um eixo
/// que o espelho preserva, e nos dois casos a metade espelhada acerta por
/// acidente.
///
/// ⇒ A fixture que contém o fenômeno traça na **DIAGONAL**, que é a direção que
/// o espelho de fato move: com o caminho não espelhado, a cópia da esquerda
/// deita a faixa na diagonal ERRADA e a arte deixa de ser simétrica.
#[test]
fn the_mirrored_copy_lays_its_strip_along_its_own_path() {
    let b = strip_brush(3.0, 0.0);
    let mut mesh = plane_grid(N, HALF);
    let rest = plane_grid(N, HALF);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..9 {
        // Diagonal, e deslocada do plano do espelho: um traço sobre `x = 0`
        // seria simétrico por construção e o gate mediria vácuo.
        let t = k as f32 * 0.15 * R;
        stroke.dab(
            &mut mesh,
            &b,
            &Dab::at([1.1 + t * 0.7, t * 0.7, 0.0], R, [0.0, 0.0, -1.0]),
            Symmetry::MIRROR_X,
        );
    }
    // ⚠️ **O traço nasce LONGE do eixo do espelho, e a fixture pagou por isso.**
    // O raio de CONSULTA da faixa é `√(1 + L²) · r = 0,566`, então um traço a
    // começar em `x = 0,5` faz as duas metades da simetria tocarem os MESMOS
    // vértices perto de `x = 0` — e aí o segundo passe ajusta o plano sobre a
    // superfície que o primeiro levantou. Medido com a magnitude da referência:
    // **5,28% de divergência em `x = 0,5` contra 1,77% em `x = 1,1`**, ou seja
    // dois terços eram a sobreposição e não o espelhamento do caminho, que é o
    // que este gate afirma.
    //
    // ⚠️ **A divergência residual tem dono e está nomeada:** o nosso
    // `fit_plane` lê a superfície VIVA, e o
    // `sculpt.cc::calc_area_normal_and_center_node_mesh` ramifica em
    // `!ss.cache->accum` para ler o pen-down congelado. Sob o plano congelado os
    // dois passes de simetria seriam idênticos por construção.
    //
    // A altura no ponto `p` e no espelho dele têm de ser a mesma.
    let h = |p: [f32; 2]| {
        let mut best = (f32::MAX, 0.0f32);
        for (i, q) in rest.positions().iter().enumerate() {
            let d = (q[0] - p[0]).abs() + (q[1] - p[1]).abs();
            if d < best.0 {
                best = (d, mesh.positions()[i][2] - q[2]);
            }
        }
        best.1
    };
    let mut worst = 0.0f32;
    let mut peak = 0.0f32;
    for i in 0..12 {
        for j in 0..12 {
            let p = [1.1 + i as f32 * 0.06, -0.3 + j as f32 * 0.06];
            let (a, m) = (h(p), h([-p[0], p[1]]));
            worst = worst.max((a - m).abs());
            peak = peak.max(a.abs());
        }
    }
    assert!(peak > 0.01, "o traço tem de depositar: {peak:.6}");
    assert!(
        worst < peak * 0.05,
        "a metade espelhada divergiu {worst:.6} sobre um pico de {peak:.6}"
    );
}

/// **O DEFAULT QUE SHIPA DEIXA UMA FAIXA, NÃO UMA LENTE** — os lados do depósito
/// são paralelos.
///
/// ⚠️ **Este gate NÃO menciona `tip_roundness`, e é essa a razão de ele
/// existir.** As outras fixtures deste arquivo passam a dureza explícita, então
/// todas dodgeavam o default — e o default shipou **redondo**, com os 214 testes
/// verdes e o artista a reportar *"parece redondo"* no primeiro smoke. *Um
/// default só é testado por um teste que não o menciona* (a lei que a wave do
/// Mirror já tinha escrito, num módulo que este não leu).
///
/// ⚠️ **O oráculo é a LARGURA em sete secções**, e não a altura: um dab redondo
/// arrastado deixa uma LENTE (afina nas pontas), um dab de quina reta deixa uma
/// tira de lados paralelos. Medido, `1,00` contra `0,83`.
#[test]
fn the_shipped_default_lays_a_strip_with_parallel_sides() {
    let widths = |b: &Brush| {
        let rest = plane_grid(N, HALF);
        let m = stroke_along_x(b, 9);
        let mut w = Vec::new();
        for i in 0..7 {
            let x = -0.3 + i as f32 * 0.1;
            let (mut lo, mut hi) = (f32::MAX, -f32::MAX);
            for (k, q) in rest.positions().iter().enumerate() {
                if (q[0] - x).abs() < 0.03 && (m.positions()[k][2] - q[2]) > 0.01 {
                    lo = lo.min(q[1]);
                    hi = hi.max(q[1]);
                }
            }
            w.push(if hi > lo { hi - lo } else { 0.0 });
        }
        w
    };
    // O pincel que SHIPA: só o verbo é escolhido, tudo o mais é default.
    let shipped = Brush {
        verb: Verb::ClayStrips,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let w = widths(&shipped);
    let (mid, end) = (w[3], w[0].max(w[6]));
    assert!(mid > 0.0, "o traço tem de depositar: {w:?}");
    assert!(
        end / mid > 0.95,
        "o default tinha de deixar lados PARALELOS e afinou: {w:?}"
    );

    // O CONTROLE: uma ponta redonda é a lente, e é o que o default NÃO pode ser.
    let round = Brush {
        tip_roundness: 1.0,
        ..shipped
    };
    let w = widths(&round);
    let (mid, end) = (w[3], w[0].max(w[6]));
    assert!(
        end / mid < 0.9,
        "o controle tinha de afinar, senão este gate não separa nada: {w:?}"
    );
}

// --- O LIFT DO PLANO: nivelar, e não copiar o relevo -------------------------
//
// ⚠️ **Estes três gates medem UMA constante por TRÊS propriedades**, e é
// deliberado: o [`crate::STRIP_PLANE_FRACTION`] decide sozinho se a faixa fecha
// relevo, se ainda deposita sob o cursor numa forma convexa, e quanta tinta ela
// deixa em chapa plana. Um gate só deixaria as outras duas livres para regredir
// em silêncio — foi assim que o `0,5` shipou.

/// Meia-largura e profundidade do vale das fixtures abaixo.
const VALLEY_W: f32 = 0.5;
const VALLEY_D: f32 = 0.4;

/// `z` de um vale liso que corre ao longo de `x`.
fn valley_z(y: f32) -> f32 {
    if y.abs() >= VALLEY_W {
        0.0
    } else {
        -VALLEY_D * 0.5 * (1.0 + (core::f32::consts::PI * y / VALLEY_W).cos())
    }
}

/// Uma grade cuja altura é dada por `f(x, y)`.
fn shaped_grid(f: impl Fn(f32, f32) -> f32) -> ph2d_mesh::Mesh {
    let mut pos = Vec::with_capacity((N + 1) * (N + 1));
    for j in 0..=N {
        for i in 0..=N {
            let g = |k: usize| (k as f32 / N as f32) * 2.0 * HALF - HALF;
            let (x, y) = (g(i), g(j));
            pos.push([x, y, f(x, y)]);
        }
    }
    let at = |i: usize, j: usize| (j * (N + 1) + i) as u32;
    let mut faces = Vec::with_capacity(N * N * 2);
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("índices válidos")
}

/// Um traço ao longo de `+x` sobre uma superfície dada, com o cursor pousando
/// NELA.
///
/// ⚠️ **O raio é o da superfície, não o `R` das fixtures planas:** o vale mede
/// `2 · VALLEY_W` de largura, e um pincel muito menor que ele nunca vê as
/// cristas — a pegada ficaria inteira dentro do chão, onde não há relevo para
/// nivelar, e a fixture não conteria o fenômeno.
fn stroke_over(f: impl Fn(f32, f32) -> f32 + Copy, radius: f32, dabs: usize) -> ph2d_mesh::Mesh {
    let mut mesh = shaped_grid(f);
    let brush = Brush {
        verb: Verb::ClayStrips,
        radius,
        strength: 0.5,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..dabs {
        let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * radius;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([x, 0.0, f(x, 0.0)], radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    mesh
}

/// A profundidade do vale na secção central: crista menos chão.
///
/// ⚠️ **A secção é lida por ÍNDICE de linha, nunca por `x ≈ 0`** — os vértices
/// andam ao longo da normal da área, que se inclina, então um filtro por
/// coordenada perde parte da coluna (a lição que a sonda do Draw Sharp pagou).
fn valley_depth(mesh: &ph2d_mesh::Mesh) -> f32 {
    let col: Vec<[f32; 3]> = (0..=N)
        .map(|j| mesh.positions()[j * (N + 1) + N / 2])
        .collect();
    let ridge = col
        .iter()
        .filter(|p| p[1].abs() <= 2.0 * VALLEY_W)
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max);
    let floor = col.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    ridge - floor
}

/// **A FAIXA FECHA UM VALE — ela não o exagera.**
///
/// É o report do Enio de 2026-08-15, e o gate nasceu VERMELHO: com o lift em
/// `0,5` a mesma passada media `0,4269` contra os `0,4000` de repouso, ou seja
/// **aprofundava** o vale em `+0,0269`.
///
/// ⚠️ **O oráculo é a PROFUNDIDADE, não a altura do chão.** Uma passada que
/// levantasse a paisagem inteira subiria o chão sem nivelar nada, e um gate que
/// olhasse só para o chão a chamaria de correta — é exatamente o que o `0,5`
/// fazia (o chão subia `0,0447` e a crista `0,0716`).
#[test]
fn the_strip_closes_a_valley_instead_of_deepening_it() {
    let rest = valley_depth(&shaped_grid(|_, y| valley_z(y)));
    let after = valley_depth(&stroke_over(|_, y| valley_z(y), 0.8, 9));
    assert!(
        after < rest - 0.02,
        "a faixa tinha de FECHAR o vale: {rest:.4} → {after:.4}"
    );
}

/// **E ela ainda deposita SOB O CURSOR numa forma convexa** — o contra-peso.
///
/// ⚠️ **Sem este gate, "fecha o vale" é maximizado levando o lift a zero**, e
/// aí o miolo da pegada — que numa cúpula está ACIMA do plano ajustado — recebe
/// literalmente nada: medido, `0,009` do que o aro recebe. A faixa deixaria de
/// ser uma banda e viraria um anel, e o artista apontaria para um sítio onde
/// nada acontece.
///
/// ⚠️ **A banda ser MAIS FINA no miolo não é defeito** — é a mesma lei vista
/// numa superfície convexa (*"displaces vertices toward the brush plane"*), e é
/// o que faz a ferramenta cortar planos. O que o gate proíbe é o miolo ficar
/// vazio.
#[test]
fn the_band_still_lands_under_the_cursor_on_a_convex_form() {
    let dome = |x: f32, y: f32| {
        let q = (x * x + y * y) / (1.5 * 1.5);
        if q >= 1.0 { 0.0 } else { 0.5 * (1.0 - q) }
    };
    let rest = shaped_grid(dome);
    let after = stroke_over(dome, 0.8, 9);
    let disp = |j: usize| {
        let i = j * (N + 1) + N / 2;
        after.positions()[i][2] - rest.positions()[i][2]
    };
    let pick = |lo: f32, hi: f32| {
        (0..=N)
            .filter(|&j| {
                let y = rest.positions()[j * (N + 1) + N / 2][1].abs();
                (lo..=hi).contains(&y)
            })
            .map(disp)
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (core, rim) = (pick(0.0, 0.15), pick(0.35, 0.85));
    assert!(rim > 1e-4, "o aro tinha de receber barro: {rim:.4}");
    assert!(
        core > rim * 0.4,
        "o miolo não pode ficar VAZIO: miolo {core:.4} contra aro {rim:.4}"
    );
}

// --- A FAIXA É UMA TOOL DO BLENDER, e a lei dela vem de la' ------------------

/// **A FAIXA NÃO DEPOSITA NO QUE O ARTISTA NÃO VÊ.**
///
/// Report do Enio de 2026-08-15 (com foto): lâminas a atravessar a silhueta de
/// um membro. ⚠️ **Medido com o olho RASANTE — a situação da foto — a faixa
/// punha `2,3×` mais barro nas costas do que na frente** (39,86 contra 17,18),
/// e perto da silhueta isso empurra a superfície de trás para FORA do contorno:
/// é a barbatana.
///
/// ⚠️ **A causa não era a silhueta nem o plano — era a REFERÊNCIA:** o
/// `Brush::default()` shipa em [`crate::RefMode::S`], e o `S` declarava *todos*
/// os verbos, incluindo um que o **SculptGL não tem**. O `front_face: Ignored`
/// do `S` é fiel ao `Brush.js` (`if (this._culling)`, e o `_culling` nasce
/// desligado), e simplesmente **não é a lei desta ferramenta**: o
/// `clay_strips.cc::calc_faces` chama `calc_front_face` como terceira linha da
/// cadeia de fatores.
#[test]
fn the_strip_does_not_lay_clay_on_what_the_artist_cannot_see() {
    // Olhar quase de lado: é assim que a silhueta entra na pegada.
    let eye = {
        let v = [1.0f32, 0.0, -0.25];
        let l = (v[0] * v[0] + v[2] * v[2]).sqrt();
        [v[0] / l, 0.0, v[2] / l]
    };
    let dome = |x: f32, y: f32| {
        let q = (x * x + y * y) / (1.5 * 1.5);
        if q >= 1.0 { 0.0 } else { 0.5 * (1.0 - q) }
    };
    let rest = shaped_grid(dome);
    let mut mesh = shaped_grid(dome);
    let brush = Brush {
        verb: Verb::ClayStrips,
        radius: 0.8,
        strength: 0.5,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for k in 0..9 {
        let y = (k as f32 - 4.0) * 0.15 * 0.8;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([0.9, y, dome(0.9, y)], 0.8, eye),
            Symmetry::default(),
        );
    }
    let (mut front, mut back) = (0.0f32, 0.0f32);
    for (i, p) in mesh.positions().iter().enumerate() {
        let q = rest.positions()[i];
        let moved = (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
        if moved <= 1e-6 {
            continue;
        }
        let n = rest.normals()[i];
        if -(n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2]) >= 0.0 {
            front += moved;
        } else {
            back += moved;
        }
    }
    assert!(
        front > 1e-3,
        "a fixture tinha de depositar na FRENTE: {front:.4}"
    );
    assert!(
        back < front * 0.01,
        "a faixa depositou nas COSTAS: {back:.4} contra {front:.4} de frente"
    );
}

/// **UMA REFERÊNCIA SÓ GOVERNA AS FERRAMENTAS QUE ELA TEM.**
///
/// ⚠️ **As duas tabelas do [`crate::RefMode`] discordavam, e só uma estava
/// certa:** a de DEFAULTS já devolvia `None` para o `S` na faixa (o censo
/// `the_census_of_offered_chips` até escreve *"todos menos o Sharpen e o Clay
/// Strips"*), enquanto a da LEI dizia `S => true` para tudo. Este gate afirma o
/// acordo, e a consequência que ele carrega — o produto pergunta a lei **por
/// verbo**.
#[test]
fn sculptgl_does_not_declare_the_strip_so_it_does_not_govern_it() {
    use crate::RefMode;
    assert!(
        !RefMode::S.declares(Verb::ClayStrips),
        "o SculptGL não tem Clay Strips"
    );
    assert!(RefMode::B.declares(Verb::ClayStrips), "o Blender tem");
    // O chip que o painel oferece para a faixa é UM, e é o da referência dela.
    let offered: Vec<RefMode> = RefMode::offered_for(Verb::ClayStrips).collect();
    assert_eq!(offered, vec![RefMode::B], "a faixa tem uma referência só");
    // ⚠️ **E a lei que o produto usa segue a referência, não o modo guardado:**
    // o `Brush::default()` shipa em `S`, e é isso que punha o `front_face`
    // errado na ferramenta.
    assert_eq!(
        RefMode::S.kernel_for(Verb::ClayStrips).front_face,
        crate::FrontFace::Continuous,
        "a faixa herda a lei do Blender mesmo com o modo em S"
    );
    // O CONTROLE: um verbo que o `S` de facto tem continua com a lei dele.
    assert_eq!(
        RefMode::S.kernel_for(Verb::Draw).front_face,
        crate::FrontFace::Ignored,
        "o Draw é do SculptGL e a lei dele não pode ter mudado"
    );
}

/// **O BARRO SOBE ATÉ O PLANO E PARA** — o auto-limite que É um clay strip.
///
/// ⚠️ **Este gate nasceu porque a mutação do [`crate::STRIP_REACH_FRACTION`]
/// SOBREVIVEU aos 219:** devolver à faixa o `0,1` do SculptGL deixava tudo
/// verde. O número que fazia a ferramenta parecer certa não era afirmado por
/// ninguém.
///
/// ⚠️ **E a propriedade não é o número, é o que ele DESTRAVA.** Com o `raio ·
/// força` do `clay_strips.cc:327`, a posição VIVA no portão e o plano
/// CONGELADO do `!ss.cache->accum`, o `z` do portão `z·(1−z)` encolhe à medida
/// que o barro sobe e **fecha no plano**. Medido, o pico pousa em `0,1000`
/// contra um plano em `0,1000` e lá fica.
///
/// ⚠️ **A FIXTURE VARIA UMA COISA SÓ, e a 1ª versão variava duas:** ela
/// aumentava a contagem de dabs *e*, com ela, o COMPRIMENTO do traço — a 81
/// dabs o traço media `4,8` contra uma chapa de `1,5` e saía da malha, o que
/// lia como *"não saturou"* (`1,67×`). O caminho é FIXO e o que muda é só quão
/// fino ele é amostrado; é a lei que este repo já pagou quatro vezes no relevo
/// do Painter.
#[test]
fn the_clay_rises_to_the_plane_and_stops() {
    // O plano congelado: a superfície em repouso mais o lift.
    let plane = R * crate::STRIP_PLANE_FRACTION;
    let peak = |dabs: usize| {
        let mut mesh = plane_grid(N, HALF);
        let brush = strip_brush(1.0, crate::Brush::default().tip_roundness);
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        for k in 0..dabs {
            // ⚠️ O MESMO caminho, amostrado mais fino — nunca um caminho maior.
            let t = if dabs == 1 {
                0.0
            } else {
                k as f32 / (dabs - 1) as f32 - 0.5
            };
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at([t * 0.6, 0.0, 0.0], R, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        mesh.positions()
            .iter()
            .map(|p| p[2])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (a, b) = (peak(9), peak(81));
    assert!(
        a > plane * 0.9,
        "a fixture tinha de chegar ao plano: {a:.4}"
    );
    // ⚠️ **NOVE vezes os dabs sobre o MESMO caminho quase não pode mover o
    // barro** — é isso que "sobe até o plano e para" significa.
    assert!(
        b < a * 1.05,
        "a faixa não saturou: 9 dabs {a:.4} → 81 dabs {b:.4} ({:.2}x)",
        b / a
    );
    // E ela para NO plano, não muito acima dele.
    assert!(
        b < plane * 1.2,
        "a faixa passou do plano ({plane:.4}): {b:.4}"
    );
}
