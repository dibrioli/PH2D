//! ⏱️⭐⭐⭐⭐ **A GRADE ASSADA CONTRA A ÁRVORE** — a pergunta que decide a wave do volume.
//!
//! # A pergunta
//!
//! O MagicaCSG (estudo de 2026-09-23, `docs/Render3d/03_o_plano.md` §W9) desenha cada grupo de
//! peças como uma **grade** de `20³`–`64³` distâncias, e por isso `100` formas custam o mesmo que
//! uma. A tabela da `W9` mediu o **custo por amostra** e concluiu *«a grandeza dominante são os
//! PASSOS»*. ⚠️ Ela não mediu a outra metade: com uma grade, o custo por passo deixa de crescer
//! com a complexidade da peça — e a grade marcha com o passo de uma escultura (`1/√2`), que pode
//! **aumentar** os passos. *Só a corrida diz qual das duas ganha.*
//!
//! # ⭐ A régua é a porta do PRODUTO nas duas colunas
//!
//! A grade entra por onde a escultura já entra: um nó [`ph2d_field::NodeKind::Sampled`] resolvido
//! pelo registo, e o dispositivo lê-a pela lei que o `ph2d_field_gpu::sculpt` já tem. ⇒ *nenhuma
//! aritmética de marcha é reescrita aqui*: o que muda entre as duas colunas é o DOCUMENTO.
//!
//! O assar é o mesmo avaliador em lote que a exportação usa (`Hybrid::eval`), sobre a caixa da
//! peça (`bounding_ball`) com `PAD` células de folga.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release -- \
//!   --ignored --exact preview::device_tests::sondas_w9::grade::diag_a_grade_contra_a_arvore \
//!   --nocapture
//! ```

use std::sync::Arc;

use ph2d_field_eval::hybrid::{Registry, Sampled, SampledGrid};

/// Células de folga à volta da caixa da peça — as mesmas oito da [`ph2d_field_mesh`].
const PAD: usize = 8;

/// Uma grade assada de um documento: a lei é a do `SampledField::at`, linha a linha.
struct Grade {
    dims: [usize; 3],
    origin: [f32; 3],
    step: f32,
    values: Vec<f32>,
    raio: f32,
}

impl Grade {
    fn hi(&self) -> [f32; 3] {
        #[allow(clippy::cast_precision_loss)]
        std::array::from_fn(|a| ((self.dims[a] - 1) as f32).mul_add(self.step, self.origin[a]))
    }

    fn dentro(&self, p: [f32; 3]) -> f32 {
        let inv = 1.0 / self.step;
        let mut i0 = [0usize; 3];
        let mut fr = [0.0f32; 3];
        for a in 0..3 {
            #[allow(clippy::cast_precision_loss)]
            let top = (self.dims[a] - 1) as f32;
            let g = if (p[a] - self.origin[a]) * inv >= 0.0 {
                ((p[a] - self.origin[a]) * inv).min(top)
            } else {
                0.0
            };
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let i = (g.floor() as usize).min(self.dims[a] - 2);
            i0[a] = i;
            #[allow(clippy::cast_precision_loss)]
            {
                fr[a] = g - i as f32;
            }
        }
        let rx = self.dims[0];
        let rxy = rx * self.dims[1];
        let base = i0[0] + i0[1] * rx + i0[2] * rxy;
        let c = |dx: usize, dy: usize, dz: usize| self.values[base + dx + dy * rx + dz * rxy];
        let lerp = |a: f32, b: f32, t: f32| (b - a).mul_add(t, a);
        let x00 = lerp(c(0, 0, 0), c(1, 0, 0), fr[0]);
        let x10 = lerp(c(0, 1, 0), c(1, 1, 0), fr[0]);
        let x01 = lerp(c(0, 0, 1), c(1, 0, 1), fr[0]);
        let x11 = lerp(c(0, 1, 1), c(1, 1, 1), fr[0]);
        lerp(lerp(x00, x10, fr[1]), lerp(x01, x11, fr[1]), fr[2])
    }
}

impl Sampled for Grade {
    fn at(&self, p: [f32; 3]) -> f32 {
        let lo = self.origin;
        let hi = self.hi();
        let mut fora = 0.0f32;
        let mut parede = p;
        for a in 0..3 {
            let d = (lo[a] - p[a]).max(p[a] - hi[a]).max(0.0);
            fora = d.mul_add(d, fora);
            parede[a] = p[a].clamp(lo[a], hi[a]);
        }
        let fora = fora.sqrt();
        if fora > 0.0 {
            return fora.max(self.dentro(parede));
        }
        self.dentro(p)
    }
    fn grid(&self) -> Option<SampledGrid<'_>> {
        Some(SampledGrid {
            dims: [
                u32::try_from(self.dims[0]).ok()?,
                u32::try_from(self.dims[1]).ok()?,
                u32::try_from(self.dims[2]).ok()?,
            ],
            origin: self.origin,
            step: self.step,
            values: &self.values,
        })
    }
    fn bounding_radius(&self) -> f32 {
        self.raio
    }
}

/// Assa `doc` numa grade com `res` células no lado MAIOR da peça. Devolve a grade e o relógio.
fn assa(doc: &ph2d_field::FieldDoc, reg: &Registry, res: usize) -> (Grade, f32) {
    let ball = ph2d_field_eval::bounds::bounding_ball(doc, reg).expect("a peça tem caixa");
    let (lo, hi) = ball.aabb();
    let ext = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    #[allow(clippy::cast_precision_loss)]
    let step = ext[0].max(ext[1]).max(ext[2]) / res as f32;
    let mut dims = [0usize; 3];
    let mut origin = [0.0f32; 3];
    for a in 0..3 {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let n = (ext[a] / step).ceil() as usize + 1;
        dims[a] = n + 2 * PAD;
        #[allow(clippy::cast_precision_loss)]
        {
            origin[a] = lo[a] - step * PAD as f32;
        }
    }
    let total = dims[0] * dims[1] * dims[2];
    let (mut xs, mut ys, mut zs) = (
        Vec::with_capacity(total),
        Vec::with_capacity(total),
        Vec::with_capacity(total),
    );
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                #[allow(clippy::cast_precision_loss)]
                {
                    xs.push((i as f32).mul_add(step, origin[0]));
                    ys.push((j as f32).mul_add(step, origin[1]));
                    zs.push((k as f32).mul_add(step, origin[2]));
                }
            }
        }
    }
    let mut melhor = f32::INFINITY;
    let mut values = Vec::new();
    for _ in 0..super::super::QUADROS_MEDIDOS {
        let t0 = std::time::Instant::now();
        let mut h = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
        let mut v = Vec::with_capacity(total);
        const LOTE: usize = 1 << 16;
        for c in (0..total).step_by(LOTE) {
            let f = (c + LOTE).min(total);
            v.extend_from_slice(h.eval(&xs[c..f], &ys[c..f], &zs[c..f]).expect("assar"));
        }
        #[allow(clippy::cast_possible_truncation)]
        let ms = t0.elapsed().as_secs_f32() * 1e3;
        melhor = melhor.min(ms);
        values = v;
    }
    let mut g = Grade {
        dims,
        origin,
        step,
        values,
        raio: 0.0,
    };
    let hi = g.hi();
    let mut r2 = 0.0f32;
    for a in 0..3 {
        let m = origin[a].abs().max(hi[a].abs());
        r2 = m.mul_add(m, r2);
    }
    g.raio = r2.sqrt();
    (g, melhor)
}

/// O documento de UM nó amostrado — o que o registo resolve para a grade.
fn doc_da_grade() -> ph2d_field::FieldDoc {
    ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Sampled {
                key: "grade".to_string(),
            },
            mods: Vec::new(),
            verb: None,
        }],
        ph2d_field::NodeId(0),
    )
    .expect("o nó da grade")
}

/// Passos por raio que acertou, pela porta do produto (`STEP_SAMPLES`).
fn passos_e_buffer(
    doc: &ph2d_field::FieldDoc,
    reg: &Registry,
    cam: &ph2d_field_render::Orbit,
    w: u32,
    h: u32,
) -> (f32, ph2d_field_render::Gbuffer) {
    use std::sync::atomic::Ordering;
    ph2d_field_render::STEP_SAMPLES.store(0, Ordering::Relaxed);
    let g = ph2d_field_render::trace(doc, reg, cam, w, h);
    let acertos = g.hit.iter().filter(|x| **x).count().max(1);
    #[allow(clippy::cast_precision_loss)]
    let pa = ph2d_field_render::STEP_SAMPLES.load(Ordering::Relaxed) as f32 / acertos as f32;
    (pa, g)
}

/// O relógio da placa pelo passe do modo de omissão (o matcap), mínimo de três.
fn placa(
    doc: &ph2d_field::FieldDoc,
    reg: &Registry,
    cam: &ph2d_field_render::Orbit,
    w: u32,
    h: u32,
) -> Option<f32> {
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    crate::gpu_frame::shared().and_then(|t| {
        let mut melhor = f32::INFINITY;
        for _ in 0..super::super::QUADROS_MEDIDOS + 1 {
            let t0 = std::time::Instant::now();
            crate::gpu_frame::pinta_matcap(
                t,
                doc,
                reg,
                cam,
                &ph2d_field_gpu::matcap::MatcapSetup {
                    rgb_linear: &foto,
                    side: lado,
                    chave: 1,
                    stops: olhar.exposure_stops,
                    view: ph2d_view_transform::wgsl::view_code(olhar.view),
                    background: [0, 0, 0, 0],
                },
                w,
                h,
            )?;
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            melhor = melhor.min(ms);
        }
        Some(melhor)
    })
}

fn quantil(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v.sort_by(f32::total_cmp);
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    let i = ((v.len() - 1) as f32 * q).round() as usize;
    v[i]
}

/// ⏱️⭐⭐⭐⭐ **Sonda: a grade assada contra a árvore, nas peças do corpus da `W9`.**
///
/// Colunas: a resolução (`árvore` = o traço de hoje) · a aresta da célula em fracção do lado maior
/// da peça · o passo da marcha (o `safe_march_step` do documento — ⚠️ até 2026-09-24 a coluna
/// imprimia `1/√L²`, que dava `0,841` onde a marcha anda `0,707`) · os passos por acerto · o quadro da placa a
/// `1920×1080` · o assar · e a QUALIDADE contra a árvore — píxeis de silhueta trocados, o desvio
/// do ponto de paragem (`p99`, em células) e o ângulo da normal (`p50`/`p99`), que é o que a luz
/// mostra como faceta.
#[test]
#[ignore = "sonda de relógio; precisa de adaptador e de máquina calma"]
fn diag_a_grade_contra_a_arvore() {
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    const W: u32 = 1920;
    const H: u32 = 1080;
    const QW: u32 = 480;
    const QH: u32 = 270;
    println!(
        "\n  {}\n  peça · res · célula/lado · passo · passos/acerto · placa ms · assar ms · MB · \
         silhueta trocada · desvio p99 (células) · normal p50/p99 °",
        super::super::contexto()
    );
    for (nome, cena) in [("vaso (5)", 5u32), ("nó de toro (28)", 28u32)] {
        let doc = crate::smoke::scene(cena);
        let passo_arvore = ph2d_field_eval::safe_march_step(&doc);
        let (pa, ref_g) = passos_e_buffer(&doc, &reg, &cam, QW, QH);
        let p = placa(&doc, &reg, &cam, W, H);
        println!(
            "  {nome:>16} · árvore · {:>6} · {passo_arvore:>5.3} · {pa:>7.1} · {:>7.2} · {:>7} · \
             {:>6} · — · — · —",
            "—",
            p.unwrap_or(f32::NAN),
            "—",
            "—"
        );
        let doc_g = doc_da_grade();
        let passo_grade = ph2d_field_eval::safe_march_step(&doc_g);
        for res in [32usize, 64, 128] {
            let (grade, assar) = assa(&doc, &reg, res);
            let lado = grade.step * res as f32;
            let celula = grade.step;
            #[allow(clippy::cast_precision_loss)]
            let mb = grade.values.len() as f32 * 4.0 / 1.0e6;
            let mut reg_g = Registry::new();
            reg_g.insert("grade".to_string(), Arc::new(grade) as Arc<dyn Sampled>);
            let (pg, g) = passos_e_buffer(&doc_g, &reg_g, &cam, QW, QH);
            let mut trocados = 0usize;
            let mut desvio = Vec::new();
            let mut angulo = Vec::new();
            for i in 0..ref_g.hit.len() {
                if ref_g.hit[i] != g.hit[i] {
                    trocados += 1;
                } else if ref_g.hit[i] {
                    let (a, b) = (ref_g.point[i], g.point[i]);
                    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2))
                        .sqrt();
                    desvio.push(d / celula);
                    let (na, nb) = (ref_g.normal[i], g.normal[i]);
                    let c = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
                    angulo.push(c.acos().to_degrees());
                }
            }
            let pgpu = placa(&doc_g, &reg_g, &cam, W, H);
            println!(
                "  {nome:>16} · {res:>6} · {:>6.4} · {passo_grade:>5.3} · {pg:>7.1} · {:>7.2} · \
                 {assar:>7.1} · {mb:>6.1} · {trocados} · {:.2} · {:.1}/{:.1}",
                celula / lado,
                pgpu.unwrap_or(f32::NAN),
                quantil(&mut desvio, 0.99),
                quantil(&mut angulo, 0.5),
                quantil(&mut angulo, 0.99),
            );
        }
    }
    println!();
}

#[path = "device_probes_w9_longe.rs"]
mod longe;
