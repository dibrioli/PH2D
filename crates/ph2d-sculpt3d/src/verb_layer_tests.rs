//! **A DEMÃO** — a W8, e o que a separa do [`Verb::Draw`].
//!
//! ⚠️ **A propriedade que estes gates existem para pinar não é *"ela levanta
//! barro"*** — o Draw também levanta. É que ela **PARA numa altura escolhida**,
//! que o falloff é uma TAXA e não um perfil, e que o número não se move quando
//! o artista muda o pincel.

use crate::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};
use ph2d_mesh::{Face, Mesh};

/// Uma grade plana `n × n` em `z = 0`, meia-largura `half`, normal `+z`.
fn grid(n: usize, half: f32) -> Mesh {
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let f = |k: usize| (k as f32 / n as f32) * 2.0 * half - half;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade")
}

/// Um pincel de demão, com o falloff mais duro que temos para o platô ser
/// legível na malha.
fn coat_brush(radius: f32, height: f32) -> Brush {
    Brush {
        verb: Verb::Layer,
        radius,
        strength: 1.0,
        layer_height: height,
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

/// Carimba `dabs` dabs no mesmo ponto e devolve a malha.
fn lay(mesh: &mut Mesh, b: &Brush, dabs: usize) {
    let mut s = SculptStroke::default();
    s.begin(mesh);
    for _ in 0..dabs {
        s.dab(
            mesh,
            b,
            &Dab::at([0.0, 0.0, 0.0], b.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
}

fn peak(mesh: &Mesh) -> f32 {
    mesh.positions()
        .iter()
        .map(|p| p[2])
        .fold(f32::NEG_INFINITY, f32::max)
}

/// **A ENTREGA — a demão PARA na altura autorada, por mais que se insista.**
#[test]
fn the_coat_stops_at_the_authored_height() {
    let h = 0.1f32;
    let mut last = 0.0f32;
    for dabs in [1usize, 8, 64, 512] {
        let mut mesh = grid(60, 1.0);
        let b = coat_brush(0.4, h);
        lay(&mut mesh, &b, dabs);
        let p = peak(&mesh);
        assert!(
            p <= h + 1e-5,
            "{dabs} dabs passaram da altura autorada: {p} > {h}"
        );
        assert!(p >= last - 1e-6, "a demão RECUOU de {last} para {p}");
        last = p;
    }
    // E ela CHEGA lá — sem esta metade, um verbo inerte passaria.
    assert!(
        last > 0.99 * h,
        "a demão nunca fecha: parou em {last} de {h}"
    );
}

/// **O falloff é uma TAXA, não um perfil** — o topo é chato na altura autorada.
///
/// ⚠️ **O CONTROLE é o [`Verb::Draw`] na mesma pegada:** ele deposita `w · reach`
/// por dab, então o perfil dele **É** a curva do falloff. Sem o controle, um
/// gate que só olhasse a demão não saberia dizer se a chatice do topo é a lei ou
/// a curva `Constant` da fixture.
#[test]
fn the_falloff_is_a_rate_and_not_a_profile() {
    let h = 0.1f32;
    let r = 0.4f32;

    // Uma curva com OMBRO, para o perfil ter o que mostrar.
    let mut coat = coat_brush(r, h);
    coat.falloff = Falloff::Smooth;
    let mut mesh = grid(80, 1.0);
    lay(&mut mesh, &coat, 512);

    // Amostra o `z` a três frações do raio, dentro da pegada.
    let sample = |mesh: &Mesh, frac: f32| -> f32 {
        let want = frac * r;
        let mut best = (f32::MAX, 0.0f32);
        for p in mesh.positions() {
            let d = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let err = (d - want).abs();
            if err < best.0 {
                best = (err, p[2]);
            }
        }
        best.1
    };

    let (a, b, c) = (sample(&mesh, 0.1), sample(&mesh, 0.45), sample(&mesh, 0.8));
    println!("DEMÃO   z a 10 %/45 %/80 % do raio: {a:.5} {b:.5} {c:.5}");
    for (name, z) in [("10 %", a), ("45 %", b), ("80 %", c)] {
        assert!(
            (z - h).abs() < 0.02 * h,
            "a demão NÃO é chata a {name} do raio: {z} contra {h}"
        );
    }

    // O CONTROLE: o mesmo gesto com o Draw desenha a CURVA.
    let mut mesh2 = grid(80, 1.0);
    let draw = Brush {
        verb: Verb::Draw,
        radius: r,
        strength: 1.0,
        falloff: Falloff::Smooth,
        accumulate: true,
        ..Brush::default()
    };
    lay(&mut mesh2, &draw, 512);
    let (x, _y, z) = (
        sample(&mesh2, 0.1),
        sample(&mesh2, 0.45),
        sample(&mesh2, 0.8),
    );
    println!("CONTROLE Draw z a 10 %/80 % do raio: {x:.5} {z:.5}");
    // ⚠️ **A régua do CONTROLE é a MESMA que a demão acabou de passar** (2 % da
    // altura), e não um número escolhido: o que ele tem de mostrar é que a
    // fixture DISTINGUE um platô de um perfil. Medido, o Draw varia 41 % entre
    // as duas amostras e a demão varia 0 %.
    assert!(
        (z - x).abs() > 0.02 * x,
        "o CONTROLE é chato como a demão: a fixture não distingue taxa de \
         perfil ({x} contra {z})"
    );
}

/// **UM DAB NÃO É A DEMÃO INTEIRA** — a rampa existe, e é ela que faz a
/// ferramenta *construir* em vez de carimbar.
///
/// ⚠️ **Sem este gate, `unit_accum` ficar `true` na demão passaria despercebido:**
/// o alvo é a camada CHEIA, então com `accum = 1` o primeiro dab a deitaria
/// inteira — e todos os gates de teto continuariam VERDES, porque a altura
/// final é a mesma. O que se perderia é a única coisa que o artista sente.
#[test]
fn one_dab_is_not_the_whole_coat() {
    let h = 0.1f32;
    let r = 0.4f32;
    let mut b = coat_brush(r, h);
    b.falloff = Falloff::Smooth; // um peso < 1 fora do centro

    let sample = |mesh: &Mesh| -> f32 {
        let want = 0.45 * r;
        let mut best = (f32::MAX, 0.0f32);
        for p in mesh.positions() {
            let d = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let err = (d - want).abs();
            if err < best.0 {
                best = (err, p[2]);
            }
        }
        best.1
    };

    let mut one = grid(80, 1.0);
    lay(&mut one, &b, 1);
    let mut many = grid(80, 1.0);
    lay(&mut many, &b, 512);
    let (a, z) = (sample(&one), sample(&many));
    println!("a 45 % do raio — 1 dab {a:.5}  ·  512 dabs {z:.5}  (altura {h})");
    assert!(
        (z - h).abs() < 0.02 * h,
        "512 dabs não fecharam a demão a 45 % do raio: {z}"
    );
    assert!(
        a < 0.75 * z,
        "o primeiro dab já deitou a demão quase inteira ({a} de {z}): a rampa \
         desapareceu e a ferramenta virou um carimbo"
    );
    assert!(a > 0.0, "o primeiro dab não deitou nada: {a}");
}

/// **A ALTURA não segue o RAIO** — a frase inteira que separa a demão do Draw.
#[test]
fn the_height_does_not_follow_the_radius() {
    let h = 0.1f32;
    let mut peaks = Vec::new();
    for r in [0.2f32, 0.4, 0.8] {
        let mut mesh = grid(80, 1.4);
        lay(&mut mesh, &coat_brush(r, h), 512);
        peaks.push(peak(&mesh));
    }
    println!("DEMÃO   picos por raio 0,2/0,4/0,8: {peaks:?}");
    for p in &peaks {
        assert!(
            (p - h).abs() < 0.02 * h,
            "a demão mudou de altura com o raio: {peaks:?}"
        );
    }

    // O CONTROLE: o Draw deposita `força · raio · 0,1`, então ELE segue o raio.
    let mut draws = Vec::new();
    for r in [0.2f32, 0.8] {
        let mut mesh = grid(80, 1.4);
        let b = Brush {
            verb: Verb::Draw,
            radius: r,
            strength: 1.0,
            falloff: Falloff::Constant,
            ..Brush::default()
        };
        lay(&mut mesh, &b, 1);
        draws.push(peak(&mesh));
    }
    println!("CONTROLE Draw picos por raio 0,2/0,8: {draws:?}");
    assert!(
        draws[1] > 3.0 * draws[0],
        "o CONTROLE não segue o raio: a fixture não distingue as duas leis"
    );
}

/// **A CERCA DE CHESTERTON, executável** — o Draw e a demão deixaram de
/// colapsar, e o que os separa é **de que número cada teto é função**.
///
/// ⚠️ **A primeira versão deste gate pedia que o Draw passasse muito do teto da
/// demão, e a medição a derrubou:** com Accumulate ele para no **RAIO** (0,4000
/// num pincel de raio 0,4), porque o `from_live` mede a distância em 3-D da
/// posição VIVA e o vértice sai da pegada ao subir. Os dois verbos TÊM teto; o
/// que difere é que o do Draw se move quando o artista muda o pincel e o da
/// demão não. *Um teto maior era um número que eu escolhi; de que grandeza ele
/// é função é o que a lei diz.*
#[test]
fn the_draw_and_the_coat_no_longer_collapse() {
    let h = 0.1f32;
    let mut coats = Vec::new();
    let mut draws = Vec::new();
    for r in [0.2f32, 0.8] {
        let mut m_coat = grid(80, 1.4);
        lay(&mut m_coat, &coat_brush(r, h), 512);
        coats.push(peak(&m_coat));

        let mut m_draw = grid(80, 1.4);
        let draw = Brush {
            verb: Verb::Draw,
            radius: r,
            strength: 1.0,
            falloff: Falloff::Constant,
            accumulate: true,
            ..Brush::default()
        };
        lay(&mut m_draw, &draw, 512);
        draws.push(peak(&m_draw));
    }
    println!("tetos por raio 0,2/0,8 — demão {coats:?}  ·  Draw+Accum {draws:?}");
    assert!(
        (coats[0] - coats[1]).abs() < 0.02 * h,
        "o teto da demão seguiu o raio: {coats:?}"
    );
    assert!(
        draws[1] > 3.0 * draws[0],
        "o teto do Draw NÃO segue o raio: a cerca voltou e as duas leis \
         colapsaram outra vez ({draws:?})"
    );
}

/// **A LEI é a recorrência da referência, escrita à mão como oráculo.**
///
/// ⚠️ **O oráculo NÃO chama [`crate::coat_step`]** — um gate que usa a função
/// sob teste para computar o que espera é sempre verde, e este arquivo já viu a
/// forma duas vezes noutras waves.
#[test]
fn the_law_is_the_reference_recurrence() {
    for (w, strength, cap) in [
        (1.0f32, 1.0f32, 1.0f32),
        (0.5, 1.0, 1.0),
        (0.25, 0.5, 1.0),
        (0.9, 1.0, 0.4),
        (0.1, 0.75, 0.65),
    ] {
        let mut mine = 0.0f32;
        let mut theirs = 0.0f32;
        for _ in 0..64 {
            mine = crate::coat_step(mine, w, strength, cap);
            // `offset_displacement_factors` + `clamp_displacement_factors`.
            theirs += w * strength * (1.05 - theirs.abs());
            theirs = theirs.clamp(0.0, cap);
            assert_eq!(
                mine, theirs,
                "a lei divergiu da referência em w={w} s={strength} teto={cap}"
            );
        }
    }
}

/// **A MÁSCARA é um TETO, não só uma taxa** — um vértice meio protegido recebe
/// meia demão, não a demão inteira mais devagar.
#[test]
fn a_masked_vertex_stops_at_a_fraction_of_the_coat() {
    let h = 0.1f32;
    let r = 0.4f32;
    let mut mesh = grid(60, 1.0);
    {
        // Metade da grade (x > 0) fica 60 % protegida.
        let n = mesh.positions().len();
        let xs: Vec<f32> = mesh.positions().iter().map(|p| p[0]).collect();
        let m = mesh.masks_mut();
        assert_eq!(m.len(), n);
        for (slot, x) in m.iter_mut().zip(xs) {
            if x > 0.02 {
                *slot = 0.6;
            }
        }
    }
    lay(&mut mesh, &coat_brush(r, h), 512);

    let side = |mesh: &Mesh, want_right: bool| -> f32 {
        mesh.positions()
            .iter()
            .filter(|p| {
                let d = (p[0] * p[0] + p[1] * p[1]).sqrt();
                d < 0.6 * r && (p[0] > 0.05) == want_right && p[0].abs() > 0.05
            })
            .map(|p| p[2])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (free, masked) = (side(&mesh, false), side(&mesh, true));
    println!("livre {free:.5}  ·  60 % protegido {masked:.5}  (teto {h})");
    assert!(
        (free - h).abs() < 0.02 * h,
        "o lado LIVRE não fechou a demão: {free}"
    );
    // `1 − 0,6 = 0,4` da demão, e não a demão inteira.
    assert!(
        (masked - 0.4 * h).abs() < 0.05 * h,
        "o lado protegido não parou na fração da máscara: {masked} contra \
         {} — a máscara virou só uma taxa",
        0.4 * h
    );
}

/// **O `Ctrl` CAVA a demão** — a mesma altura, para o outro lado.
#[test]
fn ctrl_lays_the_coat_the_other_way() {
    let h = 0.1f32;
    let mut mesh = grid(60, 1.0);
    let mut b = coat_brush(0.4, h);
    b.invert = true;
    lay(&mut mesh, &b, 512);
    let low = mesh
        .positions()
        .iter()
        .map(|p| p[2])
        .fold(f32::INFINITY, f32::min);
    println!("demão invertida: {low:.5} (teto −{h})");
    assert!(
        (low + h).abs() < 0.02 * h,
        "o Ctrl não cavou a demão inteira: {low}"
    );
}

/// **UM TRAÇO NOVO deita uma demão NOVA** — e é a referência, não um descuido.
///
/// ⚠️ **A primeira versão deste gate afirmava idempotência entre traços e
/// nasceu VERMELHA sobre produto correto** (0,1000 contra 0,1726): eu tinha
/// importado a frase *"um shape editor re-carimba a figura a cada quadro"* do
/// Painter, e este módulo não tem shape editors. O que o `pre` congelado
/// garante é dentro de UM traço — que o número de dabs não move o resultado
/// convergido, e disso trata o `the_coat_stops_at_the_authored_height`.
///
/// Entre traços a demão **empilha**, porque o `ss.cache` da referência morre no
/// pen-up (`MEM_delete`) e o traço seguinte lê o `orig` NOVO: uma segunda
/// passada deita uma segunda camada, que é o que *demão* significa.
#[test]
fn a_second_stroke_lays_a_second_coat() {
    let h = 0.1f32;
    let b = coat_brush(0.4, h);

    let mut once = grid(60, 1.0);
    lay(&mut once, &b, 512);
    let first = peak(&once);

    lay(&mut once, &b, 512); // traço NOVO sobre a mesma tinta
    let second = peak(&once);

    println!("uma demão {first:.5}  ·  duas {second:.5}  (altura {h})");
    assert!(
        (first - h).abs() < 0.02 * h,
        "a primeira demão não fechou: {first}"
    );
    assert!(
        (second - 2.0 * h).abs() < 0.04 * h,
        "o segundo traço não deitou uma segunda camada: {second} contra {}",
        2.0 * h
    );
}

/// **UMA DEMÃO FECHADA PARA DE CRESCER** — e o teto dela é a MÁSCARA, não `1`.
///
/// ⚠️ **Este gate media TRABALHO e a premissa dele MORREU com o porte do
/// `calc_translations`.** Ele afirmava *"o 64.º dab move ZERO vértices"*, o que
/// era verdade enquanto o alvo da demão era escrito de forma ABSOLUTA a partir
/// do `base`: chegado à demão cheia, re-escrever era um no-op, e um early-out o
/// poupava.
///
/// Sob a lei da referência a demão **tem de continuar a escrever** — a
/// translação sai do VIVO, e é o dab seguinte que traz de volta o vértice que
/// o auto-smooth (ou qualquer passe posterior) tirou da meta. Manter aquela
/// asserção teria prendido o produto a um early-out que ANIQUILAVA a demão
/// sob o Auto Smooth (medido: relevo `0,00000` em `auto_smooth = 0,75`).
///
/// ⚠️ **O que se perde é a defesa de CUSTO, e ela fica NOMEADA:** a demão volta
/// a mandar ao refit do octree e ao upload vértices que não se moveram. É o
/// preço de ela conviver com um alisador; se um dia doer, a cura é comparar a
/// POSIÇÃO com a meta (barato e correcto), nunca voltar a comparar o `disp`.
///
/// A propriedade que sobrevive — e a única que o artista vê — é a CONVERGÊNCIA:
/// a demão para de subir, e para na fração que a máscara deixa livre.
#[test]
fn a_finished_coat_stops_growing() {
    let h = 0.1f32;
    let b = coat_brush(0.4, h);
    let mut mesh = grid(60, 1.0);
    {
        // A pegada inteira 60 % protegida: o teto de toda ela é `0,4`.
        let m = mesh.masks_mut();
        for slot in m.iter_mut() {
            *slot = 0.6;
        }
    }
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    let d = Dab::at([0.0, 0.0, 0.0], b.radius, [0.0, 0.0, -1.0]);
    for _ in 0..32 {
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    let half = peak(&mesh);
    for _ in 0..32 {
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    let full = peak(&mesh);
    println!("demão protegida: 32 dabs -> {half}, 64 dabs -> {full}");
    assert!(
        (full - half).abs() < 1e-4,
        "a demão fechada continuou a crescer entre o 32.º e o 64.º dab: \
         {half} -> {full}"
    );
    assert!(
        (full - 0.4 * h).abs() < 0.02 * h,
        "a demão protegida não parou na fração da máscara: {full}"
    );
    // O CONTROLE: sem máscara ela para NOUTRA altura — senão este gate estaria
    // a medir *"nada se move"* em vez de *"ela converge no teto da máscara"*.
    let mut clean = grid(60, 1.0);
    let mut s2 = SculptStroke::default();
    s2.begin(&clean);
    for _ in 0..64 {
        s2.dab(&mut clean, &b, &d, Symmetry::default());
    }
    let open = peak(&clean);
    println!("demão livre: 64 dabs -> {open}");
    assert!(
        open > full + 0.02 * h,
        "o CONTROLE sem máscara parou na mesma altura da protegida: \
         {open} contra {full}"
    );
}

/// **As TRÊS leis de acumulação são mutuamente exclusivas**, em todo verbo e em
/// toda combinação das duas flags.
#[test]
fn the_three_accumulation_laws_are_mutually_exclusive() {
    let mut coats = 0usize;
    for verb in Verb::ALL {
        for accumulate in [false, true] {
            for field in [false, true] {
                let law = verb.grip_law(accumulate, field);
                let n =
                    usize::from(law.unit_accum) + usize::from(law.additive) + usize::from(law.coat);
                assert!(
                    n <= 1,
                    "{verb:?} (acc {accumulate}, campo {field}) declara {n} leis \
                     de acumulação: {law:?}"
                );
                coats += usize::from(law.coat);
            }
        }
    }
    assert!(coats > 0, "nenhum verbo declara a demão: o gate mede vácuo");
}

/// **A demão NÃO oferece o Accumulate** — a referência não o lê, e ela tem o
/// próprio motor de saturação.
#[test]
fn the_coat_does_not_offer_accumulate() {
    assert!(!Verb::Layer.accumulates(), "a demão oferece o Accumulate");
    // E o interruptor não muda a lei dela, venha ele de onde vier.
    let off = Verb::Layer.grip_law(false, false);
    let on = Verb::Layer.grip_law(true, false);
    assert_eq!(off, on, "o Accumulate move a lei da demão");
    // O CONTROLE: num verbo que o oferece, ele MOVE alguma coisa.
    assert!(Verb::Draw.accumulates());
    assert_ne!(
        Verb::Draw.grip_law(false, false),
        Verb::Draw.grip_law(true, false),
        "o CONTROLE não distingue: o Accumulate ficou inerte em toda parte"
    );
}
