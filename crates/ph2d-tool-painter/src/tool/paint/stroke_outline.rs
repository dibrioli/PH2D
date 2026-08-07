//! **O CONTORNO de uma figura** — o produtor ÚNICO de *"que forma esta figura tem"*, lido pelos dois
//! consumidores que precisam concordar: o **gizmo que a shell desenha** e o **hit-test que decide se um
//! clique alcança a figura**.
//!
//! ## Por que isto virou uma porta
//!
//! Enquanto a tinta aparecia sob a mão (antes de `super::shape_draft`), uma figura parqueada podia ser
//! representada por uma **caixa** — o desenho pintado dizia onde ela estava, e a caixa só precisava dizer
//! *"ainda dá para clicar aqui"*. Com o gesto rascunhado a caixa passou a ser a ÚNICA coisa na tela, e as
//! duas metades do acordo quebraram de uma vez (Enio, 2026-08-07):
//!
//! - **o que se VÊ:** *"o gizmo fica invisível ao criar outro círculo"* — quatro círculos viravam quatro
//!   retângulos apagados, e o artista perdia de vista as figuras que ainda estava compondo;
//! - **o que se CLICA:** *"se clicar dentro de uma forma já desenhada, não aceita desenhar outra"* — o
//!   hit-test de figura parqueada era `bbox_contains`, ou seja o **INTERIOR** da caixa, e a caixa de um
//!   círculo de 900 px cobre `4/π` da área dele. Com quatro círculos grandes sobrepostos a união das
//!   caixas cobria quase a tela toda ⇒ todo Down caía em *"reativar uma figura parqueada"* e **nunca**
//!   chegava em *"parquear a ativa e começar outra"* (`stroke_multi::maybe_switch_or_new_shape`, passos
//!   4 e 5). O sintoma se lê como intermitente porque depende de o clique cair fora de TODAS as caixas.
//!
//! ⚠️ **As duas metades são o mesmo defeito**, e é por isso que a cura é uma porta só: **o que é
//! DESENHADO é o que é CLICÁVEL**. Um contorno é a coisa que o artista vê; a caixa era um proxy que
//! ninguém desenhou de propósito e que ninguém queria clicar.
//!
//! ## A lei de cada família (não é escolha — cada uma espelha o gizmo ATIVO dela)
//!
//! O contorno de uma figura parqueada tem de ser o gêmeo do gizmo que ela teria se estivesse ativa,
//! senão reativá-la faria a forma SALTAR:
//!
//! - **Ellipse / Polygon** — com o **Offset aplicado** (`ellipse_offset_radii`), exatamente como
//!   `ellipse_overlay`/`polygon_overlay`: ali o guia e a tinta andam juntos.
//! - **Curve / Line** — **PRISTINO**, sem offset, exatamente como `curve_overlay`: o offset é
//!   *drawing-only* (Enio 2026-07-05, *"ponto e linha ficassem parados e apenas o desenho sofresse o
//!   offset"*), então o CHROME nunca o carrega.
//!
//! ⚠️ **Isto NÃO é `parked_shape_dabs`**, e a diferença é o ponto: aquele produz a lista de **dabs**
//! (o pigmento), este produz a **linha** (a forma). O badge pedia a caixa ao primeiro — construía a lista
//! de dabs INTEIRA de cada figura parqueada, **a cada quadro**, para jogar tudo fora menos quatro floats
//! (o suspeito que a sonda `measure_idle_frame_of_a_live_shape` já nomeava).

use super::*;
use crate::undo::ShapeEditState;

/// A linha de uma figura, em px de imagem — o que o gizmo desenha e o que o clique alcança.
pub(super) struct ShapeOutline {
    /// Os pontos da linha, na ordem. **Nunca** repete o primeiro no fim: quem fecha é o consumidor
    /// (`stroke_box` na shell, [`Self::hit`] aqui), pela mesma razão que o `ellipse_perimeter` não fecha.
    pub points: Vec<[f32; 2]>,
    /// `true` quando a linha fecha (elipse, polígono, curva/linha fechadas). Um contorno ABERTO fechado
    /// por engano ganha um segmento que a figura não tem — visível no desenho e no hit-test.
    pub closed: bool,
}

impl ShapeOutline {
    /// `true` quando `pos` está a até `band` px da linha — **o hit-test do contorno**.
    ///
    /// ⚠️ O segmento de fecho entra aqui e em lugar nenhum: num polígono de 3 lados ele é **um terço da
    /// figura**, e esquecê-lo deixaria uma aresta inteira inalcançável.
    pub fn hit(&self, pos: [f32; 2], band: f32) -> bool {
        if super::stroke_router::point_near_polyline(pos, &self.points, band) {
            return true;
        }
        self.closed
            && self.points.len() >= 3
            && super::stroke_router::point_near_polyline(
                pos,
                &[self.points[self.points.len() - 1], self.points[0]],
                band,
            )
    }

    /// A AABB da LINHA (não dos dabs) — o centro do badge sai daqui.
    pub fn bbox(&self) -> Option<[f32; 4]> {
        let mut bb = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for p in &self.points {
            bb[0] = bb[0].min(p[0]);
            bb[1] = bb[1].min(p[1]);
            bb[2] = bb[2].max(p[0]);
            bb[3] = bb[3].max(p[1]);
        }
        (bb[0] <= bb[2]).then_some(bb)
    }
}

impl PainterTool {
    /// O contorno de uma figura (ativa ou parqueada) — ver o cabeçalho do módulo para a lei de cada
    /// família. `None` quando a figura ainda não tem linha (menos de 2 pontos).
    pub(super) fn shape_state_outline(&self, st: &ShapeEditState) -> Option<ShapeOutline> {
        use crate::undo::ShapeEditState as S;
        let off = self.shape_offset_px();
        let mut points = Vec::new();
        let closed = match st {
            S::Ellipse(e) => {
                let (erx, ery) = self.ellipse_offset_radii(e.rx, e.ry);
                ph2d_painter_brush::ellipse_perimeter(e.center, e.u, erx, ery, &mut points);
                true
            }
            S::Polygon(p) => {
                let (erx, ery) = ((p.rx + off).max(0.5), (p.ry + off).max(0.5));
                ph2d_painter_brush::polygon_perimeter(
                    p.center,
                    p.u,
                    erx,
                    ery,
                    p.sides,
                    &mut points,
                );
                true
            }
            // PRISTINO: o offset da curva é drawing-only (a lei do `curve_overlay`).
            S::Curve(c) => {
                curve_geom::flatten_spine(&c.points, &c.handles, c.closed, &mut points);
                c.closed
            }
            // PRISTINO pelo mesmo motivo; as quinas Fillet/Chamfer SÃO a forma, então entram.
            S::Line(l) => {
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                points = line_corner::cooked_path(
                    &l.points,
                    l.closed,
                    &mods,
                    self.paint.shape_grab_tol_px,
                );
                l.closed
            }
        };
        (points.len() >= 2).then_some(ShapeOutline { points, closed })
    }
}
