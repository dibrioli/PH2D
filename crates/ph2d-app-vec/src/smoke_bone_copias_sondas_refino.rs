//! A13 — o REFINO LOCAL do bake na barra `0` da `=6` (a sem riscas): a régua (a imagem EXACTA do
//! contorno em repouso contra o desenho), a sonda das leis e, no irmão, os gates.

use super::a13::{dist_pl, fechados_de, voltas};
use super::*;
use ph2d_vec_skin::curva::{Bake, CampoIndexado, Refino};
use std::rc::Rc;

/// A `=6` presa e dobrada em S a `g1`/`g2`.
pub(crate) fn cena(
    g1: f32,
    g2: f32,
) -> (SimWorld, VecScene, crate::state::VecState, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    let pend = st.bone_smoke_pend.take().expect("pendentes");
    for (id, raiz) in &pend {
        assert_eq!(
            ph2d_skeleton_live::skin_live::bind(&mut sim, &mut scene, &st.entities, &[*id], *raiz),
            1
        );
        crate::smoke_bone_par::dobra_duas(&mut sim, raiz.expect("raiz"), g1, g2);
    }
    let ids = pend.iter().map(|p| p.0).collect();
    (sim, scene, st, ids)
}

/// O que o bake do produto lê para uma barra presa: a pele, a fonte (só fechados) e a tabela.
pub(crate) struct Barra {
    pele: ph2d_skeleton::Skin,
    prep: Rc<ph2d_skeleton_live::skin_desenho::Preparado>,
    fonte: ph2d_vec_scene::VecPath,
    tabela: Vec<f64>,
    correcoes: Vec<ph2d_skeleton::Correccao>,
    /// A largura da peça — a escala da grelha.
    pub(crate) t: f64,
}

impl Barra {
    pub(crate) fn de(sim: &SimWorld, st: &crate::state::VecState, id: VecPathId) -> Self {
        use ph2d_skeleton_live::skin_desenho as sd;
        let e = st
            .entities
            .get(&id)
            .and_then(|b| Entity::try_from_bits(*b))
            .expect("entidade");
        let bind = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .expect("bind")
            .clone();
        let prep = sd::lida(e.to_bits(), &bind).expect("fonte");
        let g = &prep.guardado;
        assert!(
            sd::os_nos_servem(&g.path),
            "a barra da =6 não tem quinas vivas"
        );
        let tabela = bind
            .pesos_do_quadro(if g.valida() { &g.pesos } else { &[] })
            .to_vec();
        Self {
            pele: ph2d_skeleton_live::skin_live::skin_of(sim, e).expect("pele"),
            fonte: g.path.clone(),
            tabela,
            correcoes: bind.correcoes_resolvidas(),
            prep,
            t: crate::smoke_bone_efeitos::PECA.1,
        }
    }

    /// As amostras por segmento do bake do produto (só fechados: um segmento por nó).
    pub(crate) fn amostras(&self) -> usize {
        let segs: usize = (0..self.fonte.contour_count())
            .filter_map(|c| self.fonte.contour(c))
            .filter(|(_, f)| *f)
            .map(|(v, _)| v.len())
            .sum();
        ph2d_skeleton_live::skin_desenho::amostras_por_segmento(segs)
    }

    /// A tolerância do ajuste do bake do produto (fracção da diagonal das âncoras e alças).
    pub(crate) fn tolerancia(&self) -> f64 {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in self.fonte.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
        }
        ph2d_skeleton_live::skin_desenho::TOLERANCIA_DA_DIAGONAL
            * (hi[0] - lo[0]).hypot(hi[1] - lo[1])
    }

    /// O bake do produto com `refino` (a contagem e a tolerância do `skin_desenho`).
    pub(crate) fn assa(&self, refino: Option<Refino>) -> ph2d_vec_scene::VecPath {
        ph2d_vec_skin::curva::assa_a_pele(
            &self.pele,
            &self.fonte,
            &self.tabela,
            &self.correcoes,
            true,
            CampoIndexado {
                campo: self.prep.guardado.campo.as_ref(),
                indice: self.prep.indice.as_ref(),
                suave: None,
            },
            Bake {
                amostras: self.amostras(),
                tolerancia: self.tolerancia(),
                refino,
            },
        )
    }

    /// A IMAGEM EXACTA do contorno em repouso: `n` pontos por segmento postos pela lei do bake.
    pub(crate) fn imagem(&self, n: usize) -> Vec<Vec<[f64; 2]>> {
        let campo = self.prep.guardado.campo.as_ref().expect("campo");
        polilinhas(&fechados_de(&self.fonte), n)
            .into_iter()
            .map(|l| {
                l.into_iter()
                    .map(|p| {
                        let linha = campo
                            .linha_com(p, self.prep.indice.as_ref())
                            .expect("o contorno está no domínio");
                        let mut w = self.pele.scratch();
                        self.pele
                            .point_corrected(p, Some(&linha), &mut w, &self.correcoes)
                    })
                    .collect()
            })
            .collect()
    }
}

/// ⭐ **A régua A13** de um desenho contra a imagem exacta: `(o maior afastamento imagem → desenho,
/// as células onde a imagem PINTA (grau ≠ 0) e o desenho não)`, numa grelha de `t/20` sem as duas
/// bordas (`2` células). Uma célula de grau `0` coberta por camadas de sinais opostos é a
/// cancelação verdadeira, e fica de fora por construção.
pub(crate) fn regua(
    imagem: &[Vec<[f64; 2]>],
    desenho: &ph2d_vec_scene::VecPath,
    t: f64,
) -> (f64, usize) {
    let fino = polilinhas(&fechados_de(desenho), 64);
    let gap = imagem
        .iter()
        .flatten()
        .map(|p| dist_pl(&fino, *p))
        .fold(0.0, f64::max);
    let d = polilinhas(&fechados_de(desenho), 32);
    let cel = t / 20.0;
    let mut cx = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for p in imagem.iter().chain(&d).flatten() {
        cx = [
            cx[0].min(p[0]),
            cx[1].min(p[1]),
            cx[2].max(p[0]),
            cx[3].max(p[1]),
        ];
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caixa finita"
    )]
    let (gx, gy) = (
        ((cx[2] - cx[0]) / cel) as usize + 1,
        ((cx[3] - cx[1]) / cel) as usize + 1,
    );
    let mut borda = vec![false; gx * gy];
    for s in imagem.iter().chain(&d).flat_map(|l| l.windows(2)) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "finito"
        )]
        let n = ((s[1][0] - s[0][0]).hypot(s[1][1] - s[0][1]) / (cel / 4.0)) as usize + 1;
        for m in 0..=n {
            #[expect(clippy::cast_precision_loss, reason = "um punhado")]
            let u = m as f64 / n as f64;
            #[expect(clippy::cast_possible_truncation, reason = "finito")]
            let (i, j) = (
                ((s[0][0] + u * (s[1][0] - s[0][0]) - cx[0]) / cel) as i64,
                ((s[0][1] + u * (s[1][1] - s[0][1]) - cx[1]) / cel) as i64,
            );
            for (a, b) in (-2..=2).flat_map(|di| (-2..=2).map(move |dj| (i + di, j + dj))) {
                if let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b))
                    && a < gx
                    && b < gy
                {
                    borda[a * gy + b] = true;
                }
            }
        }
    }
    let mut falta = 0;
    for i in 0..gx {
        for j in 0..gy {
            if borda[i * gy + j] {
                continue;
            }
            #[expect(clippy::cast_precision_loss, reason = "um punhado")]
            let c = [
                cx[0] + (i as f64 + 0.5) * cel,
                cx[1] + (j as f64 + 0.5) * cel,
            ];
            if voltas(imagem, c) != 0 && voltas(&d, c) == 0 {
                falta += 1;
            }
        }
    }
    (gap, falta)
}

/// Sem refino, e `k ∈ {2, 3}`.
fn leis() -> Vec<(String, Option<Refino>)> {
    let mut v = vec![("sem".to_string(), None)];
    for k in [2.0, 3.0] {
        v.push((format!("k{k}"), Some(Refino { k })));
    }
    v
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// ⭐ **SONDA — as leis de refino na barra `0` da `=6`**: a régua A13 a `170°`, e (em `--release`)
/// nós e µs por bake, mínimo de `5` rondas intercaladas, com a carga ao lado.
#[test]
#[ignore = "sonda: imprime; o relógio só vale em --release"]
fn diag_a13_as_leis_de_refino_na_barra() {
    for (g1, g2) in [
        (170f32, 110f32),
        (170.0, 140.0),
        (170.0, 150.0),
        (170.0, 170.0),
    ] {
        let (sim, _, st, ids) = cena(g1, g2);
        let b = Barra::de(&sim, &st, ids[0]);
        let img = b.imagem(256);
        println!("{g1}/{g2}:");
        for (nome, lei) in leis() {
            let (gap, falta) = regua(&img, &b.assa(lei), b.t);
            println!("    {nome:>10}: afastamento {gap:.4} · células que faltam {falta}");
        }
    }
    for (g1, g2) in [(90f32, 90f32), (110.0, 110.0), (170.0, 140.0)] {
        let (sim, _, st, ids) = cena(g1, g2);
        let b = Barra::de(&sim, &st, ids[0]);
        let ls = leis();
        let feitos: Vec<_> = ls.iter().map(|(_, l)| b.assa(*l)).collect();
        let mut us = vec![f64::MAX; ls.len()];
        let antes = carga();
        for _ in 0..5 {
            for (k, (_, l)) in ls.iter().enumerate() {
                let t = std::time::Instant::now();
                let mut n = 0u32;
                while t.elapsed().as_millis() < 40 {
                    std::hint::black_box(b.assa(*l));
                    n += 1;
                }
                us[k] = us[k].min(t.elapsed().as_secs_f64() * 1e6 / f64::from(n));
            }
        }
        println!("{g1}/{g2} · carga {antes} → {}", carga());
        for (k, (nome, _)) in ls.iter().enumerate() {
            println!(
                "    {nome:>10}: {:>4} nós · {:>7.1} µs · ao bit do sem: {}",
                feitos[k].verts_all().count(),
                us[k],
                feitos[k] == feitos[0]
            );
        }
    }
}

#[path = "smoke_bone_copias_refino_tests.rs"]
mod gates;
