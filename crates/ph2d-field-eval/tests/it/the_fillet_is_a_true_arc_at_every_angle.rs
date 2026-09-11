//! ⭐⭐⭐ **O FILETE TEM DE SER UM ARCO DE RAIO `r` EM QUALQUER QUINA — não só a 90°.**
//!
//! # A pergunta que este ficheiro responde
//!
//! *«O número que o artista escreve em **Fillet** é o raio que ele recebe?»* Até 2026-09-02 a
//! resposta era **sim a 90° e não em todo o resto**: o operador publicado mede `‖(u, v)‖` com
//! Pitágoras, que só é a distância euclidiana se os dois gradientes forem ortogonais.
//!
//! ⚠️ **A régua é ANALÍTICA, não uma fixtura.** Uma cunha de meio-ângulo `α` arredondada com raio
//! `r` tem o centro do arco a `r/sin α` do vértice, logo a superfície recua **exactamente**
//! `r·(1/sin α − 1)`. Nada aqui é medido contra outra implementação nossa — o oráculo é a
//! trigonometria, que é a mesma disciplina do resto desta crate.
//!
//! ```text
//! cargo test -p ph2d-field-eval --test the_fillet_is_a_true_arc_at_every_angle
//! ```

use fidget::context::Tree;
use fidget::shape::EzShape;
use ph2d_field_eval::ops_bool;

/// As duas faces de uma cunha 2D (prismática em `z`) com o vértice na origem, aberta para `+x` e
/// com **meio-ângulo interno** `alpha`.
///
/// As normais exteriores são `(−sin α, ±cos α, 0)`, logo `n_a · n_b = sin²α − cos²α = −cos 2α` —
/// que é o `cos_faces` que o operador quer.
fn wedge(alpha: f64) -> (Tree, Tree, f64) {
    let (s, c) = (alpha.sin(), alpha.cos());
    let a = Tree::x() * Tree::constant(-s) + Tree::y() * Tree::constant(c);
    let b = Tree::x() * Tree::constant(-s) - Tree::y() * Tree::constant(c);
    (a, b, -(2.0 * alpha).cos())
}

/// Onde a superfície corta o semi-eixo `+x` — o **recuo do vértice**, que é a grandeza que o olho
/// lê como "o tamanho do arredondamento".
fn recess(tree: &Tree) -> f64 {
    let shape = ph2d_field_eval::Engine::from(tree.clone());
    let tape = shape.ez_float_slice_tape();
    let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
    #[allow(clippy::cast_possible_truncation)]
    let mut at = |x: f64| -> f64 {
        let v = ev.eval(&tape, &[x as f32], &[0.0], &[0.0]).expect("avalia");
        f64::from(v[0])
    };
    // O vértice foi cortado ⇒ a origem está FORA (`f > 0`) e o interior está longe em `+x`.
    let (mut lo, mut hi) = (0.0_f64, 4.0_f64);
    assert!(at(lo) > 0.0, "a origem devia estar fora da peça");
    assert!(at(hi) < 0.0, "o fundo devia estar dentro da peça");
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if at(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// A intersecção arredondada das duas faces, por De Morgan — o mesmo caminho do produto.
fn rounded_wedge(alpha: f64, r: f64, cos_faces: f64) -> Tree {
    let (a, b, _) = wedge(alpha);
    let neg = |t: &Tree| Tree::constant(0.0) - t.clone();
    let u = ops_bool::union_round_at(&neg(&a), &neg(&b), r, cos_faces);
    Tree::constant(0.0) - u
}

/// Os meio-ângulos que a régua percorre — a quina recta no meio, uma ponta de estrela em baixo e
/// uma parede de hexágono em cima. ⚠️ **Uma varredura só de agudos deixaria o lado obtuso por
/// medir**, que é precisamente onde a cura anterior partia o prisma.
const HALF_ANGLES_DEG: &[f64] = &[15.0, 19.2, 30.0, 45.0, 60.0, 75.0];

/// ⭐⭐⭐ **A LEI: o recuo é `r·(1/sin α − 1)` em toda quina.**
///
/// ⛔ Prova de mutação: passar `0.0` como `cos_faces` (o operador de hoje) reprova em **cinco** dos
/// seis ângulos e passa exactamente num — os 45°, que é o único caso que toda fixtura canónica
/// desta crate exercita.
#[test]
fn the_fillet_recess_is_the_true_arc_at_every_angle() {
    const R: f64 = 0.2;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let (_, _, c) = wedge(alpha);
        let esperado = R * (1.0 / alpha.sin() - 1.0);
        let medido = recess(&rounded_wedge(alpha, R, c));
        assert!(
            (medido - esperado).abs() <= 2.0e-4,
            "a {deg}°: o arco pede {esperado:.6} e o operador entregou {medido:.6}"
        );
    }
}

/// ⭐⭐⭐ **A OUTRA METADE, e ela reprovou primeiro: a FACE PLANA não se mexe.**
///
/// ⛔ A 1.ª versão do operador aplicava o factor `1/√(1−c²)` à raiz inteira. O recuo no vértice
/// ficava exacto — este ficheiro passava — e **a face saía do sítio**, porque onde só uma face
/// está activa a lei devolvia `u/√(1−c²)` em vez de `u`. Na sonda de arestas o prisma foi de
/// `0,0 %` para `3,3 %` de superfície sobre um vinco.
///
/// ⚠️ **Um gate sobre o VÉRTICE não vê uma face deslocada** — ele mede um ponto, e a face é todo o
/// resto da peça. Esta é a metade que faltava, e ela mede **longe** da quina, onde a mistura já
/// acabou e a resposta tem de ser a distância ao plano, ao dígito.
#[test]
fn the_flat_face_does_not_move_when_the_corner_knows_its_angle() {
    const R: f64 = 0.2;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let (a, b, c) = wedge(alpha);
        let neg = |t: &Tree| Tree::constant(0.0) - t.clone();
        let tree = neg(&ops_bool::union_round_at(&neg(&a), &neg(&b), R, c));
        let shape = ph2d_field_eval::Engine::from(tree);
        let tape = shape.ez_float_slice_tape();
        let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
        // Pontos sobre a face de cima (a recta `y = x·tan α`), bem para lá do alcance da mistura:
        // ali o campo TEM de ser zero, e um passo `d` para fora tem de valer `d`.
        for k in 1..=6 {
            let dist_ao_vertice = 1.0 + f64::from(k);
            let (px, py) = (dist_ao_vertice * alpha.cos(), dist_ao_vertice * alpha.sin());
            // A normal exterior da face de cima é `(−sin α, cos α)`.
            for fora in [0.0_f64, 0.05, 0.15] {
                let (qx, qy) = (px - fora * alpha.sin(), py + fora * alpha.cos());
                #[allow(clippy::cast_possible_truncation)]
                let v = ev
                    .eval(&tape, &[qx as f32], &[qy as f32], &[0.0])
                    .expect("avalia");
                let lido = f64::from(v[0]);
                assert!(
                    (lido - fora).abs() <= 2.0e-4,
                    "a {deg}°, a {dist_ao_vertice} do vértice e {fora} fora da face: \
                     o campo devia valer {fora:.4} e vale {lido:.6} — a FACE deslocou-se"
                );
            }
        }
    }
}

/// Onde a superfície de uma árvore corta o raio que sai da origem na direcção `dir` (unitária).
fn boundary_along(tree: &Tree, dir: [f64; 3], fora: f64) -> f64 {
    let shape = ph2d_field_eval::Engine::from(tree.clone());
    let tape = shape.ez_float_slice_tape();
    let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
    #[allow(clippy::cast_possible_truncation)]
    let mut at = |d: f64| -> f64 {
        let p = [d * dir[0], d * dir[1], d * dir[2]];
        let v = ev
            .eval(&tape, &[p[0] as f32], &[p[1] as f32], &[p[2] as f32])
            .expect("avalia");
        f64::from(v[0])
    };
    let (mut lo, mut hi) = (0.0_f64, fora);
    assert!(at(lo) < 0.0, "a origem devia estar dentro da peça");
    assert!(at(hi) > 0.0, "o fim devia estar fora da peça");
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if at(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐⭐⭐ **AS FORMAS QUE DECLARAM UM ÂNGULO SÃO MEDIDAS POR ELE** — o censo que faltava.
///
/// ⛔⛔ **Este gate nasceu de uma mutação que SOBREVIVEU**: pôr o `COS_FACES` do octaedro a `0`
/// (isto é, mentir-lhe sobre a própria geometria) não matava um único teste da crate. A estrela
/// tinha o gate analítico dela e o prisma tinha a sonda de curvatura; o octaedro **não tinha
/// ninguém** — *escrevi a guarda certa e não a gateei*, pela quarta vez registada neste repo.
///
/// A régua é analítica nos dois casos: uma quina de meio-ângulo `α` arredondada com raio `ρ` recua
/// `ρ·(1/sin α − 1)` medido na **bissectriz** da quina, que aqui é a direcção radial da aresta.
///
/// | forma | quina | `α` | recuo do arco | recuo da lei ortogonal | razão |
/// |---|---|---|---|---|---|
/// | prisma hexagonal | parede–parede | `60,0°` | `0,1547 ρ` | `0,3382 ρ` | **`2,19×` a MAIS** |
/// | octaedro | face–face | `54,7°` | `0,2247 ρ` | `0,3587 ρ` | **`1,60×` a MAIS` |
///
/// ⚠️ **Numa quina OBTUSA a lei antiga arredondava DE MAIS**, e é por isso que a sonda de vinco a
/// premiava: uma mistura mais larga apaga mais aresta. *Uma régua que conta «quanto vinco sobrou»
/// não distingue «o filete está certo» de «o filete é grande demais».*
#[test]
fn the_shapes_that_declare_an_angle_are_measured_by_it() {
    // ⚠️ O prisma é a interseção das paredes com a laje; a quina lateral vive no equador (`z = 0`),
    // longe do aro, para que o que se mede seja SÓ a mistura das duas paredes.
    const RHO: f64 = 0.04;
    let raio = 0.45;
    let prisma = ph2d_field_eval::ops::sd_prism(6, raio, raio, 0.30, RHO, 0.0);
    let alfa_prisma = 60.0_f64.to_radians();
    // A quina de um hexágono está na direcção `2πk/n`; `k = 0` é o eixo `+x`.
    let medido = boundary_along(&prisma, [1.0, 0.0, 0.0], 2.0);
    let esperado = raio - RHO * (1.0 / alfa_prisma.sin() - 1.0);
    assert!(
        (medido - esperado).abs() < 1.0e-3,
        "a quina do prisma mede {medido:.6} e o arco de raio {RHO} diz {esperado:.6}"
    );

    // O octaedro `x+y+z = r`: a aresta entre as faces `(+,+,+)` e `(+,+,−)` está em `z = 0`,
    // `x + y = r`, e a bissectriz dela é a direcção `(1,1,0)/√2`.
    let r_oct = 0.5;
    let oct = ph2d_field_eval::ops_solids::sd_octahedron(r_oct, RHO, 0.0);
    let alfa_oct = 0.5 * (std::f64::consts::PI - (1.0_f64 / 3.0).acos());
    let inv2 = std::f64::consts::FRAC_1_SQRT_2;
    let medido = boundary_along(&oct, [inv2, inv2, 0.0], 2.0);
    let esperado = r_oct * inv2 - RHO * (1.0 / alfa_oct.sin() - 1.0);
    assert!(
        (medido - esperado).abs() < 1.0e-3,
        "a aresta do octaedro mede {medido:.6} e o arco de raio {RHO} diz {esperado:.6}"
    );

    // ⛔ **O CONTROLE**: sem filete as duas quinas estão nos raios autorados, e o filete tem de as
    // mover. Sem esta metade, um construtor que devolvesse a peça encolhida passaria as de cima.
    let vivo_prisma = ph2d_field_eval::ops::sd_prism(6, raio, raio, 0.30, 0.0, 0.0);
    assert!(
        (boundary_along(&vivo_prisma, [1.0, 0.0, 0.0], 2.0) - raio).abs() < 1.0e-4,
        "sem filete a quina do prisma tem de estar no circunraio autorado"
    );
    let vivo_oct = ph2d_field_eval::ops_solids::sd_octahedron(r_oct, 0.0, 0.0);
    assert!(
        (boundary_along(&vivo_oct, [inv2, inv2, 0.0], 2.0) - r_oct * inv2).abs() < 1.0e-4,
        "sem filete a aresta do octaedro tem de estar onde a fórmula a põe"
    );
}

/// ⭐⭐⭐ **O CHANFRO recua o que o slider DIZ, em qualquer quina** (W111).
///
/// O painel promete que *Chamfer* é o recuo **ao longo de cada face**, e a lei anterior —
/// `(a + b + c)·√½` — só o entregava a `90°`: fora dali ela descia `c/sin 2α`, que numa ponta de
/// estrela (`α = 19,2°`) é **`1,61×`** o número pedido.
///
/// ⚠️ **Este gate atravessa o PRODUTO** (`ops_joint::intersection_joint`), e não uma fórmula
/// copiada para o teste: a versão anterior dele escrevia o plano à mão para não usar a função sob
/// teste, e o preço foi ficar **verde sobre uma lei que o produto já não usava**. *Contra um oráculo
/// analítico, a régua tem de morder o caminho que o artista percorre.*
#[test]
fn the_chamfer_recess_is_the_number_the_slider_says() {
    const C: f64 = 0.15;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let (a, b, cos_faces) = wedge(alpha);
        let tree = ph2d_field_eval::ops_joint::intersection_joint(
            &a,
            &b,
            ph2d_field_eval::ops_joint::Edge::at(0.0, C, cos_faces),
        );
        let shape = ph2d_field_eval::Engine::from(tree);
        let tape = shape.ez_float_slice_tape();
        let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
        // Caminha SOBRE a face de cima, do vértice para fora: o corte acaba onde o campo chega a 0.
        #[allow(clippy::cast_possible_truncation)]
        let mut na_face = |d: f64| -> f64 {
            let (px, py) = (d * alpha.cos(), d * alpha.sin());
            let v = ev
                .eval(&tape, &[px as f32], &[py as f32], &[0.0])
                .expect("avalia");
            f64::from(v[0])
        };
        // ⚠️ **O piso não é conforto: sobre a própria face o `a` é uma subtracção que se cancela**
        // (`−d sinα cosα + d sinα cosα`), e em `f32` ela devolve `±1e-7`. Uma bissecção que pergunta
        // `> 0` prende-se a esse ruído e devolve o **fim do intervalo**; o corte cresce a `~0,5` por
        // unidade, logo este piso desloca a leitura `2e-5`. *Uma régua sobre uma superfície tem de
        // saber a que distância dela o zero deixa de ser zero.*
        const PISO: f64 = 1.0e-5;
        let (mut lo, mut hi) = (0.0_f64, 4.0_f64);
        assert!(na_face(lo) > PISO, "o vértice devia estar cortado fora");
        assert!(
            na_face(hi) < PISO,
            "longe do vértice a face devia ser a peça"
        );
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if na_face(mid) > PISO {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let medido = 0.5 * (lo + hi);
        assert!(
            (medido - C).abs() <= 5.0e-4,
            "a {deg}°: o slider pede {C:.6} de recuo ao longo da face e o corte deu {medido:.6}"
        );
        // ⛔ **O CONTROLE, e ele é o tamanho da mentira que isto apagou:** a lei ortogonal descia
        // `C/sin 2α`, e fora dos 45° isso NÃO é o pedido. Sem esta metade, um operador que voltasse
        // à fórmula antiga passaria a barra de cima em exactamente um dos seis ângulos e o gate
        // leria «quase tudo bem».
        let antiga = C / (2.0 * alpha).sin();
        let razao = antiga / C;
        if (deg - 45.0).abs() < 1.0e-9 {
            assert!(
                (razao - 1.0).abs() <= 1.0e-3,
                "a 45° as duas leis TÊM de coincidir"
            );
        } else {
            assert!(
                razao > 1.05,
                "a {deg}° a lei antiga devia cortar mais fundo do que o pedido, e deu {razao:.4}"
            );
            assert!(
                (medido - antiga).abs() > 5.0e-4,
                "a {deg}° o corte devolveu o número da lei ANTIGA ({antiga:.6}) — o operador \
                 regrediu"
            );
        }
    }
}

/// ⭐⭐⭐ **O PLANO DO CHANFRO É UMA DISTÂNCIA** — e este era o segundo erro, o que escondia o
/// primeiro (W110).
///
/// A lei anterior escalava a soma por `√½` seja qual for o ângulo, e `‖∇(a+b)‖ = √(2+2κ)`: numa
/// ponta de estrela isso deixava `‖∇plano‖ = 0,4644`, a **subestimar `2,15×`**. Um campo `2,15×`
/// menor torna a região `{|plano| < r}` — onde o filete mistura — `2,15×` mais larga, e era essa
/// largura a mais que tapava o vinco da ponta.
///
/// ⚠️ **É por isso que as duas metades não se movem uma sem a outra**, e por que a cura de W110 que
/// só honrava o recuo media pior: ela estreitava a mistura sem dar à mistura o ângulo das arestas
/// novas.
#[test]
fn the_chamfer_plane_is_a_true_distance_at_every_angle() {
    const C: f64 = 0.15;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let (a, b, cos_faces) = wedge(alpha);
        let tree = ph2d_field_eval::ops_joint::intersection_joint(
            &a,
            &b,
            ph2d_field_eval::ops_joint::Edge::at(0.0, C, cos_faces),
        );
        let shape = ph2d_field_eval::Engine::from(tree);
        let tape = shape.ez_float_slice_tape();
        let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
        #[allow(clippy::cast_possible_truncation)]
        let mut at = |x: f64, y: f64| -> f64 {
            let v = ev
                .eval(&tape, &[x as f32], &[y as f32], &[0.0])
                .expect("avalia");
            f64::from(v[0])
        };
        // Sobre a bissectriz, ANTES do corte, só o plano está activo. A derivada ao longo de `+x`
        // é `−‖∇plano‖`, e o passo é grande de propósito: `f32` em diferenças finitas.
        const H: f64 = 0.02;
        let x0 = -0.30;
        let grad = (at(x0, 0.0) - at(x0 + H, 0.0)) / H;
        assert!(
            (grad - 1.0).abs() <= 5.0e-3,
            "a {deg}°: o plano do chanfro devia ser uma distância e mede ‖∇‖ = {grad:.4}"
        );
    }
}

/// ⭐⭐⭐ **QUANDO O FILETE NÃO CABE NA FACETA, ELE COME O CHANFRO — e a transição não tem degrau.**
///
/// A faceta sobrevive à erosão por `r` enquanto `r < c·sin α(1 + sin α)/cos α`. No limite os três
/// planos deslocados são concorrentes, os centros dos dois arcos coincidem e as duas leis devolvem
/// a **mesma** peça; acima dele o plano do corte já não pertence ao erodido, logo a abertura do
/// sólido chanfrado é **idêntica** à do sólido vivo.
///
/// ⚠️ **Este gate mede as duas metades da afirmação**: a continuidade no limite (as leis coincidem)
/// e a identidade acima dele (o chanfro é invisível). ⛔ Sem a segunda, prender `r` no limite
/// passaria a primeira — e é essa a alternativa que ficou recusada, porque deixa o *Fillet* inerte
/// sem nada na tela a dizê-lo.
#[test]
fn a_fillet_that_outgrows_the_facet_eats_the_chamfer_without_a_step() {
    const C: f64 = 0.15;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let (a, b, cos_faces) = wedge(alpha);
        let (s, k) = (alpha.sin(), alpha.cos());
        let limite = C * s * (1.0 + s) / k;
        let com_chanfro = |r: f64| {
            ph2d_field_eval::ops_joint::intersection_joint(
                &a,
                &b,
                ph2d_field_eval::ops_joint::Edge::at(r, C, cos_faces),
            )
        };
        let so_filete = |r: f64| {
            ph2d_field_eval::ops_joint::intersection_joint(
                &a,
                &b,
                ph2d_field_eval::ops_joint::Edge::at(r, 0.0, cos_faces),
            )
        };
        // ⭐ A CONTINUIDADE: um fio abaixo do limite e um fio acima têm de dar o mesmo recuo.
        let (abaixo, acima) = (limite * 0.999, limite * 1.001);
        let (r_abaixo, r_acima) = (recess(&com_chanfro(abaixo)), recess(&com_chanfro(acima)));
        assert!(
            (r_abaixo - r_acima).abs() <= 2.0e-3,
            "a {deg}°: a transição tem um degrau — {r_abaixo:.6} contra {r_acima:.6}"
        );
        // ⭐ A IDENTIDADE: acima do limite o chanfro não se vê.
        for f in [1.001_f64, 1.5, 2.5] {
            let r = limite * f;
            let (com, sem) = (recess(&com_chanfro(r)), recess(&so_filete(r)));
            assert!(
                (com - sem).abs() <= 5.0e-4,
                "a {deg}° com r = {f:.3}× o limite: o chanfro devia ser invisível e mede \
                 {com:.6} contra {sem:.6}"
            );
        }
        // ⛔ O CONTROLE que impede isto de ser lido como «o chanfro nunca faz nada»: bem abaixo do
        // limite ele TEM de se ver.
        let r = limite * 0.25;
        let (com, sem) = (recess(&com_chanfro(r)), recess(&so_filete(r)));
        assert!(
            (com - sem).abs() > 5.0e-3,
            "a {deg}° com r = 0,25× o limite o chanfro devia mudar a peça, e mede {com:.6} \
             contra {sem:.6}"
        );
    }
}

/// ⭐⭐ **O CONTROLE: a suposição ortogonal é o erro MEDIDO, e ele tem o sinal dos dois lados.**
///
/// Sem o ângulo, o recuo é `(1 − 1/√2)·r/sin α`. ⚠️ Numa quina **aguda** isso é *menos* filete do
/// que se pediu (a queixa que abriu esta obra) e numa **obtusa** é *mais* — foi por isso que
/// compensar as obtusas pelo raio estreitava a mistura e criava o vinco que ela devia curar.
///
/// Este teste é o que impede a lei nova de ser confundida com um no-op: se alguém apagar o termo
/// cruzado da [`ops_bool::union_round_at`], os dois testes deste ficheiro passam a concordar, e
/// **este** diz que eles não deviam.
#[test]
fn the_orthogonal_assumption_is_the_measured_error() {
    const R: f64 = 0.2;
    for &deg in HALF_ANGLES_DEG {
        let alpha = deg.to_radians();
        let sem_angulo = recess(&rounded_wedge(alpha, R, 0.0));
        let previsto = (1.0 - std::f64::consts::FRAC_1_SQRT_2) * R / alpha.sin();
        assert!(
            (sem_angulo - previsto).abs() <= 2.0e-4,
            "a {deg}°: a lei ortogonal devia recuar {previsto:.6} e recuou {sem_angulo:.6}"
        );
        let arco = R * (1.0 / alpha.sin() - 1.0);
        let razao = arco / sem_angulo;
        if (deg - 45.0).abs() < 1.0e-9 {
            assert!(
                (razao - 1.0).abs() <= 1.0e-3,
                "a 45° as duas leis TÊM de coincidir, e a razão deu {razao:.4}"
            );
        } else {
            assert!(
                (razao - 1.0).abs() > 0.05,
                "a {deg}° as duas leis deviam divergir, e a razão deu {razao:.4}"
            );
        }
    }
}
