//! ⭐⭐⭐ **A MOLA E A REDE (W124), PROVADAS ANTES DE SEREM LIGADAS.**
//!
//! > **Enio, 05/09:** *«diante do sucesso da espiral, faça pesquisa de shapes geradas por fórmulas
//! > que ainda não temos»* — e depois *«vamos lá. siga implementando»*.
//!
//! As duas de maior alcance do levantamento ([doc 08 §7](../../../docs/3DModeling/08_formas_por_formula.md)),
//! e as duas cujo mecanismo esta linha **acabou de pagar**: a volta mais próxima por `round()`
//! (W123) e o minorante por gradiente (a onda do `Document`).

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

const NA_PELE: f64 = 3.0e-3;

#[track_caller]
fn dentro(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(
        v < -NA_PELE,
        "{porque}: {p:?} devia estar DENTRO e leu {v:.5}"
    );
}

#[track_caller]
fn fora(f: &Field, p: [f64; 3], porque: &str) {
    let v = f.at(p[0], p[1], p[2]);
    assert!(v > NA_PELE, "{porque}: {p:?} devia estar FORA e leu {v:.5}");
}

fn uma_mola(turns: f32) -> Primitive {
    Primitive::Helix {
        radius: 0.30,
        pitch: 0.14,
        turns,
        thickness: 0.045,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐ **A MOLA SOBE `pitch` por volta** — o tubo está onde a hélice o põe, e no meio não está.
///
/// ⚠️ Os pontos saem da fórmula: com `turns = 3` a altura é `0,42` e a peça vai de `−0,21` a
/// `+0,21`; no ângulo `0` o tubo está em `z = −0,21 + 0,14·k`.
#[test]
fn the_coil_climbs_one_pitch_per_turn() {
    let f = campo(uma_mola(3.0));
    for k in 0..3 {
        let z = -0.21 + 0.14 * f64::from(k);
        dentro(&f, [0.30, 0.0, z + 0.01], &format!("o tubo na volta {k}"));
    }
    // ⛔ **Entre duas voltas não há nada** — sem isto isto seria um cilindro.
    fora(&f, [0.30, 0.0, -0.14], "entre a 1.ª e a 2.ª volta");
    fora(&f, [0.30, 0.0, 0.0], "entre a 2.ª e a 3.ª volta");
    // O eixo é vazio, e fora do cilindro também.
    fora(&f, [0.0, 0.0, 0.0], "o eixo da mola");
    fora(&f, [0.45, 0.0, -0.21], "para lá do cilindro");
    fora(&f, [0.30, 0.0, 0.30], "acima do fim");
}

/// ⭐⭐⭐ **E A MEIA VOLTA ELA ESTÁ A MEIO PASSO** — é o gate que separa uma mola de uma pilha de
/// anéis.
///
/// ⚠️ Uma fieira de anéis concêntricos passa em **todos** os pontos do ângulo zero e reprova aqui.
#[test]
fn at_half_a_turn_the_coil_is_half_a_pitch_up() {
    let f = campo(uma_mola(3.0));
    for k in 0..2 {
        let z = -0.21 + 0.14 * (f64::from(k) + 0.5);
        dentro(
            &f,
            [-0.30, 0.0, z],
            &format!("meia volta acima da volta {k}"),
        );
        // ⛔ E à mesma altura, do outro lado, está o VALE.
        fora(
            &f,
            [0.30, 0.0, z],
            &format!("o vale do lado oposto, volta {k}"),
        );
    }
}

/// ⭐⭐ **Mais voltas só crescem para cima** — o que já lá estava fica igual.
#[test]
fn adding_turns_only_grows_the_coil_upwards() {
    let curta = campo(uma_mola(2.0));
    let longa = campo(uma_mola(4.0));
    // ⚠️ **As duas são centradas**, logo o que se compara é a peça deslocada: a curta vai de
    // `−0,14` a `+0,14` e a longa de `−0,28` a `+0,28`. O tubo da BASE de cada uma está no fundo.
    for (a, b) in [(-0.14, -0.28), (-0.07, -0.21)] {
        let (va, vb) = (curta.at(0.30, 0.0, a), longa.at(0.30, 0.0, b));
        assert!(
            (va - vb).abs() < 1.0e-6,
            "a mesma altura relativa devia ler o mesmo: {va:.6} contra {vb:.6}"
        );
    }
}

fn uma_rede(thickness: f32) -> Primitive {
    Primitive::Gyroid {
        half: [0.40; 3],
        cell: 0.20,
        thickness,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// ⭐⭐⭐ **A REDE PASSA PELA SUPERFÍCIE DE SCHOEN, e o VAZIO dela é o que a torna uma rede.**
///
/// ⚠️ Os pontos saem da fórmula `sin x·cos y + sin y·cos z + sin z·cos x`: a origem e os múltiplos
/// de meia célula em qualquer eixo anulam-na (`sin 0 = sin π = 0`), e o centro de um octante não.
#[test]
fn the_lattice_is_the_schoen_surface_and_it_has_holes() {
    let f = campo(uma_rede(0.022));
    let c = 0.20_f64;
    dentro(&f, [0.0, 0.0, 0.0], "a origem, que está na superfície");
    dentro(&f, [c * 0.5, 0.0, 0.0], "meia célula em X");
    dentro(&f, [0.0, c * 0.5, 0.0], "meia célula em Y");
    dentro(&f, [0.0, 0.0, c * 0.5], "meia célula em Z");
    // ⛔ **O VAZIO** — `g` longe de zero. ⚠️ O máximo **não** está a um quarto da célula (ali
    // `g = 0`, e a primeira versão deste gate acusou o produto por isso): pondo `a = b = c` fica
    // `g = 3·sin a·cos a = 1,5·sin 2a`, que vale `3/2` em `a = π/4` — isto é, a **um OITAVO** da
    // célula nos três eixos.
    let q = c / 8.0;
    fora(&f, [q, q, q], "o centro do canal, onde `g` é máximo");
    fora(&f, [-q, -q, -q], "e o do canal oposto");
}

/// ⭐⭐ **A PAREDE engrossa com o controlo** — e o vazio encolhe com ela.
#[test]
fn the_wall_thickens_and_the_channel_shrinks() {
    let fina = campo(uma_rede(0.008));
    // ⚠️ **`0,024` e não mais**: a cerca é `2·thickness ≤ cell × 0,25`, e acima dela a rede FECHA
    // (ver [`ph2d_field::MAX_GYROID_FILL`]) — o documento recusa, e é isso que ele tem de fazer.
    let grossa = campo(uma_rede(0.024));
    let q = 0.20 / 8.0;
    // No mesmo ponto do canal, a fina deixa mais vazio que a grossa.
    let (a, b) = (fina.at(q, q, q), grossa.at(q, q, q));
    assert!(
        a > b + 0.01,
        "a parede fina devia deixar mais vazio: {a:.4} contra {b:.4}"
    );
    // E a superfície continua dentro nas duas.
    dentro(
        &fina,
        [0.0, 0.0, 0.0],
        "a fina ainda tem parede na superfície",
    );
    dentro(&grossa, [0.0, 0.0, 0.0], "e a grossa também");
}

/// ⭐⭐ **A CAIXA é a peça** — a rede não sai dela.
#[test]
fn the_box_is_the_piece_and_the_lattice_does_not_leave_it() {
    let f = campo(uma_rede(0.022));
    for p in [[0.55, 0.0, 0.0], [0.0, 0.55, 0.0], [0.0, 0.0, 0.55]] {
        fora(&f, p, "para lá da caixa");
    }
}

/// ⭐⭐⭐ **NENHUMA DAS DUAS PROMETE MAIS DO QUE ANDA** — é o gate que a wave inteira precisa.
///
/// ⚠️ **É a régua que a W123 estabeleceu**: nenhuma destas duas tem distância exacta, e nenhuma
/// precisa — o que o módulo exige é `‖∇f‖ ≤ 1` perto da superfície, que é onde a marcha decide.
#[test]
fn neither_the_coil_nor_the_lattice_overpromises() {
    for (nome, p, e) in [
        ("mola", uma_mola(3.0), 0.5),
        ("rede", uma_rede(0.022), 0.45),
    ] {
        let f = campo(p);
        let mut pior: f64 = 0.0;
        let passos = 70;
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / passos as f64;
        for i in 0..passos {
            for j in 0..passos {
                for k in 0..passos {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if f.at(x, y, z).abs() > 0.02 {
                        continue;
                    }
                    pior = pior.max(f.gradient_norm(x, y, z, 1.0e-4));
                }
            }
        }
        assert!(
            pior <= 1.02,
            "«{nome}»: ‖∇f‖ = {pior:.4} — acima de 1 o campo promete mais do que anda"
        );
    }
}

// ─────────────────────────── W140 ───────────────────────────

/// A meia-largura do tubo numa direcção, por **varredura** até à primeira troca de sinal.
///
/// ⛔ **Uma bissecção não serve aqui, e a razão é medida:** ela pressupõe o extremo superior FORA da
/// peça, e num passo curto o ponto de partida dela cai na **volta seguinte** da mola — o intervalo
/// deixa de conter a fronteira e ela converge para o extremo, devolvendo `0,40` onde a verdade era
/// `0,06`. *Uma bissecção com o extremo por verificar devolve o extremo.*
fn meia_largura(f: &Field, de: [f64; 3], dir: [f64; 3]) -> f64 {
    let em = |t: f64| f.at(de[0] + dir[0] * t, de[1] + dir[1] * t, de[2] + dir[2] * t);
    assert!(
        em(0.0) < 0.0,
        "o centro da medição tem de estar DENTRO da peça"
    );
    const PASSOS: usize = 20_000;
    let mut anterior = 0.0_f64;
    for i in 1..=PASSOS {
        let t = 0.5 * (i as f64) / (PASSOS as f64);
        if em(t) >= 0.0 {
            let (mut lo, mut hi) = (anterior, t);
            for _ in 0..50 {
                let m = f64::midpoint(lo, hi);
                if em(m) < 0.0 {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            return lo;
        }
        anterior = t;
    }
    f64::NAN
}

/// ⭐⭐⭐ **O TUBO DA MOLA TEM A ESPESSURA QUE O PAINEL DIZ** (W140) — e não tinha.
///
/// # ⛔⛔ O defeito estava NOMEADO no §5 desde a W134 e a cura escrita DUAS vezes
///
/// A fórmula era `hypot(dr, dz) · c − thickness`: o divisor multiplicava a **corda inteira**, logo
/// o zero do campo caía em `corda = thickness/c` e o tubo saía **`1/c` mais gordo**.
///
/// ⭐ **A conta que o corrige cabe numa linha:** um deslocamento **radial** já é perpendicular à
/// hélice (a tangente `(0, R, b)/√(R²+b²)` não tem componente em `ρ`), então ele não se encolhe;
/// só o **vertical** tem uma parte ao longo da curva. A distância perpendicular é
/// `√(dr² + (dz·sinβ)²)`, e o `sinβ` **é o mesmo `c`** — uma posição adentro.
///
/// ⚠️ **MEDIDO**, com a espessura pedida em `0,060`:
///
/// | passo | `c` | radial ANTES | radial DEPOIS | vertical (não muda) |
/// |---|---:|---:|---:|---:|
/// | 0,20 | 0,9940 | 0,06036 | **0,06000** | 0,06036 |
/// | 1,00 | 0,8767 | 0,06844 | **0,06000** | 0,06844 |
/// | 1,60 | 0,7514 | 0,07985 | **0,06000** | 0,07985 |
///
/// ⚠️ **A meia-largura VERTICAL não muda, e está CERTA nos dois:** uma subida em `z` é em parte um
/// passeio ao longo da curva, logo o tubo estende-se `thickness/c` em `dz` por construção. ⇒ *a
/// régua que apanha o defeito é a RAZÃO radial/vertical, que tem de ler `c` e lia `1,0000`.*
#[test]
fn the_spring_tube_is_as_thick_as_the_panel_says() {
    let (radius, turns, thickness) = (0.35_f32, 3.0_f32, 0.06_f32);
    for pitch in [0.20_f32, 0.50, 1.00, 1.60] {
        let f = campo(Primitive::Helix {
            radius,
            pitch,
            turns,
            thickness,
            round: 0.0,
            chamfer: 0.0,
        });
        let b = f64::from(pitch) / std::f64::consts::TAU;
        let dentro_r = f64::from(radius - thickness);
        let c = dentro_r / dentro_r.hypot(b);
        let altura = f64::from(pitch) * f64::from(turns);
        let z = (f64::from(turns) / 2.0).floor() * f64::from(pitch) - altura * 0.5;
        let centro = [f64::from(radius), 0.0, z];
        let rad = meia_largura(&f, centro, [1.0, 0.0, 0.0]);
        let ver = meia_largura(&f, centro, [0.0, 0.0, 1.0]);
        assert!(
            (rad - f64::from(thickness)).abs() < 1.0e-3,
            "passo {pitch}: o tubo mede {rad:.5} de raio contra os {thickness} que o painel diz"
        );
        // ⭐ E a RAZÃO, que é a régua que separa o campo curado do que engordava.
        assert!(
            (rad / ver - c).abs() < 5.0e-3,
            "passo {pitch}: radial/vertical = {:.4} e tem de ser o `c` = {c:.4} — a ler `1` o tubo \
             está `1/c` mais gordo do que o pedido",
            rad / ver
        );
    }
}
