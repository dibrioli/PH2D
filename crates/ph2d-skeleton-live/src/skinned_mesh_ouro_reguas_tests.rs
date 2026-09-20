//! ═══════════════════════════════════════════════════════════════════════════════════════════
//! LENTE B — AS RÉGUAS **LOCAIS** DA DEFORMAÇÃO VECTORIAL (auditoria ordenada em 2026-09-20)
//!
//! ⛔⛔ A casa mede sobretudo `pior_desvio_do_desenho`, que é um MÁXIMO GLOBAL entre DUAS SAÍDAS
//! NOSSAS. Ele responde *«a lei A e a lei B concordam?»* e é cego a tudo o que um artista chama de
//! irregularidade: área perdida, contorno a cruzar-se, vinco, braço a afinar.
//!
//! ⭐ Estas sondas medem contra um PADRÃO-OURO — a mesma forma deformada como a mídia IMAGEM faz
//! (cada ponto pelo peso DELE, lido do `CampoDoDominio` guardado no bind) — e são todas p50/p90/max.
//! ═══════════════════════════════════════════════════════════════════════════════════════════//!
//! ⚠️⚠️ **Tudo aqui é `pub(super)` de propósito:** os três irmãos que medem são módulos SEPARADOS
//! (tecto de LOC), e um item privado não é visível de um irmão. `pub(super)` abre-o ao pai e aos
//! descendentes dele — que é exactamente esta família — e **não** à crate, que é o que um
//! `pub(crate)` faria com uma instrumentação de auditoria.
//!
//! ⚠️ **Ele saiu do [`super::skinned_mesh_tests`] por TECTO DE LOC** (2026-09-20, `2 304` contra
//! `700`), e o corte é por RESPONSABILIDADE e não pelo excesso: o irmão mede *«as duas leis
//! concordam?»* e este mede *«a nossa lei bate o PADRÃO-OURO?»*. ⛔ Uma entrada nova no
//! `FILE_OVERAGE_OK` teria deixado as duas perguntas no mesmo ficheiro para sempre.

/// Amostras por segmento de cúbica. ⚠️ Fixo, e a parametrização `(segmento, t)` é a MESMA na fonte
/// e no produto — é isso que deixa comparar ponto a ponto além de curva a curva.
pub(super) const B_N: usize = 32;

pub(super) fn b_cub(v: &[ph2d_vec_scene::VecVertex], k: usize) -> [[f64; 2]; 4] {
    let n = v.len();
    let (a, b) = (&v[k], &v[(k + 1) % n]);
    [a.anchor, a.out_handle, b.in_handle, b.anchor]
}

pub(super) fn b_eval(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * c[0][0] + w1 * c[1][0] + w2 * c[2][0] + w3 * c[3][0],
        w0 * c[0][1] + w1 * c[1][1] + w2 * c[2][1] + w3 * c[3][1],
    ]
}

/// O contorno `0` amostrado na parametrização `(segmento, t)`, `t ∈ [0,1)`.
pub(super) fn b_amostra(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let cozido = p.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return Vec::new();
    };
    let n = v.len();
    let mut out = Vec::with_capacity(n * B_N);
    for k in 0..n {
        let c = b_cub(v, k);
        for i in 0..B_N {
            #[expect(clippy::cast_precision_loss, reason = "i < B_N")]
            out.push(b_eval(&c, i as f64 / B_N as f64));
        }
    }
    out
}

/// As âncoras do contorno `0`, na ordem dos nós.
pub(super) fn b_ancoras(p: &ph2d_vec_scene::VecPath) -> Vec<[f64; 2]> {
    let cozido = p.cooked();
    cozido
        .contour(0)
        .map(|(v, _)| v.iter().map(|x| x.anchor).collect())
        .unwrap_or_default()
}

pub(super) fn b_pct(v: &mut [f64]) -> (f64, f64, f64) {
    if v.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    v.sort_by(|a, b| a.total_cmp(b));
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "índice de percentil"
    )]
    let q = |f: f64| v[(((v.len() - 1) as f64) * f).round() as usize];
    (q(0.5), q(0.9), v[v.len() - 1])
}

/// A distância de `p` à POLILINHA FECHADA `poli` — *«o desenho passa por aqui?»*.
pub(super) fn b_dist(p: [f64; 2], poli: &[[f64; 2]]) -> f64 {
    let n = poli.len();
    let mut melhor = f64::INFINITY;
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l2 = dx.mul_add(dx, dy * dy);
        let t = if l2 > 0.0 {
            (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let d = (p[0] - t.mul_add(dx, a[0])).hypot(p[1] - t.mul_add(dy, a[1]));
        if d < melhor {
            melhor = d;
        }
    }
    melhor
}

/// O perfil de afastamento de `a` a `b` — p50 / p90 / max.
pub(super) fn b_perfil(a: &[[f64; 2]], b: &[[f64; 2]]) -> (f64, f64, f64) {
    let mut d: Vec<f64> = a.iter().map(|&p| b_dist(p, b)).collect();
    b_pct(&mut d)
}

pub(super) fn b_area(poli: &[[f64; 2]]) -> f64 {
    ph2d_poly2d::signed_area(poli).abs()
}

/// Quantos pares de segmentos NÃO adjacentes se cruzam de verdade.
pub(super) fn b_auto(poli: &[[f64; 2]], eps: f64) -> usize {
    let n = poli.len();
    let curto = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= eps;
    let sinal = |o: [f64; 2], u: [f64; 2], v: [f64; 2]| {
        ((u[0] - o[0]) * (v[1] - o[1]) - (v[0] - o[0]) * (u[1] - o[1])).signum()
    };
    let mut c = 0usize;
    for i in 0..n {
        let (a1, a2) = (poli[i], poli[(i + 1) % n]);
        if curto(a1, a2) {
            continue;
        }
        for j in (i + 2)..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (b1, b2) = (poli[j], poli[(j + 1) % n]);
            if curto(b1, b2) {
                continue;
            }
            if a1[0].max(a2[0]) < b1[0].min(b2[0])
                || b1[0].max(b2[0]) < a1[0].min(a2[0])
                || a1[1].max(a2[1]) < b1[1].min(b2[1])
                || b1[1].max(b2[1]) < a1[1].min(a2[1])
            {
                continue;
            }
            if sinal(a1, a2, b1) != sinal(a1, a2, b2) && sinal(b1, b2, a1) != sinal(b1, b2, a2) {
                c += 1;
            }
        }
    }
    c
}

/// Os pesos do vértice da malha do domínio mais próximo — a resposta que a mídia imagem daria a um
/// ponto sem triângulo (a malha do bind é uma GRELHA, logo a curva pode sair dela).
pub(super) fn b_mais_proximo(
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    p: [f64; 2],
) -> Vec<f64> {
    let mut melhor = (f64::INFINITY, 0usize);
    for i in 0..campo.malha.rest.len() {
        if let Some(q) = campo.local_do_vertice(i) {
            let d = (q[0] - p[0]).hypot(q[1] - p[1]);
            if d < melhor.0 {
                melhor = (d, i);
            }
        }
    }
    campo
        .linha_do_vertice(melhor.1)
        .map(<[f64]>::to_vec)
        .unwrap_or_default()
}

/// ⭐⭐⭐ **O PADRÃO-OURO NUM PONTO** — a lei da mídia IMAGEM aplicada ao contorno: cada ponto pelo
/// peso DELE, lido do campo, misturado pela mesma [`ph2d_skeleton::Skin::blend`] do produto.
///
/// Devolve `(imagem, veio_do_campo)`.
pub(super) fn b_ouro_pt(
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
    p: [f64; 2],
) -> ([f64; 2], bool) {
    let mut w = pele.scratch();
    let dentro = campo.linha(p);
    let ok = dentro.is_some();
    let linha = dentro.unwrap_or_else(|| b_mais_proximo(campo, p));
    pele.weights_corrected(p, Some(&linha), &mut w, correcoes);
    (pele.blend(p, &w), ok)
}

/// ⭐ **A LARGURA DO BRAÇO ao longo dele** — o par de pontos de repouso que estão um por cima do
/// outro, medido DEPOIS da deformação. Em repouso a barra tem `1,0` de espessura em todo o troço
/// recto (`x ∈ [-8, -2]`), logo toda leitura diferente de `1,0` é encolhimento ou inchaço.
pub(super) fn b_larguras(rest: &[[f64; 2]], def: &[[f64; 2]]) -> Vec<f64> {
    let mut out = Vec::new();
    for s in 0..=12 {
        let x = f64::from(s).mul_add(6.0 / 12.0, -8.0);
        let (mut cima, mut baixo) = ((f64::INFINITY, usize::MAX), (f64::INFINITY, usize::MAX));
        for (i, r) in rest.iter().enumerate() {
            let d = (r[0] - x).abs();
            if (r[1] - 3.0).abs() < 1e-6 {
                if d < cima.0 {
                    cima = (d, i);
                }
            } else if (r[1] - 2.0).abs() < 1e-6 && d < baixo.0 {
                baixo = (d, i);
            }
        }
        if cima.1 == usize::MAX || baixo.1 == usize::MAX || cima.0 > 0.25 || baixo.0 > 0.25 {
            continue;
        }
        let (a, b) = (def[cima.1], def[baixo.1]);
        out.push((a[0] - b[0]).hypot(a[1] - b[1]));
    }
    out
}

/// A QUEBRA DA TANGENTE em cada nó, em graus — `0` num nó liso.
pub(super) fn b_quebra_nos(p: &ph2d_vec_scene::VecPath) -> Vec<f64> {
    let cozido = p.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return Vec::new();
    };
    v.iter()
        .filter_map(|x| {
            let ent = [x.anchor[0] - x.in_handle[0], x.anchor[1] - x.in_handle[1]];
            let sai = [x.out_handle[0] - x.anchor[0], x.out_handle[1] - x.anchor[1]];
            let (le, ls) = (ent[0].hypot(ent[1]), sai[0].hypot(sai[1]));
            if le <= 1e-12 || ls <= 1e-12 {
                return None;
            }
            let cruz = ent[0].mul_add(sai[1], -(ent[1] * sai[0]));
            let esc = ent[0].mul_add(sai[0], ent[1] * sai[1]);
            Some(cruz.atan2(esc).abs().to_degrees())
        })
        .collect()
}

/// A MELHOR cúbica possível para um segmento — pontas presas no padrão-ouro, as duas alças livres,
/// mínimos quadrados sobre `amostras` pontos. ⭐ **É o CHÃO do modelo**: nenhum ajuste de alças pode
/// fazer melhor do que isto com esta contagem de nós.
pub(super) fn b_melhor_cubica(ouro: &[[f64; 2]], ts: &[f64]) -> [[f64; 2]; 4] {
    let (p0, p3) = (ouro[0], ouro[ouro.len() - 1]);
    let (mut a11, mut a12, mut a22) = (0.0_f64, 0.0_f64, 0.0_f64);
    let (mut b1, mut b2) = ([0.0_f64; 2], [0.0_f64; 2]);
    for (i, &t) in ts.iter().enumerate() {
        let u = 1.0 - t;
        let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        let d = [
            w3.mul_add(-p3[0], w0.mul_add(-p0[0], ouro[i][0])),
            w3.mul_add(-p3[1], w0.mul_add(-p0[1], ouro[i][1])),
        ];
        a11 = w1.mul_add(w1, a11);
        a12 = w1.mul_add(w2, a12);
        a22 = w2.mul_add(w2, a22);
        b1 = [w1.mul_add(d[0], b1[0]), w1.mul_add(d[1], b1[1])];
        b2 = [w2.mul_add(d[0], b2[0]), w2.mul_add(d[1], b2[1])];
    }
    let det = a12.mul_add(-a12, a11 * a22);
    let p1 = [
        (b1[0] * a22 - b2[0] * a12) / det,
        (b1[1] * a22 - b2[1] * a12) / det,
    ];
    let p2 = [
        (b2[0] * a11 - b1[0] * a12) / det,
        (b2[1] * a11 - b1[1] * a12) / det,
    ];
    [p0, p1, p2, p3]
}

/// A escala a que o VINCO é medido — `5 %` da espessura da barra.
///
/// ⚠️ **Ela é a régua, e tem de ser dita:** a curvatura de um canto depende da escala a que se
/// olha. A esta, uma quina de `180°` satura em `κ = 2/h = 40`, a tampa da cápsula (raio `0,5`) lê
/// `2,0` e uma aresta recta lê `0`.
pub(super) const B_H: f64 = 0.05;

/// O comprimento de arco acumulado de uma polilinha FECHADA (`len + 1` casas; a última é o
/// perímetro).
pub(super) fn b_cum(poli: &[[f64; 2]]) -> Vec<f64> {
    let n = poli.len();
    let mut c = Vec::with_capacity(n + 1);
    c.push(0.0);
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        c.push(c[i] + (a[0] - b[0]).hypot(a[1] - b[1]));
    }
    c
}

/// O ponto da polilinha ao comprimento de arco `s` (com volta).
pub(super) fn b_em(poli: &[[f64; 2]], cum: &[f64], s: f64) -> [f64; 2] {
    let per = cum[cum.len() - 1];
    if per <= 0.0 {
        return poli[0];
    }
    let s = s.rem_euclid(per);
    let (mut lo, mut hi) = (0usize, cum.len() - 1);
    while hi - lo > 1 {
        let m = (lo + hi) / 2;
        if cum[m] <= s { lo = m } else { hi = m }
    }
    let seg = cum[lo + 1] - cum[lo];
    let u = if seg > 0.0 { (s - cum[lo]) / seg } else { 0.0 };
    let n = poli.len();
    let (a, b) = (poli[lo % n], poli[(lo + 1) % n]);
    [u.mul_add(b[0] - a[0], a[0]), u.mul_add(b[1] - a[1], a[1])]
}

/// ⭐⭐⭐ **O VINCO — curvatura de MENGER à escala `h`**, uma por amostra, na MESMA ordem do
/// repouso.
///
/// ⛔⛔ **Ela substitui duas réguas que MENTIRAM nesta auditoria.** A curvatura tirada da
/// amostragem em `t` divide o ângulo por um comprimento que a própria deformação COMPRIME e leu
/// `7,07e15` sobre a cápsula (segmentos de comprimento zero) e `752` sobre uma aresta sã; a versão
/// que reamostra o arco e mede vizinhos consecutivos **ALIASA** — dois pontos dentro do mesmo
/// segmento da polilinha dão viragem `0` e dois que o atravessam dão a viragem inteira, o que
/// concentra o sinal e inflaciona o máximo (leu `124,75` onde esta lê muito menos).
///
/// ⭐ Esta pega os pontos a **`±h` de ARCO** e lê o círculo que passa pelos três: ela não conhece a
/// amostragem, e uma aresta recta lê `0` mesmo com amostras irregulares.
pub(super) fn b_menger(poli: &[[f64; 2]], h: f64) -> Vec<f64> {
    let cum = b_cum(poli);
    let n = poli.len();
    (0..n)
        .map(|i| {
            let s = cum[i];
            let (a, b, c) = (b_em(poli, &cum, s - h), poli[i], b_em(poli, &cum, s + h));
            let ab = (a[0] - b[0]).hypot(a[1] - b[1]);
            let bc = (b[0] - c[0]).hypot(b[1] - c[1]);
            let ca = (c[0] - a[0]).hypot(c[1] - a[1]);
            if ab <= 0.0 || bc <= 0.0 || ca <= 0.0 {
                return 0.0;
            }
            let area2 = (b[0] - a[0])
                .mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])))
                .abs();
            2.0 * area2 / (ab * bc * ca)
        })
        .collect()
}

/// O vinco em GRAUS DE QUINA — o número que o artista lê. Satura em `180°`.
pub(super) fn b_quina(k: f64, h: f64) -> f64 {
    2.0 * (k * h * 0.5).min(1.0).asin().to_degrees()
}

/// **A CURVATURA COM SINAL**, sobre a mesma janela física da [`b_menger`] — irmã dela, e a única
/// diferença é não tomar o módulo.
///
/// ⚠️ **O sinal é o que a torna útil aqui:** a magnitude diz *quanto* a linha curva e o report do
/// dono é sobre *para que LADO* — uma aresta que devia ser um arco só e vai para um lado, volta
/// para o outro, e outra vez.
pub(super) fn b_menger_com_sinal(poli: &[[f64; 2]], h: f64) -> Vec<f64> {
    let cum = b_cum(poli);
    let n = poli.len();
    (0..n)
        .map(|i| {
            let s = cum[i];
            let (a, b, c) = (b_em(poli, &cum, s - h), poli[i], b_em(poli, &cum, s + h));
            let ab = (a[0] - b[0]).hypot(a[1] - b[1]);
            let bc = (b[0] - c[0]).hypot(b[1] - c[1]);
            let ca = (c[0] - a[0]).hypot(c[1] - a[1]);
            if ab <= 0.0 || bc <= 0.0 || ca <= 0.0 {
                return 0.0;
            }
            let cruz = (b[0] - a[0]).mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])));
            2.0 * cruz / (ab * bc * ca)
        })
        .collect()
}

/// As amostras cujo REPOUSO está no troço RECTO da barra (`y = 2` ou `y = 3`) — ⛔ a tampa da
/// cápsula lê `κ = 2` por construção e afogaria o sinal.
pub(super) fn b_rectas(rest: &[[f64; 2]]) -> Vec<usize> {
    (0..rest.len())
        .filter(|&i| (rest[i][1] - 2.0).abs() < 1e-9 || (rest[i][1] - 3.0).abs() < 1e-9)
        .collect()
}

/// O palco das sondas da lente B — a barra da cena do dono, montada UMA vez.
pub(super) struct BPalco {
    pub(super) sim: ph2d_ecs::SimWorld,
    pub(super) scene: ph2d_vec_scene::VecScene,
    pub(super) id: ph2d_vec_scene::VecPathId,
    pub(super) alvo: ph2d_ecs::Entity,
    pub(super) ossos: Vec<ph2d_ecs::Entity>,
    pub(super) campo: ph2d_vec_skin::pesos::CampoDoDominio,
    pub(super) correcoes: Vec<ph2d_skeleton::Correccao>,
    pub(super) fonte: ph2d_vec_scene::VecPath,
}

pub(super) fn b_palco(subdividir: bool) -> BPalco {
    use crate::barra_da_cena_tests_support::barra_da_cena_com;
    use ph2d_ecs::Entity;
    use ph2d_skeleton_ecs::SkinBind;
    let (sim, scene, map, id, ossos) = barra_da_cena_com(subdividir);
    let alvo = Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(alvo).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("a fonte lê-se");
    BPalco {
        sim,
        scene,
        id,
        alvo,
        ossos,
        campo: g.campo.clone().expect("o bind desta wave guarda o campo"),
        correcoes: skin.correcoes_resolvidas(),
        fonte: g.path.clone(),
    }
}

impl BPalco {
    pub(super) fn dobra(&mut self, graus: f32) {
        use ph2d_ecs::Transform;
        for o in &self.ossos[1..] {
            self.sim
                .world_mut()
                .get_mut::<Transform>(*o)
                .expect("Transform")
                .rotation = graus.to_radians();
        }
    }
    /// ⭐⭐⭐ **A DOBRA EM S — os dois ossos para lados OPOSTOS, que é a pose da cena do dono.**
    ///
    /// A [`BPalco::dobra`] põe os dois no MESMO sentido e dá um **C**; a cena do dono autora
    /// `ARM_SHOULDER_BEND = −0,45` e `ARM_ELBOW_BEND = +0,45`, que é um **S**. Ela existe para a
    /// fixtura ser a POSE que o dono fotografou.
    ///
    /// ⛔⛔ **E NÃO é ela que faz o canto — eu escrevi que era e a mutação desmentiu-me.** Medido
    /// (`diag_b_as_duas_escolhas_que_a_mutacao_nao_mata`), a pior quina a `90°` lê **`155,3°` no S
    /// e `158,7°` no C**: a mesma coisa. *O que me fez ler o 1.º desenho como liso não foi a pose
    /// — foi o ZOOM: o canto é local e a uma vista da peça inteira ele passa por uma dobra normal.*
    pub(super) fn dobra_em_s(&mut self, graus: f32) {
        use ph2d_ecs::Transform;
        for (k, o) in self.ossos.iter().enumerate().skip(1) {
            let sinal = if k % 2 == 1 { -1.0 } else { 1.0 };
            self.sim
                .world_mut()
                .get_mut::<Transform>(*o)
                .expect("Transform")
                .rotation = (sinal * graus).to_radians();
        }
    }

    /// ⭐⭐⭐ **REPARTE A DOBRA POR SUB-OSSOS** — o *bendy bone* que o painel chama «Curve Handles».
    ///
    /// `segments` põe cada osso a dobrar em `n` pedaços e [`ph2d_skeleton::bend::Handles::Auto`]
    /// tira as alças das tangentes dos VIZINHOS ⇒ a corrente inteira vira uma curva lisa em vez de
    /// uma cadeia de segmentos rígidos. ⚠️ **Com `segments = 1` a saída é a de sempre, ao bit.**
    pub(super) fn reparte(&mut self, segments: u8) {
        self.reparte_com(segments, true);
    }

    /// ⚠️⚠️ **As DUAS metades separadas, e a separação é obrigatória:** `Segments` é um número que
    /// o artista escreve, e *Curve Handles* é uma fileira que o painel **só pinta com
    /// `Segments > 1`**. Mexer nas duas juntas e depois dizer ao dono *«suba o Segments»* seria
    /// prometer-lhe um resultado que só a outra metade produz — e em 2026-09-20 eu quase o fiz.
    pub(super) fn reparte_com(&mut self, segments: u8, alcas_da_corrente: bool) {
        use ph2d_skeleton_ecs::{Bone, BoneHandles};
        for o in &self.ossos {
            if let Some(mut b) = self.sim.world_mut().get_mut::<Bone>(*o) {
                b.segments = segments;
                b.handles = if alcas_da_corrente && segments > 1 {
                    BoneHandles::Auto
                } else {
                    BoneHandles::Authored
                };
            }
        }
    }

    /// ⭐⭐⭐ **A LEI DO PESO que o painel oferece** — a fileira *Deform By*.
    ///
    /// ⛔⛔ **Ela NÃO é o `PH2D_SKIN_CAMPO`, e confundi-las custou-me um passo de smoke errado.**
    /// O `campo` do [`crate::skin_live::recook_com_mistura`] corta só a consulta ao campo **ENTRE
    /// os nós** — a tabela BBW continua a mandar nos nós. A [`SkinLaw::Envelope`] (*Bone Reach*)
    /// faz o `pesos_do_quadro` devolver **vazio** e a lei euclidiana manda em tudo.
    pub(super) fn lei_do_peso(&mut self, envelope: bool) {
        use ph2d_skeleton_ecs::{SkinBind, SkinLaw};
        if let Some(mut b) = self.sim.world_mut().get_mut::<SkinBind>(self.alvo) {
            b.law = if envelope {
                SkinLaw::Envelope
            } else {
                SkinLaw::Auto
            };
        }
    }

    pub(super) fn pele(&self) -> ph2d_skeleton::Skin {
        crate::skin_live::skin_of(&self.sim, self.alvo).expect("pele resolvida")
    }
    /// O PADRÃO-OURO sobre uma amostragem de repouso — a lei da mídia IMAGEM, ponto a ponto.
    pub(super) fn ouro(&self, pele: &ph2d_skeleton::Skin, rest: &[[f64; 2]]) -> Vec<[f64; 2]> {
        rest.iter()
            .map(|&x| b_ouro_pt(pele, &self.campo, &self.correcoes, x).0)
            .collect()
    }
    /// A saída do PRODUTO, pela porta do produto.
    pub(super) fn produto(&self, curva: bool, campo: bool) -> ph2d_vec_scene::VecPath {
        let mut sc = self.scene.clone();
        crate::skin_live::recook_com_mistura(&self.sim, &mut sc, curva, true, campo);
        sc.paths()
            .iter()
            .find(|p| p.id == self.id)
            .expect("path")
            .clone()
    }
}

/// ⭐ **O CHÃO DO MODELO** — a melhor cúbica possível em CADA segmento desta fonte, com as pontas
/// presas no padrão-ouro. Devolve `(polilinha do chão, pior erro mesmo-`t`)`.
///
/// ⛔ Nenhum ajuste de alças pode fazer melhor do que isto com esta contagem de nós: o que sobra
/// abaixo desta linha é **modelo**, e o que está acima dela é **procedimento**.
pub(super) fn b_chao(
    fonte: &ph2d_vec_scene::VecPath,
    pele: &ph2d_skeleton::Skin,
    campo: &ph2d_vec_skin::pesos::CampoDoDominio,
    correcoes: &[ph2d_skeleton::Correccao],
) -> (Vec<[f64; 2]>, f64, Vec<f64>) {
    #[expect(clippy::cast_precision_loss, reason = "i <= B_N")]
    let ts: Vec<f64> = (0..=B_N).map(|i| i as f64 / B_N as f64).collect();
    let cozido = fonte.cooked();
    let Some((v, _)) = cozido.contour(0) else {
        return (Vec::new(), 0.0, Vec::new());
    };
    let n = v.len();
    let mut poli = Vec::new();
    let mut pior = 0.0_f64;
    let mut por_seg = Vec::with_capacity(n);
    for k in 0..n {
        let c = b_cub(v, k);
        let ouro: Vec<[f64; 2]> = ts
            .iter()
            .map(|&t| b_ouro_pt(pele, campo, correcoes, b_eval(&c, t)).0)
            .collect();
        let best = b_melhor_cubica(&ouro, &ts);
        let mut e = 0.0_f64;
        for (i, &t) in ts.iter().enumerate() {
            let q = b_eval(&best, t);
            e = e.max((q[0] - ouro[i][0]).hypot(q[1] - ouro[i][1]));
        }
        por_seg.push(e);
        pior = pior.max(e);
        for &t in &ts[..B_N] {
            poli.push(b_eval(&best, t));
        }
    }
    (poli, pior, por_seg)
}
