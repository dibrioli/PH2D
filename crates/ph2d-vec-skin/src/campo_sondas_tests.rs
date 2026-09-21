//! ⭐⭐⭐ **AS SONDAS DA LENTE A** — a auditoria que o dono ordenou em 2026-09-20, ao report
//! *«os pesos não estão na malha mas sim nos pontos do vetor. não houve nenhuma melhora.»*
//!
//! ⛔ **Estas funções são SONDAS: elas IMPRIMEM.** Os gates que AFIRMAM vivem no irmão
//! [`super::campo_tests`], e esse é o corte (tecto de LOC, `1 621` contra `700`): *uma medição
//! exploratória e uma lei são responsabilidades diferentes, e juntá-las faz o ficheiro crescer a
//! cada report sem que nenhuma lei nova entre.*
//!
//! ⚠️ **Nenhuma delas mede RELÓGIO** — a workstation esteve a `load 26` durante a auditoria, e a
//! lei da casa diz que nenhuma leitura de tempo vale acima de `load ~5`. Todas as grandezas aqui
//! são GEOMÉTRICAS, logo imunes à carga.

use super::campo_tests::*;
use super::pesos::*;
use ph2d_skeleton::Skin;
use ph2d_skin_weights::Handle;
use ph2d_vec_scene::VecPath;

/// `p50`, `p90` e o MÁXIMO — ⛔ nunca só uma média: *uma régua que mede um extremo global não vê
/// um defeito local, e uma que mede a média não vê nenhum dos dois.*
pub(super) fn percentis(v: &mut [f64]) -> (f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    v.sort_by(f64::total_cmp);
    let q = |fr: f64| v[((f(v.len() - 1)) * fr).round() as usize];
    (q(0.5), q(0.9), *v.last().expect("não vazia"))
}

/// A diagonal da caixa do caminho — o denominador que torna um desvio LEGÍVEL.
pub(super) fn diagonal(path: &VecPath) -> f64 {
    let c = path.cooked();
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for k in 0..c.contour_count() {
        let Some((vs, _)) = c.contour(k) else {
            continue;
        };
        for v in vs {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                x0 = x0.min(p[0]);
                y0 = y0.min(p[1]);
                x1 = x1.max(p[0]);
                y1 = y1.max(p[1]);
            }
        }
    }
    (x1 - x0).hypot(y1 - y0)
}

// ─────────────────────────── A BARRA REAL DA CENA DO DONO ───────────────────────────
//
// ⚠️ Ela é o `cook(RoundRect, [-8.5, 2.0], [-1.5, 3.0], &[0.5])` do
// `ph2d_skeleton_live::barra_da_cena_tests_support`, com a cadeia de TRÊS ossos daquele ficheiro.
// ⛔ Reconstruída aqui porque aquela crate DEPENDE desta — importá-la seria um ciclo.

// ─────────────────────── A SUBDIVISÃO DO BIND, replicada ───────────────────────

/// ⭐⭐⭐ **SONDA 0 — A FIXTURA ESTÁ VIVA?**
///
/// Ela responde a uma pergunta que precede todas as outras: *o peso guardado chega a mover alguma
/// coisa?* Se todos os ossos partilharem o tendão `0`, a tabela é lida e **normalizada para
/// uniforme**, e toda medição feita sobre ela mede outro programa.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_saude_da_fixtura -- --nocapture
/// ```
#[test]
pub(super) fn diag_a_saude_da_fixtura() {
    eprintln!("\n=== SONDA 0 — a fixtura está viva? ===");
    for (nome, pele) in [
        ("pele_degenerada (tendões [0,0])", pele_degenerada(0.8)),
        ("pele_com_tendoes (à mão)", pele_com_tendoes(0.8)),
    ] {
        let t: Vec<u32> = pele.bones().iter().map(|b| b.tendon).collect();
        // Dois pontos bem separados, com linhas de peso OPOSTAS.
        let mut w = pele.scratch();
        pele.weights_corrected([1.0, H * 0.5], Some(&[1.0, 0.0]), &mut w, &[]);
        let a = w.clone();
        pele.weights_corrected([1.0, H * 0.5], Some(&[0.0, 1.0]), &mut w, &[]);
        eprintln!(
            "  {nome:34} tendões={t:?}  w(linha=[1,0])={a:?}  w(linha=[0,1])={w:?}  \
             {}",
            if a == w {
                "⛔ A TABELA NÃO MOVE NADA"
            } else {
                "ok"
            }
        );
    }
    let pd = pele_do_dono(0.6);
    let t: Vec<u32> = pd.bones().iter().map(|b| b.tendon).collect();
    eprintln!("  barra do dono (3 ossos)            tendões={t:?}");
}

/// ⭐⭐⭐ **SONDA 1 — A PROVENIÊNCIA DE CADA PONTO QUE SE MOVE.**
///
/// Para cada uma das três metades de um vértice, quanto de cada camada:
///
/// | camada | o que é |
/// |---|---|
/// | `|ingénuo − fonte|` | a lei dos NÓS: peso da tabela guardada, linha `3k`, **sem campo** |
/// | `|curva − ingénuo|` | o que o ajuste das alças acrescenta (sem campo) |
/// | `|campo − curva|`   | **o que o CAMPO DO DOMÍNIO acrescenta** |
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_a_proveniencia -- --nocapture
/// ```
#[test]
pub(super) fn diag_a_proveniencia_de_cada_ponto() {
    eprintln!("\n=== SONDA 1 — de onde vem o peso de cada ponto ===");
    for (nome, mut fonte, eixos, pele_f) in [
        (
            "barra do dono (como o bind entrega)",
            barra_do_dono(),
            eixos_do_dono(),
            pele_do_dono(0.6),
        ),
        (
            "rectângulo 40x10, 2 ossos",
            forma(),
            eixos(),
            pele_com_tendoes(0.8),
        ),
    ] {
        let cortes = subdivide_como_o_bind(&mut fonte, &eixos);
        let Some(campo) = campo_do_caminho(&fonte, &eixos) else {
            eprintln!("  {nome}: sem campo");
            continue;
        };
        let tabela = pesos_dos_pontos(&fonte, &campo);
        let nos = fonte.verts_all().count();
        let diag = diagonal(&fonte);

        let corre = |curva: bool, com_campo: bool| {
            let mut p = fonte.clone();
            if curva {
                crate::curva::aplica_pela_curva_com(
                    &pele_f,
                    &mut p,
                    &tabela,
                    &[],
                    true,
                    com_campo.then_some(&campo),
                );
            } else {
                crate::aplica_corrigido_com(&pele_f, &mut p, &tabela, &[], true);
            }
            p
        };
        let ingenuo = corre(false, false);
        let curva_sem = corre(true, false);
        let curva_com = corre(true, true);

        let lista = |a: &VecPath, b: &VecPath, j: usize| -> Vec<f64> {
            let (ca, cb) = (a.cooked(), b.cooked());
            let mut out = Vec::new();
            for k in 0..ca.contour_count() {
                let (Some((va, _)), Some((vb, _))) = (ca.contour(k), cb.contour(k)) else {
                    continue;
                };
                for (x, y) in va.iter().zip(vb) {
                    let (p, q) = match j {
                        0 => (x.anchor, y.anchor),
                        1 => (x.in_handle, y.in_handle),
                        _ => (x.out_handle, y.out_handle),
                    };
                    out.push((p[0] - q[0]).hypot(p[1] - q[1]));
                }
            }
            out
        };

        eprintln!(
            "\n  ── {nome} — {nos} nós ({cortes} cortes do bind), diagonal {diag:.4}, \
             {} ossos",
            campo.ossos()
        );
        eprintln!(
            "     {:<12} {:>32} {:>32} {:>32}",
            "", "|ingénuo−fonte| (lei do NÓ)", "|curva−ingénuo| (ajuste)", "|campo−curva| (CAMPO)"
        );
        for (j, classe) in ["âncora", "alça in", "alça out"].iter().enumerate() {
            let cols = [
                (&fonte, &ingenuo),
                (&ingenuo, &curva_sem),
                (&curva_sem, &curva_com),
            ]
            .map(|(a, b)| percentis(&mut lista(a, b, j)));
            eprintln!(
                "     {classe:<12} {:>10.6}/{:>9.6}/{:>9.6} {:>10.6}/{:>9.6}/{:>9.6} \
                 {:>10.6}/{:>9.6}/{:>9.6}",
                cols[0].0,
                cols[0].1,
                cols[0].2,
                cols[1].0,
                cols[1].1,
                cols[1].2,
                cols[2].0,
                cols[2].1,
                cols[2].2,
            );
        }
        eprintln!("     (cada célula é p50/p90/max)");

        // De onde veio a linha guardada de cada NÓ: do campo amostrado nele, ou herdada do
        // vértice de malha mais próximo (a âncora caiu FORA da malha)?
        let cozido = fonte.cooked();
        let (mut dentro, mut fora) = (0usize, 0usize);
        for k in 0..cozido.contour_count() {
            let Some((vs, _)) = cozido.contour(k) else {
                continue;
            };
            for v in vs {
                if campo.linha(v.anchor).is_some() {
                    dentro += 1;
                } else {
                    fora += 1;
                }
            }
        }
        eprintln!(
            "     âncoras DENTRO da malha: {dentro} · FORA (peso herdado do vizinho): {fora}"
        );
    }
}

// ─────────────────── O CONTRAFACTUAL: a lei da MALHA DENSA (a mídia IMAGEM) ───────────────────

/// Cada vértice da malha deformado pelo peso DELE — é literalmente o que a 2.ª mídia faz.
pub(super) fn malha_deformada(campo: &CampoDoDominio, pele: &Skin) -> Vec<[f64; 2]> {
    let mut w = pele.scratch();
    (0..campo.malha.rest.len())
        .map(|i| {
            let local = campo.local_do_vertice(i).expect("régua não-nula");
            let linha = campo.linha_do_vertice(i).expect("linha").to_vec();
            pele.weights_corrected(local, Some(&linha), &mut w, &[]);
            pele.blend(local, &w)
        })
        .collect()
}

/// O ponto `p` pela lei da MALHA DENSA — baricêntricas sobre as posições JÁ deformadas.
///
/// ⚠️ **Ela NÃO é a lei da curva.** A curva interpola os PESOS e mistura uma vez; esta mistura
/// cada vértice e interpola as POSIÇÕES. São duas leis, e a diferença entre elas é medida.
pub(super) fn ponto_da_malha_densa(
    campo: &CampoDoDominio,
    def: &[[f64; 2]],
    p_local: [f64; 2],
) -> Option<[f64; 2]> {
    let pm = [
        (p_local[0] - campo.regua[0]) * campo.regua[2],
        (p_local[1] - campo.regua[1]) * campo.regua[2],
    ];
    for t in &campo.malha.tris {
        let (i0, i1, i2) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (a, b, c) = (
            campo.malha.rest[i0],
            campo.malha.rest[i1],
            campo.malha.rest[i2],
        );
        let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let u = ((pm[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (pm[1] - a[1])) / den;
        let v = ((b[0] - a[0]) * (pm[1] - a[1]) - (pm[0] - a[0]) * (b[1] - a[1])) / den;
        if u < -1e-9 || v < -1e-9 || u + v > 1.0 + 1e-9 {
            continue;
        }
        let k = 1.0 - u - v;
        return Some([
            v.mul_add(def[i2][0], u.mul_add(def[i1][0], k * def[i0][0])),
            v.mul_add(def[i2][1], u.mul_add(def[i1][1], k * def[i0][1])),
        ]);
    }
    None
}

/// Uma cúbica de um contorno AUTORADO (o mesmo que o algoritmo percorre).
pub(super) fn cubica_de(vs: &[ph2d_vec_scene::VecVertex], k: usize, n: usize) -> [[f64; 2]; 4] {
    let (a, b) = (&vs[k], &vs[(k + 1) % n]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

pub(super) fn eval_cubica(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b3.mul_add(
            c[3][0],
            b2.mul_add(c[2][0], b1.mul_add(c[1][0], b0 * c[0][0])),
        ),
        b3.mul_add(
            c[3][1],
            b2.mul_add(c[2][1], b1.mul_add(c[1][1], b0 * c[0][1])),
        ),
    ]
}

pub(super) fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// A linha do nó `k` (índice PLANO) na tabela por ponto de controlo.
pub(super) fn linha_plana(tabela: &[f64], k: usize, ossos: usize) -> &[f64] {
    &tabela[k * 3 * ossos..k * 3 * ossos + ossos]
}

/// Uma fixtura completa, já subdividida como o bind faz.
pub(super) struct Caso {
    pub(super) nome: &'static str,
    pub(super) fonte: VecPath,
    pub(super) campo: CampoDoDominio,
    pub(super) tabela: Vec<f64>,
    pub(super) pele: Skin,
}

pub(super) fn casos() -> Vec<Caso> {
    let mut out = Vec::new();
    let mut monta = |nome, mut fonte: VecPath, eixos: Vec<Handle>, pele: Skin, sub: bool| {
        if sub {
            subdivide_como_o_bind(&mut fonte, &eixos);
        }
        if let Some(campo) = campo_do_caminho(&fonte, &eixos) {
            let tabela = pesos_dos_pontos(&fonte, &campo);
            out.push(Caso {
                nome,
                fonte,
                campo,
                tabela,
                pele,
            });
        }
    };
    monta(
        "barra do dono · 8 nós (como o artista desenha)",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(0.6),
        false,
    );
    monta(
        "barra do dono · 34 nós (como o BIND entrega)",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(0.6),
        true,
    );
    monta(
        "barra do dono · 34 nós · dobra FORTE 1,2",
        barra_do_dono(),
        eixos_do_dono(),
        pele_do_dono(1.2),
        true,
    );
    monta(
        "rectângulo 40x10 · 20 nós · 2 ossos",
        forma(),
        eixos(),
        pele_com_tendoes(0.8),
        true,
    );
    out
}

/// ⭐⭐⭐ **SONDA 2 — QUANTAS AMOSTRAS LEEM O CAMPO, e quantas caem na mistura.**
///
/// Replica exactamente o que a [`crate::curva::correccao_das_alcas`] pergunta: `AMOSTRAS` pontos
/// por segmento em `t = (i + ½)/N`.
///
/// ⚠️ **Eram `AMOSTRAS + 2` até 2026-09-20** — as duas consultas extra eram o EIXO de cada alça,
/// que a conciliação lia em `t = 0` e `t = 1`. Com aquele passe apagado elas saíram, e o recook da
/// barra da cena passou de `427,3` para **`335,7 µs`** (`−21 %`, `--release`, `load 3,8`).
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_quantas_amostras -- --nocapture
/// ```
#[test]
pub(super) fn diag_quantas_amostras_leem_o_campo() {
    eprintln!("\n=== SONDA 2 — o campo é consultado quantas vezes, e responde? ===");
    eprintln!("  (AMOSTRAS = {})", crate::curva::AMOSTRAS);
    for caso in casos() {
        let (mut sim_aj, mut nao_aj) = (0usize, 0usize);
        let (mut sim_dir, mut nao_dir) = (0usize, 0usize);
        for c in 0..caso.fonte.contour_count() {
            let Some((vs, fechado)) = caso.fonte.contour(c) else {
                continue;
            };
            let n = vs.len();
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let cub = cubica_de(vs, k, n);
                for i in 0..crate::curva::AMOSTRAS {
                    let t = (f(i) + 0.5) / f(crate::curva::AMOSTRAS);
                    if caso.campo.linha(eval_cubica(&cub, t)).is_some() {
                        sim_aj += 1;
                    } else {
                        nao_aj += 1;
                    }
                }
                for t in [0.0, 1.0] {
                    if caso.campo.linha(eval_cubica(&cub, t)).is_some() {
                        sim_dir += 1;
                    } else {
                        nao_dir += 1;
                    }
                }
            }
        }
        let tot = sim_aj + nao_aj;
        eprintln!(
            "  {:<48} ajuste: {sim_aj}/{tot} no campo ({:.1} %) · direcção: {sim_dir}/{} ({:.1} %)",
            caso.nome,
            100.0 * f(sim_aj) / f(tot.max(1)),
            sim_dir + nao_dir,
            100.0 * f(sim_dir) / f((sim_dir + nao_dir).max(1)),
        );
    }
}

/// ⭐⭐⭐ **SONDA 3 — O TECTO: quanto do erro é do PESO e quanto é do AJUSTE da cúbica.**
///
/// Cinco curvas, todas amostradas densamente no MESMO `t`:
///
/// | curva | o que é |
/// |---|---|
/// | `malha` | a lei da MALHA DENSA — baricêntricas sobre vértices deformados (a mídia IMAGEM) |
/// | `lei+campo` | `blend(C(t), campo.linha(C(t)))` — o que a lei da curva PERSEGUE |
/// | `lei+mist` | `blend(C(t), lerp(ra, rb, t))` — a mesma sem campo |
/// | `entregue` | a CÚBICA que o produto escreve (com ou sem campo) |
/// | `ingénuo` | a cúbica da lei dos nós sozinha |
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-vec-skin --lib diag_o_tecto_do_ajuste -- --nocapture
/// ```
#[test]
pub(super) fn diag_o_tecto_do_ajuste_da_cubica() {
    eprintln!("\n=== SONDA 3 — o tecto: o PESO ou o AJUSTE? (p50/p90/max, e % da diagonal) ===");
    const DENSO: usize = 200;
    for caso in casos() {
        let diag = diagonal(&caso.fonte);
        let ossos = caso.campo.ossos();
        let def = malha_deformada(&caso.campo, &caso.pele);

        let corre = |curva: bool, com: bool| {
            let mut p = caso.fonte.clone();
            if curva {
                crate::curva::aplica_pela_curva_com(
                    &caso.pele,
                    &mut p,
                    &caso.tabela,
                    &[],
                    true,
                    com.then_some(&caso.campo),
                );
            } else {
                crate::aplica_corrigido_com(&caso.pele, &mut p, &caso.tabela, &[], true);
            }
            p
        };
        let (ingenuo, sem, com) = (corre(false, false), corre(true, false), corre(true, true));

        // Uma coluna por grandeza medida.
        let mut e_lei = Vec::new(); // |lei+campo − lei+mist|   : o que o campo VALE na lei
        let mut e_malha_lei = Vec::new(); // |malha − lei+campo| : as duas leis do padrão-ouro
        let mut e_fit = Vec::new(); // |entregue(campo) − lei+campo| : o resíduo do AJUSTE
        let mut e_com = Vec::new(); // |entregue(campo) − malha|     : o produto contra o ouro
        let mut e_sem = Vec::new(); // |entregue(sem)   − malha|
        let mut e_ing = Vec::new(); // |ingénuo         − malha|

        let mut w = caso.pele.scratch();
        let mut base = 0usize;
        for c in 0..caso.fonte.contour_count() {
            let (Some((vf, fechado)), Some((vi, _)), Some((vs, _)), Some((vc, _))) = (
                caso.fonte.contour(c),
                ingenuo.contour(c),
                sem.contour(c),
                com.contour(c),
            ) else {
                continue;
            };
            let n = vf.len();
            let segs = if fechado { n } else { n.saturating_sub(1) };
            for k in 0..segs {
                let src = cubica_de(vf, k, n);
                let (ci, cs, cc) = (
                    cubica_de(vi, k, n),
                    cubica_de(vs, k, n),
                    cubica_de(vc, k, n),
                );
                let ra = linha_plana(&caso.tabela, base + k, ossos).to_vec();
                let rb = linha_plana(&caso.tabela, base + (k + 1) % n, ossos).to_vec();
                for i in 0..=DENSO {
                    let t = f(i) / f(DENSO);
                    let p = eval_cubica(&src, t);
                    let mistura: Vec<f64> = (0..ossos)
                        .map(|j| (rb[j] - ra[j]).mul_add(t, ra[j]))
                        .collect();
                    let do_campo = caso.campo.linha(p).unwrap_or_else(|| mistura.clone());
                    caso.pele.weights_corrected(p, Some(&do_campo), &mut w, &[]);
                    let lei_campo = caso.pele.blend(p, &w);
                    caso.pele.weights_corrected(p, Some(&mistura), &mut w, &[]);
                    let lei_mist = caso.pele.blend(p, &w);
                    let Some(ouro) = ponto_da_malha_densa(&caso.campo, &def, p) else {
                        continue; // fora da malha: não há padrão-ouro para comparar
                    };
                    e_lei.push(dist(lei_campo, lei_mist));
                    e_malha_lei.push(dist(ouro, lei_campo));
                    e_fit.push(dist(eval_cubica(&cc, t), lei_campo));
                    e_com.push(dist(eval_cubica(&cc, t), ouro));
                    e_sem.push(dist(eval_cubica(&cs, t), ouro));
                    e_ing.push(dist(eval_cubica(&ci, t), ouro));
                }
            }
            base += n;
        }

        eprintln!("\n  ── {} — diagonal {diag:.4}", caso.nome);
        let linha = |rot: &str, v: &mut Vec<f64>| -> (f64, f64, f64) {
            let (a, b, c) = percentis(v);
            eprintln!(
                "     {rot:<40} {a:>10.6} {b:>10.6} {c:>10.6}   (max = {:.3} % da peça)",
                100.0 * c / diag
            );
            (a, b, c)
        };
        eprintln!("     {:<40} {:>10} {:>10} {:>10}", "", "p50", "p90", "max");
        let lei = linha("o que o CAMPO vale na LEI", &mut e_lei);
        linha("|malha densa − lei da curva|", &mut e_malha_lei);
        let fit = linha("resíduo do AJUSTE (entregue−lei)", &mut e_fit);
        let com = linha("PRODUTO com campo vs OURO", &mut e_com);
        let sem = linha("PRODUTO sem campo vs OURO", &mut e_sem);
        let ing = linha("lei dos NÓS sozinha vs OURO", &mut e_ing);
        eprintln!(
            "     ⇒ o campo compra {:.6} do erro (max {:.6} → {:.6}, {:.1} %); \
             o AJUSTE deixa {:.6} ({:.1} % do erro que fica)",
            sem.2 - com.2,
            sem.2,
            com.2,
            100.0 * (sem.2 - com.2) / sem.2.max(1e-30),
            fit.2,
            100.0 * fit.2 / com.2.max(1e-30),
        );
        eprintln!(
            "     ⇒ o ajuste das alças compra {:.6} sobre a lei dos nós (max {:.6} → {:.6}); \
             a LEI do campo vale {:.6}",
            ing.2 - sem.2,
            ing.2,
            sem.2,
            lei.2,
        );
    }
}
