//! ⭐ **A ferramenta do arranjo do HUD** (W5.5, ADR-0194; decisão do dono 27/10: setas na tela, nunca lista no
//! Inspector). Puxar de um elemento de um canvas de HUD para um irmão cria «ao lado» no eixo dominante, com a
//! folga de AGORA (juntar não mexe no desenho); um texto que entra ganha «largura do texto» (encolhe antes de
//! invadir). Com a seta escolhida, a ④ diz como ela prende (`ph2d_panel_vector::state_hud`).
//!
//! O que se vê é o RETRATO do passe do arranjo ([`ph2d_ecs::hud_arranjo::Visto`], em `VecState::hud_visto`): o
//! desenho não tem o mundo. O mundo só se escreve no `frame` (o soltar, os botões) — pela porta
//! [`UiArranjo::junta`], que recusa o círculo de lados (o Cassowary aceitá-lo-ia e violaria um `REQUIRED`).

use std::cell::RefCell;

use ph2d_ecs::hud_arranjo::{Eixo, FilhoVisto, Forca, Lado, Relacao, UiArranjo, Visto};
use ph2d_ecs::{Entity, SimWorld, StableId};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state_hud::{self, HudVista, PedidoHud, TipoSeta};
use ph2d_tool_vector::DrawMode;
use ph2d_vector::{Affine, Point};

use crate::state::VecState;

/// O par que uma seta prende, e o eixo dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Par {
    pub canvas: Entity,
    /// Quem segue.
    pub a: StableId,
    pub b: StableId,
    pub eixo: Eixo,
}

/// A seta a ser puxada: de quem, e a ponta no ecrã.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Puxada {
    pub canvas: Entity,
    pub de: StableId,
    pub ponta: [f64; 2],
}

/// Uma seta desenhada: o par, as pontas e o selo (o rect que se pinta é o que se acerta).
#[derive(Clone, Debug, PartialEq)]
pub struct SetaNoEcra {
    pub par: Par,
    pub de: [f64; 2],
    pub para: [f64; 2],
    pub selo: Rect,
    pub tipo: Option<TipoSeta>,
    pub centrado: bool,
}

/// O plano no ECRÃ, feito pelo desenho e lido pelo clique.
#[derive(Clone, Debug, Default)]
pub struct Plano {
    pub cam: Affine,
    pub elementos: Vec<(Entity, StableId, Rect)>,
    pub setas: Vec<SetaNoEcra>,
}

#[derive(Default)]
pub struct HudTool {
    pub puxada: Option<Puxada>,
    pub escolhida: Option<Par>,
    pub recusada: bool,
    /// Os soltares do botão já vistos (`ShapeHandleInput::soltos`).
    soltos: u32,
    pub plano: RefCell<Option<Plano>>,
}

fn rect(lo: [f64; 2], hi: [f64; 2]) -> Rect {
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    Rect::new(
        lo[0].min(hi[0]) as f32,
        lo[1].min(hi[1]) as f32,
        (hi[0] - lo[0]).abs() as f32,
        (hi[1] - lo[1]).abs() as f32,
    )
}

fn dentro(r: &Rect, p: [f64; 2]) -> bool {
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    r.contains(p[0] as f32, p[1] as f32)
}

fn centro(c: ([f64; 2], [f64; 2])) -> [f64; 2] {
    [(c.0[0] + c.1[0]) * 0.5, (c.0[1] + c.1[1]) * 0.5]
}

fn outro(e: Eixo) -> Eixo {
    match e {
        Eixo::X => Eixo::Y,
        Eixo::Y => Eixo::X,
    }
}

impl Plano {
    #[must_use]
    pub fn no_ecra(&self, w: [f64; 2]) -> [f64; 2] {
        let q = self.cam * Point::new(w[0], w[1]);
        [q.x, q.y]
    }

    fn seta_em(&self, p: [f64; 2]) -> Option<Par> {
        self.setas
            .iter()
            .find(|s| dentro(&s.selo, p))
            .map(|s| s.par)
    }

    /// O elemento sob `p` — o de cima (o último desenhado) ganha.
    fn elemento_em(&self, p: [f64; 2]) -> Option<(Entity, StableId)> {
        self.elementos
            .iter()
            .rev()
            .find(|(_, _, r)| dentro(r, p))
            .map(|&(c, id, _)| (c, id))
    }
}

/// O eixo dominante entre duas caixas (o da maior separação relativa ao tamanho).
fn eixo_entre(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> Eixo {
    let (ca, cb) = (centro(a), centro(b));
    let w = (a.1[0] - a.0[0] + b.1[0] - b.0[0]).max(1e-9);
    let h = (a.1[1] - a.0[1] + b.1[1] - b.0[1]).max(1e-9);
    if (ca[0] - cb[0]).abs() / w >= (ca[1] - cb[1]).abs() / h {
        Eixo::X
    } else {
        Eixo::Y
    }
}

/// ⭐ A relação de `tipo` entre `a` (segue) e `b` no `eixo`, com a folga de AGORA em unidades locais do canvas.
/// «Ao lado» guarda a folga (nada se mexe ao juntar); «não invade» deixa encostar (folga `≤ 0`).
#[must_use]
pub fn relacao(
    v: &Visto,
    a: &FilhoVisto,
    b: &FilhoVisto,
    eixo: Eixo,
    tipo: TipoSeta,
) -> Relacao<StableId> {
    let (k, s) = match eixo {
        Eixo::X => (0, v.escala[0]),
        Eixo::Y => (1, v.escala[1]),
    };
    let depois = centro(a.caixa)[k] > centro(b.caixa)[k];
    // Gravada já na grelha do solver: juntar a seta não mexe em nada.
    let folga = ph2d_ecs::hud_arranjo::na_grelha(
        if depois {
            a.caixa.0[k] - b.caixa.1[k]
        } else {
            b.caixa.0[k] - a.caixa.1[k]
        } / s,
    );
    let (a, b) = (a.id, b.id);
    let lado = |depois_x, antes_x, depois_y, antes_y| match (eixo, depois) {
        (Eixo::X, true) => depois_x,
        (Eixo::X, false) => antes_x,
        (Eixo::Y, true) => depois_y,
        (Eixo::Y, false) => antes_y,
    };
    match tipo {
        TipoSeta::AoLado => Relacao::AoLado {
            a,
            b,
            lado: lado(Lado::Direita, Lado::Esquerda, Lado::Acima, Lado::Abaixo),
            folga,
        },
        TipoSeta::NaoInvade => Relacao::NaoInvade {
            a,
            b,
            lado: lado(Lado::Esquerda, Lado::Direita, Lado::Abaixo, Lado::Acima),
            folga: folga.min(0.0),
        },
    }
}

/// O tipo de uma relação de posição (`None` para centrado/largura).
#[must_use]
pub fn tipo_de(r: &Relacao<StableId>) -> Option<TipoSeta> {
    match r {
        Relacao::AoLado { .. } => Some(TipoSeta::AoLado),
        Relacao::NaoInvade { .. } => Some(TipoSeta::NaoInvade),
        _ => None,
    }
}

fn do_par(r: &Relacao<StableId>, a: StableId, b: StableId) -> bool {
    let (x, y) = r.ids();
    y.is_some_and(|y| (x, y) == (a, b) || (x, y) == (b, a))
}

/// As setas de um retrato: uma por par e eixo de posição; um centrado sem posição no eixo cruzado também é seta.
#[must_use]
pub fn setas_de(v: &Visto) -> Vec<(Par, Option<TipoSeta>, bool)> {
    let mut out: Vec<(Par, Option<TipoSeta>, bool)> = Vec::new();
    for (r, _) in &v.lacos {
        let (a, Some(b)) = r.ids() else { continue };
        let centrado_cruzado = |e: Eixo| {
            v.lacos.iter().any(|(c, _)| {
                matches!(c, Relacao::Centrado { eixo, .. } if *eixo == outro(e)) && do_par(c, a, b)
            })
        };
        let par = match r {
            Relacao::Centrado { eixo, .. } => {
                let e = outro(*eixo);
                let tem_posicao = v
                    .lacos
                    .iter()
                    .any(|(p, _)| tipo_de(p).is_some() && p.eixo() == e && do_par(p, a, b));
                if tem_posicao {
                    continue;
                }
                (
                    Par {
                        canvas: v.canvas,
                        a,
                        b,
                        eixo: e,
                    },
                    None,
                    true,
                )
            }
            _ => (
                Par {
                    canvas: v.canvas,
                    a,
                    b,
                    eixo: r.eixo(),
                },
                tipo_de(r),
                centrado_cruzado(r.eixo()),
            ),
        };
        if !out.iter().any(|(p, _, _)| {
            p.canvas == par.0.canvas && p.eixo == par.0.eixo && do_par_ids(p, par.0.a, par.0.b)
        }) {
            out.push(par);
        }
    }
    out
}

fn do_par_ids(p: &Par, a: StableId, b: StableId) -> bool {
    (p.a, p.b) == (a, b) || (p.a, p.b) == (b, a)
}

/// O press com a ferramenta na mão (pela porta `shape_handles_live::on::press`): o selo de uma seta escolhe-a; um
/// elemento começa uma seta; o vazio limpa. Consome SEMPRE — um clique no vazio não pode cair na cadeia e desenhar.
pub fn press(vec: &mut VecState, w: [f64; 2]) -> bool {
    let t = &mut vec.hud_tool;
    let (p, seta, elemento) = {
        let plano = t.plano.borrow();
        let Some(pl) = plano.as_ref() else {
            return true;
        };
        let p = pl.no_ecra(w);
        (p, pl.seta_em(p), pl.elemento_em(p))
    };
    t.recusada = false;
    t.puxada = None;
    t.escolhida = seta;
    if seta.is_none()
        && let Some((canvas, de)) = elemento
    {
        t.puxada = Some(Puxada {
            canvas,
            de,
            ponta: p,
        });
    }
    true
}

/// O arrasto da seta: a ponta segue o cursor. `false` = não havia seta (a cadeia continua).
pub fn drag(vec: &mut VecState, w: [f64; 2]) -> bool {
    let t = &mut vec.hud_tool;
    let Some(pux) = t.puxada.as_mut() else {
        return false;
    };
    if let Some(pl) = t.plano.borrow().as_ref() {
        pux.ponta = pl.no_ecra(w);
    }
    true
}

/// O Esc: larga a seta puxada e a escolhida. `true` = havia o que largar.
pub fn escape(vec: &mut VecState) -> bool {
    let t = &mut vec.hud_tool;
    let havia = t.puxada.is_some() || t.escolhida.is_some();
    t.puxada = None;
    t.escolhida = None;
    havia
}

/// `f` sobre o arranjo do canvas (posto vazio se ainda não tinha).
fn com_arranjo<R>(
    sim: &mut SimWorld,
    canvas: Entity,
    f: impl FnOnce(&mut UiArranjo) -> R,
) -> Option<R> {
    let mut e = sim.world_mut().get_entity_mut(canvas).ok()?;
    if e.get::<UiArranjo>().is_none() {
        e.insert(UiArranjo::default());
    }
    e.get_mut::<UiArranjo>().map(|mut a| f(&mut a))
}

/// Junta `nova` e, para cada ponta que é texto, a «largura do texto». `false` = recusada (círculo de lados).
fn junta(sim: &mut SimWorld, v: &Visto, nova: Relacao<StableId>, forca: Forca) -> bool {
    com_arranjo(sim, v.canvas, |arr| {
        if !arr.junta(nova, forca) {
            return false;
        }
        let (a, b) = nova.ids();
        for id in [Some(a), b].into_iter().flatten() {
            let texto = v.filhos.iter().any(|f| f.id == id && f.texto);
            let tem = arr
                .lacos
                .iter()
                .any(|(r, _)| *r == Relacao::LarguraDoConteudo { a: id });
            if texto && !tem {
                arr.junta(Relacao::LarguraDoConteudo { a: id }, Forca::Media);
            }
        }
        true
    })
    .unwrap_or(false)
}

/// O soltar de uma seta: sobre um irmão (outro que não a origem) prende os dois «ao lado».
fn solta(t: &mut HudTool, sim: &mut SimWorld, visto: &[Visto], pux: Puxada) {
    let alvo = t
        .plano
        .borrow()
        .as_ref()
        .and_then(|pl| pl.elemento_em(pux.ponta));
    let Some((canvas, para)) = alvo.filter(|&(c, id)| c == pux.canvas && id != pux.de) else {
        return;
    };
    let Some(v) = visto.iter().find(|v| v.canvas == canvas) else {
        return;
    };
    let (Some(a), Some(b)) = (
        v.filhos.iter().find(|f| f.id == pux.de),
        v.filhos.iter().find(|f| f.id == para),
    ) else {
        return;
    };
    let eixo = eixo_entre(a.caixa, b.caixa);
    if junta(
        sim,
        v,
        relacao(v, a, b, eixo, TipoSeta::AoLado),
        Forca::Requerida,
    ) {
        t.escolhida = Some(Par {
            canvas,
            a: a.id,
            b: b.id,
            eixo,
        });
    } else {
        t.recusada = true;
    }
}

/// Um botão da ④ sobre a seta escolhida.
fn aplica(t: &mut HudTool, sim: &mut SimWorld, visto: &[Visto], pedido: PedidoHud) {
    let Some(par) = t.escolhida else {
        return;
    };
    let Some(v) = visto.iter().find(|v| v.canvas == par.canvas) else {
        return;
    };
    let (Some(a), Some(b)) = (
        v.filhos.iter().find(|f| f.id == par.a),
        v.filhos.iter().find(|f| f.id == par.b),
    ) else {
        return;
    };
    match pedido {
        PedidoHud::Tipo(tipo) => {
            t.recusada = !junta(sim, v, relacao(v, a, b, par.eixo, tipo), Forca::Requerida);
        }
        PedidoHud::Centro => {
            let cruzado = outro(par.eixo);
            com_arranjo(sim, par.canvas, |arr| {
                let antes = arr.lacos.len();
                arr.lacos.retain(|(r, _)| {
                    !(matches!(r, Relacao::Centrado { eixo, .. } if *eixo == cruzado)
                        && do_par(r, par.a, par.b))
                });
                if arr.lacos.len() == antes {
                    arr.junta(
                        Relacao::Centrado {
                            a: par.a,
                            b: par.b,
                            eixo: cruzado,
                        },
                        Forca::Forte,
                    );
                }
            });
        }
        PedidoHud::Apaga => {
            com_arranjo(sim, par.canvas, |arr| {
                arr.lacos.retain(|(r, _)| !do_par(r, par.a, par.b))
            });
            t.escolhida = None;
        }
    }
}

/// ⭐ **Um quadro da ferramenta** (pela porta `cad_tool::frame`, que a shell já chama): o soltar da seta, os
/// pedidos da ④, a escolhida que deixou de existir (undo), e a publicação da ④ e da dica.
pub fn frame(sim: &mut SimWorld, vec: &mut VecState) {
    let ativa = vec.draw_config.mode == DrawMode::HudLayout;
    let soltos = vec.shape_handles.soltos();
    let VecState {
        hud_tool: t,
        hud_visto: visto,
        ..
    } = vec;
    let soltou = std::mem::replace(&mut t.soltos, soltos) != soltos;
    let pedidos = state_hud::drena();
    if !ativa {
        t.puxada = None;
        t.escolhida = None;
        t.recusada = false;
        state_hud::set_hud_vista(HudVista::default());
        return;
    }
    if soltou && let Some(pux) = t.puxada.take() {
        solta(t, sim, visto, pux);
    }
    for p in pedidos {
        aplica(t, sim, visto, p);
    }
    let setas: Vec<_> = visto.iter().flat_map(setas_de).collect();
    if t.escolhida
        .is_some_and(|e| !setas.iter().any(|(p, _, _)| *p == e))
        && !soltou
    {
        t.escolhida = None;
    }
    let escolhida = t
        .escolhida
        .and_then(|e| setas.iter().find(|(p, _, _)| *p == e));
    let ancora = t.escolhida.and_then(|e| {
        t.plano
            .borrow()
            .as_ref()?
            .setas
            .iter()
            .find(|s| s.par == e)
            .map(|s| s.selo)
    });
    state_hud::set_hud_vista(HudVista {
        escolhida: escolhida
            .map(|&(_, tipo, centrado)| (tipo.unwrap_or(TipoSeta::AoLado), centrado)),
        recusada: t.recusada,
        ancora,
    });
}

/// O plano no ecrã de um retrato (puro: o desenho chama-o, os gates também).
#[must_use]
pub fn plano(visto: &[Visto], cam: Affine, selo_px: f64) -> Plano {
    let mut p = Plano {
        cam,
        ..Plano::default()
    };
    for v in visto {
        for f in &v.filhos {
            p.elementos.push((
                v.canvas,
                f.id,
                rect(p.no_ecra(f.caixa.0), p.no_ecra(f.caixa.1)),
            ));
        }
        for (par, tipo, centrado) in setas_de(v) {
            let caixa = |id| v.filhos.iter().find(|f| f.id == id).map(|f| f.caixa);
            let (Some(a), Some(b)) = (caixa(par.a), caixa(par.b)) else {
                continue;
            };
            let (ca, cb) = (p.no_ecra(centro(a)), p.no_ecra(centro(b)));
            // As pontas nas bordas que se olham, no eixo da seta; no eixo cruzado, a meio da sobreposição.
            let k = usize::from(par.eixo == Eixo::Y);
            let (ea, eb) = (
                (p.no_ecra(a.0), p.no_ecra(a.1)),
                (p.no_ecra(b.0), p.no_ecra(b.1)),
            );
            let borda = |e: ([f64; 2], [f64; 2]), para_mais: bool| {
                let (lo, hi) = (e.0[k].min(e.1[k]), e.0[k].max(e.1[k]));
                if para_mais { hi } else { lo }
            };
            let mais = ca[k] < cb[k];
            let mut de = ca;
            let mut para = cb;
            de[k] = borda(ea, mais);
            para[k] = borda(eb, !mais);
            let meio = [(de[0] + para[0]) * 0.5, (de[1] + para[1]) * 0.5];
            let selo = rect(
                [meio[0] - selo_px, meio[1] - selo_px],
                [meio[0] + selo_px, meio[1] + selo_px],
            );
            p.setas.push(SetaNoEcra {
                par,
                de,
                para,
                selo,
                tipo,
                centrado,
            });
        }
    }
    p
}

/// ⭐ **O desenho das setas** (pela porta `cad_tool_plano::draw`, só com a ferramenta na mão): faz o plano, guarda-o
/// para o press e o pinta.
pub fn draw(
    vec: &VecState,
    cam: Affine,
    hero: &ph2d_editor_core::HeroScreen,
    target: &mut ph2d_vector::VectorScene,
) {
    let t = &vec.hud_tool;
    if vec.draw_config.mode != DrawMode::HudLayout {
        *t.plano.borrow_mut() = None;
        return;
    }
    let p = plano(&vec.hud_visto, cam, crate::hud_tool_pinta::SELO_PX);
    let hover = hero
        .store
        .last_pointer()
        .map(|q| [f64::from(q[0]), f64::from(q[1])]);
    crate::hud_tool_pinta::pinta(&p, t, hover, hero.theme, target);
    *t.plano.borrow_mut() = Some(p);
}

#[cfg(test)]
#[path = "hud_tool_tests.rs"]
mod tests;
