//! ⭐⭐ **A NUVEM PELA PORTA DO PAINEL** — o report do Enio de 2026-09-05: *«cloud completamente
//! bugado. Tail não tem efeito, Span e joint criam formas gigantes»*.
//!
//! ⚠️ **Ela arrasta cada linha como o painel arrasta** (`ph2d_field::dims` + `set_dim`), e mede o
//! que o artista vê: **onde a peça chega** em cada eixo. ⛔ Uma sonda que construísse a primitiva à
//! mão mediria outro programa — a mesma lição que a sonda do undo pagou em 04/09.
//!
//! ```text
//! cargo test -p ph2d-field-eval --test probe_cloud_report -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Span, Xform};
use ph2d_field_eval::Field;

/// A nuvem **como a paleta a cria** — ver `field3d_shapes_make_signs::a_cloud`.
fn da_paleta(r: f32) -> Primitive {
    Primitive::Cloud {
        lobes: 5,
        half_width: r,
        half_span: r * 0.50,
        tail: 0.0,
        half_height: r * 0.25,
        round: r * 0.1,
        chamfer: 0.0,
    }
}

/// ⭐ `passo × ‖∇f‖` — acima de `1` a marcha **atravessa a superfície**, e o que se vê é uma peça
/// rasgada ou um borrão que não é a forma.
fn marcha(p: &Primitive) -> f64 {
    let Ok(doc) = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    ) else {
        return f64::NAN;
    };
    let f = Field::new(&doc);
    let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
    let mut g = 0.0f64;
    let n = 46usize;
    #[allow(clippy::cast_precision_loss)]
    let at = |t: usize| -1.6 + 3.2 * (t as f64 + 0.5) / n as f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let (x, y, z) = (at(i), at(j), at(k));
                if f.at(x, y, z).abs() < 0.06 {
                    let gg = f.gradient_norm(x, y, z, 1.0e-4);
                    if gg.is_finite() {
                        g = g.max(gg);
                    }
                }
            }
        }
    }
    passo * g
}

/// Até onde a peça chega em cada eixo, medido no CAMPO.
fn extensao(p: &Primitive) -> [f64; 3] {
    let Ok(doc) = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    ) else {
        return [f64::NAN; 3];
    };
    let f = Field::new(&doc);
    let mut ext = [0.0f64; 3];
    let n = 120usize;
    #[allow(clippy::cast_precision_loss)]
    let at = |t: usize| -3.0 + 6.0 * (t as f64 + 0.5) / n as f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let (x, y, z) = (at(i), at(j), at(k));
                if f.at(x, y, z) < 0.0 {
                    for (eixo, c) in [x, y, z].iter().enumerate() {
                        ext[eixo] = ext[eixo].max(c.abs());
                    }
                }
            }
        }
    }
    ext
}

#[test]
#[ignore = "sonda: o report da nuvem"]
fn probe_cloud_report() {
    let base = da_paleta(0.5);
    let linhas = ph2d_field::dims(&base);
    println!("\n  a nuvem da paleta: {:?}", base);
    println!(
        "  a caixa que o documento DECLARA: {:?}\n",
        ph2d_field::bounding_half_extents(&base)
    );
    for (i, d) in linhas.iter().enumerate() {
        // Três pontos de trabalho por linha, dentro da faixa que ela declara.
        let alvos: Vec<f32> = match d.span {
            #[allow(clippy::cast_precision_loss)]
            Span::Count { min, max } => vec![min as f32, ((min + max) / 2) as f32, max as f32],
            Span::Wall(w) | Span::WallFromZero(w) => vec![w * 0.1, w * 0.5, w * 0.9],
            _ => vec![
                d.value * 0.5,
                d.value.max(0.05) * 2.0,
                d.value.max(0.05) * 4.0,
            ],
        };
        println!("  {} (linha {i}, agora {:.3})", d.key, d.value);
        for alvo in alvos {
            let mut q = base.clone();
            match ph2d_field::set_dim(&mut q, 0, i, alvo) {
                Err(e) => println!("      {alvo:>7.3} -> RECUSADO {e:?}"),
                Ok(()) => {
                    ph2d_field::clamp_round(&mut q);
                    let ext = extensao(&q);
                    let caixa = ph2d_field::bounding_half_extents(&q);
                    let m = marcha(&q);
                    println!(
                        "      {alvo:>7.3} -> peça até [{:.3} {:.3} {:.3}]   caixa [{:.3} {:.3} {:.3}]   marcha {m:.3}{}{}",
                        ext[0],
                        ext[1],
                        ext[2],
                        caixa[0],
                        caixa[1],
                        caixa[2],
                        if ext[0] > f64::from(caixa[0]) * 1.02
                            || ext[1] > f64::from(caixa[1]) * 1.02
                        {
                            "  <== SAI DA CAIXA"
                        } else {
                            ""
                        },
                        if m > 1.02 { "  <== A MARCHA FURA" } else { "" }
                    );
                }
            }
        }
    }
    println!();
}

/// ⭐ **ONDE o escudo e a chave deixam de marchar** — as duas que sobraram do gate de faixa (05/09).
///
/// ⚠️ **A cerca de cada uma escreve-se a partir DESTA tabela**, e não de um número escolhido: o §0
/// manda medir antes de limitar.
#[test]
#[ignore = "sonda: as duas cercas que faltam"]
fn probe_shield_and_brace_ranges() {
    let marchar = |p: &Primitive| -> f64 {
        let Ok(doc) = FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
            NodeId(0),
        ) else {
            return f64::NAN;
        };
        let f = Field::new(&doc);
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        let mut g = 0.0f64;
        let n = 54usize;
        #[allow(clippy::cast_precision_loss)]
        let at = |t: usize| -1.2 + 2.4 * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if f.at(x, y, z).abs() < 0.05 {
                        let gg = f.gradient_norm(x, y, z, 1.0e-4);
                        if gg.is_finite() {
                            g = g.max(gg);
                        }
                    }
                }
            }
        }
        passo * g
    };
    println!("\n  ESCUDO — a razão altura/largura (a cerca de hoje e' 0,70)");
    println!("  {:>8} {:>16}", "s/w", "passo x |grad|");
    for razao in [0.80_f32, 0.90, 1.00, 1.10, 1.20, 1.30] {
        let w = 0.34_f32;
        // ⚠️ **Com o filete do REPRESENTANTE** (`0,04`), e não um menor: a 1.ª varredura usou `0,02`
        // e leu `0,88` onde o gate lia `1,05`. *Uma sonda com o knob noutro ponto mede outra peça.*
        let mut p = Primitive::Shield {
            half_width: w,
            half_span: w * razao,
            half_height: 0.10,
            round: 0.04,
            chamfer: 0.0,
        };
        ph2d_field::clamp_round(&mut p);
        println!("  {razao:>8.2} {:>16.4}", marchar(&p));
    }
    println!("\n  CHAVE — a espessura em fracção da parede (`half_span/2 = 0,22`)");
    println!(
        "  {:>8} {:>10} {:>16}",
        "fracção", "thickness", "passo x |grad|"
    );
    for f in [0.15_f32, 0.25, 0.35, 0.50, 0.70, 0.90] {
        let s = 0.44_f32;
        let p = Primitive::Brace {
            half_span: s,
            thickness: s * 0.5 * f,
            half_height: 0.10,
            round: 0.01,
            chamfer: 0.0,
        };
        println!("  {f:>8.2} {:>10.4} {:>16.4}", s * 0.5 * f, marchar(&p));
    }
    println!();
}
