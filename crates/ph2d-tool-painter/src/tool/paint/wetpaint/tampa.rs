//! ⭐ **As PONTAS RECTAS do traço** (J6, lacuna de motor 2 do `docs/Painter/Pinceis/01_CATALOGO.md` §3d;
//! [`ph2d_painter_brush::BrushSpec::pontas_rectas`], [`ph2d_painter_brush::BrushSpec::pontas_alem`]). O pastel e o
//! carvão do Rebelle começam e acabam cortados a direito, perpendiculares ao rumo (uma FITA), e CHEIOS até ao
//! corte; a cadeia de dabs redondos acaba na meia-lua do último e esmaece num raio.
//!
//! A lei: cada dab pousa só a METADE DE TRÁS da sua pegada (o semiplano atrás do centro, pelo rumo dele), e
//! [`COMPENSA`] devolve a soma pela porta do padrão (que multiplica DEPOIS do portão do papel, onde o depósito por
//! célula é linear — `trail/deposit.rs::cell`): no meio cada célula recebe metade dos dabs, cada um a dobrar. O
//! filme (o máximo por célula) não precisa: a união é a mesma. Os dois cortes passam `pontas_alem` raios dos centros
//! das pontas (`0` = no centro; `1` = o *square cap*).
//! - O INÍCIO corta-se pelo semiplano à frente de `c₀ − alem·r₀·rumo₀` (o 1.º dab não tem rumo: não pousa). Cada
//!   célula à frente de `c₀` recebe TODOS os seus dabs (os que vêm depois); entre o corte e `c₀` faltam os dabs que
//!   haveria antes de `c₀` — a ABERTURA ([`PainterTool::abre_as_pontas_rectas`]) pousa-os logo que a lane conhece o
//!   rumo, ao passo dela, com a pegada, o grão e o peso de um dab do traço.
//! - O FIM: a metade de trás acaba a direito no último centro a cada instante do gesto, mas as células atrás dele
//!   ficam sem os dabs que ainda viriam. Ao soltar, o FECHO ([`PainterTool::fecha_as_pontas_rectas`]) continua o
//!   traço a direito além do fim, cortado pelo plano `c_N + alem·r_N·rumo_N`, e a janela do rasto que sobra POUSA
//!   (`Engine::land_direct_remainders`) em vez de cair — a fita cortada, cheia, ao bit da conta da soma.
//!
//! O corte é anti-serrilhado por área (a rampa de 1 px de um semiplano), simétrica: a soma ao longo do traço é a
//! do degrau. Gate: `tampa_tests`.

use ph2d_painter_brush::{BrushSpec, Dab};

use super::session::Lane;
use crate::tool::PainterTool;

/// O depósito de cada célula × isto: metade dos dabs a cobre.
pub(super) const COMPENSA: f64 = 2.0;

/// Até onde (em raios do dab, pelo caminho, além do `alem`) a tampa do início ainda corta: um dab só chega atrás
/// do corte enquanto o caminho desde `c₀` é menor que o raio; a folga cobre a curva do arranque. Mais longe, um
/// traço que volta ao ponto de partida por trás não pode ser cortado.
const ALCANCE_DO_INICIO: f32 = 1.5;

/// Os dabs virtuais vão até `raio × (alem + isto)` além da ponta (os mais longe não chegam atrás do corte).
const ALCANCE_DO_FIM: f32 = 1.05;

/// O passo mínimo dos dabs virtuais (px): um traço parado não pode pedir dabs sem fim.
const PASSO_MINIMO: f32 = 0.5;

/// De que lote é o dab: da mão, da abertura (antes do 1.º centro) ou do fecho (além do último).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Lote {
    Mao,
    Abertura,
    Fecho,
}

/// O corte de UM dab ([`Self::do_dab`]).
#[derive(Clone, Copy, Debug)]
pub(super) struct Tampa {
    centro: [f32; 2],
    rumo: [f32; 2],
    /// O semiplano do início (a origem do corte e o rumo), só perto do início.
    inicio: Option<([f32; 2], [f32; 2])>,
    /// O semiplano do fim (só no fecho).
    fim: Option<([f32; 2], [f32; 2])>,
}

fn unitario(v: [f32; 2]) -> Option<[f32; 2]> {
    let n = v[0].hypot(v[1]);
    (n > 1e-6).then(|| [v[0] / n, v[1] / n])
}

fn anda(c: [f32; 2], r: [f32; 2], a: f32) -> [f32; 2] {
    [c[0] + r[0] * a, c[1] + r[1] * a]
}

/// A cobertura do semiplano `{(p − o)·n ≤ 0}` no píxel de centro `p` (a rampa de 1 px).
#[inline]
fn atras(p: [f32; 2], o: [f32; 2], n: [f32; 2]) -> f32 {
    let u = (p[0] - o[0]) * n[0] + (p[1] - o[1]) * n[1];
    (0.5 - u).clamp(0.0, 1.0)
}

impl Tampa {
    /// O corte do dab `d` na lane `lane`, ou `None` — o dab sem rumo, que não pousa. No lote da mão a lane aprende
    /// o 1.º rumo e o último dab; no fecho o corte leva o plano do fim. `alem` = [`BrushSpec::pontas_alem`].
    pub(super) fn do_dab(d: &Dab, lane: &mut Lane, lote: Lote, alem: f32) -> Option<Self> {
        let rumo = unitario(d.dir)?;
        let fim = match lote {
            Lote::Fecho => {
                let u = lane.ultimo?;
                let n = unitario(u.dir).unwrap_or(rumo);
                Some((anda(u.center, n, alem * u.radius_px), n))
            }
            Lote::Abertura => None,
            Lote::Mao => {
                if lane.rumo0 == [0.0; 2] {
                    lane.rumo0 = rumo;
                }
                lane.passo = d.arc_len - lane.arco;
                lane.ultimo = Some(*d);
                None
            }
        };
        let perto = lote == Lote::Abertura
            || d.arc_len - lane.arco0
                < (ALCANCE_DO_INICIO + alem) * d.radius_px.max(d.stroke_radius_px);
        let n0 = lane.rumo0;
        Some(Self {
            centro: d.center,
            rumo,
            inicio: perto.then(|| (anda(lane.inicio, n0, -alem * lane.raio0), n0)),
            fim,
        })
    }

    /// A fracção `0..1` da pegada que pousa no píxel `(x, y)` (centro do sub-ponto, px de tela).
    #[inline]
    pub(super) fn cobre(&self, x: f32, y: f32) -> f32 {
        let p = [x, y];
        let mut c = atras(p, self.centro, self.rumo);
        if let Some((o, n)) = self.inicio {
            c *= atras(p, o, [-n[0], -n[1]]);
        }
        if let Some((o, n)) = self.fim {
            c *= atras(p, o, n);
        }
        c
    }
}

/// O passo de caminho de um dab virtual da lane (o do último dab da mão; nunca abaixo de [`PASSO_MINIMO`]).
pub(super) fn passo_virtual(lane: &Lane) -> f32 {
    lane.passo.max(PASSO_MINIMO)
}

/// Os dabs virtuais de um lote, por ponto do caminho e as lanes pela ordem (como as cópias de um ponto chegam):
/// cada `(lane, dab-modelo, rumo, passo, quantos)` anda `sentido·passo·(k + 1)` a partir do modelo. A abertura
/// (`sentido < 0`) sai do mais longe para o 1.º centro — anda para a frente, como um traço. Devolve os dabs e a lane
/// de cada um.
fn virtuais(vivas: &[(usize, Dab, [f32; 2], f32, usize)], sentido: f32) -> (Vec<Dab>, Vec<usize>) {
    let (mut dabs, mut de) = (Vec::new(), Vec::new());
    let n = vivas.iter().map(|v| v.4).max().unwrap_or(0);
    let ordem: Vec<usize> = if sentido < 0.0 {
        (0..n).rev().collect()
    } else {
        (0..n).collect()
    };
    for k in ordem {
        for &(i, u, r, passo, nk) in vivas {
            if k >= nk {
                continue;
            }
            let a = sentido * passo * (k + 1) as f32;
            let mut d = u;
            d.center = anda(u.center, r, a);
            d.dir = r;
            d.arc_len = u.arc_len + a;
            // Um dab por ponto: os virtuais já andam ao passo do traço.
            d.step = [0.0, 0.0];
            d.ante = None;
            dabs.push(d);
            de.push(i);
        }
    }
    (dabs, de)
}

fn quantos(raio: f32, alem: f32, passo: f32) -> usize {
    ((alem + ALCANCE_DO_FIM) * raio / passo).ceil() as usize
}

impl PainterTool {
    /// Pousa um lote virtual: cada dab na lane que `de` lhe dá ([`Lote::Abertura`] ou [`Lote::Fecho`]).
    fn pousa_virtuais(&mut self, dabs: &[Dab], de: Vec<usize>, abrindo: bool, brush: &BrushSpec) {
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // Pousa JÁ (a fila de adiados perderia a lane de cada dab).
        sess.bring_home();
        sess.fecho = de;
        sess.abrindo = abrindo;
        let grupos = std::mem::take(&mut self.paint.dab_groups);
        self.stamp_dabs_wetpaint(dabs, brush);
        self.paint.dab_groups = grupos;
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.fecho.clear();
            sess.abrindo = false;
        }
    }

    /// O `pontas_alem` em raios do MOTOR: o do painel é em meias-larguras PROMETIDAS (o número do painel), e o Wet
    /// Paint pinta com o raio do motor cuja largura medida é a prometida (`largura.rs`; o redondo de 40 px: 66,9).
    pub(super) fn alem_no_motor(&self, brush: &BrushSpec) -> f32 {
        let razao = (self.paint.brush.radius_px / brush.radius_px.max(1e-3)).min(1.0);
        brush.pontas_alem * razao
    }

    /// ⭐ **A ABERTURA das pontas rectas** — logo que uma lane conhece o 1.º rumo: os dabs que haveria antes do 1.º
    /// centro, até um raio além do corte do início. `alem = 0` não precisa deles (cada célula à frente de `c₀` tem
    /// os seus dabs): só marca a lane.
    pub(in crate::tool::paint) fn abre_as_pontas_rectas(&mut self, brush: &BrushSpec) {
        let alem = self.alem_no_motor(brush);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let mut vivas = Vec::new();
        for (i, l) in sess.lanes.iter_mut().enumerate() {
            if l.aberta || l.rumo0 == [0.0; 2] {
                continue;
            }
            l.aberta = true;
            let Some(u) = l.ultimo.filter(|_| alem > 0.0) else {
                continue;
            };
            let passo = passo_virtual(l);
            let mut modelo = u;
            modelo.center = l.inicio;
            modelo.arc_len = l.arco0;
            modelo.radius_px = l.raio0;
            // O 1.º dab (sem rumo) entra também: o modelo anda `-passo` antes do 1.º virtual.
            modelo.center = anda(modelo.center, l.rumo0, passo);
            modelo.arc_len += passo;
            vivas.push((i, modelo, l.rumo0, passo, 1 + quantos(l.raio0, alem, passo)));
        }
        if vivas.is_empty() {
            return;
        }
        // A lane volta onde estava: o dab seguinte da mão anda desde o SEU anterior (o peso, a janela do rasto).
        let onde: Vec<(usize, [f32; 2], f32)> =
            vivas.iter().map(|v| (v.0, sess.lanes[v.0].pos, sess.lanes[v.0].arco)).collect();
        let (dabs, de) = virtuais(&vivas, -1.0);
        self.pousa_virtuais(&dabs, de, true, brush);
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            for (i, pos, arco) in onde {
                sess.lanes[i].pos = pos;
                sess.lanes[i].arco = arco;
            }
        }
    }

    /// ⭐ **O FECHO das pontas rectas** — ao soltar a caneta, com o traço ainda vivo: os dabs que continuariam o
    /// traço além do fim, cortados pelo plano do fim ([`Tampa::cobre`]), e a janela do rasto que sobra pousa. Sem a
    /// caixa, ou sem uma lane que tenha andado, não faz nada.
    pub(in crate::tool::paint) fn fecha_as_pontas_rectas(&mut self) {
        if self.paint.eraser || !self.paint.wetpaint.live_gesture {
            return;
        }
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let Some(brush) = sess.pincel_do_fecho.take() else {
            return;
        };
        if !sess.stroke_open || !brush.pontas_rectas {
            return;
        }
        let alem = self.alem_no_motor(&brush);
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        let vivas: Vec<_> = sess
            .lanes
            .iter()
            .enumerate()
            .filter_map(|(i, l)| {
                let u = l.ultimo?;
                let r = unitario(u.dir)?;
                let passo = passo_virtual(l);
                let raio = u.radius_px.max(u.stroke_radius_px);
                Some((i, u, r, passo, quantos(raio, alem, passo)))
            })
            .collect();
        if vivas.is_empty() {
            return;
        }
        let (dabs, de) = virtuais(&vivas, 1.0);
        let lanes: Vec<usize> = vivas.iter().map(|v| v.0).collect();
        self.pousa_virtuais(&dabs, de, false, &brush);
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            // A janela que sobra pousa: sem isto o último pedaço do traço caía (`end_direct_stroke`) e esmaecia.
            sess.bring_home();
            sess.engine.land_direct_remainders(&lanes);
        }
        self.wetpaint_composite();
    }
}
