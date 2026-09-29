//! ⭐⭐⭐ **AS FORMAS NA PLACA** — a rota da CPU do passe instanciado (doc 121 do Motion, W2).
//!
//! Ordem do dono (2026-09-29): *«não só as estrelas mas todas as shapes devem ser o mais
//! otimizadas possível»*, sem perder a nitidez que ele recusou trocar por uma imagem assada
//! (2026-09-20). O passe é o [`ph2d_shape_gpu`]: cada geometria DISTINTA é preparada uma vez e a
//! placa desenha as `N` cópias numa chamada, com a área de cada pixel calculada **como o
//! rasterizador fino do Vello a calcula** (paridade de pixel medida no gate daquela crate).
//!
//! ## A decisão é TUDO-OU-NADA por quadro
//!
//! ⚠️ **A ordem das linhas é o desenho** — uma cópia posterior fica por cima de uma anterior, e
//! uma folha de IMAGEM no meio de uma planta tem de ficar à frente dos galhos que vêm antes dela.
//! Partir a lista entre a placa e o Vello poria as duas metades em camadas diferentes e
//! **trocaria a ordem** entre elas. ⇒ ou o quadro inteiro vai à placa, ou vai inteiro ao Vello,
//! byte a byte o de sempre. O que devolve o quadro ao Vello (doc 121 §2.1):
//!
//! - um quad de IMAGEM (`geometry_id == 0`) — é uma textura, não uma forma;
//! - uma linha com MISTURA própria ou de grupo ([`crate::motion_shape_gen::mistura`]) — pede uma
//!   camada fora do alvo;
//! - uma geometria que a placa não desenha como o Vello
//!   ([`ph2d_vec_render::forma_para_a_placa`] devolve `None`: tinta própria, traço de padrão ou
//!   de pincel);
//! - ⛔ **um TRAÇO sob afim NÃO conforme** — a lei do dono (bug #27: *«quando engrossa, engrossa
//!   por igual nos dois eixos»*) põe a caneta no MUNDO, e o contorno expandido no espaço LOCAL
//!   daria uma caneta elíptica. É a divergência que a W4 resolve; até lá, o Vello.
//!
//! ⚠️ **E a placa desenha numa CAMADA própria de meio-float**, que o presente cola no acumulador
//! do mundo entre o documento e os gizmos: o Vello grava a cor SEPARADA do alfa, logo o passe não
//! pode pintar por cima do alvo dele, e um alvo de 8 bits arredondaria a cor a cada sobreposição
//! (medido na W1: o desvio dobra).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use ph2d_eval_motion::VectorInstance;
use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{
    Copias, FillRule, ShapeGeometry, ShapeInput, ShapeInstance, ShapePass, ShapeView, StrokeInput,
};
use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, Fill};

use crate::motion_shape_gen::VecPathStore;

/// O formato da camada: meio-float, para a cor não arredondar a cada sobreposição.
pub const FORMATO_DA_CAMADA: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// O que uma geometria É para a placa.
enum Entrada {
    /// A placa desenha-a. `traco` = ela tem traço, e então só vai com afim conforme.
    Pronta {
        geometria: Box<ShapeGeometry>,
        traco: bool,
    },
    /// Não desenha nada (sem preenchimento nem traço) — a cópia é saltada, como no Vello.
    Vazia,
    /// A placa não a sabe desenhar como o Vello — o quadro inteiro fica no Vello.
    Recusada,
}

/// Prepara a geometria de uma forma pelos MESMOS passos do desenho de hoje
/// ([`ph2d_vec_render::forma_para_a_placa`]).
fn prepara(path: &VecPath) -> Entrada {
    let Some(f) = ph2d_vec_render::forma_para_a_placa(path) else {
        return Entrada::Recusada;
    };
    let fill = f.fill.as_ref().map(|(bp, regra)| {
        let regra = match regra {
            Fill::NonZero => FillRule::NonZero,
            Fill::EvenOdd => FillRule::EvenOdd,
        };
        (bp, regra)
    });
    let strokes: Vec<StrokeInput<'_>> = f
        .tracos
        .iter()
        .map(|(bp, style)| StrokeInput {
            path: bp,
            style,
            color: f.cor_do_traco,
        })
        .collect();
    let traco = !strokes.is_empty() || !f.preenchimentos_do_traco.is_empty();
    let input = ShapeInput {
        fill,
        strokes,
        stroke_fills: f.preenchimentos_do_traco.iter().collect(),
    };
    match ShapeGeometry::prepare(&input) {
        Some(g) => Entrada::Pronta {
            geometria: Box::new(g),
            traco,
        },
        None => Entrada::Vazia,
    }
}

/// **A pose da cópia é CONFORME?** (rotação + escala uniforme, com ou sem espelho) — a única
/// em que o contorno expandido no espaço local é a caneta redonda que o Vello desenha.
///
/// ⚠️ A folga é RELATIVA à escala: um `f32` que atravessou um `basis` rodado não é exacto, e uma
/// folga absoluta recusaria as cópias pequenas e aceitaria as grandes.
#[must_use]
pub fn conforme(inst: &VectorInstance) -> bool {
    let [b0, b1, b2, b3] = inst.basis;
    let [sx, sy] = inst.size;
    let (a, b, c, d) = (b0 * sx, b1 * sx, b2 * sy, b3 * sy);
    let escala = a.abs() + b.abs() + c.abs() + d.abs();
    let folga = 1e-4 * escala;
    let roda = (a - d).abs() + (b + c).abs() <= folga;
    let espelha = (a + d).abs() + (b - c).abs() <= folga;
    roda || espelha
}

/// A porta de bissecção do produto: `PH2D_FORMAS_NA_PLACA=0` devolve toda forma ao Vello.
///
/// ⚠️ **Lida UMA vez e só aqui**, na porta do produto — a decisão ([`PlacaDeFormas::decide`])
/// recebe-a como argumento, para um gate medir a LEI e não o ambiente.
#[must_use]
pub fn por_ordem() -> bool {
    static LIGADA: OnceLock<bool> = OnceLock::new();
    *LIGADA.get_or_init(|| std::env::var("PH2D_FORMAS_NA_PLACA").map_or(true, |v| v != "0"))
}

/// O lado da placa: o passe e a camada, criados na primeira vez que um quadro vai lá.
struct Gpu {
    passe: ShapePass,
    textura: wgpu::Texture,
    vista: wgpu::TextureView,
    tamanho: (u32, u32),
}

impl Gpu {
    fn camada(gpu: &GpuContext, tamanho: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
        let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: tamanho.0.max(1),
                height: tamanho.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMATO_DA_CAMADA,
            // ⚠️ `COPY_SRC` é o que deixa o gate de paridade LER a camada que o produto desenha.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let v = t.create_view(&wgpu::TextureViewDescriptor::default());
        (t, v)
    }
}

/// ⭐⭐⭐ **As formas do quadro, a caminho da placa** — o cache de geometria (por handle, que o
/// [`VecPathStore`] nunca recicla), as cópias deste quadro e a camada onde elas se desenham.
#[derive(Default)]
pub struct PlacaDeFormas {
    cache: BTreeMap<u32, Entrada>,
    copias: Vec<ShapeInstance>,
    ativa: bool,
    gpu: Option<Gpu>,
    /// A rota do quadro anterior — o diagnóstico fala na MUDANÇA, nunca por quadro.
    anterior: Option<bool>,
    /// O afim mundo→pixel do quadro — o MESMO que a cena Vello recebe.
    cam: Affine,
}

impl PlacaDeFormas {
    /// **Este quadro vai à placa?** Decide TUDO-OU-NADA (ver o cabeçalho) e, se vai, deixa as
    /// cópias prontas. `ligada` = a ferramenta Motion na mão, sem vidro, e a porta por ordem;
    /// `cam` = o afim mundo→pixel que a cena Vello receberia.
    ///
    /// ⚠️ Sem cópia nenhuma a resposta é `false` — o quadro de sempre, byte a byte.
    pub fn decide(
        &mut self,
        ligada: bool,
        insts: &[VectorInstance],
        store: &VecPathStore,
        cam: Affine,
    ) -> bool {
        self.cam = cam;
        self.ativa = ligada && !insts.is_empty() && self.monta(insts, store);
        if !self.ativa {
            self.copias.clear();
        }
        if self.anterior != Some(self.ativa) && !insts.is_empty() {
            eprintln!(
                "[formas] {} copias de {} geometrias pela {}",
                insts.len(),
                self.cache.len(),
                if self.ativa { "PLACA" } else { "cena Vello" }
            );
            self.anterior = Some(self.ativa);
        }
        self.ativa
    }

    /// As cópias, ou `false` ao primeiro motivo para o quadro ficar no Vello.
    fn monta(&mut self, insts: &[VectorInstance], store: &VecPathStore) -> bool {
        self.copias.clear();
        // ⚠️ Um handle que o store largou nunca volta (não é reciclado) — a entrada dele é lixo.
        self.cache.retain(|h, _| store.get(*h).is_some());
        for inst in insts {
            if inst.geometry_id == 0 || crate::motion_shape_gen::mistura::precisa_do_vello(inst) {
                return false;
            }
            let entrada = self.cache.entry(inst.geometry_id).or_insert_with(|| {
                store
                    .get(inst.geometry_id)
                    .map_or(Entrada::Recusada, prepara)
            });
            match entrada {
                Entrada::Recusada => return false,
                Entrada::Vazia => {}
                Entrada::Pronta { traco, .. } => {
                    if *traco && !conforme(inst) {
                        return false;
                    }
                    self.copias.push(ShapeInstance {
                        pos: inst.world_pos,
                        size: inst.size,
                        basis: inst.basis,
                        anchor: inst.anchor,
                        geometry: inst.geometry_id,
                        _pad: 0,
                        tint: inst.tint,
                    });
                }
            }
        }
        true
    }

    /// O quadro vai à placa? (a resposta do último [`Self::decide`]).
    #[must_use]
    pub fn ativa(&self) -> bool {
        self.ativa
    }

    /// As cópias deste quadro, na ordem de desenho — o que a placa recebe.
    #[must_use]
    pub fn copias(&self) -> &[ShapeInstance] {
        &self.copias
    }

    /// **Desenha as formas na camada** e devolve-a, para o presente a colar no acumulador do
    /// mundo, com o afim que o [`Self::decide`] recebeu; `tamanho` é o do alvo.
    /// `None` quando o quadro não vai à placa.
    pub fn desenha(&mut self, gpu: &GpuContext, tamanho: (u32, u32)) -> Option<&wgpu::TextureView> {
        // ⚠️ **A decisão vale UM quadro** e é consumida aqui: um quadro que não a tome (sem o ecrã
        // do herói, que é onde ela corre) volta ao caminho de sempre em vez de colar as cópias velhas.
        if !std::mem::take(&mut self.ativa) {
            return None;
        }
        let g = self.gpu.get_or_insert_with(|| {
            let (textura, vista) = Gpu::camada(gpu, tamanho);
            Gpu {
                passe: ShapePass::new(gpu, FORMATO_DA_CAMADA),
                textura,
                vista,
                tamanho,
            }
        });
        if g.tamanho != tamanho {
            (g.textura, g.vista) = Gpu::camada(gpu, tamanho);
            g.tamanho = tamanho;
        }
        g.passe.set_geometries(
            gpu,
            self.cache.iter().filter_map(|(h, e)| match e {
                Entrada::Pronta { geometria, .. } => Some((*h, &**geometria)),
                _ => None,
            }),
        );
        g.passe.upload_instances(gpu, &self.copias);
        let c = self.cam.as_coeffs();
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        #[expect(clippy::cast_precision_loss, reason = "um alvo cabe num f32")]
        let vista = ShapeView {
            lin: [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32],
            t: [c[4] as f32, c[5] as f32],
            alvo: [tamanho.0.max(1) as f32, tamanho.1.max(1) as f32],
        };
        let count = u32::try_from(self.copias.len()).ok()?;
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        g.passe.draw(
            gpu,
            &mut enc,
            &g.vista,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            vista,
            Copias {
                buffer: g.passe.uploaded()?,
                count,
            },
        );
        gpu.queue.submit([enc.finish()]);
        Some(&g.vista)
    }
}

impl PlacaDeFormas {
    /// A textura da camada — para o gate de paridade a ler de volta.
    #[cfg(test)]
    pub(crate) fn textura_da_camada(&self) -> Option<&wgpu::Texture> {
        self.gpu.as_ref().map(|g| &g.textura)
    }
}

#[cfg(test)]
#[path = "motion_shape_placa_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_shape_placa_gpu_tests.rs"]
mod gpu_tests;
