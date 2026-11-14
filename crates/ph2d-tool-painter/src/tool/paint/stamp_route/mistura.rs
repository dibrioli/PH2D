//! A rota do **Color mixing «Wet blend»** do Digital ([`ph2d_painter_brush::mistura_opaca`], a lei medida do CSP):
//! só com o modo ligado e sem apagar; `Off` nunca chega aqui (a rota de sempre, ao bit).
//!
//! - **Reservatório:** um por cópia da Symmetry (as cópias saem em blocos de `copies()` dabs,
//!   `symmetry::push_symmetric`), para o lado espelhado não levar a cor do outro. As cópias do Tiling
//!   (`dab_groups`) partem todas do MESMO estado e só a primeira o avança — a costura casa.
//! - **Renasce** no pen-down e em cada re-carimbo de figura (`stamp_drag_preview`).
//! - **Amostra a tela do pen-DESTE traço**, sem o undo (que agrupa traços sobrepostos, W16, e vai
//!   deixar de guardar a tela inteira): na tela directa, os ladrilhos [`Ladrilhos`] que o traço vai
//!   escrever, guardados antes da 1.ª escrita (só com a mistura ligada; `Off` não paga nada); no plano
//!   do traço (o dual, L8), a `base` dele, que já é o pré-traço. O re-carimbo de figura repõe a tela e
//!   recomeça os dois.
//! - **Pressão:** a do último evento da caneta (o `Dab` não a leva), para todos os dabs do lote.
//! - **Alpha lock:** a cor nova entra na proporção do alfa de antes e o alfa fica (o `preserve_alpha`).
//!   A proteção e a seleção vêm do invólucro (`stamp_dabs_gated`), como em toda rota.
//! - **A pegada:** o kernel de sempre pinta o dab numa máscara (`pegada`: falloff, Shape, Grain, a cápsula da cauda),
//!   com o MESMO stream `tex_rng` que a rota normal consome por dab; o alfa dela pesa a amostra e a cobertura.
//! - **A tela escrita** vive numa `Sombra` `f32` do traço (a lei acumula centenas de carimbos de α pequeno);
//!   quem repõe a tela por baixo dela (re-carimbo de figura, a cauda) recomeça-a.
//! - A 2.ª ponta (dual) carimba a máscara SEM a mistura (`DualTip::pincel`): partilhava o reservatório.

use super::super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::mistura_opaca::{
    Carimbo, Pegada, ReservatorioOpaco, Sombra, Tela, mistura_opaca_carimbo,
};
use ph2d_painter_brush::{BrushSpec, Dab};
use std::sync::Arc;

mod lote;
mod meia; // a meia definição do véu (Bug #84) — filho por LOC // o lote: as pegadas e a película em ordem — filho por LOC

/// O lado de um ladrilho da tela de antes (texels): 64² × 4 = 16 KiB por ladrilho tocado.
const LADO: u32 = 64;

/// A tela do pen-down, só nos ladrilhos que o traço tocou (o resto ainda É o pen-down na tela viva).
#[derive(Clone, Debug, Default)]
pub(in crate::tool::paint) struct Ladrilhos {
    w: u32,
    h: u32,
    colunas: u32,
    guardados: Vec<Option<Box<[u8]>>>,
}

impl Ladrilhos {
    fn recomeca(&mut self) {
        self.guardados.clear();
    }

    /// Guarda (uma vez por traço) os ladrilhos da caixa `x0..x1 × y0..y1` de `tela`, antes de ela ser escrita.
    fn guarda(&mut self, tela: &[u8], w: u32, h: u32, (x0, y0, x1, y1): (u32, u32, u32, u32)) {
        if self.w != w || self.h != h || self.guardados.is_empty() {
            self.w = w;
            self.h = h;
            self.colunas = w.div_ceil(LADO);
            self.guardados.clear();
            self.guardados
                .resize((self.colunas * h.div_ceil(LADO)) as usize, None);
        }
        for ty in y0 / LADO..=(y1 - 1) / LADO {
            for tx in x0 / LADO..=(x1 - 1) / LADO {
                let slot = &mut self.guardados[(ty * self.colunas + tx) as usize];
                if slot.is_some() {
                    continue;
                }
                let (lx, ly) = (tx * LADO, ty * LADO);
                let (lw, lh) = (LADO.min(w - lx), LADO.min(h - ly));
                let mut b = Vec::with_capacity((LADO * LADO * 4) as usize);
                for y in ly..ly + lh {
                    let i = ((y * w + lx) * 4) as usize;
                    b.extend_from_slice(&tela[i..i + (lw * 4) as usize]);
                }
                *slot = Some(b.into_boxed_slice());
            }
        }
    }

    /// O RGBA do byte `i` da tela do pen-down (o ladrilho tem de estar guardado).
    fn le(&self, i: usize) -> [u8; 4] {
        let p = (i / 4) as u32;
        let (x, y) = (p % self.w, p / self.w);
        let (tx, ty) = (x / LADO, y / LADO);
        let lw = LADO.min(self.w - tx * LADO);
        let t = self.guardados[(ty * self.colunas + tx) as usize]
            .as_deref()
            .expect("ladrilho guardado antes da leitura");
        let j = (((y - ty * LADO) * lw + (x - tx * LADO)) * 4) as usize;
        [t[j], t[j + 1], t[j + 2], t[j + 3]]
    }

    /// Bytes guardados agora (a memória que a mistura paga).
    #[cfg(test)]
    pub(in crate::tool::paint) fn bytes(&self) -> usize {
        self.guardados.iter().flatten().map(|b| b.len()).sum()
    }
}

/// O estado da mistura opaca num traço.
#[derive(Clone, Debug)]
pub(in crate::tool::paint) struct MisturaDoTraco {
    /// Um reservatório por cópia da Symmetry.
    pub res: Vec<ReservatorioOpaco>,
    /// A pressão da caneta do último evento (`1` no rato).
    pub pressao: f32,
    /// A tela do pen-down nos ladrilhos tocados (a tela directa).
    pub antes: Ladrilhos,
    /// A `base` do plano do traço durante um lote no plano (`traco_plano::carimba_no_plano`).
    pub base_do_plano: Option<Arc<Vec<u8>>>,
    /// A máscara RGBA (tamanho da tela, transparente fora do dab) onde o kernel pinta a pegada de cada dab.
    pub pegada: Vec<u8>,
    /// A mesma pegada sem o Grain (só com o Grain ativo).
    pub pegada_lisa: Vec<u8>,
    /// A tela escrita em `f32` durante o traço.
    pub sombra: Sombra,
    /// A película a meia definição em RGBA8 (`meia::`, `MisturaOpaca::meia_definicao`); vazia fora dela.
    pub meia: Vec<u8>,
    /// Os mapas pré-multiplicados de cada camada do Shape de cor (Per-Layer Color) para a pegada de UM carimbo.
    pub camadas: Vec<Vec<u8>>,
    /// Salta a cópia do véu no mesmo centro ([`ReservatorioOpaco::repete_o_centro`]); `false` só na sonda de custo.
    pub salta_a_copia_do_veu: bool,
    /// B07.3 (diagnóstico): texels de tela inteira que a meia definição AMPLIOU (`meia::amplia`), acumulados.
    pub texels_ampliados: u64,
}

impl Default for MisturaDoTraco {
    fn default() -> Self {
        Self {
            res: Vec::new(),
            pressao: 1.0,
            antes: Ladrilhos::default(),
            base_do_plano: None,
            pegada: Vec::new(),
            pegada_lisa: Vec::new(),
            sombra: Sombra::default(),
            meia: Vec::new(),
            camadas: Vec::new(),
            salta_a_copia_do_veu: true,
            texels_ampliados: 0,
        }
    }
}

impl PainterTool {
    /// O pincel começa limpo e a tela de antes é a de agora (pen-down, re-carimbo de figura — que já repôs a tela).
    pub(in crate::tool::paint) fn mistura_opaca_recomeca(&mut self) {
        let paint = &mut self.paint;
        paint.mistura_visivel = None;
        for m in std::iter::once(&mut paint.mistura_opaca).chain(paint.mistura_camada.iter_mut()) {
            m.res.clear();
            m.antes.recomeca();
            m.sombra.recomeca();
            m.meia.clear();
        }
    }

    /// A composição das camadas visíveis de antes do traço, para a amostra (`None` = amostra a própria camada).
    fn mistura_amostra_visivel(
        &mut self,
        brush: &BrushSpec,
        base: Option<&Arc<Vec<u8>>>,
        n4: usize,
    ) -> Option<Arc<Vec<u8>>> {
        use ph2d_painter_brush::mistura_opaca::AmostraDaMistura;
        if brush.mistura_opaca.amostra != AmostraDaMistura::TodasVisiveis || self.layers.len() < 2 {
            return None;
        }
        if let Some(v) = self
            .paint
            .mistura_visivel
            .as_ref()
            .filter(|v| v.len() == n4)
        {
            return Some(Arc::clone(v));
        }
        let (w, h) = self.source_size;
        let ativa = match base {
            Some(b) => Arc::clone(b),
            None => Arc::clone(&self.canvas_rgba),
        };
        let src = crate::tool::internal::ToolPixelSource {
            active_id: self.layers.active().unwrap_or(crate::layers::LayerId(0)),
            active_rgba: &ativa,
            images: &self.images,
        };
        let v = Arc::new(crate::compositor::composite(&self.layers, &src, w, h));
        self.paint.mistura_visivel = Some(Arc::clone(&v));
        Some(v)
    }

    /// A sonda de custo compara com o véu de antes no MESMO processo (`MEDIR_VELOCIDADE.md`).
    #[doc(hidden)]
    pub fn set_mistura_salta_a_copia_do_veu(&mut self, sim: bool) {
        let paint = &mut self.paint;
        for m in std::iter::once(&mut paint.mistura_opaca).chain(paint.mistura_camada.iter_mut()) {
            m.salta_a_copia_do_veu = sim;
        }
    }

    /// B07.3: os texels de tela inteira que o véu a meia definição ampliou até agora (o contador do portão por CONTAGEM).
    #[doc(hidden)]
    #[must_use]
    pub fn mistura_texels_do_veu(&self) -> u64 {
        self.paint.mistura_opaca.texels_ampliados
    }

    /// A sonda de custo compara a meia definição com a inteira no MESMO processo (`MisturaOpaca::meia_definicao`).
    #[doc(hidden)]
    pub fn set_mistura_meia_definicao(&mut self, sim: bool) {
        self.paint.brush.mistura_opaca.meia_definicao = sim;
    }

    /// A pressão do evento que vai gerar o próximo lote.
    pub(in crate::tool::paint) fn mistura_opaca_pressao(&mut self, p: f32) {
        self.paint.mistura_opaca.pressao = if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            1.0
        };
    }
}

/// A PELÍCULA de um carimbo da mistura sobre a tela viva: no plano do traço a tela de antes é a base dele; noutro plano
/// os ladrilhos do dab, guardados ANTES de ele escrever (nenhum é lido depois de escrito).
#[allow(clippy::too_many_arguments)]
fn pelicula(
    buf: &mut [u8],
    sombra: &mut Sombra,
    ladrilhos: &mut Ladrilhos,
    w: u32,
    h: u32,
    alpha_locked: bool,
    base: Option<&[u8]>,
    visivel: Option<&[u8]>,
    spec: &BrushSpec,
    carimbo: Carimbo,
    pegada: Pegada<'_>,
    alvo: &mut ReservatorioOpaco,
    caixa: (u32, u32, u32, u32),
) -> Option<ph2d_painter_brush::DirtyRect> {
    let mut tela = Tela {
        buf,
        width: w,
        height: h,
        sombra,
        trava_alfa: alpha_locked,
        plano: base.is_some(),
    };
    if let Some(b) = base {
        let antes = |i: usize| [b[i], b[i + 1], b[i + 2], b[i + 3]];
        match visivel {
            Some(v) => {
                let amostra = |i: usize| [v[i], v[i + 1], v[i + 2], v[i + 3]];
                mistura_opaca_carimbo(&mut tela, antes, amostra, spec, carimbo, pegada, alvo)
            }
            None => mistura_opaca_carimbo(&mut tela, antes, antes, spec, carimbo, pegada, alvo),
        }
    } else {
        ladrilhos.guarda(tela.buf, w, h, caixa);
        let antes = |i: usize| ladrilhos.le(i);
        match visivel {
            Some(v) => {
                let amostra = |i: usize| [v[i], v[i + 1], v[i + 2], v[i + 3]];
                mistura_opaca_carimbo(&mut tela, antes, amostra, spec, carimbo, pegada, alvo)
            }
            None => mistura_opaca_carimbo(&mut tela, antes, antes, spec, carimbo, pegada, alvo),
        }
    }
}

/// A pegada do dab `d` (só o Grain, ou nada) num buffer da caixa dele: o kernel da rota normal, a branco, força 1,
/// Flow 1.
fn pegada_do_dab(
    spec: &BrushSpec,
    basis: Option<&ph2d_painter_brush::texture::TexDabBasis>,
    d: &Dab,
    w: u32,
    h: u32,
    image: Option<&ph2d_painter_brush::texture::ImageMask>,
    po: Option<ph2d_painter_brush::texture::PoDoPapel<'_>>,
) -> Option<(
    ph2d_painter_brush::PegadaLocal,
    Option<ph2d_painter_brush::DirtyRect>,
    Option<ph2d_painter_brush::DirtyRect>,
)> {
    let branco = BrushSpec {
        color: [1.0; 3],
        flow: 1.0,
        blend: ph2d_painter_brush::BrushBlend::default(),
        ..*spec
    };
    ph2d_painter_brush::pegada_textured_masked_po_com_liso(
        w,
        h,
        d.center,
        &branco,
        1.0,
        basis,
        image,
        spec.dab_rotor(d),
        d.capsula(&branco),
        po,
        // Sem o Grain, o dente e o pó a pegada lisa é a própria máscara: a película lê-a como sem ela.
        basis.is_some() || spec.dente.is_some() || po.is_some(),
    )
}

/// A pegada de um dab com o SHAPE (e o Grain, se houver) num buffer próprio — o que a rota em série deixa nas máscaras
/// da tela: a máscara (o kernel a branco com o Shape, o Grain e o pó) e a lisa SEM eles (a queda, sem dente nem pó),
/// as duas na UNIÃO das caixas (missão 4K onda 3). `Some((local, Some(união), lisa))` só se a máscara pinta.
#[allow(clippy::too_many_arguments)]
fn pegada_com_forma(
    spec: &BrushSpec,
    basis: Option<&ph2d_painter_brush::texture::TexDabBasis>,
    forma: ph2d_painter_brush::ShapeInput<'_>,
    d: &Dab,
    w: u32,
    h: u32,
    image: Option<&ph2d_painter_brush::texture::ImageMask>,
    po: Option<ph2d_painter_brush::texture::PoDoPapel<'_>>,
) -> Option<(
    ph2d_painter_brush::PegadaLocal,
    Option<ph2d_painter_brush::DirtyRect>,
    Option<ph2d_painter_brush::DirtyRect>,
)> {
    use ph2d_painter_brush::DirtyRect;
    let branco = BrushSpec {
        color: [1.0; 3],
        flow: 1.0,
        blend: ph2d_painter_brush::BrushBlend::default(),
        ..*spec
    };
    let (m, r) = ph2d_painter_brush::pegada_local(
        w,
        h,
        d.center,
        &branco,
        1.0,
        basis,
        image,
        Some(forma),
        spec.dab_rotor(d),
        d.capsula(&branco),
        po,
    )?;
    let r = r?;
    let plano = BrushSpec {
        texture: Default::default(),
        dente: None,
        ..branco
    };
    let lisa = ph2d_painter_brush::pegada_local(
        w,
        h,
        d.center,
        &plano,
        1.0,
        None,
        None,
        None,
        spec.dab_rotor(d),
        d.capsula(&branco),
        None,
    );
    let r2 = lisa.as_ref().and_then(|l| l.1);
    let u = r2.map_or(r, |r2| {
        let (x0, y0) = (r.x.min(r2.x), r.y.min(r2.y));
        DirtyRect {
            x: x0,
            y: y0,
            w: (r.x + r.w).max(r2.x + r2.w) - x0,
            h: (r.y + r.h).max(r2.y + r2.h) - y0,
        }
    });
    // O buffer `v` da caixa `de` na caixa `u` (a zeros fora dela).
    let na_uniao = |v: Vec<u8>, de: DirtyRect| -> Vec<u8> {
        if de == u {
            return v;
        }
        let mut o = vec![0u8; (u.w * u.h * 4) as usize];
        let n = (de.w * 4) as usize;
        for y in 0..de.h {
            let a = ((y * de.w) * 4) as usize;
            let b = (((de.y + y - u.y) * u.w + (de.x - u.x)) * 4) as usize;
            o[b..b + n].copy_from_slice(&v[a..a + n]);
        }
        o
    };
    let mascara = na_uniao(m.mascara, m.caixa);
    let liso = match lisa {
        Some((l, Some(_))) => na_uniao(l.mascara, l.caixa),
        _ => vec![0u8; (u.w * u.h * 4) as usize],
    };
    Some((
        ph2d_painter_brush::PegadaLocal {
            caixa: u,
            mascara,
            liso,
        },
        Some(u),
        r2.map(|_| u),
    ))
}
