//! ⭐⭐⭐ **AS SONDAS DO DESEMARANHAMENTO** — irmã de [`super::field_wakes`] pelo teto de LOC
//! da shell (HR-18, 600), cortada por RESPONSABILIDADE: lá as réguas do **campo** e do
//! **traçado** (singularidades, paredes, `SoG`, dobras por casca); aqui as três que medem o
//! **desemaranhador** — a costura livre, a contagem de dobras de um mapa, e o passe restrito.
//!
//! ⚠️ **Elas são chamadas pela sonda irmã**, e é por isso que são `pub(super)`: uma sonda que
//! duplicasse a construção dos elementos mediria a duplicação, não o produto.

use ph2d_untangle::Element;

/// ⭐⭐⭐ **A OBRA GRANDE, medida** — a injectividade como objectivo, nas variáveis reduzidas.
///
/// ⚠️ **É a irmã honesta da [`seam_free_probe`]**, que media a mesma ideia por **projecção** e
/// estagnava a oscilar. Aqui a costura é a **variável**, e não há nada a desfazer.
pub(super) fn injective_probe(
    work: &ph2d_mesh::Mesh,
    cut: &ph2d_gridmap::CutMesh,
    combed: &ph2d_gridmap::Combed,
    target: f32,
) {
    let (mut map, _) = ph2d_gridmap::solve_welded(
        work,
        cut,
        combed,
        ph2d_gridmap::Step::uniform(target),
        ph2d_gridmap::RoundOptions::default().welded_rounds,
    );
    let (w, _) = ph2d_gridmap::weld(cut, combed);
    let relogio = std::time::Instant::now();
    // ⚠️ **O orçamento entra pela env** porque a 1.ª medição gastou-o TODO
    // (`64 externas / 2048 internas` = exactamente os tectos) — e um número que bate no tecto não
    // diz onde o método pára, diz onde o relógio parou. *`33` truncado e `33` convergido são a
    // mesma impressão e duas conclusões opostas.*
    let base = ph2d_untangle::Settings::default();
    let set = ph2d_untangle::Settings {
        max_outer: std::env::var("PH2D_INJ_OUTER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(base.max_outer),
        max_inner: std::env::var("PH2D_INJ_INNER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(base.max_inner),
        ..base
    };
    // ⚠️ **O MESMO passo que resolveu o mapa** — o repouso da energia vive em unidades de
    // célula, e uma sonda que passasse outro passo mediria outro problema.
    let rep = ph2d_gridmap::make_injective(
        work,
        cut,
        &w,
        &mut map,
        ph2d_gridmap::Step::uniform(target),
        set,
    );
    eprintln!(
        "  OBRA GRANDE (injectividade nas variaveis reduzidas): dobras {} -> {}  |  min det \
         {:.3e} -> {:.3e}  |  {} externas / {} internas  |  {:.0} ms  --  {}",
        rep.flipped_before,
        rep.flipped_after,
        rep.min_det.0,
        rep.min_det.1,
        rep.outer,
        rep.inner,
        relogio.elapsed().as_secs_f64() * 1000.0,
        if rep.flipped_after == 0 {
            "⭐⭐⭐ ZERA"
        } else if rep.flipped_after * 4 < rep.flipped_before {
            "⭐ desce MUITO"
        } else if rep.flipped_after < rep.flipped_before {
            "⚠️ desce"
        } else {
            "⛔ NAO desce"
        }
    );
}

/// ⭐⭐⭐ **A COSTURA LIVRE, por PROJECÇÃO** — o teste de viabilidade da obra grande.
pub(super) fn seam_free_probe(
    work: &ph2d_mesh::Mesh,
    cut: &ph2d_gridmap::CutMesh,
    combed: &ph2d_gridmap::Combed,
    target: f32,
) {
    let (mut map, _) = ph2d_gridmap::solve_welded(
        work,
        cut,
        combed,
        ph2d_gridmap::Step::uniform(target),
        ph2d_gridmap::RoundOptions::default().welded_rounds,
    );
    let (w, _) = ph2d_gridmap::weld(cut, combed);
    let antes = flips_of(work, cut, &map);
    let relogio = std::time::Instant::now();
    let pos = work.positions();
    // ⚠️ **Poucas iterações internas por ronda, de propósito:** a projecção tem de entrar
    // cedo e muitas vezes. *Descer até ao fundo e só depois projectar devolve um ponto que a
    // projecção desfaz.*
    let set = ph2d_untangle::Settings {
        max_outer: 4,
        max_inner: 8,
        ..ph2d_untangle::Settings::default()
    };
    let mut curva = Vec::new();
    for _ in 0..12 {
        for (p, tris) in cut.tris.iter().enumerate() {
            let (Some(origin), Some(uvp)) = (cut.origin.get(p), map.uv.get(p)) else {
                continue;
            };
            let elements: Vec<_> = tris
                .iter()
                .filter_map(|t| element_for(pos, origin, *t))
                .collect();
            let mut uv: Vec<[f64; 2]> = uvp
                .iter()
                .map(|c| [f64::from(c[0]), f64::from(c[1])])
                .collect();
            if ph2d_untangle::flipped(&elements, &uv) == 0 {
                continue;
            }
            // ⛔ NADA preso — é isto que distingue esta sonda da do produto.
            let livre = vec![false; uv.len()];
            ph2d_untangle::untangle(&elements, &mut uv, &livre, set);
            if let Some(slot) = map.uv.get_mut(p) {
                *slot = uv
                    .iter()
                    .map(|c| {
                        #[expect(
                            clippy::cast_possible_truncation,
                            reason = "o mapa e' f32; a descida corre em f64 e volta"
                        )]
                        [c[0] as f32, c[1] as f32]
                    })
                    .collect();
            }
        }
        // ⭐ A PROJECÇÃO: a costura volta a valer, exactamente.
        for c in 0..w.classes() {
            w.derive(&mut map, c);
        }
        curva.push(flips_of(work, cut, &map));
    }
    let depois = curva.last().copied().unwrap_or(antes);
    eprintln!(
        "  COSTURA LIVRE (descida projectada): dobras {antes} -> {depois}  |  curva {curva:?}  |  {:.0} ms",
        relogio.elapsed().as_secs_f64() * 1000.0
    );
    // ⛔⛔ **O VEREDITO LÊ A CURVA, e não só o último número** — e a 1.ª redacção disto lia só
    // o último, com um limiar de metade: sobre `120 → 66` ela imprimia *«a obra grande está
    // condenada»*, que é **mais do que esta sonda mede**.
    //
    // ⚠️ **O que ela de facto mede é uma APROXIMAÇÃO da obra grande**, e uma aproximação com um
    // defeito conhecido: a costura entra por **projecção** (`derive` empurra o valor da cópia
    // RAIZ para as outras), logo **todo o trabalho que a descida fez nas cópias não-raiz é
    // deitado fora a cada ronda**. *A projecção luta com a descida* — e é isso que uma curva
    // que estabiliza e depois **oscila** diz.
    //
    // ⇒ ⛔ **Um planalto com oscilação não condena a obra grande; condena ESTE instrumento.**
    // A obra grande exprime a energia **nas variáveis livres** do `ClosureSystem` — a costura
    // por **eliminação**, não por projecção —, e aí a descida nunca produz um estado que a
    // restrição tenha de desfazer.
    let oscila = curva.windows(2).filter(|w| w[1] > w[0]).count();
    eprintln!(
        "  -> {}",
        if depois == 0 {
            "⭐⭐⭐ ZERA: a liberdade da costura CHEGA, e a obra grande tem sujeito"
        } else if oscila >= 2 {
            "⚠️ PLANALTO COM OSCILACAO: a projeccao luta com a descida (o `derive` deita fora o \
             trabalho das copias nao-raiz) -- INCONCLUSIVO sobre a obra grande, que elimina em \
             vez de projectar"
        } else {
            "⚠️ desce e para, sem oscilar -- o limite pode ser da liberdade, e ai' a obra grande \
             ajuda sem resolver"
        }
    );
}

/// Quantos triângulos do mapa estão dobrados — a régua partilhada pelos dois lados do A/B.
pub(super) fn flips_of(
    work: &ph2d_mesh::Mesh,
    cut: &ph2d_gridmap::CutMesh,
    map: &ph2d_gridmap::GridMap,
) -> usize {
    let pos = work.positions();
    let mut total = 0;
    for (p, tris) in cut.tris.iter().enumerate() {
        let (Some(origin), Some(uvp)) = (cut.origin.get(p), map.uv.get(p)) else {
            continue;
        };
        let elements: Vec<_> = tris
            .iter()
            .filter_map(|t| element_for(pos, origin, *t))
            .collect();
        let uv: Vec<[f64; 2]> = uvp
            .iter()
            .map(|c| [f64::from(c[0]), f64::from(c[1])])
            .collect();
        total += ph2d_untangle::flipped(&elements, &uv);
    }
    total
}

/// O elemento de um triângulo, com o repouso achatado isometricamente.
fn element_for(pos: &[[f32; 3]], origin: &[u32], t: [u32; 3]) -> Option<Element> {
    let q: Vec<[f64; 3]> = t
        .iter()
        .map(|&l| {
            let g = *origin.get(l as usize).unwrap_or(&0) as usize;
            let v = pos.get(g).copied().unwrap_or([0.0; 3]);
            [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]
        })
        .collect();
    let e1 = [q[1][0] - q[0][0], q[1][1] - q[0][1], q[1][2] - q[0][2]];
    let e2 = [q[2][0] - q[0][0], q[2][1] - q[0][1], q[2][2] - q[0][2]];
    let l1 = e1[0]
        .mul_add(e1[0], e1[1].mul_add(e1[1], e1[2] * e1[2]))
        .sqrt();
    if !l1.is_finite() || l1 <= 0.0 {
        return None;
    }
    let u = [e1[0] / l1, e1[1] / l1, e1[2] / l1];
    let x = e2[0].mul_add(u[0], e2[1].mul_add(u[1], e2[2] * u[2]));
    let sq = e2[0].mul_add(e2[0], e2[1].mul_add(e2[1], e2[2] * e2[2])) - x * x;
    let y = if sq > 0.0 { sq.sqrt() } else { 0.0 };
    Element::from_rest(t, [0.0, 0.0], [l1, 0.0], [x, y])
}

/// ⭐⭐⭐ **O DESEMARANHADOR SOBRE O NOSSO MAPA** — retalho a retalho, fronteira presa.
///
/// ⚠️ **O repouso é o triângulo 3D achatado ISOMETRICAMENTE** (`p0` na origem, `p1` no eixo
/// `x`): ele preserva os comprimentos das arestas, então a energia mede a distorção **do mapa**
/// e não a de um achatamento que já distorce.
///
/// ⚠️ **A fronteira do retalho fica presa** — é a metade que mantém a costura intacta.
pub(super) fn untangle_probe(
    work: &ph2d_mesh::Mesh,
    cut: &ph2d_gridmap::CutMesh,
    map: &ph2d_gridmap::GridMap,
) {
    use std::collections::BTreeMap;
    let pos = work.positions();
    let (mut antes, mut depois, mut desistiu, mut patches) = (0usize, 0usize, 0usize, 0usize);
    let relogio = std::time::Instant::now();
    for (p, tris) in cut.tris.iter().enumerate() {
        let (Some(origin), Some(uvp)) = (cut.origin.get(p), map.uv.get(p)) else {
            continue;
        };
        if tris.is_empty() || uvp.is_empty() {
            continue;
        }
        // As arestas com UMA face só são a fronteira do retalho.
        let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for t in tris {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
            }
        }
        let mut locked = vec![false; uvp.len()];
        for (e, c) in &n {
            if *c == 1 {
                locked[e.0 as usize] = true;
                locked[e.1 as usize] = true;
            }
        }
        let mut elements = Vec::with_capacity(tris.len());
        for t in tris {
            let q: Vec<[f64; 3]> = t
                .iter()
                .map(|&l| {
                    let g = origin[l as usize] as usize;
                    [
                        f64::from(pos[g][0]),
                        f64::from(pos[g][1]),
                        f64::from(pos[g][2]),
                    ]
                })
                .collect();
            let e1 = [q[1][0] - q[0][0], q[1][1] - q[0][1], q[1][2] - q[0][2]];
            let e2 = [q[2][0] - q[0][0], q[2][1] - q[0][1], q[2][2] - q[0][2]];
            let l1 = e1[0]
                .mul_add(e1[0], e1[1].mul_add(e1[1], e1[2] * e1[2]))
                .sqrt();
            if l1 <= 0.0 {
                continue;
            }
            let u = [e1[0] / l1, e1[1] / l1, e1[2] / l1];
            let x = e2[0].mul_add(u[0], e2[1].mul_add(u[1], e2[2] * u[2]));
            let sq = e2[0].mul_add(e2[0], e2[1].mul_add(e2[1], e2[2] * e2[2])) - x * x;
            let y = if sq > 0.0 { sq.sqrt() } else { 0.0 };
            if let Some(el) = Element::from_rest(*t, [0.0, 0.0], [l1, 0.0], [x, y]) {
                elements.push(el);
            }
        }
        let mut uv: Vec<[f64; 2]> = uvp
            .iter()
            .map(|c| [f64::from(c[0]), f64::from(c[1])])
            .collect();
        let f0 = ph2d_untangle::flipped(&elements, &uv);
        if f0 == 0 {
            continue;
        }
        patches += 1;
        let rep = ph2d_untangle::untangle(
            &elements,
            &mut uv,
            &locked,
            ph2d_untangle::Settings::default(),
        );
        antes += rep.flipped_before;
        depois += rep.flipped_after;
        if rep.gave_up {
            desistiu += 1;
        }
    }
    eprintln!(
        "  DESEMARANHADOR (retalho a retalho, fronteira presa): {patches} retalho(s) com dobra  \
         |  dobras {antes} -> {depois}  |  {desistiu} sem fechar  |  {:.0} ms",
        relogio.elapsed().as_secs_f64() * 1000.0
    );
    if antes > 0 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "contagem de dobras para uma percentagem de diagnostico"
        )]
        let pct = 100.0 * (antes - depois) as f64 / antes as f64;
        eprintln!(
            "  -> {pct:.1} % das dobras desfeitas SEM tocar na costura -- {}",
            if depois == 0 {
                "⭐ TODAS: a cura existe e o preco esta' medido"
            } else {
                "⚠️ as que sobram vivem na FRONTEIRA dos retalhos, e sao outra wave"
            }
        );
    }
}
