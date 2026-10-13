//! **P6 — a silhueta das SPRITES com filtro** (doc 132 §6): quantos texels mudam entre as rotas (a
//! fatia da CPU × o buffer da placa) e entre os escopos (cada forma × camada inteira), e ONDE — a
//! distância do centro de cada texel à aresta do quad mais próxima.

use super::*;

/// Os quatro cantos do quad de `s` em pixels da camada (a câmera da [`Banca`]: `K` px por unidade,
/// centrada) — a mesma soma do `vs_main` do `sprite.wgsl`: `world_pos + B·(anchor + q·size)`.
fn cantos(s: &RenderInstance) -> [[f32; 2]; 4] {
    let [bx, by, bz, bw] = s.basis;
    [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]].map(|q| {
        let l = [
            s.anchor[0] + q[0] * s.size[0],
            s.anchor[1] + q[1] * s.size[1],
        ];
        let w = [
            s.world_pos[0] + l[0] * bx + l[1] * bz,
            s.world_pos[1] + l[0] * by + l[1] * bw,
        ];
        [W as f32 * 0.5 + w[0] * K, H as f32 * 0.5 - w[1] * K]
    })
}

/// A distância (px) do centro do texel `(x, y)` à aresta de quad mais próxima.
fn ate_a_aresta(quads: &[[[f32; 2]; 4]], x: u32, y: u32) -> f32 {
    let p = [x as f32 + 0.5, y as f32 + 0.5];
    let mut d = f32::MAX;
    for q in quads {
        for k in 0..4 {
            let (a, b) = (q[k], q[(k + 1) % 4]);
            let ab = [b[0] - a[0], b[1] - a[1]];
            let ap = [p[0] - a[0], p[1] - a[1]];
            let t =
                ((ap[0] * ab[0] + ap[1] * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1])).clamp(0.0, 1.0);
            let e = [ap[0] - ab[0] * t, ap[1] - ab[1] * t];
            d = d.min((e[0] * e[0] + e[1] * e[1]).sqrt());
        }
    }
    d
}

/// As sprites da cena: a sozinha do gate e mais quatro, rodadas, em posições sub-pixel.
pub(super) fn cena(b: &Banca) -> Vec<RenderInstance> {
    let mut v = vec![sprite(b, 0.37, -0.45)];
    for (x, y) in [(-6.13, 4.07), (6.29, 4.11), (-6.21, -4.33), (6.07, -4.19)] {
        v.push(sprite(b, x, y));
    }
    v
}

/// Uma comparação: os texels que diferem por mais de 1 nível, separados em silhueta (da cena crua
/// pela CPU) e fora dela, o pior nível de cada lado, e o histograma da distância à aresta.
fn compara(nome: &str, a: &[u8], c: &[u8], crua: &[u8], quads: &[[[f32; 2]; 4]]) -> usize {
    let alfa = |x: u32, y: u32| crua[((y * W + x) * 4 + 3) as usize] > 0;
    let (mut borda, mut fora, mut pior_b, mut pior_f, mut pintados) = (0, 0, 0u8, 0u8, 0);
    let mut hist = [0usize; 4];
    for y in 1..H - 1 {
        for x in 1..W - 1 {
            pintados += usize::from(alfa(x, y));
            let o = ((y * W + x) * 4) as usize;
            let d = (0..4)
                .map(|k| a[o + k].abs_diff(c[o + k]))
                .max()
                .unwrap_or(0);
            if d <= 1 {
                continue;
            }
            let silhueta = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                .iter()
                .any(|&(i, j)| alfa(i, j) != alfa(x, y));
            if silhueta {
                borda += 1;
                pior_b = pior_b.max(d);
                let e = ate_a_aresta(quads, x, y);
                hist[match e {
                    e if e < 0.01 => 0,
                    e if e < 0.1 => 1,
                    e if e < 0.75 => 2,
                    _ => 3,
                }] += 1;
            } else {
                fora += 1;
                pior_f = pior_f.max(d);
            }
        }
    }
    eprintln!(
        "[p6] {nome:<34}: silhueta {borda} texels (pior {pior_b}) · fora {fora} (pior {pior_f}) · \
         {pintados} pintados · aresta a <0,01 px {} · <0,1 {} · <0,75 {} · ≥0,75 {}",
        hist[0], hist[1], hist[2], hist[3]
    );
    borda
}

/// Sonda (não afirma nada): a cena de sprites, crua e com dois filtros (a Drop Shadow do gate da
/// sprite sozinha e um Outer Glow), pelas duas rotas e nos dois escopos.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-fx-cells --test it medida_p6 -- --ignored --nocapture`
#[test]
#[ignore = "medição de GPU"]
fn medida_p6_a_silhueta_das_sprites() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    eprintln!("[p6] load {}", carga());
    // (rótulo, a faixa da placa, o tag de amostragem das sprites da CPU): a partição VAZIA (o sampler
    // do projecto: trilinear, anisotropia 16); a faixa explícita do tag `0` (até à cura do P6 ligava
    // OUTRO sampler na CPU — linear, um nível, isotrópico —: 61 texels entre as rotas); e as duas
    // rotas no tag `6` (`LinearAniso`, o descritor do sampler do projecto).
    for (rotulo, faixa, tag) in [
        ("vazia (produto)", None, 0u32),
        ("faixa tag 0", Some(0u32), 0),
        ("as duas no tag 6", Some(6), 6),
    ] {
        b.faixa = faixa;
        let ss: Vec<RenderInstance> = cena(&b)
            .into_iter()
            .map(|s| RenderInstance { sampling: tag, ..s })
            .collect();
        eprintln!("[p6] ══ {rotulo}");
        rodada(&mut b, &ss);
    }
    b.faixa = None;
    eprintln!("[p6] load {}", carga());
}

fn rodada(b: &mut Banca, ss: &[RenderInstance]) {
    let quads: Vec<_> = ss.iter().map(cantos).collect();
    let crua_cpu = b.inteira_por(&[], Rota::Cpu(ss), &[]);
    let crua_placa = b.inteira_por(&[], Rota::Placa(ss), &[]);
    compara(
        "crua · placa × cpu",
        &crua_placa,
        &crua_cpu,
        &crua_cpu,
        &quads,
    );
    let mut sombra = sombra_interna(false);
    sombra.kind = FxOp::DROP_SHADOW;
    sombra.offset_px = [2.5, 3.25];
    let glow = FxOpGpu {
        kind: FxOp::GLOW,
        sigma_px: 5.3,
        offset_px: [0.0; 2],
        ..sombra.clone()
    };
    for (filtro, ops) in [("Drop Shadow", [sombra]), ("Outer Glow", [glow])] {
        let inteira_cpu = b.inteira_por(&[], Rota::Cpu(ss), &ops);
        let inteira_placa = b.inteira_por(&[], Rota::Placa(ss), &ops);
        let cada_cpu = b.cada_um(&[], Rota::Cpu(ss), &ops);
        let cada_placa = b.cada_um(&[], Rota::Placa(ss), &ops);
        let c = |n: &str| format!("{filtro} · {n}");
        compara(
            &c("inteira · placa × cpu"),
            &inteira_placa,
            &inteira_cpu,
            &crua_cpu,
            &quads,
        );
        compara(
            &c("cada · placa × cpu"),
            &cada_placa,
            &cada_cpu,
            &crua_cpu,
            &quads,
        );
        compara(
            &c("cpu · cada × inteira"),
            &cada_cpu,
            &inteira_cpu,
            &crua_cpu,
            &quads,
        );
        compara(
            &c("placa · cada × inteira"),
            &cada_placa,
            &inteira_placa,
            &crua_cpu,
            &quads,
        );
    }
}
