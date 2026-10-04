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
//!   ⭐ O traço sob escala não uniforme vai à placa: o shader constrói-o no ecrã a partir do EIXO,
//!   com a caneta redonda `w·√|det|` (W4) e, desde o §9.9 do doc 121, o tracejado cortado pelo
//!   comprimento de arco no ecrã ([`ph2d_shape_gpu`], `eixo`). Um padrão que o eixo não exprime
//!   ([`ph2d_shape_gpu::FLAG_SO_CONFORME`] — o `kurbo_stroke` da casa nunca o faz) é desta lista.
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
    /// A placa desenha-a.
    Pronta { geometria: Box<ShapeGeometry> },
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
    let input = ShapeInput {
        fill,
        strokes,
        stroke_fills: f.preenchimentos_do_traco.iter().collect(),
    };
    match ShapeGeometry::prepare(&input) {
        // ⚠️ Um tracejado que o eixo não exprime só se desenharia conforme — e a placa não o sabe
        // sob escala não uniforme. O `kurbo_stroke` da casa nunca o produz; se um dia produzir, o
        // quadro fica no Vello em vez de desenhar outra coisa.
        Some(g) if g.record.flags & ph2d_shape_gpu::FLAG_SO_CONFORME != 0 => Entrada::Recusada,
        Some(g) => Entrada::Pronta {
            geometria: Box::new(g),
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

/// ⭐⭐⭐ **As geometrias que a placa sabe desenhar** — o cache por handle (que o [`VecPathStore`]
/// nunca recicla), PARTILHADO pelas duas rotas (doc 121 W3): a da CPU pergunta-o cópia a cópia, a
/// do dispositivo pergunta-o ANTES de cozinhar, handle a handle, e o presente liga as prontas ao
/// passe. ⚠️ **Um cache só, e é essa a razão de ele morar no `MotionState`:** duas respostas à
/// pergunta *«esta forma vai à placa?»* divergiriam na primeira forma nova, e a rota do
/// dispositivo não tem volta — uma forma que ela aceitasse e a placa recusasse simplesmente
/// desapareceria (o dispositivo já a calou no buffer das sprites).
#[derive(Default)]
pub struct GeometriasDaPlaca {
    cache: BTreeMap<u32, Entrada>,
}

/// O que uma geometria é para a placa, sem a geometria — a resposta que a ponte lê.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Veredito {
    /// A placa desenha-a.
    Pronta,
    /// Não desenha nada (sem preenchimento nem traço).
    Vazia,
    /// A placa não a sabe desenhar como o Vello.
    Recusada,
    /// O handle não está no store (largado, ou nunca publicado) — não há o que desenhar.
    Ausente,
}

impl GeometriasDaPlaca {
    /// Larga as entradas cujo handle o store largou — ⚠️ um handle largado nunca volta.
    pub fn varre(&mut self, store: &VecPathStore) {
        self.cache.retain(|h, _| store.get(*h).is_some());
    }

    fn entrada(&mut self, handle: u32, store: &VecPathStore) -> Option<&Entrada> {
        let path = store.get(handle)?;
        Some(self.cache.entry(handle).or_insert_with(|| prepara(path)))
    }

    /// **Esta geometria vai à placa?** Prepara-a da primeira vez que alguém pergunta.
    pub fn veredito(&mut self, handle: u32, store: &VecPathStore) -> Veredito {
        match self.entrada(handle, store) {
            None => Veredito::Ausente,
            Some(Entrada::Recusada) => Veredito::Recusada,
            Some(Entrada::Vazia) => Veredito::Vazia,
            Some(Entrada::Pronta { .. }) => Veredito::Pronta,
        }
    }

    /// As geometrias prontas, para o passe.
    fn prontas(&self) -> impl Iterator<Item = (u32, &ShapeGeometry)> {
        self.cache.iter().filter_map(|(h, e)| match e {
            Entrada::Pronta { geometria, .. } => Some((*h, &**geometria)),
            _ => None,
        })
    }

    /// Quantas geometrias estão no cache (o diagnóstico).
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
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

/// A porta de bissecção do CONTORNO CALCULADO (doc 121 §9.5): `PH2D_CONTORNO_CALCULADO=0` devolve o
/// traço sob escala não-uniforme ao caminho pixel a pixel. ⚠️ A mesma imagem pelos dois caminhos
/// (gate `o_contorno_calculado_desenha_o_que_o_eixo_desenha`); a porta existe para medir o RELÓGIO.
#[must_use]
pub fn contorno_por_ordem() -> bool {
    static LIGADO: OnceLock<bool> = OnceLock::new();
    *LIGADO.get_or_init(|| std::env::var("PH2D_CONTORNO_CALCULADO").map_or(true, |v| v != "0"))
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

/// ⭐⭐⭐ **As formas do quadro, a caminho da placa** — as cópias deste quadro (ou a contagem das
/// que o dispositivo escreveu) e a camada onde elas se desenham. As geometrias vivem no
/// [`GeometriasDaPlaca`] do `MotionState`, partilhado com a ponte.
#[derive(Default)]
pub struct PlacaDeFormas {
    copias: Vec<ShapeInstance>,
    ativa: bool,
    /// ⭐ **As cópias vêm do DISPOSITIVO** (doc 121 W3): quantas o cozimento escreveu no buffer
    /// dele — `None` na rota da CPU, onde elas são as [`Self::copias`].
    do_dispositivo: Option<u32>,
    gpu: Option<Gpu>,
    /// A rota do quadro anterior (placa? do dispositivo?) — o diagnóstico fala na MUDANÇA.
    anterior: Option<(bool, bool)>,
    /// O afim mundo→pixel do quadro — o MESMO que a cena Vello recebe.
    cam: Affine,
    /// `true` ⇒ o traço do eixo sai pixel a pixel mesmo com a porta do contorno aberta — para os
    /// gates e a sonda compararem os dois caminhos sem ler o ambiente.
    sem_contorno: bool,
    /// O [`Self::desenha`] DESTE quadro desenhou a camada — o que o [`Self::redesenha_em`] repete.
    desenhou: bool,
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
        geometrias: &mut GeometriasDaPlaca,
        cam: Affine,
    ) -> bool {
        self.cam = cam;
        self.do_dispositivo = None;
        self.ativa = ligada && !insts.is_empty() && self.monta(insts, store, geometrias);
        if !self.ativa {
            self.copias.clear();
        }
        self.diz(insts.len(), geometrias.len());
        self.ativa
    }

    /// ⭐⭐⭐ **As cópias vêm do DISPOSITIVO** (doc 121 W3) — o cozimento escreveu `n` cópias no
    /// buffer dele, e a ponte já perguntou handle a handle se a placa as desenha
    /// ([`GeometriasDaPlaca::veredito`], a MESMA porta). Aqui só se garante que cada geometria viva
    /// está preparada para o passe.
    ///
    /// ⚠️ **`ligada` NÃO pergunta pelo vidro**, e é de propósito: o dispositivo já CALOU as formas
    /// no buffer das sprites, logo com a placa desligada elas não se desenhariam em sítio nenhum.
    /// Com o vidro subido, as formas desenham-se por baixo dele, como o resto do mundo.
    pub fn decide_do_dispositivo(
        &mut self,
        ligada: bool,
        n: u32,
        vivas: &[u32],
        store: &VecPathStore,
        geometrias: &mut GeometriasDaPlaca,
        cam: Affine,
    ) -> bool {
        self.cam = cam;
        self.copias.clear();
        for &h in vivas {
            let _ = geometrias.veredito(h, store);
        }
        self.ativa = ligada && n > 0;
        self.do_dispositivo = self.ativa.then_some(n);
        self.diz(n as usize, geometrias.len());
        self.ativa
    }

    /// O diagnóstico fala na MUDANÇA de rota, nunca por quadro.
    ///
    /// ⚠️ Todo o texto mora DENTRO do formato do `eprintln!` — é terminal, e o censo do HR-15 só o
    /// reconhece como tal ali (um literal passado por argumento lê-se como texto de ecrã).
    fn diz(&mut self, n: usize, geometrias: usize) {
        let rota = (self.ativa, self.do_dispositivo.is_some());
        if n > 0 && self.anterior != Some(rota) {
            match rota {
                (true, true) => eprintln!(
                    "[formas] {n} copias de {geometrias} geometrias pela PLACA (do dispositivo)"
                ),
                (true, false) => {
                    eprintln!("[formas] {n} copias de {geometrias} geometrias pela PLACA");
                }
                (false, _) => {
                    eprintln!("[formas] {n} copias de {geometrias} geometrias pela cena Vello");
                }
            }
            self.anterior = Some(rota);
        }
    }

    /// As cópias, ou `false` ao primeiro motivo para o quadro ficar no Vello.
    fn monta(
        &mut self,
        insts: &[VectorInstance],
        store: &VecPathStore,
        geometrias: &mut GeometriasDaPlaca,
    ) -> bool {
        self.copias.clear();
        // ⚠️ Um handle que o store largou nunca volta (não é reciclado) — a entrada dele é lixo.
        geometrias.varre(store);
        for inst in insts {
            if inst.geometry_id == 0 || crate::motion_shape_gen::mistura::precisa_do_vello(inst) {
                return false;
            }
            match geometrias.veredito(inst.geometry_id, store) {
                // ⚠️ Na rota da CPU um handle ausente devolve o quadro ao Vello, como sempre fez.
                Veredito::Recusada | Veredito::Ausente => return false,
                Veredito::Vazia => {}
                Veredito::Pronta => {
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
    ///
    /// ⭐ Na rota do dispositivo (doc 121 W3) as cópias são o `buffer` do cozimento, passado em
    /// `dispositivo` — nunca lidas de volta.
    pub fn desenha(
        &mut self,
        gpu: &GpuContext,
        tamanho: (u32, u32),
        geometrias: &GeometriasDaPlaca,
        dispositivo: Option<&wgpu::Buffer>,
    ) -> Option<&wgpu::TextureView> {
        // ⚠️ **A decisão vale UM quadro** e é consumida aqui: um quadro que não a tome (sem o ecrã
        // do herói, que é onde ela corre) volta ao caminho de sempre em vez de colar as cópias velhas.
        self.desenhou = false;
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
        g.passe.set_geometries(gpu, geometrias.prontas());
        g.passe
            .com_contorno(contorno_por_ordem() && !self.sem_contorno);
        // ⚠️ A contagem do dispositivo sem o buffer dele é um quadro sem cópias: o presente só
        // passa o buffer com o cozimento vivo, e colar cópias velhas seria pior que não desenhar.
        // ⚠️ Um clone do handle (o `wgpu::Buffer` é contado por referência): o `draw` muta o passe
        // (o contorno, doc 121 §9.5) e não pode receber um empréstimo dele próprio.
        let (buffer, count) = match self.do_dispositivo {
            Some(n) => (dispositivo?.clone(), n),
            None => {
                g.passe.upload_instances(gpu, &self.copias);
                (
                    g.passe.uploaded()?.clone(),
                    u32::try_from(self.copias.len()).ok()?,
                )
            }
        };
        let c = self.cam.as_coeffs();
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        #[expect(clippy::cast_precision_loss, reason = "um alvo cabe num f32")]
        let vista = ShapeView {
            lin: [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32],
            t: [c[4] as f32, c[5] as f32],
            alvo: [tamanho.0.max(1) as f32, tamanho.1.max(1) as f32],
        };
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
                buffer: &buffer,
                count,
            },
        );
        gpu.queue.submit([enc.finish()]);
        self.desenhou = true;
        Some(&g.vista)
    }

    /// ⭐ doc 121 §9.14 (c) — **as formas deste quadro outra vez, sobre `alvo`** (o RT `Rgba16Float` do
    /// halo do `fx.glow`, com o tamanho da camada): a silhueta EXACTA e o `tint` HDR, nas duas rotas —
    /// na do dispositivo as cópias nem existem na CPU, e era por isso que o brilho recusava a placa.
    /// `false` ⇒ o [`Self::desenha`] deste quadro não desenhou (ou o alvo é de outro tamanho).
    pub fn redesenha_em(
        &self,
        gpu: &GpuContext,
        alvo: &wgpu::TextureView,
        tamanho: (u32, u32),
    ) -> bool {
        let Some(g) = self
            .gpu
            .as_ref()
            .filter(|g| self.desenhou && g.tamanho == tamanho)
        else {
            return false;
        };
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        g.passe.redesenha(gpu, &mut enc, alvo);
        gpu.queue.submit([enc.finish()]);
        true
    }

    /// O [`Self::desenha`] deste quadro desenhou a camada.
    #[must_use]
    pub fn desenhou(&self) -> bool {
        self.desenhou
    }

    /// A camada que o [`Self::desenha`] deste quadro desenhou — para o presente a colar no mundo
    /// DEPOIS do halo (doc 121 §9.14: o desenho corre antes do brilho, a colagem onde sempre correu).
    #[must_use]
    pub fn camada(&self) -> Option<&wgpu::TextureView> {
        self.gpu
            .as_ref()
            .filter(|_| self.desenhou)
            .map(|g| &g.vista)
    }
}

impl PlacaDeFormas {
    /// Desliga o contorno calculado (doc 121 §9.5) nesta placa — o caminho pixel a pixel.
    #[cfg(test)]
    pub(crate) fn sem_contorno(&mut self) {
        self.sem_contorno = true;
    }

    /// Quantas das cópias do último desenho ganharam o contorno calculado (doc 121 §9.5).
    #[cfg(test)]
    pub(crate) fn copias_com_contorno(&self, gpu: &GpuContext, n: u32) -> (u32, u64) {
        self.gpu
            .as_ref()
            .map_or((0, 0), |g| g.passe.copias_com_contorno(gpu, n))
    }

    /// As células que o último desenho pediu, e a capacidade (doc 121 §9.12).
    #[cfg(test)]
    pub(crate) fn celulas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        self.gpu
            .as_ref()
            .map_or((0, 0), |g| g.passe.celulas_do_ultimo_quadro(gpu))
    }

    /// As arestas do último desenho: reservadas, escritas e do contorno (doc 121 §9.15).
    #[cfg(test)]
    pub(crate) fn arestas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64, u64) {
        self.gpu
            .as_ref()
            .map_or((0, 0, 0), |g| g.passe.arestas_do_ultimo_quadro(gpu))
    }

    /// As células que o último desenho tocou, e as que usou (doc 121 §9.13).
    #[cfg(test)]
    pub(crate) fn celulas_tocadas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        self.gpu
            .as_ref()
            .map_or((0, 0), |g| g.passe.celulas_tocadas_do_ultimo_quadro(gpu))
    }

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
pub(crate) mod gpu_tests;
