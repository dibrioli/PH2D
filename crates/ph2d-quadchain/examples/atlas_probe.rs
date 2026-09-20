//! ⭐⭐⭐ **A PARAMETRIZAÇÃO VISTA COMO ATLAS** — as cinco perguntas da §9 da avaliação
//! `docs/3D/25_avaliacao_o_painter_na_malha.md`, medidas.
//!
//! ```text
//! cargo run --release -p ph2d-quadchain --example atlas_probe -- <peca.obj|esfera:48> [escala]
//! ```
//!
//! ⛔ **`--release`.** O solver contínuo gasta rondas de Gauss–Seidel aos milhares; em
//! `debug` isto é minutos por peça.
//!
//! # Por que esta sonda existe, e o que ela NÃO é
//!
//! A `ph2d-gridmap` foi construída para **extrair quads**, e a pergunta desta sonda é
//! outra: *ela serve de ATLAS para uma textura?* São grandezas diferentes e nenhuma
//! régua desta casa as media —
//!
//! | a extracção pergunta | um atlas pergunta |
//! |---|---|
//! | as isolinhas inteiras fecham? | o mapa **DOBRA** sobre si mesmo? |
//! | o `χ` sobrevive? | quantas **ILHAS**, e quanta **COSTURA**? |
//! | quantos quads saem? | que **RESOLUÇÃO** a peça pede, e quanto do atlas se desperdiça? |
//!
//! ⚠️⚠️ **E a pergunta que decide a arquitectura inteira é a primeira linha da tabela:**
//! a cadeia do botão REMALHA antes de parametrizar (F1), e uma textura tem de viver na
//! malha que o artista esculpiu. *Se a parametrização só funcionar depois do F1, o
//! caminho da textura destrói o trabalho dele* — por isso cada peça é medida **duas
//! vezes**, CRUA e remalhada, lado a lado.
//!
//! ⛔⛔ **Ela NÃO corre a [`ph2d_quadchain::quads_from_mesh_raw`]**, e a razão é que ela
//! mede um caminho que aquele não tem: um atlas precisa do mapa **CONTÍNUO** (G3) e o
//! botão precisa dele **INTEIRO** (G3+G5, que a `ChainTiming` soma numa coluna só).
//! *Arredondar a grade a inteiros é trabalho da extracção, nunca de uma textura.* As
//! fases partilhadas são as mesmas funções com os mesmos argumentos, e o número de
//! G3+G5 continua a ler-se no `chain_time`, que corre a porta do produto.

use ph2d_gridmap::{CutMesh, GridMap};
use ph2d_mesh::Mesh;

/// Uma peça: um caminho de `.obj`, ou uma forma da casa.
fn load(name: &str) -> Mesh {
    if let Some(rest) = name.strip_prefix("esfera:") {
        let n: usize = rest.parse().unwrap_or(48);
        return ph2d_mesh::shapes::uv_sphere(n, n * 3 / 2, 1.0);
    }
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

/// O comprimento de uma polilinha de vértices **globais**, em unidades de mundo.
fn chain_len(pos: &[[f32; 3]], chain: &[u32]) -> f64 {
    chain
        .windows(2)
        .map(|w| {
            let (a, b) = (pos[w[0] as usize], pos[w[1] as usize]);
            let d = [
                f64::from(a[0] - b[0]),
                f64::from(a[1] - b[1]),
                f64::from(a[2] - b[2]),
            ];
            d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
        })
        .sum()
}

/// A área **com sinal** de um triângulo no plano `(u, v)`.
fn uv_area2(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f64 {
    let (ux, uy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
    let (vx, vy) = (f64::from(c[0] - a[0]), f64::from(c[1] - a[1]));
    ux.mul_add(vy, -(uy * vx)) * 0.5
}

/// ⭐ O que a sonda mede de um par `(malha, mapa)`.
#[derive(Default)]
struct Atlas {
    /// Ilhas: um patch é uma carta.
    patches: usize,
    /// Quantas delas saíram discos (`χ = 1`) — o único resultado bom.
    discs: usize,
    /// Costuras interiores.
    seams: usize,
    /// O comprimento somado delas, em unidades de mundo.
    seam_len: f64,
    /// ⛔ Triângulos cuja área em `(u, v)` tem o sinal da MINORIA do patch: o mapa
    /// **dobrou** ali, e um atlas dobrado pinta dois sítios da peça no mesmo texel.
    folds: usize,
    /// Quantos triângulos entraram na conta.
    tris: usize,
    /// ⛔⛔ Quantos ficaram de FORA por o patch não ter `(u, v)` naquele local.
    ///
    /// ⚠️ **Sem esta coluna a sonda mente com a cara de um resultado perfeito:** a 1.ª
    /// corrida sobre a malha CRUA imprimiu `dobras 0/0 (0,00 %)`, que se lê como *«o
    /// mapa não dobra em lado nenhum»* e queria dizer *«nada foi medido»*. É a lei que
    /// esta casa já pagou nas duas réguas de valência do quad remesh — *um zero de «não
    /// medido» e um de «perfeito» são o mesmo byte*.
    sem_uv: usize,
    /// A área somada das cartas em `(u, v)`, em unidades de grade ao quadrado.
    uv_area: f64,
    /// A área somada das CAIXAS que envolvem cada carta — o que um empacotador de
    /// rectângulos teria de arrumar. ⚠️ É um **limite superior optimista** do
    /// aproveitamento: ele mede o desperdício DENTRO da caixa, e um empacotador
    /// desperdiça mais ainda entre elas.
    box_area: f64,
}

fn medir_atlas(mesh: &Mesh, cut: &CutMesh, map: &GridMap) -> Atlas {
    let pos = mesh.positions();
    let mut a = Atlas {
        patches: cut.origin.len(),
        seams: cut.seams.len(),
        seam_len: cut.seams.iter().map(|s| chain_len(pos, &s.chain)).sum(),
        ..Atlas::default()
    };
    for (p, tris) in cut.tris.iter().enumerate() {
        let Some(uv) = map.uv.get(p) else { continue };
        let (mut pos_n, mut neg_n) = (0usize, 0usize);
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        let mut soma = 0.0f64;
        for t in tris {
            // ⚠️ Um triângulo que aponte para um local que o patch não tem é um defeito
            // do CORTE e não do mapa — saltá-lo evita misturar dois achados.
            let (Some(&p0), Some(&p1), Some(&p2)) = (
                uv.get(t[0] as usize),
                uv.get(t[1] as usize),
                uv.get(t[2] as usize),
            ) else {
                a.sem_uv += 1;
                continue;
            };
            let s = uv_area2(p0, p1, p2);
            if s > 0.0 {
                pos_n += 1;
            } else if s < 0.0 {
                neg_n += 1;
            }
            soma += s.abs();
            for q in [p0, p1, p2] {
                lo[0] = lo[0].min(q[0]);
                lo[1] = lo[1].min(q[1]);
                hi[0] = hi[0].max(q[0]);
                hi[1] = hi[1].max(q[1]);
            }
        }
        a.tris += pos_n + neg_n;
        a.folds += pos_n.min(neg_n);
        a.uv_area += soma;
        if lo[0] <= hi[0] {
            a.box_area += f64::from(hi[0] - lo[0]) * f64::from(hi[1] - lo[1]);
        }
    }
    a
}

/// ⭐⭐⭐ **QUANTAS ILHAS UM ATLAS DE FACTO TERIA** — e não quantos patches o traçado fez.
///
/// ⛔⛔ *Um patch NÃO é uma ilha.* O mapa soldado acopla os dois lados de cada costura, e
/// onde o salto de período é `0 (mod 4)` **os dois lados leem a mesma função**: ali não há
/// corte nenhum, há uma linha que só existe porque o traçado partiu a peça em quads. Uma
/// ilha de textura só nasce onde a transição **RODA** — aí as duas cartas não cabem no
/// mesmo plano.
///
/// ⚠️ Devolve `(ilhas, coladas, rodadas, soltas, corte_len)` — e o último é **o
/// comprimento que um pintor SENTE**: só as costuras que rodam são cortes de verdade, e
/// somar todas conta linhas que não existem no atlas.
///
/// ⛔⛔ **O resíduo da costura NÃO se mede aqui, e a 1.ª redacção media-o — mal, duas
/// vezes.** A casa já tem [`ph2d_gridmap::seam_residual`], e a convenção dela é
/// `zb − turn2(za, jump) − shift`: eu rodei o lado ERRADO e depois esqueci a translação,
/// e li `2,17e1` e `3,84e1` células de grade sobre um solder que solda. *Uma terceira
/// cópia de uma lei é onde ela diverge* — hoje o número vem da porta que o produto usa.
fn ilhas_de_facto(
    pos: &[[f32; 3]],
    cut: &CutMesh,
    jumps: &[Option<i32>],
) -> (usize, usize, usize, usize, f64) {
    let n = cut.origin.len();
    let mut pai: Vec<usize> = (0..n).collect();
    fn raiz(pai: &mut [usize], mut x: usize) -> usize {
        while pai[x] != x {
            pai[x] = pai[pai[x]];
            x = pai[x];
        }
        x
    }
    let (mut coladas, mut rodadas, mut soltas) = (0usize, 0usize, 0usize);
    let mut corte_len = 0.0f64;
    for (s, seam) in cut.seams.iter().enumerate() {
        let Some(Some(j)) = jumps.get(s).copied() else {
            soltas += 1;
            corte_len += chain_len(pos, &seam.chain);
            continue;
        };
        let (pa, pb) = (seam.side[0].patch as usize, seam.side[1].patch as usize);
        if j.rem_euclid(4) == 0 {
            coladas += 1;
            let (ra, rb) = (raiz(&mut pai, pa), raiz(&mut pai, pb));
            if ra != rb {
                pai[ra] = rb;
            }
        } else {
            rodadas += 1;
            corte_len += chain_len(pos, &seam.chain);
        }
    }
    let ilhas = (0..n).filter(|&p| raiz(&mut pai, p) == p).count();
    (ilhas, coladas, rodadas, soltas, corte_len)
}

/// Corre F2 → G3 sobre uma malha e imprime o bloco dela.
fn corrida(rotulo: &str, mesh: &Mesh, alvo: f32, marca: &str) {
    let mut clock = std::time::Instant::now();
    let mut lap = || {
        let ms = clock.elapsed().as_secs_f64() * 1000.0;
        clock = std::time::Instant::now();
        ms
    };

    let dual = ph2d_crossfield::Dual::build(mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let ms_campo = lap();
    let layout = ph2d_trace::trace_patches(mesh, &dual, &field);
    let ms_trace = lap();
    let (cut, cutrep) = ph2d_gridmap::cut_along_patches(mesh, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(mesh, &layout, &cut);
    let ms_corte = lap();
    // ⛔⛔ **A PRIMEIRA REDACÇÃO DESTA SONDA CHAMOU `ph2d_gridmap::solve`, e era o motor
    // ERRADO.** Aquele é o G3 com a costura **PENALIZADA** (`SEAM_WEIGHT`), que a obra A
    // de 24/08 substituiu pela costura **ELIMINADA**; o produto corre o soldado desde
    // então. Medido na esfera de 24: o penalizado deu `10,76 %` de triângulos dobrados e
    // `5 784 ms`, o soldado dá o que a tabela abaixo imprime — e a diferença de relógio é
    // a das rondas (`160 000` contra `8 000`, `20×`, que o doc do `welded_rounds` já
    // escrevia). *Escolher a função pelo nome mais óbvio mede um programa que o produto
    // deixou de correr.*
    let rondas = ph2d_gridmap::RoundOptions::default().welded_rounds;
    let (map, wrel) = ph2d_gridmap::solve_welded(
        mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(alvo),
        rondas,
    );
    let rel = wrel.solve;
    let ms_mapa = lap();

    let mut a = medir_atlas(mesh, &cut, &map);
    a.discs = cutrep.discs;
    let area = f64::from(mesh.surface_area());
    let total = ms_campo + ms_trace + ms_corte + ms_mapa;

    println!(
        "\n-- {rotulo}: {} V | {} F | area {area:.4}",
        mesh.vert_count(),
        mesh.face_count()
    );
    println!(
        "   relogio  campo {ms_campo:8.1} | tracado {ms_trace:7.1} | corte {ms_corte:7.1} | \
         mapa-continuo {ms_mapa:9.1}   TOTAL {total:9.1} ms"
    );
    println!(
        "   ilhas    {} patches ({} discos) | {} costuras | fronteira {:.3} de mundo = {:.2}x sqrt(area)",
        a.patches,
        a.discs,
        a.seams,
        a.seam_len,
        a.seam_len / area.sqrt().max(1.0e-12)
    );
    let pct_fold = if a.tris == 0 {
        0.0
    } else {
        100.0 * as_f64(a.folds) / as_f64(a.tris)
    };
    let jumps = ph2d_gridmap::jumps_only(mesh, &cut, &combed);
    let (ilhas, coladas, rodadas, soltas, corte_len) =
        ilhas_de_facto(mesh.positions(), &cut, &jumps);
    // ⭐ O resíduo vem da PORTA da casa — ver a nota em [`ilhas_de_facto`].
    let (w, _) = ph2d_gridmap::weld(&cut, &combed);
    let sr = ph2d_gridmap::seam_residual(&w, &map);
    println!(
        "   ILHAS    {ilhas} de facto (de {} patches) | costuras {coladas} coladas + {rodadas} rodadas + {soltas} soltas | \
         CORTE que o pintor sente {corte_len:.3} = {:.2}x sqrt(area) ({:.0}% da fronteira)",
        a.patches,
        corte_len / area.sqrt().max(1.0e-12),
        100.0 * corte_len / a.seam_len.max(1.0e-12)
    );
    println!(
        "   costura  {} elos eliminados: p50 {:.2e} max {:.2e} | {} fechos: max {:.2e}",
        sr.links, sr.p50, sr.max, sr.closures, sr.closure_max
    );
    println!(
        "   solver   {} triangulos na energia | {} saltados | {} pares de costura | {} costuras soltas",
        rel.triangles, rel.skipped, rel.pairs, rel.loose_seams
    );
    // ⭐ DUAS medidas da MESMA grandeza, de propósito: a minha (sinal da área UV, por
    // patch) e a do solver (`folded_after`). *Quando uma página imprime duas medidas da
    // mesma coisa e elas discordam, isso É o achado* — a lei que a régua do `reach`
    // pagou em 31/08.
    println!(
        "   dobras   {}/{} triangulos com a area UV invertida ({pct_fold:.2}%) | {} SEM (u,v) | \
         o solver conta {} antes e {} depois",
        a.folds, a.tris, a.sem_uv, wrel.folded_before, wrel.folded_after
    );
    // ⛔ O piso de população: sem ele um `0/0` lê-se como um mapa impecável.
    assert!(
        a.tris > 0 || a.sem_uv > 0,
        "nem um triangulo chegou a ser medido em {rotulo} — a sonda nao mediu nada"
    );
    let fill = if a.box_area <= 0.0 {
        0.0
    } else {
        100.0 * a.uv_area / a.box_area
    };
    println!(
        "   atlas    caixas somadas {:.1} | cartas {:.1} => aproveitamento DENTRO da caixa {fill:.1}%",
        a.box_area, a.uv_area
    );
    // ⚠️ A área em `(u, v)` vem em unidades de GRADE, e uma unidade de grade vale `alvo`
    // de mundo — sem essa conversão os dois números não são comparáveis.
    let uv_em_mundo = a.uv_area * f64::from(alvo) * f64::from(alvo);
    println!(
        "   escala   area em UV {uv_em_mundo:.4} de mundo contra {area:.4} de superficie = {:.3}x",
        uv_em_mundo / area.max(1.0e-12)
    );
    // ⭐⭐⭐ **O ATLAS DE VERDADE** — as ilhas juntas, assentes e arrumadas em `[0,1]²`.
    // Tudo acima é a matéria-prima; isto é o que uma textura recebe.
    let relogio = std::time::Instant::now();
    let atl = ph2d_uv_atlas::build(mesh, &cut, &map, &jumps);
    let ms_atlas = relogio.elapsed().as_secs_f64() * 1000.0;
    let r = atl.relatorio;
    println!(
        "   ATLAS    {} ilhas | {} cantos, {} orfaos | {} cortes que o atlas obrigou (rasgo {:.2e}) | \
         cola_max {:.2e} | aproveitamento {:.1}% | {ms_atlas:.1} ms",
        r.ilhas,
        r.cantos,
        r.orfaos,
        r.ciclos,
        r.holonomia_max,
        r.cola_max,
        100.0 * f64::from(r.aproveitamento)
    );
    let (cobertos, dobrados) = sobreposicao(&atl, mesh, 1024);
    println!(
        "   dobra    {dobrados} de {cobertos} texels de 1024^2 pintados MAIS DE UMA VEZ ({:.2}%)",
        if cobertos == 0 {
            0.0
        } else {
            100.0 * as_f64(dobrados) / as_f64(cobertos)
        }
    );
    if let Ok(dir) = std::env::var("PH2D_ATLAS_DUMP") {
        desenha(&atl, mesh, &format!("{dir}/atlas_{marca}.ppm"), 1024);
    }
    // ⭐ A resolução que a peça pede: com o atlas aproveitado a `fill`, quanto mede um texel.
    for n in [1024u32, 2048, 4096] {
        let uteis = f64::from(n) * f64::from(n) * (fill / 100.0);
        let texel = (area / uteis.max(1.0)).sqrt();
        println!(
            "   {n}^2     um texel mede {texel:.5} de mundo = 1/{:.1} do quad pedido ({alvo:.5})",
            f64::from(alvo) / texel.max(1.0e-12)
        );
    }
}

/// ⭐ **DESENHA O ATLAS** num `.ppm` — uma cor por ilha, os triângulos preenchidos.
///
/// ⚠️ É a única forma de o dono ver o que a arrumação fez. *Uma tabela de aproveitamento
/// não diz se as ilhas ficaram legíveis* — e esta linha já leu duas imagens ao contrário
/// por decidir por tabela.
fn desenha(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh, caminho: &str, lado: usize) {
    let mut px = vec![[24u8, 24, 28]; lado * lado];
    let cor = |i: u32| -> [u8; 3] {
        // Uma roda de matizes: ilhas vizinhas nunca saem parecidas.
        let h = f32::from(u16::try_from(i % 12).unwrap_or(0)) / 12.0 * 6.0;
        let f = h - h.floor();
        let (a, b) = ((255.0 * f) as u8, (255.0 * (1.0 - f)) as u8);
        match h as u32 {
            0 => [255, a, 40],
            1 => [b, 255, 40],
            2 => [40, 255, a],
            3 => [40, b, 255],
            4 => [a, 40, 255],
            _ => [255, 40, b],
        }
    };
    let (base, _) = ph2d_uv_atlas::bases_dos_cantos(mesh);
    for (f, face) in mesh.faces().iter().enumerate() {
        let n = face.verts().len();
        if n < 3 {
            continue;
        }
        let b = base[f] as usize;
        let c = cor(atlas.ilha[b]);
        // Leque a partir do canto 0 — chega para triângulos e quads.
        for k in 1..(n - 1) {
            let t = [atlas.uv[b], atlas.uv[b + k], atlas.uv[b + k + 1]];
            preenche(&mut px, lado, t, c);
        }
    }
    let mut out = format!("P6\n{lado} {lado}\n255\n").into_bytes();
    for p in &px {
        out.extend_from_slice(p);
    }
    if let Err(e) = std::fs::write(caminho, out) {
        println!("   (nao consegui escrever {caminho}: {e})");
    } else {
        println!("   desenho  {caminho}");
    }
}

/// ⭐⭐⭐ **QUANTOS TEXELS O ATLAS PINTA DUAS VEZES** — a única pergunta de CORRECÇÃO que
/// sobra depois de as ilhas caberem no quadrado.
///
/// ⛔ Uma ilha assentada ao longo de uma árvore não tem holonomia **e pode dobrar-se sobre
/// si mesma** — nada no assentamento o impede. Um texel coberto por dois sítios da
/// superfície é tinta que aparece onde ninguém a pôs. ⚠️ *Nenhum gate desta casa media
/// isto*: as réguas do atlas olham caixas, e uma dobra acontece DENTRO de uma caixa.
///
/// Devolve `(texels cobertos, texels cobertos MAIS DE UMA VEZ)`.
fn sobreposicao(atlas: &ph2d_uv_atlas::Atlas, mesh: &Mesh, lado: usize) -> (usize, usize) {
    let mut n = vec![0u16; lado * lado];
    let (base, _) = ph2d_uv_atlas::bases_dos_cantos(mesh);
    for (f, face) in mesh.faces().iter().enumerate() {
        let k = face.verts().len();
        if k < 3 {
            continue;
        }
        let b = base[f] as usize;
        for j in 1..(k - 1) {
            conta(
                &mut n,
                lado,
                [atlas.uv[b], atlas.uv[b + j], atlas.uv[b + j + 1]],
            );
        }
    }
    let cobertos = n.iter().filter(|&&c| c > 0).count();
    let dobrados = n.iter().filter(|&&c| c > 1).count();
    (cobertos, dobrados)
}

/// A mesma varredura do [`preenche`], a CONTAR em vez de pintar.
///
/// ⚠️ Ela é uma segunda travessia do mesmo rectângulo de propósito: o pintor escreve a
/// última cor e a contagem soma, e juntar as duas num só percurso faria a imagem depender
/// de quem se sobrepõe. *Duas perguntas, duas varreduras.*
fn conta(n: &mut [u16], lado: usize, t: [[f32; 2]; 3]) {
    varre(lado, t, &mut |i| n[i] = n[i].saturating_add(1));
}

/// Um triângulo cheio, em coordenadas `[0,1]²`.
fn preenche(px: &mut [[u8; 3]], lado: usize, t: [[f32; 2]; 3], c: [u8; 3]) {
    varre(lado, t, &mut |i| px[i] = c);
}

/// ⭐ **A VARREDURA, uma só** — quem pinta e quem conta percorrem exactamente os mesmos
/// texels, senão a imagem e o número descrevem atlas diferentes.
fn varre(lado: usize, t: [[f32; 2]; 3], f: &mut dyn FnMut(usize)) {
    let n = lado as f32;
    let p: Vec<[f32; 2]> = t.iter().map(|z| [z[0] * n, (1.0 - z[1]) * n]).collect();
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for q in &p {
        lo[0] = lo[0].min(q[0]);
        lo[1] = lo[1].min(q[1]);
        hi[0] = hi[0].max(q[0]);
        hi[1] = hi[1].max(q[1]);
    }
    let y0 = lo[1].floor().max(0.0) as usize;
    let y1 = (hi[1].ceil().max(0.0) as usize).min(lado);
    let x0 = lo[0].floor().max(0.0) as usize;
    let x1 = (hi[0].ceil().max(0.0) as usize).min(lado);
    for y in y0..y1 {
        for x in x0..x1 {
            let q = [x as f32 + 0.5, y as f32 + 0.5];
            let w = |a: [f32; 2], b: [f32; 2]| {
                (b[0] - a[0]).mul_add(q[1] - a[1], -((b[1] - a[1]) * (q[0] - a[0])))
            };
            let (u, v, s) = (w(p[0], p[1]), w(p[1], p[2]), w(p[2], p[0]));
            let dentro = (u >= 0.0 && v >= 0.0 && s >= 0.0) || (u <= 0.0 && v <= 0.0 && s <= 0.0);
            if dentro {
                f(y * lado + x);
            }
        }
    }
}

/// Uma contagem como `f64`, sem o `as` solto que o clippy da casa recusa.
fn as_f64(n: usize) -> f64 {
    u32::try_from(n).map_or(f64::from(u32::MAX), f64::from)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera:48"));
    let escala: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut crua = load(&name);
    crua.triangulate();
    let alvo = ph2d_remesh_iso::target_edge(&crua, ph2d_remesh_iso::ALPHA) * escala;
    println!("peca {name} | alvo de aresta {alvo:.5}");

    // ⭐⭐⭐ **A PERGUNTA QUE DECIDE A ARQUITECTURA:** a mesma medição nas duas entradas.
    corrida("CRUA (a malha do artista)", &crua, alvo, "crua");
    let f1 = ph2d_quadchain::phase_zero(&crua, alvo);
    corrida("F1 (remalhada, como o botao faz)", &f1, alvo, "f1");
}
