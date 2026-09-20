//! ⭐⭐ **O ESQUELETO VIVO NO DOCUMENTO** — prender uma forma (ou uma imagem) aos ossos, e
//! responder o que a corrente deles é.
//!
//! # Por que esta folha existe, e por que ela NÃO é a `ph2d-skeleton-ecs`
//!
//! O esqueleto é módulo próprio desde [ADR-0169], com a lei em `ph2d-skeleton` e os componentes em
//! `ph2d-skeleton-ecs`. O que vivia na shell era o degrau do meio: as operações **sobre o mundo**
//! — `bind`, `bind_image`, `bone_segments`, `chain_to` — e elas são **puras** (`SimWorld` mais
//! tipos de crates; zero `App`, zero `gfx`).
//!
//! ⛔⛔ **Elas não podem morar na `ph2d-skeleton-ecs`, e o motivo é um CICLO:** o `bind` precisa do
//! mapa `caminho ⟺ entidade` da [`ph2d_vec_entities`], e essa crate **já depende** da
//! `ph2d-skeleton-ecs` (o `settle_origins` dela salta uma forma presa a um osso). Pôr a lei lá
//! fecharia `skeleton-ecs → vec-entities → skeleton-ecs`, que o cargo recusa. *Uma folha nova não
//! é escolha de gosto quando a alternativa é um ciclo.*
//!
//! # O que a tirou da shell
//!
//! A cena `PH2D_VEC_BONE_SMOKE` é um dos **cinco** roteadores da família `vec`, e a catraca
//! `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` é **all-or-nothing por família**: enquanto uma cena
//! precisasse de um módulo da shell, **nenhum** dos cinco podia ser declarado. Esta crate é o que
//! fecha essa conta.
//!
//! ⚠️ **Os módulos da shell NÃO foram apagados** — `skeleton_live` e `skeleton_skin_image` ficaram
//! como re-exportação de uma linha, e os **21** ficheiros que os nomeiam continuam byte a byte
//! iguais ([HOWTO §1.2]).
//!
//! [ADR-0169]: ../../../docs/architecture/decisions/0169-the-skeleton-is-its-own-module-and-each-medium-answers-only-what-a-point-is.md
//! [HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

/// ⭐ A CURVATURA de um osso, viva — irmão do `skin_live` pelo tecto de LOC, cortado por assunto.
mod ancora_da_mancha;
#[cfg(test)]
#[path = "barra_da_cena_tests_support.rs"]
mod barra_da_cena_tests_support;
pub mod bend_live;
pub mod bone;
#[cfg(test)]
#[path = "entalhe_tests.rs"]
mod entalhe_tests;
/// ⭐⭐⭐ **ESPELHAR um ramo** — construir o lado esquerdo a partir do direito.
pub mod espelho;
pub mod esqueletos;
#[cfg(test)]
#[path = "esqueletos_tests_support.rs"]
mod esqueletos_tests_support;
pub mod goal;
/// ⭐⭐⭐ **O PESO À MÃO** — o gesto e o olho da correcção que o artista pinta.
pub mod peso_a_mao;
#[cfg(test)]
#[path = "peso_entre_os_nos_tests.rs"]
mod peso_entre_os_nos_tests;
/// ⭐⭐⭐ **A POSE DE REPOUSO** — voltar ao repouso, e o que o *Reset Transform* quer dizer num osso.
/// ⭐⭐⭐ Um ponto NOVO numa forma presa — ele entra na fonte guardada e sobrevive ao quadro.
pub mod ponto_novo;
pub mod pose_de_repouso;
pub mod recusa_do_osso;
/// ⭐ O ORÇAMENTO de peças do quadro — irmão do `skin_image` pelo tecto de LOC, cortado por assunto.
pub mod skin_bake;
pub mod skin_bake_cache;
pub mod skin_budget;
pub mod skin_gpu;
pub mod skin_image;
/// ⭐ **PRENDER uma IMAGEM** — irmão do `skin_live` pelo tecto de LOC, cortado por assunto.
pub mod skin_image_bind;
pub mod skin_live;
pub mod skin_refine;
pub mod skinned_mesh;
/// ⭐⭐⭐ **A SUBDIVISÃO DO BIND** — os pontos nascem visíveis quando a forma é presa.
pub mod subdivisao;

/// ⚠️ **Os auxiliares que ATRAVESSAM a fronteira, e nada mais** (HOWTO §2.5).
///
/// O gate `probe_the_smoke_sequence` ficou na SHELL porque o sujeito dele (a sequência do smoke,
/// que inclui uma POSE) ficou lá — e ele usa dois auxiliares que eram `#[cfg(test)]` desta crate.
/// Um `cfg(test)` é **falso** quando a crate é dependência, logo eles tinham de atravessar ou ser
/// copiados. ⛔ Copiar seriam duas definições da mesma régua, e a que envelhece é a de fora.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use ph2d_ecs::SimWorld;
    use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

    /// Re-cozinha a cena e devolve o caminho `id` como ele ficou.
    pub fn quadro(sim: &SimWorld, scene: &mut VecScene, id: VecPathId) -> VecPath {
        crate::skin_live::recook(sim, scene);
        scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .expect("o path")
            .clone()
    }

    /// ⭐⭐⭐ **UMA PELE QUE CAIU NA LEI DERIVADA, presa a `ossos`** — a fixtura de *«aqui o envelope
    /// ainda manda»*.
    ///
    /// ⛔⛔ **Ela atravessa a fronteira porque uma fixtura escrita à mão do outro lado já se partiu
    /// DUAS vezes.** O gate `when_two_handles_overlap_the_nearer_one_wins` (na
    /// `ph2d-app-skeleton`) precisa de um osso com região de influência, e montava a
    /// [`ph2d_skeleton_ecs::SkinBind`] com `source: Vec::new()`. Isso funcionou enquanto a lei
    /// perguntava pela MÍDIA, e deixou de funcionar no dia em que ela passou a perguntar pelo
    /// **BIND** — *uma fixtura montada à mão fica abaixo da lei que se está a medir*, e a cura é
    /// sempre da fixtura, nunca da lei.
    ///
    /// ⚠️ **O que a torna honesta é a tabela VAZIA e o caminho ABERTO**: é exactamente o que o
    /// [`crate::skin_live::bind`] escreve quando o padrão-ouro não tem domínio para resolver.
    /// ⛔ Uma `source` que nem descodifica NÃO serve — ali o quadro pula a pele e não há deformação
    /// nenhuma para o alcance governar.
    #[must_use]
    pub fn pele_na_lei_derivada(ossos: &[ph2d_ecs::StableId]) -> ph2d_skeleton_ecs::SkinBind {
        let guardado = crate::skinned_mesh::SkinnedPath {
            path: ph2d_vec_scene::cook(
                ph2d_vec_scene::ShapeKind::Line,
                [0.0, 0.0],
                [1.0, 0.0],
                &[],
            ),
            pesos: Vec::new(),
        };
        ph2d_skeleton_ecs::SkinBind::new(
            crate::skinned_mesh::grava(&guardado).expect("a pele codifica"),
            ossos
                .iter()
                .map(|bone| ph2d_skeleton_ecs::Tendon {
                    bone: *bone,
                    rest: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                })
                .collect(),
        )
    }

    /// ⭐⭐⭐ **O maior afastamento entre os DESENHOS de dois caminhos** — irmã da
    /// [`pior_desvio`], e a diferença entre elas é a que esta casa já pagou duas vezes.
    ///
    /// A [`pior_desvio`] emparelha **vértice com vértice**: ela responde *«a REPRESENTAÇÃO mudou?»*.
    /// Esta amostra as duas curvas e mede ponto-a-polilinha: ela responde *«o DESENHO mudou?»*.
    ///
    /// ⛔⛔ **Quando a contagem de vértices muda, a primeira não afirma nada** — ela emparelha o
    /// canto de uma com o meio da aresta da outra e devolve um número enorme sobre geometria
    /// idêntica (medido: `37,5` ao prender um rectângulo de `40×10` depois de a subdivisão do bind
    /// existir). *É a mesma lição da F30, um nível acima: o desenho estava certo e oito gates da
    /// casa mediam a representação.*
    #[must_use]
    pub fn pior_desvio_do_desenho(a: &VecPath, b: &VecPath) -> f64 {
        let da = amostra_densa(a);
        let db = amostra_densa(b);
        afastamento_max(&da, &db).max(afastamento_max(&db, &da))
    }

    /// A curva COZIDA achatada com densidade constante POR COMPRIMENTO.
    ///
    /// ⚠️ **Por comprimento e não por segmento:** `N` amostras por segmento afinam a polilinha à
    /// medida que a forma é subdividida, e a comparação lê «mexeu» sobre um corte exacto.
    fn amostra_densa(p: &VecPath) -> Vec<[f64; 2]> {
        const POR_UNIDADE: f64 = 1000.0;
        let cozido = p.cooked();
        let mut out = Vec::new();
        for c in 0..cozido.contour_count() {
            let Some((v, fechado)) = cozido.contour(c) else {
                continue;
            };
            let n = v.len();
            let ultimo = if fechado { n } else { n.saturating_sub(1) };
            for i in 0..ultimo {
                let (a, b) = (&v[i], &v[(i + 1) % n]);
                let d = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
                let poli = d(a.anchor, a.out_handle)
                    + d(a.out_handle, b.in_handle)
                    + d(b.in_handle, b.anchor);
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "a contagem e' presa a [8, 20 000] a seguir"
                )]
                let amostras = ((poli * POR_UNIDADE) as usize).clamp(8, 20_000);
                for k in 0..amostras {
                    #[expect(clippy::cast_precision_loss, reason = "k < amostras <= 20 000")]
                    let t = k as f64 / amostras as f64;
                    let u = 1.0 - t;
                    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    out.push([
                        w3.mul_add(
                            b.anchor[0],
                            w2.mul_add(
                                b.in_handle[0],
                                w1.mul_add(a.out_handle[0], w0 * a.anchor[0]),
                            ),
                        ),
                        w3.mul_add(
                            b.anchor[1],
                            w2.mul_add(
                                b.in_handle[1],
                                w1.mul_add(a.out_handle[1], w0 * a.anchor[1]),
                            ),
                        ),
                    ]);
                }
            }
        }
        out
    }

    /// A maior distância de um ponto de `a` à polilinha de `b`. As consultas são subamostradas —
    /// ver o cabeçalho da [`pior_desvio_do_desenho`].
    fn afastamento_max(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
        const CONSULTAS: usize = 2_000;
        if a.is_empty() || b.len() < 2 {
            return 0.0;
        }
        let salto = a.len().div_ceil(CONSULTAS).max(1);
        let d2 = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
            let (vx, vy) = (r[0] - q[0], r[1] - q[1]);
            let l2 = vy.mul_add(vy, vx * vx);
            let t = if l2 <= f64::EPSILON {
                0.0
            } else {
                (((p[1] - q[1]) * vy + (p[0] - q[0]) * vx) / l2).clamp(0.0, 1.0)
            };
            let (x, y) = (t.mul_add(vx, q[0]) - p[0], t.mul_add(vy, q[1]) - p[1]);
            y.mul_add(y, x * x)
        };
        a.iter()
            .step_by(salto)
            .map(|p| {
                (0..b.len())
                    .map(|i| d2(*p, b[i], b[(i + 1) % b.len()]))
                    .fold(f64::INFINITY, f64::min)
                    .sqrt()
            })
            .fold(0.0, f64::max)
    }

    /// O maior desvio, em qualquer eixo, entre os VÉRTICES de dois estados do mesmo caminho.
    ///
    /// ⚠️ **Ela só afirma alguma coisa quando os dois lados têm a mesma representação** — ver a
    /// irmã [`pior_desvio_do_desenho`].
    #[must_use]
    pub fn pior_desvio(a: &VecPath, b: &VecPath) -> f64 {
        a.verts_all()
            .zip(b.verts_all())
            .flat_map(|(x, y)| {
                [
                    (x.anchor, y.anchor),
                    (x.in_handle, y.in_handle),
                    (x.out_handle, y.out_handle),
                ]
            })
            .fold(0.0_f64, |m, (p, q)| {
                m.max((p[0] - q[0]).abs()).max((p[1] - q[1]).abs())
            })
    }
}

#[cfg(test)]
#[path = "sonda_do_envelope_tests.rs"]
mod sonda_do_envelope_tests;

#[cfg(test)]
#[path = "sonda_do_envelope_no_vector_tests.rs"]
mod sonda_do_envelope_no_vector_tests;

#[cfg(test)]
#[path = "sonda_do_reset_na_hierarquia_tests.rs"]
mod sonda_do_reset_na_hierarquia_tests;

#[cfg(test)]
#[path = "alcas_tests.rs"]
mod alcas_tests;
