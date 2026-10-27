//! **Os gizmos das regras, em px de ECRÃ** (W4.6, §3.7): a cor de cada lado das formas com regras (livre / definido
//! / conflito), o SELO de cada regra junto do que ela liga (só com a tool na mão), a COTA de cada medida (sempre:
//! linhas de extensão, setas, o número) e, com a tool na mão, os alvos que se podem escolher. Os rects que se pintam
//! são os rects que se acertam ([`Plano::acerta`]) — uma lista, duas leituras.

use std::collections::BTreeMap;

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::readout::{CHIP_H_PX, chip_rect, chip_width};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state_cad::Possivel;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing};
use ph2d_vec_constraints::{Estado, EstadoRegra, Liberdade};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraId, SegCad, circulo_de};
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState, VecXforms, Xform};
use ph2d_vector::{Affine, Point};

use crate::cad_tool::{Alvo, CadTool, Unidades};

/// O raio de acerto de um ponto, de um lado e da borda de um círculo — o das alças (a mão já o conhece).
const HIT_PX: f64 = ph2d_editor_core::HANDLE_SIZE_PX as f64;

/// O que o ponteiro acerta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acerto {
    Numero(RegraId),
    Selo(RegraId),
    Alvo(Alvo),
}

/// Um selo: o glifo da regra (o MESMO do botão da ④) e, num `Igual`, o número do par.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Selo {
    pub regra: RegraId,
    pub icon: IconId,
    pub par: Option<u32>,
    pub rect: Rect,
    pub cor: ColorToken,
}

/// Uma cota: os traços (extensão, linha da cota, arco), as pontas das setas e o número.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cota {
    pub regra: RegraId,
    pub tracos: Vec<Vec<[f64; 2]>>,
    pub setas: Vec<([f64; 2], [f64; 2])>,
    pub texto: String,
    pub rect: Rect,
    pub cor: ColorToken,
}

/// ⭐ O plano de um quadro.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Plano {
    /// O afim mundo → ecrã com que foi feito (o press chega em mundo).
    pub cam: Affine,
    /// Cada lado de uma forma com regras, na cor do que ela é.
    pub lados: Vec<([f64; 2], [f64; 2], ColorToken)>,
    pub selos: Vec<Selo>,
    pub cotas: Vec<Cota>,
    /// Os alvos escolhíveis (só com a tool na mão): pontos, lados e círculos.
    pub pontos: Vec<(PontoCad, [f64; 2])>,
    pub segmentos: Vec<(SegCad, [f64; 2], [f64; 2])>,
    pub circulos: Vec<(VecPathId, [f64; 2], f64)>,
    /// O que cada regra liga, no ecrã (o hover e a escolhida acendem-no).
    pub ligados: BTreeMap<RegraId, Vec<[f64; 2]>>,
}

fn tela(
    scene: &VecScene,
    view: &VecViewState,
    xf: &VecXforms,
    cam: Affine,
    p: PontoCad,
) -> Option<[f64; 2]> {
    let path = scene.path(p.path()).filter(|_| !view.is_hidden(p.path()))?;
    let local = match p {
        PontoCad::Vert(_, v) => path.vert_by_id(v)?.anchor,
        PontoCad::Centro(_) => circulo_de(path, Xform::IDENTITY)?.0,
    };
    let q = ph2d_vec_render::overlay_transform(view, xf, p.path(), cam)
        * Point::new(local[0], local[1]);
    Some([q.x, q.y])
}

fn add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn esc(a: [f64; 2], k: f64) -> [f64; 2] {
    [a[0] * k, a[1] * k]
}
fn unit(a: [f64; 2]) -> [f64; 2] {
    let n = a[0].hypot(a[1]);
    if n > 0.0 { esc(a, 1.0 / n) } else { [1.0, 0.0] }
}
fn meio(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    esc(add(a, b), 0.5)
}
/// A distância de `p` ao segmento `a..b`.
fn ao_segmento(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = sub(b, a);
    let l2 = d[0] * d[0] + d[1] * d[1];
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = add(a, esc(d, t));
    (p[0] - q[0]).hypot(p[1] - q[1])
}
fn rect_em(c: [f64; 2], w: f64, h: f64) -> Rect {
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    Rect::new(
        (c[0] - w * 0.5) as f32,
        (c[1] - h * 0.5) as f32,
        w as f32,
        h as f32,
    )
}
fn dentro(r: &Rect, p: [f64; 2]) -> bool {
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    r.contains(p[0] as f32, p[1] as f32)
}

/// A cor de um lado: em conflito se uma regra em conflito cita os dois pontos; definido se os dois estão definidos.
fn cor_do_lado(a: PontoCad, b: PontoCad, e: &Estado, conflito: &[Vec<PontoCad>]) -> ColorToken {
    if conflito.iter().any(|ps| ps.contains(&a) && ps.contains(&b)) {
        return ColorToken::Danger;
    }
    let def = |p| e.pontos.get(&p) == Some(&Liberdade::Definido);
    if def(a) && def(b) {
        ColorToken::Text1
    } else {
        ColorToken::Info
    }
}

fn cor_da_regra(est: Option<EstadoRegra>, referencia: bool) -> ColorToken {
    match est {
        Some(EstadoRegra::Conflito) => ColorToken::Danger,
        Some(EstadoRegra::Redundante) => ColorToken::Warn,
        Some(EstadoRegra::Ref) => ColorToken::Text3,
        _ if referencia => ColorToken::Text3,
        _ => ColorToken::Accent,
    }
}

/// O centro da caixa dos pontos de um path no ecrã (o lado de FORA de uma cota aponta para longe dele).
fn centro_do(
    path: VecPathId,
    scene: &VecScene,
    view: &VecViewState,
    xf: &VecXforms,
    cam: Affine,
) -> Option<[f64; 2]> {
    let p = scene.path(path)?;
    let pts: Vec<[f64; 2]> = p
        .verts
        .iter()
        .filter_map(|v| tela(scene, view, xf, cam, PontoCad::Vert(path, v.id)))
        .collect();
    (!pts.is_empty()).then(|| {
        esc(
            pts.iter().fold([0.0; 2], |s, q| add(s, *q)),
            1.0 / pts.len() as f64,
        )
    })
}

/// Os entradas que o plano lê, além da cena.
pub(crate) struct Entrada<'a> {
    pub scene: &'a VecScene,
    pub view: &'a VecViewState,
    pub xf: &'a VecXforms,
    pub cam: Affine,
    pub estado: Option<&'a Estado>,
    pub ativa: bool,
    pub u: Unidades,
}

/// ⭐ **O plano**: a geometria de AGORA, a cor da última análise.
pub(crate) fn plano(e: &Entrada<'_>, text: &mut TextSystem) -> Plano {
    let Entrada {
        scene,
        view,
        xf,
        cam,
        ..
    } = *e;
    let ponto = |p| tela(scene, view, xf, cam, p);
    let regras = scene.regras();
    let estados: Vec<Option<EstadoRegra>> = (0..regras.len())
        .map(|i| e.estado.and_then(|s| s.regras.get(i).copied()))
        .collect();
    let conflito: Vec<Vec<PontoCad>> = regras
        .iter()
        .zip(&estados)
        .filter(|(_, s)| **s == Some(EstadoRegra::Conflito))
        .map(|(r, _)| r.lei.pontos())
        .collect();
    let mut plano = Plano {
        cam,
        lados: Vec::new(),
        selos: Vec::new(),
        cotas: Vec::new(),
        pontos: Vec::new(),
        segmentos: Vec::new(),
        circulos: Vec::new(),
        ligados: BTreeMap::new(),
    };
    let com_regras: std::collections::BTreeSet<VecPathId> = regras
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .collect();
    for p in scene.paths().iter().filter(|p| !view.is_hidden(p.id)) {
        if e.ativa
            && let Some((c, r)) = circulo_de(p, Xform::IDENTITY)
        {
            let t = ph2d_vec_render::overlay_transform(view, xf, p.id, cam);
            let cc = t * Point::new(c[0], c[1]);
            let borda = t * Point::new(c[0] + r, c[1]);
            plano
                .circulos
                .push((p.id, [cc.x, cc.y], (borda - cc).hypot()));
            continue;
        }
        let contornos = std::iter::once((&p.verts, p.closed))
            .chain(p.subpaths.iter().map(|c| (&c.verts, c.closed)));
        for (verts, fechado) in contornos {
            let n = verts.len();
            let pares = if fechado { n } else { n.saturating_sub(1) };
            for i in 0..pares {
                let (a, b) = (
                    PontoCad::Vert(p.id, verts[i].id),
                    PontoCad::Vert(p.id, verts[(i + 1) % n].id),
                );
                let (Some(qa), Some(qb)) = (ponto(a), ponto(b)) else {
                    continue;
                };
                if com_regras.contains(&p.id)
                    && let Some(s) = e.estado
                {
                    plano.lados.push((qa, qb, cor_do_lado(a, b, s, &conflito)));
                }
                if e.ativa {
                    plano.segmentos.push((SegCad(a, b), qa, qb));
                }
            }
            if e.ativa {
                plano.pontos.extend(verts.iter().filter_map(|v| {
                    let a = PontoCad::Vert(p.id, v.id);
                    ponto(a).map(|q| (a, q))
                }));
            }
        }
    }
    // Os selos pousam fora do lado; vários no mesmo sítio empilham-se ao longo dele.
    let mut ocupados: BTreeMap<(i64, i64), u32> = BTreeMap::new();
    let lado_px = f64::from(CHIP_H_PX);
    let fora = f64::from(Spacing::Lg.px());
    let mut pares = 0u32;
    for (r, est) in regras.iter().zip(&estados) {
        let cor = cor_da_regra(*est, r.referencia);
        let ps: Vec<[f64; 2]> = r.lei.pontos().into_iter().filter_map(ponto).collect();
        plano.ligados.insert(r.id, ps.clone());
        if let Some(valor) = e.u.texto(&r.lei) {
            // REF: a medida que o desenho já fixa mostra-se entre parênteses, como nos CAD — não se escreve.
            let texto = if r.referencia {
                ph2d_i18n::tr_with("app.vec.cad.referencia", &[("valor", &valor)])
            } else {
                valor
            };
            if let Some(c) = cota(&r.lei, &ps, &texto, r.id, cor, e, text) {
                plano.cotas.push(c);
            }
            continue;
        }
        if !e.ativa {
            continue;
        }
        let Some(tipo) = crate::cad_tool::tipo_de(&r.lei) else {
            continue;
        };
        let par = matches!(r.lei, Lei::Igual(..)).then(|| {
            pares += 1;
            pares
        });
        // Onde: o meio de cada lado citado (os dois do Igual), ou o ponto (Coincidente, Fixo).
        let sitios: Vec<([f64; 2], [f64; 2])> = match ps[..] {
            [a] | [a, _] if matches!(r.lei, Lei::Coincidente(..) | Lei::Fixo(..)) => {
                vec![(a, [0.7, -0.7])]
            }
            [a, b] => vec![(meio(a, b), unit([b[1] - a[1], a[0] - b[0]]))],
            [a, b, c, d] => vec![
                (meio(a, b), unit([b[1] - a[1], a[0] - b[0]])),
                (meio(c, d), unit([d[1] - c[1], c[0] - d[0]])),
            ],
            _ => Vec::new(),
        };
        // O selo de um lado pousa para DENTRO da forma (a cota do mesmo lado vai para fora — nunca se tapam).
        let dentro_de = r
            .lei
            .dono()
            .and_then(|d| centro_do(d, scene, view, xf, cam));
        for (m, mut n) in sitios {
            if let Some(cc) = dentro_de
                && !matches!(r.lei, Lei::Coincidente(..) | Lei::Fixo(..))
                && n[0] * (cc[0] - m[0]) + n[1] * (cc[1] - m[1]) < EMPATE_PX
            {
                n = esc(n, -1.0);
            }
            let base = add(m, esc(n, fora + lado_px * 0.5));
            #[expect(
                clippy::cast_possible_truncation,
                reason = "célula de ecrã para empilhar"
            )]
            let k = (
                (base[0] / lado_px).round() as i64,
                (base[1] / lado_px).round() as i64,
            );
            let ja = ocupados.entry(k).or_insert(0);
            let c = add(
                base,
                esc(
                    [n[1], -n[0]],
                    f64::from(*ja) * (lado_px + f64::from(Spacing::Xxs.px())),
                ),
            );
            *ja += 1;
            let w = if par.is_some() {
                lado_px * 2.0
            } else {
                lado_px
            };
            plano.selos.push(Selo {
                regra: r.id,
                icon: Possivel::icon(tipo),
                par,
                rect: rect_em(c, w, lado_px),
                cor,
            });
        }
    }
    desencosta_selos(
        &mut plano.selos,
        &plano.cotas,
        lado_px + f64::from(Spacing::Xxs.px()),
    );
    plano
}

/// O centro da forma em cima do próprio lado (um traço aberto de dois pontos): «selo para dentro, cota para fora» empata
/// e os dois iriam para o mesmo lado. No empate a cota fica do lado da normal e o selo vai para o outro.
const EMPATE_PX: f64 = 1e-6;

fn cruza(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn centro_de(r: &Rect) -> [f64; 2] {
    [f64::from(r.x + r.w * 0.5), f64::from(r.y + r.h * 0.5)]
}

/// ⭐ **Nenhum selo tapa um número nem outro selo** (report do dono, 26/10: o «= 1» da tampa tapava a cota nova): o
/// número é clicável e manda; um selo que ainda bata em alguém afasta-se dele, passo a passo, até ficar livre.
fn desencosta_selos(selos: &mut [Selo], cotas: &[Cota], passo: f64) {
    const TENTATIVAS: usize = 8;
    for i in 0..selos.len() {
        for _ in 0..TENTATIVAS {
            let r = selos[i].rect;
            let Some(outro) = cotas
                .iter()
                .map(|c| c.rect)
                .chain(selos[..i].iter().map(|s| s.rect))
                .find(|o| cruza(o, &r))
            else {
                break;
            };
            let d = sub(centro_de(&r), centro_de(&outro));
            let dir = if d[0].hypot(d[1]) > EMPATE_PX {
                unit(d)
            } else {
                [0.0, -1.0]
            };
            #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
            {
                selos[i].rect.x += (dir[0] * passo) as f32;
                selos[i].rect.y += (dir[1] * passo) as f32;
            }
        }
    }
}

/// A cota de uma medida: linhas de extensão + linha com setas + o número (distância); arco + número (ângulo); haste
/// do centro à borda + «R n» (raio).
fn cota(
    lei: &Lei,
    ps: &[[f64; 2]],
    texto: &str,
    regra: RegraId,
    cor: ColorToken,
    e: &Entrada<'_>,
    text: &mut TextSystem,
) -> Option<Cota> {
    let w = f64::from(chip_width(text, texto, 0.0));
    let afasta = f64::from(Spacing::Xl2.px());
    let ponta = f64::from(Spacing::Sm.px());
    let seta = |de: [f64; 2], para: [f64; 2]| (para, unit(sub(para, de)));
    let mut c = Cota {
        regra,
        tracos: Vec::new(),
        setas: Vec::new(),
        texto: texto.to_owned(),
        rect: rect_em([0.0; 2], 0.0, 0.0),
        cor,
    };
    let numero;
    match (lei, ps) {
        (Lei::Distancia(pa, ..), [a, b]) => {
            let mut n = unit([b[1] - a[1], a[0] - b[0]]);
            if let Some(cc) = centro_do(pa.path(), e.scene, e.view, e.xf, e.cam)
                && (n[0] * (meio(*a, *b)[0] - cc[0]) + n[1] * (meio(*a, *b)[1] - cc[1]))
                    < -EMPATE_PX
            {
                n = esc(n, -1.0);
            }
            let (a2, b2) = (add(*a, esc(n, afasta)), add(*b, esc(n, afasta)));
            c.tracos
                .push(vec![add(*a, esc(n, ponta)), add(a2, esc(n, ponta))]);
            c.tracos
                .push(vec![add(*b, esc(n, ponta)), add(b2, esc(n, ponta))]);
            c.tracos.push(vec![a2, b2]);
            c.setas.push(seta(b2, a2));
            c.setas.push(seta(a2, b2));
            numero = meio(a2, b2);
        }
        (Lei::Angulo(s, t, _), [a, b, cc, d]) => {
            // O vértice: o ponto que os dois lados partilham; senão, o meio dos dois inícios.
            let (v, p1, p2) = if s.0 == t.0 {
                (*a, *b, *d)
            } else if s.1 == t.0 {
                (*b, *a, *d)
            } else if s.0 == t.1 {
                (*a, *b, *cc)
            } else if s.1 == t.1 {
                (*b, *a, *cc)
            } else {
                (meio(*a, *cc), *b, *d)
            };
            let r = f64::from(Spacing::Xl3.px());
            let (u1, u2) = (unit(sub(p1, v)), unit(sub(p2, v)));
            let (a1, a2) = (u1[1].atan2(u1[0]), u2[1].atan2(u2[0]));
            let mut d = (a2 - a1).rem_euclid(std::f64::consts::TAU);
            if d > std::f64::consts::PI {
                d -= std::f64::consts::TAU;
            }
            let passos = 24;
            let arco: Vec<[f64; 2]> = (0..=passos)
                .map(|i| {
                    #[expect(clippy::cast_precision_loss, reason = "24 passos")]
                    let ang = a1 + d * i as f64 / passos as f64;
                    add(v, [r * ang.cos(), r * ang.sin()])
                })
                .collect();
            if let (Some(&f0), Some(&f1)) = (arco.get(1), arco.last()) {
                c.setas.push(seta(f0, arco[0]));
                c.setas.push(seta(arco[passos - 1], f1));
            }
            c.tracos.push(arco);
            let m = a1 + d * 0.5;
            numero = add(
                v,
                [
                    (r + f64::from(Spacing::Lg.px()) + w * 0.5) * m.cos(),
                    (r + f64::from(Spacing::Lg.px())) * m.sin(),
                ],
            );
        }
        (Lei::Raio(id, _), [centro]) => {
            let path = e.scene.path(*id)?;
            let (lc, lr) = circulo_de(path, Xform::IDENTITY)?;
            let t = ph2d_vec_render::overlay_transform(e.view, e.xf, *id, e.cam);
            let borda = t * Point::new(
                lc[0] + lr * std::f64::consts::FRAC_1_SQRT_2,
                lc[1] + lr * std::f64::consts::FRAC_1_SQRT_2,
            );
            let b = [borda.x, borda.y];
            c.tracos.push(vec![*centro, b]);
            c.setas.push(seta(*centro, b));
            numero = add(b, esc(unit(sub(b, *centro)), afasta * 0.5 + w * 0.5));
        }
        _ => return None,
    }
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    let rect = chip_rect([numero[0] as f32, numero[1] as f32], w as f32);
    c.rect = rect;
    Some(c)
}

impl Plano {
    /// O ponto do mundo no ecrã deste plano.
    #[must_use]
    pub fn no_ecra(&self, w: [f64; 2]) -> [f64; 2] {
        let q = self.cam * Point::new(w[0], w[1]);
        [q.x, q.y]
    }

    /// ⭐ O que está em `p` (px de ecrã): o número de uma cota, depois um selo, depois o ponto, o lado e o círculo
    /// mais perto dentro do alcance da mão.
    #[must_use]
    pub fn acerta(&self, p: [f64; 2]) -> Option<Acerto> {
        if let Some(c) = self.cotas.iter().find(|c| dentro(&c.rect, p)) {
            return Some(Acerto::Numero(c.regra));
        }
        if let Some(s) = self.selos.iter().find(|s| dentro(&s.rect, p)) {
            return Some(Acerto::Selo(s.regra));
        }
        let d = |q: [f64; 2]| (q[0] - p[0]).hypot(q[1] - p[1]);
        let perto = |v: f64| v <= HIT_PX;
        if let Some((a, _)) = self
            .pontos
            .iter()
            .map(|(a, q)| (a, d(*q)))
            .filter(|x| perto(x.1))
            .min_by(|x, y| x.1.total_cmp(&y.1))
        {
            return Some(Acerto::Alvo(Alvo::Ponto(*a)));
        }
        if let Some((s, _)) = self
            .segmentos
            .iter()
            .map(|(s, a, b)| (s, ao_segmento(p, *a, *b)))
            .filter(|x| perto(x.1))
            .min_by(|x, y| x.1.total_cmp(&y.1))
        {
            return Some(Acerto::Alvo(Alvo::Lado(*s)));
        }
        self.circulos
            .iter()
            .find(|(_, c, r)| perto((d(*c) - r).abs()))
            .map(|(id, ..)| Acerto::Alvo(Alvo::Circulo(*id)))
    }

    /// Os pontos de ecrã de um alvo.
    fn do_alvo(&self, a: &Alvo) -> Vec<[f64; 2]> {
        match a {
            Alvo::Ponto(p) => self
                .pontos
                .iter()
                .filter(|(q, _)| q == p)
                .map(|(_, x)| *x)
                .collect(),
            Alvo::Lado(s) => self
                .segmentos
                .iter()
                .filter(|(q, ..)| q == s)
                .flat_map(|(_, x, y)| [*x, *y])
                .collect(),
            Alvo::Circulo(id) => self
                .circulos
                .iter()
                .filter(|(q, ..)| q == id)
                .flat_map(|(_, c, r)| [add(*c, [-r, -r]), add(*c, [*r, *r])])
                .collect(),
        }
    }

    /// Onde a ④ pousa: a caixa da escolha, ou a do número/selo da regra escolhida.
    #[must_use]
    pub fn ancora(&self, escolha: &[Alvo], regra: Option<RegraId>) -> Option<Rect> {
        let mut pts: Vec<[f64; 2]> = escolha.iter().flat_map(|a| self.do_alvo(a)).collect();
        if pts.is_empty()
            && let Some(id) = regra
        {
            pts = self.ligados.get(&id).cloned().unwrap_or_default();
        }
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for q in &pts {
            lo = [lo[0].min(q[0]), lo[1].min(q[1])];
            hi = [hi[0].max(q[0]), hi[1].max(q[1])];
        }
        #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
        (!pts.is_empty()).then(|| {
            Rect::new(
                lo[0] as f32,
                lo[1] as f32,
                (hi[0] - lo[0]) as f32,
                (hi[1] - lo[1]) as f32,
            )
        })
    }
}

/// ⭐ **O desenho das regras** (a fase do overlay da shell, em todo modo com a tool do vetor na mão): faz o plano,
/// guarda-o para o press e o hover, e pinta-o. Selos e alvos só com a tool Regras; as cotas e a cor da forma sempre.
#[allow(clippy::too_many_arguments)] // o mesmo quadro que a shell parte para cada overlay
pub fn draw(
    vec: &crate::state::VecState,
    scene: &VecScene,
    view: &VecViewState,
    xf: &VecXforms,
    cam: Affine,
    hero: &ph2d_editor_core::HeroScreen,
    text: &mut TextSystem,
    target: &mut ph2d_vector::VectorScene,
) {
    let tool: &CadTool = &vec.cad_tool;
    let ativa = vec.draw_config.mode == ph2d_tool_vector::DrawMode::Rules;
    if scene.regras().is_empty() && !ativa {
        *tool.plano.borrow_mut() = None;
        return;
    }
    let analise = vec.cad_cores.ve(scene, xf);
    let entrada = Entrada {
        scene,
        view,
        xf,
        cam,
        estado: analise.as_deref(),
        ativa,
        u: Unidades::of(hero),
    };
    let p = plano(&entrada, text);
    crate::cad_tool_pinta::pinta(&p, tool, hero.theme, text, target);
    *tool.plano.borrow_mut() = Some(p);
}

#[cfg(test)]
#[path = "cad_tool_plano_tests.rs"]
mod tests;
