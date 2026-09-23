//! ⏱️⭐⭐⭐⭐ **A DECOMPOSIÇÃO POR LINHA DE UMA PRIMITIVA DE PERFIL** — quanto é a DISTÂNCIA e quanto
//! é o SINAL.
//!
//! # Porque esta sonda existe antes de qualquer cura
//!
//! A `W9` recusou levar o índice de perfil ao dispositivo (a granularidade que é coerente não poda,
//! e a que poda não é coerente — `docs/Render3d/03` §W9), e deixou a lever no lugar certo: **o custo
//! POR PRIMITIVA**. O vaso da cena `5` paga `38,8` linhas por primitiva, uma recta custa `26,7` e
//! um arco **`51,0`** — `1,9×`.
//!
//! ⚠️ **Mas «26,7» e «51,0» são o TOTAL de uma primitiva**, e as duas metades dela têm curas
//! opostas: a DISTÂNCIA é um `min` que só uma poda encurta (e a poda está recusada), o SINAL é uma
//! SOMA de contribuições que um sinal tirado da primitiva mais próxima trocaria por **uma**. ⇒
//! *sem esta tabela, «atacar o sinal» é uma frase sem tecto.*
//!
//! # A régua, e porque ela não é uma cópia da lei
//!
//! A metade da distância monta-se pelas **mesmas portas** que o produto monta
//! ([`crate::profile_arc::dist2_recta_tree`] · [`crate::profile_arc::dist2_tree`]), na mesma ordem
//! e com o mesmo acumulador de `min`. ⭐ O que se conta é a diferença: o SINAL é
//! `total − distância − (a raiz e o produto do fim)`.
//!
//! ⚠️ **A inclinação em `N`, não o valor absoluto** — assim a raiz, o produto e a redução do
//! enrolamento (que são `O(1)`) saem da conta.

use crate::Field;
use fidget::context::Tree;
use ph2d_field::{FillRule, Profile};

/// Um polígono regular de `n` lados, e a versão dele com **toda** primitiva a ser um arco.
fn poligono(n: usize, bulge: f32) -> Profile {
    #[allow(clippy::cast_precision_loss)]
    let pt = |k: usize| {
        let t = std::f32::consts::TAU * k as f32 / n as f32;
        [t.cos(), t.sin()]
    };
    let pts: Vec<[f32; 2]> = (0..n).map(pt).collect();
    if bulge == 0.0 {
        return Profile::new(vec![pts], FillRule::NonZero, 1.0e-4).expect("um polígono é válido");
    }
    // ⚠️ **A polilinha viaja ao lado da decomposição** e não é opcional: ela é o que os
    // consumidores que não sabem ler um arco vêem. Aqui ela é a grosseira de propósito — esta sonda
    // conta o caminho dos ARCOS, que é o que `sd_profile` toma quando a decomposição existe.
    let com: Vec<([f32; 2], f32)> = pts.iter().map(|p| (*p, bulge)).collect();
    Profile::with_arcs(vec![(pts, com)], FillRule::NonZero, 1.0e-4)
        .expect("um polígono de arcos é válido")
}

/// A metade da DISTÂNCIA, pelas portas do produto — o acumulador de `min` e mais nada.
fn so_a_distancia(p: &Profile) -> Tree {
    let (u, v) = (Tree::x(), Tree::y());
    let mut dist2: Option<Tree> = None;
    for (ci, contour) in p.contours().iter().enumerate() {
        let arcs = p.arcs().get(ci).filter(|a| !a.is_empty());
        let pts: Vec<[f32; 2]> = match arcs {
            Some(a) => a.iter().map(|(q, _)| *q).collect(),
            None => contour.clone(),
        };
        for i in 0..pts.len() {
            let j = (i + 1) % pts.len();
            let (a, b) = (
                [f64::from(pts[i][0]), f64::from(pts[i][1])],
                [f64::from(pts[j][0]), f64::from(pts[j][1])],
            );
            let bulge = arcs.map_or(0.0, |x| f64::from(x[i].1));
            let d = if bulge == 0.0 {
                crate::profile_arc::dist2_recta_tree(
                    &(u.clone() - Tree::constant(a[0])),
                    &(v.clone() - Tree::constant(a[1])),
                    [b[0] - a[0], b[1] - a[1]],
                )
            } else {
                let k = crate::profile_arc::arco(a, b, bulge);
                crate::profile_arc::dist2_tree(&u, &v, a, b, &k)
            };
            dist2 = Some(match dist2 {
                None => d,
                Some(acc) => acc.min(d),
            });
        }
    }
    dist2.expect("um perfil válido tem ao menos uma primitiva")
}

fn linhas(t: &Tree) -> usize {
    Field::from_tree(t).tape_shape().map_or(0, |s| s.guardados)
}

/// ⏱️⭐⭐⭐⭐ **A sonda.** Ver o cabeçalho do módulo.
#[test]
#[ignore = "sonda de diagnóstico: conta as duas metades de uma primitiva de perfil"]
fn diag_a_decomposicao_de_uma_primitiva() {
    println!("\n  primitiva · N · total · distância · sinal · por primitiva (tot · dist · sinal)");
    for (nome, bulge) in [("recta", 0.0f32), ("arco", 0.35)] {
        let mut medidas = Vec::new();
        for n in [8usize, 16, 32, 64] {
            let p = poligono(n, bulge);
            let (t, d) = (
                linhas(&crate::profile::sd_profile(&p, &Tree::x(), &Tree::y())),
                linhas(&so_a_distancia(&p)),
            );
            medidas.push((n, t, d));
            #[allow(clippy::cast_precision_loss)]
            let por = |x: usize| x as f32 / n as f32;
            println!(
                "  {nome:>9} · {n:>2} · {t:>5} · {d:>9} · {:>5} · {:>6.1} · {:>5.1} · {:>5.1}",
                t.saturating_sub(d),
                por(t),
                por(d),
                por(t.saturating_sub(d)),
            );
        }
        // ⭐ A INCLINAÇÃO é o que tira o `O(1)` (a raiz, o produto do sinal, a redução).
        let (n0, t0, d0) = medidas[0];
        let (n1, t1, d1) = medidas[medidas.len() - 1];
        #[allow(clippy::cast_precision_loss)]
        let por = |a: usize, b: usize| (a - b) as f32 / (n1 - n0) as f32;
        let (tot, dis) = (por(t1, t0), por(d1, d0));
        println!(
            "  ⇒ {nome}: por primitiva, total {tot:.2} · distância {dis:.2} ({:.0} %) · \
             sinal {:.2} ({:.0} %)",
            100.0 * dis / tot,
            tot - dis,
            100.0 * (tot - dis) / tot,
        );
    }
    println!();
}
