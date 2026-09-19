//! As sondas da SUPERFÍCIE — filhas das [`super`], e a fronteira é o que cada
//! régua julga.
//!
//! ⛔⛔⛔ **Elas nasceram de um report com FOTO** (2026-09-19: *«o resultado fica
//! pior que o original, com irregularidade a 90 graus da direcção do
//! movimento»*). As réguas do irmão medem todas a **LIGAÇÃO** — que direcção as
//! arestas tomam, que forma os triângulos têm, quantas mudaram de rumo — e
//! **nenhuma** mede o RELEVO. *Uma malha pode ficar mais alinhada e mais feia ao
//! mesmo tempo, e até este report esta linha não tinha como o dizer.*
//!
//! ⛔ Saíram para cá por TECTO DE LOC (`981` contra `700`), e o corte é por
//! RESPONSABILIDADE.

use super::*;

/// ⛔⛔⛔ **SONDA — QUAL das três metades enruga a superfície**, e quanto.
///
/// O report de 19/09 (*«o resultado fica pior que o original, com irregularidade
/// a 90 graus da direcção do movimento»*) é sobre o **RELEVO**, e o pente tem
/// três metades que agem em sítios diferentes do dab. *Uma cura escolhida sem
/// atribuição é um palpite.*
///
/// Colunas: a [`rugosidade`] (`|p − centroide|`, que é função da LIGAÇÃO
/// também), o **vinco** da porta (o que a luz lê), a **razão** entre o
/// comprimento das arestas ao longo e atravessadas, e a grade.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_quem_enruga -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_quem_enruga() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let e = RUMOS[0].1;
    eprintln!(
        "{:<24} {:>8} {:>9} {:>8} {:>7}",
        "metades", "rug p50", "vinco p90", "razao", "grade"
    );
    for (nome, campo, flip, desloca) in [
        ("nenhuma (o controlo)", false, false, false),
        ("so' o CAMPO", true, false, false),
        ("so' o FLIP", false, true, false),
        ("so' o DESLOCAMENTO", false, false, true),
        ("campo + flip", true, true, false),
        ("campo + deslocamento", true, false, true),
        ("flip + deslocamento", false, true, true),
        ("TUDO (o que shipa)", true, true, true),
    ] {
        let (m, c, _) = traco_por_metades(e, raio, alvo, campo, flip, desloca);
        eprintln!("{nome:<24}{}", colunas_da_superficie(&m, &c, raio));
    }
    // ⭐ O CONTROLO da curvatura: tudo ligado e o carimbo a ZERO — a esfera
    // continua curva, e o traço não levanta relevo nenhum.
    let (m, c, _) = traco_por_metades_com(e, raio, alvo, true, true, true, 0.0);
    eprintln!("{:<24}{}", "TUDO, carimbo a ZERO", colunas_da_superficie(&m, &c, raio));
}

/// ⭐⭐⭐ **SONDA — A ESCADA DO BOTÃO contra a ondulação.**
///
/// É ela que diz onde o pente **melhora** a superfície e onde começa a trocá-la
/// por alinhamento — a pergunta que o report de 19/09 faz e que nenhuma régua
/// desta linha sabia responder.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_a_escada_do_botao -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_a_escada_do_botao() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    eprintln!(
        "{:<8} {:<16} {:>8} {:>9} {:>8} {:>7}",
        "pente", "rumo", "rug p50", "vinco p90", "razao", "grade"
    );
    for (nome, e) in RUMOS {
        for pente in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let (m, c, _) = traco_contado(pente, e, raio, alvo);
            eprintln!("{pente:<8.2} {nome:<16}{}", colunas_da_superficie(&m, &c, raio));
        }
    }
}

/// ⛔⛔⛔ **SONDA — o esticão é da LEI ou da CURVATURA?**
///
/// O mesmo traço sobre uma CHAPA, onde não há curvatura nenhuma: *se ali o
/// esticão desaparece, ele é da curvatura; se fica, é da lei*.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_a_chapa_estica -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_a_chapa_estica() {
    let base = chapa_sacudida(61, 3.0);
    const ALVO: f32 = 0.035;
    let raio = 0.35f32;
    eprintln!(
        "{:<24} {:>8} {:>9} {:>8} {:>7}",
        "metades (na CHAPA)", "rug p50", "vinco p90", "razao", "grade"
    );
    for (nome, campo, flip, desloca) in [
        ("nenhuma (o controlo)", false, false, false),
        ("so' o CAMPO", true, false, false),
        ("so' o FLIP", false, true, false),
        ("TUDO (o que shipa)", true, true, true),
    ] {
        let (m, c) = traco_na_chapa(&base, raio, ALVO, campo, flip, desloca);
        eprintln!("{nome:<24}{}", colunas_da_superficie(&m, &c, raio));
    }
}

/// ⛔⛔⛔ **SONDA — EM QUE PASSO do dab nasce o vinco.**
///
/// *O passo em que o número salta é o culpado* — e ele não foi o que parecia: o
/// salto está no **COLAPSO**, não na troca de diagonal que a wave do pente tinha
/// acabado de soltar.
///
/// ```text
/// cargo test -p ph2d-app-sculpt3d --release --lib diag_em_que_passo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda"]
fn diag_em_que_passo() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let e = RUMOS[0].1;
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente: 0.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    let passo = raio * 0.15;
    let mut pior = 0.0f64;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let direccao = stroke.direccao_do_traco(centro);
        let alvo_col = ph2d_mesh::collapse_target(alvo);
        let cc = ph2d_sculpt3d::campo_do_pente(
            alvo_col,
            direccao,
            0.0,
            ph2d_sculpt3d::Porta::Colapso,
        );
        if matches!(
            ph2d_mesh::collapse_in_sphere_com(
                &mut malha,
                centro,
                raio,
                alvo_col,
                Some(&cc),
                ph2d_mesh::Guarda::ETambemAForma,
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        let apos_colapso = vinco_da_faixa(&malha, &centros, raio).2;
        let cr =
            ph2d_sculpt3d::campo_do_pente(alvo, direccao, 0.0, ph2d_sculpt3d::Porta::Refino);
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha,
            centro,
            raio,
            alvo,
            Some(&cr),
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        let apos_refino = vinco_da_faixa(&malha, &centros, raio).2;
        let pref = ph2d_sculpt3d::preferencia_do_pente(direccao, 1.0);
        let trocas = ph2d_mesh::alinha_arestas(&mut malha, centro, raio, &pref, &mut region);
        let apos_flip = vinco_da_faixa(&malha, &centros, raio).2;
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        let apos_dab = vinco_da_faixa(&malha, &centros, raio).2;
        let maior = apos_colapso.max(apos_refino).max(apos_flip).max(apos_dab);
        if maior > pior + 5.0 || k == 23 {
            eprintln!(
                "dab {k:>2}  colapso {apos_colapso:>8.3}  refino {apos_refino:>8.3}  \
                 flip {apos_flip:>8.3} ({trocas:>4})  dab {apos_dab:>8.3}"
            );
            pior = pior.max(maior);
        }
    }
}

/// As quatro colunas da superfície, numa linha formatada.
fn colunas_da_superficie(m: &ph2d_mesh::Mesh, c: &[[f32; 3]], raio: f32) -> String {
    let (rp50, _, _) = rugosidade(m, c, raio);
    let (_, vp90, _, _) = vinco_da_faixa(m, c, raio);
    let l = comprimento_por_direccao(m, c, raio);
    let (bins, nb) = grade_da_faixa(m, c, raio);
    format!(
        " {rp50:>8.4} {vp90:>9.3} {:>8.3} {:>6.1}%",
        l[0] / l[2].max(1e-9),
        100.0 * bins[0] as f64 / nb.max(1) as f64
    )
}

/// `|p − centroide(anel)| / aresta média do anel`, sobre os vértices da faixa.
///
/// ⚠️ **Adimensional de propósito:** o passe muda a densidade da malha, e uma
/// medida em unidades de mundo leria a malha mais fina como mais lisa.
///
/// ⚠️⚠️ **Ela é função da LIGAÇÃO, não só da forma** — uma troca de diagonal
/// muda o anel sem mover um vértice. Quem julga o que a LUZ vê é a
/// [`vinco_da_faixa`] da porta; esta fica como a segunda testemunha.
fn rugosidade(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32) -> (f64, f64, usize) {
    let pos = m.positions();
    let mut anel: Vec<Vec<u32>> = vec![Vec::new(); pos.len()];
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k] as usize, vs[(k + 1) % vs.len()] as usize);
            if !anel[a].contains(&(b as u32)) {
                anel[a].push(b as u32);
            }
            if !anel[b].contains(&(a as u32)) {
                anel[b].push(a as u32);
            }
        }
    }
    let mut vals: Vec<f64> = Vec::new();
    for (i, p) in pos.iter().enumerate() {
        if anel[i].len() < 3 {
            continue;
        }
        let mut perto = f32::MAX;
        for c in percurso {
            perto = perto.min((p[0] - c[0]).hypot(p[1] - c[1]).hypot(p[2] - c[2]));
        }
        if perto > raio * 0.5 {
            continue;
        }
        let (mut cx, mut cy, mut cz, mut l) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for v in &anel[i] {
            let q = pos[*v as usize];
            cx += f64::from(q[0]);
            cy += f64::from(q[1]);
            cz += f64::from(q[2]);
            l += f64::from((q[0] - p[0]).hypot(q[1] - p[1]).hypot(q[2] - p[2]));
        }
        let k = anel[i].len() as f64;
        let (cx, cy, cz, l) = (cx / k, cy / k, cz / k, l / k);
        if l <= 0.0 {
            continue;
        }
        let d = ((f64::from(p[0]) - cx).powi(2)
            + (f64::from(p[1]) - cy).powi(2)
            + (f64::from(p[2]) - cz).powi(2))
        .sqrt();
        vals.push(d / l);
    }
    vals.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    let n = vals.len();
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    (vals[n / 2], vals[(n * 9 / 10).min(n - 1)], n)
}

/// Comprimento MÉDIO das arestas da faixa em três faixas de ângulo ao traço:
/// `0-15°` (ao longo), `37,5-52,5°` (diagonal) e `75-90°` (atravessado).
///
/// ⭐ **A razão `ao longo / atravessado` é a régua do ESTICÃO:** uma lei de
/// QUATRO dobras tem de deixar as duas famílias da grade **iguais** e as
/// diagonais mais longas — que é o que a saída do alvo faz (`1,04`–`1,13`, com a
/// diagonal no topo).
fn comprimento_por_direccao(m: &ph2d_mesh::Mesh, percurso: &[[f32; 3]], raio: f32) -> [f64; 3] {
    let pos = m.positions();
    let mut vistas = std::collections::BTreeSet::new();
    let mut soma = [0.0f64; 3];
    let mut n = [0usize; 3];
    for f in m.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            if !vistas.insert((a.min(b), a.max(b))) {
                continue;
            }
            let (pa, pb) = (pos[a as usize], pos[b as usize]);
            let meio = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let (mut perto, mut dir) = (f32::MAX, [1.0f32, 0.0, 0.0]);
            for par in percurso.windows(2) {
                let d = (meio[0] - par[1][0])
                    .hypot(meio[1] - par[1][1])
                    .hypot(meio[2] - par[1][2]);
                if d < perto {
                    perto = d;
                    dir = [
                        par[1][0] - par[0][0],
                        par[1][1] - par[0][1],
                        par[1][2] - par[0][2],
                    ];
                }
            }
            if perto > raio * 0.5 {
                continue;
            }
            let ar = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let la = f64::from(ar[0].hypot(ar[1]).hypot(ar[2]));
            let ld = f64::from(dir[0].hypot(dir[1]).hypot(dir[2]));
            if la <= 0.0 || ld <= 0.0 {
                continue;
            }
            let c = (f64::from(ar[0] * dir[0] + ar[1] * dir[1] + ar[2] * dir[2]) / (la * ld))
                .clamp(-1.0, 1.0);
            let ang = c.acos().to_degrees();
            let ang = ang.min(180.0 - ang);
            let faixa = if ang < 15.0 {
                Some(0)
            } else if (37.5..52.5).contains(&ang) {
                Some(1)
            } else if ang > 75.0 {
                Some(2)
            } else {
                None
            };
            if let Some(i) = faixa {
                soma[i] += la;
                n[i] += 1;
            }
        }
    }
    [
        soma[0] / n[0].max(1) as f64,
        soma[1] / n[1].max(1) as f64,
        soma[2] / n[2].max(1) as f64,
    ]
}

/// Uma chapa sacudida no interior, com a borda quieta — a mesma da bancada da
/// lei, para a sonda da curvatura ter uma peça PLANA de referência.
fn chapa_sacudida(n: usize, lado: f32) -> ph2d_mesh::Mesh {
    let passo = lado / (n - 1) as f32;
    let meio = lado * 0.5;
    let sacode = |i: usize, j: usize, sal: u32| -> f32 {
        let mut h = (i as u32).wrapping_mul(0x9E37_79B9)
            ^ (j as u32).wrapping_mul(0x85EB_CA6B)
            ^ sal.wrapping_mul(0xC2B2_AE35);
        h ^= h >> 15;
        h = h.wrapping_mul(0x2545_F491);
        h ^= h >> 13;
        f32::from(u16::try_from(h & 0xFFFF).expect("16 bits")) / 65535.0 - 0.5
    };
    let mut pos = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            let dentro = i > 0 && j > 0 && i < n - 1 && j < n - 1;
            let (dx, dy) = if dentro {
                (sacode(i, j, 1) * passo * 0.7, sacode(i, j, 2) * passo * 0.7)
            } else {
                (0.0, 0.0)
            };
            pos.push([
                i as f32 * passo - meio + dx,
                j as f32 * passo - meio + dy,
                0.0,
            ]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let (a, b, c, d) = (
                u32::try_from(j * n + i).expect("indice"),
                u32::try_from(j * n + i + 1).expect("indice"),
                u32::try_from((j + 1) * n + i + 1).expect("indice"),
                u32::try_from((j + 1) * n + i).expect("indice"),
            );
            faces.push(ph2d_mesh::Face::tri(a, b, c));
            faces.push(ph2d_mesh::Face::tri(a, c, d));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("a chapa e' uma malha valida")
}

/// O traço da sonda da chapa — recto, sobre a peça plana.
fn traco_na_chapa(
    base: &ph2d_mesh::Mesh,
    raio: f32,
    alvo: f32,
    campo: bool,
    flip: bool,
    desloca: bool,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut malha = base.clone();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente: if desloca { 1.0 } else { 0.0 },
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    for k in 0..24 {
        let centro = [-1.2 + 0.1 * k as f32, 0.0, 0.0];
        centros.push(centro);
        let direccao = stroke.direccao_do_traco(centro);
        let forca = if campo { 1.0 } else { 0.0 };
        let alvo_col = ph2d_mesh::collapse_target(alvo);
        let cc = ph2d_sculpt3d::campo_do_pente(
            alvo_col,
            direccao,
            forca,
            ph2d_sculpt3d::Porta::Colapso,
        );
        if matches!(
            ph2d_mesh::collapse_in_sphere_com(
                &mut malha,
                centro,
                raio,
                alvo_col,
                Some(&cc),
                ph2d_mesh::Guarda::ETambemAForma,
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        let cr =
            ph2d_sculpt3d::campo_do_pente(alvo, direccao, forca, ph2d_sculpt3d::Porta::Refino);
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha,
            centro,
            raio,
            alvo,
            Some(&cr),
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        if flip {
            let pref = ph2d_sculpt3d::preferencia_do_pente(direccao, 1.0);
            let _ = ph2d_mesh::alinha_arestas(&mut malha, centro, raio, &pref, &mut region);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros)
}

/// O traço da cena com cada metade do pente por PARÂMETRO — ver [`diag_quem_enruga`].
fn traco_por_metades(
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    campo: bool,
    flip: bool,
    desloca: bool,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>, usize) {
    traco_por_metades_com(e, raio, alvo, campo, flip, desloca, 0.25)
}

/// O mesmo, com a FORÇA do carimbo por parâmetro — `0` risca sem levantar
/// relevo, que é o que separa a curvatura da PEÇA da que o traço cria.
fn traco_por_metades_com(
    e: [f32; 2],
    raio: f32,
    alvo: f32,
    campo: bool,
    flip: bool,
    desloca: bool,
    forca: f32,
) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>, usize) {
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = Brush {
        verb: Verb::Draw,
        radius: raio,
        strength: forca,
        // ⚠️ É o `Brush::pente` que o `stroke.dab` lê para DESLOCAR — as outras
        // duas metades entram por argumento dos motores.
        pente: if desloca { 1.0 } else { 0.0 },
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&malha);
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    let mut centros = Vec::new();
    let mut trocas = 0usize;
    let passo = raio * 0.15;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * e[0], u.sin() * e[1], u.cos()];
        centros.push(centro);
        let direccao = stroke.direccao_do_traco(centro);
        let forca_do_campo = if campo { 1.0 } else { 0.0 };
        let alvo_col = ph2d_mesh::collapse_target(alvo);
        let cc = ph2d_sculpt3d::campo_do_pente(
            alvo_col,
            direccao,
            forca_do_campo,
            ph2d_sculpt3d::Porta::Colapso,
        );
        if matches!(
            ph2d_mesh::collapse_in_sphere_com(
                &mut malha,
                centro,
                brush.radius,
                alvo_col,
                Some(&cc),
                ph2d_mesh::Guarda::ETambemAForma,
                &mut remap,
                &mut region,
            ),
            ph2d_mesh::Collapse::Done { .. }
        ) {
            stroke.shrink_with(&remap);
        }
        let cr = ph2d_sculpt3d::campo_do_pente(
            alvo,
            direccao,
            forca_do_campo,
            ph2d_sculpt3d::Porta::Refino,
        );
        let _ = ph2d_mesh::refine_in_sphere_sized(
            &mut malha,
            centro,
            brush.radius,
            alvo,
            Some(&cr),
            &mut births,
            &mut region,
        );
        stroke.grow_with(&malha, &births);
        if flip {
            let pref = ph2d_sculpt3d::preferencia_do_pente(direccao, 1.0);
            trocas +=
                ph2d_mesh::alinha_arestas(&mut malha, centro, brush.radius, &pref, &mut region);
        }
        stroke.dab(
            &mut malha,
            &brush,
            &Dab::at(centro, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    (malha, centros, trocas)
}

/// ⭐⭐⭐ **GATE — O COLAPSO NÃO VIRA UMA FACE DO AVESSO.**
///
/// ⛔⛔⛔ **Ele nasceu do report de 19/09** (a foto do relevo), e a atribuição
/// levou-o a um sítio que ninguém suspeitava: medindo o pior vinco **depois de
/// cada um dos quatro passos do dab**, o número salta no **COLAPSO** (`5,49° →
/// 177,8°` no dab 4) e não na troca de diagonal que a wave do pente tinha
/// acabado de soltar. *A troca prepara a configuração; quem a dobra é a fusão
/// seguinte.*
///
/// ⚠️ **O cabeçalho do [`ph2d_mesh::collapse_in_sphere`] dizia, desde que
/// existe, «as quatro recusas, e todas são TOPOLOGIA»** — e isso lia-se como um
/// facto arrumado quando era uma **ausência**. A quinta recusa é geométrica.
///
/// ⚠️ **A fixtura é a configuração que REPRODUZ** — o traço com a troca de
/// diagonal ligada e o resto desligado, que é onde a dobra aparecia a `180,000°`
/// exactos. Com a recusa, a mesma corrida lê `~16°`.
///
/// ⛔ **A barra é `90°` e não um número afinado:** acima dela as duas faces
/// apontam para lados opostos, ou seja a superfície **dobrou sobre si mesma**.
/// *Uma dobra é um FACTO, não um grau de qualidade.*
#[test]
fn o_colapso_nao_vira_uma_face_do_avesso() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let (m, c, trocas) = traco_por_metades(RUMOS[0].1, raio, alvo, false, true, false);
    // O CONTROLO: sem trocas não há a configuração que dobrava, e a asserção de
    // baixo passaria sobre um passe inerte.
    assert!(
        trocas > 500,
        "so' {trocas} trocas de diagonal — a fixtura deixou de conter o \
         fenomeno que fazia o colapso dobrar"
    );
    let (_, _, max, n) = vinco_da_faixa(&m, &c, raio);
    assert!(
        n > 200,
        "a faixa tem {n} aresta(s) com duas faces — a regua esta' a medir o nada"
    );
    assert!(
        max < 90.0,
        "duas faces vizinhas ficaram a {max:.3}° uma da outra: a superficie \
         DOBROU sobre si mesma (medido 180,000° sem a quinta recusa do colapso, \
         ~16° com ela)"
    );
}
