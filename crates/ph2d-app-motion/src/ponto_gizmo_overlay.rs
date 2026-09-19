//! **A TINTA do gizmo de uma corrente de posições** — a geometria vive em [`super::ponto_gizmo`];
//! aqui mora só o desenho, com o **vocabulário que o artista já aprendeu** (as constantes do
//! `warp_overlay`: a mesma cor, o mesmo casing escuro, a mesma espessura). *Um manipulador que ele
//! já leu noutro sítio não se reaprende.*
//!
//! ## As três feições
//!
//! | feição | o que se vê |
//! |---|---|
//! | **Osso** | a silhueta de armadura: larga na junta que manda, afilada para a ponta, com um anel na junta |
//! | **Corda** | a polilinha dos consecutivos, com uma marca em cada nó |
//! | **Ponto** | um anel pequeno e — **se o grafo der direcção** — a agulha que a mostra |
//!
//! ⚠️ **O osso é um LOSANGO afilado e não uma linha**, e é isso que faz uma cadeia ler-se como um
//! esqueleto em vez de um arame: a direcção é visível sem se seguir a ordem dos pontos.
//!
//! ## ⭐⭐⭐ As DUAS leis que a ordem do dono de 2026-09-19 impõe
//!
//! > *«os gizmos devem ter tamanho absoluto (não relativo ao zoom) e precisam responder aos grafos
//! > (como o scale do oscilador). Ou seja, eles não aparecem em runtime mas no canvas simulam
//! > qualquer grafo normalmente.»*
//!
//! **1. TAMANHO ABSOLUTO.** Todo glifo é construído já em coordenadas de TELA e traçado com
//! `Affine::IDENTITY` — as constantes abaixo são pixels, e `stroke` **multiplica** a espessura pelo
//! transform (a lei do cabeçalho do `warp_overlay`). ⚠️ **O que SEGUE o zoom é a GEOMETRIA** — onde
//! as juntas estão, quão comprido é um osso, por onde a corda passa —, e isso é obrigatório: elas
//! são factos de MUNDO. *O que não segue é a espessura do símbolo que as marca.*
//!
//! **2. O GLIFO RESPONDE AO GRAFO.** A coluna `size` entra como **MULTIPLICADOR dos pixels**
//! (nunca como medida de mundo) e a `rot` como a **agulha da direcção** — é isso que faz o `Scale`
//! de um `motion.oscillator` PULSAR e o `Rotation` GIRAR num canvas sem forma nenhuma ligada.
//!
//! ⚠️⚠️ **As duas leis não brigam, e a composição é que é a resposta:** o multiplicador é do
//! GRAFO e o pixel é da TELA, logo aproximar a câmara não engorda o símbolo e um oscilador engorda-o
//! — que é exactamente o pedido.
//!
//! ⛔ **A agulha só é desenhada quando a corrente TRAZ a coluna `rot`** — uma agulha a apontar para
//! a direita em toda a nuvem seria ruído sobre um grafo que nunca falou de direcção.
//!
//! ⛔ **O `tint` NÃO entra**, e é decisão declarada: o gizmo é chrome e a cor dele é o que o torna
//! legível sobre qualquer arte; uma corrente com alfa `0` apagaria o gizmo e o artista leria
//! *«o nó parou de funcionar»*, que é o defeito que esta wave inteira existe para não ter.

use super::ponto_gizmo::{Feicao, Grupo, PontoGizmoView};
use super::warp_overlay::{CASE_PX, CASE_RGBA, HANDLE_RGBA, OUTLINE_PX, TANGENT_RGBA};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Shape, Stroke, VectorScene};

/// Meia-largura do osso na junta que manda, em pixels de tela (com `size = 1`).
const OSSO_PX: f64 = 4.0;
/// O raio do anel de uma junta.
const JUNTA_PX: f64 = 2.5;
/// O raio do anel de um ponto solto.
const PONTO_PX: f64 = 2.0;
/// O comprimento da agulha que mostra a direcção de um ponto.
const AGULHA_PX: f64 = 7.0;
/// A tolerância com que um círculo vira curvas.
const CIRCULO_TOL_PX: f64 = 0.1;

/// ⚠️ **Um osso curto demais desenha-se como uma JUNTA e mais nada.** Sem esta cerca, o losango de
/// um segmento de comprimento ~0 fica com as duas pontas do lado errado da largura e pinta uma
/// gravata — uma cadeia com juntas coincidentes (o caso normal de uma corda em repouso) ficaria
/// coberta de borrões.
const OSSO_MIN_PX: f64 = OSSO_PX * 2.0;

/// **O PISO do glifo, e ele nomeia o recurso: a TOLERÂNCIA com que uma curva é achatada.**
///
/// Um círculo de raio abaixo dela não tem como virar segmentos — o caminho sai degenerado e o
/// traçador pode não emitir nada. ⭐ **Acima disso não é preciso piso nenhum**, e a razão está
/// medida no próprio desenho: o traço tem `OUTLINE_PX` de espessura mais o casing, logo um anel de
/// raio `0,1` ainda pinta uma marca de `~4 px` — *o que garante a visibilidade é a ESPESSURA, não
/// o raio*.
///
/// ⛔⛔ **A 1.ª redacção usava o `OUTLINE_PX` (`1,5`) e um gate reprovou-a:** com `size = 0,5` o
/// anel do ponto pede `1,0 px` e o piso devolvia `1,5` ⇒ **o gizmo deixava de responder ao grafo
/// exactamente na faixa que o artista usa**. *Um piso de legibilidade que morde no regime normal
/// não protege a legibilidade: revoga a lei.*
const GLIFO_MIN_PX: f64 = CIRCULO_TOL_PX;

/// O glifo de `base` pixels **escalado pelo grafo**, com o piso de legibilidade.
///
/// ⚠️ **Sem `camera` na assinatura, e isso é a lei 1 escrita no tipo:** esta função não tem como
/// depender do zoom. Um gate mede-o na mesma (dois `to_screen` diferentes dão o mesmo glifo), mas
/// quem lê o código vê primeiro a assinatura.
#[must_use]
pub(crate) fn glifo_px(base: f64, escala: f32) -> f64 {
    let s = f64::from(escala);
    if !s.is_finite() {
        return base; // uma coluna envenenada não apaga o gizmo
    }
    (base * s.abs()).max(GLIFO_MIN_PX)
}

/// **Desenha o gizmo publicado.** No-op sem grupos.
pub fn draw(
    v: &PontoGizmoView,
    camera: &Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`.
    let to_screen =
        camera.world_to_screen_affine(super::warp_gizmo::scene_window(center_split, full_window));
    let (cheios, tracos) = caminhos(v, &|w: [f32; 2]| {
        to_screen * Point::new(f64::from(w[0]), f64::from(w[1]))
    });
    if cheios.is_empty() && tracos.is_empty() {
        return;
    }
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    // O casing PRIMEIRO, mais grosso — a lei do `warp_overlay`.
    for caminho in [&cheios, &tracos] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            caminho,
        );
    }
    vector_scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &dim, None, &cheios);
    for caminho in [&cheios, &tracos] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &brush,
            None,
            caminho,
        );
    }
}

/// ⭐⭐ **A PORTA ÚNICA que constrói os caminhos** — devolve `(o que se preenche, o que se traça)`,
/// já em pixels de tela. Dois leitores: o [`draw`] e os gates, que a chamam com DOIS `to_screen`
/// diferentes para medir a lei do tamanho absoluto.
///
/// ⚠️ **Dois caminhos e não um por grupo:** o Vello encoda um caminho de uma vez, e uma cena com
/// treze ilhas (a `=110`) pagaria treze codificações por feição.
pub(crate) fn caminhos(v: &PontoGizmoView, pt: &dyn Fn([f32; 2]) -> Point) -> (BezPath, BezPath) {
    let mut cheios = BezPath::new();
    let mut tracos = BezPath::new();
    for g in &v.grupos {
        match g.feicao {
            Feicao::Osso => desenha_ossos(g, pt, &mut cheios, &mut tracos),
            Feicao::Corda => desenha_corda(g, pt, &mut tracos),
            Feicao::Ponto => desenha_pontos(g, pt, &mut tracos),
        }
    }
    (cheios, tracos)
}

/// A silhueta de cada osso mais o anel de cada junta.
///
/// ⚠️ **A LARGURA responde ao `size` e a DIRECÇÃO não lê o `rot`** — ela já está na geometria: numa
/// cadeia o ângulo de cada junta é o que PÔS as posições onde elas estão (a cinemática já correu),
/// e aplicá-lo outra vez ao losango contaria a mesma rotação duas vezes.
fn desenha_ossos(
    g: &Grupo,
    pt: &dyn Fn([f32; 2]) -> Point,
    cheios: &mut BezPath,
    tracos: &mut BezPath,
) {
    for [de, para] in &g.segmentos {
        let (a, b) = (pt(g.pontos[*de]), pt(g.pontos[*para]));
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let comp = dx.hypot(dy);
        if comp < OSSO_MIN_PX {
            continue; // ver `OSSO_MIN_PX`
        }
        // A meia-largura é a do FILHO: é o elemento que este osso representa.
        let meia = glifo_px(OSSO_PX, g.escala_em(*para));
        let (nx, ny) = (-dy / comp * meia, dx / comp * meia);
        // O ombro fica a um quinto do caminho: é onde a armadura do referencial o põe, e é o
        // que dá a direcção sem engordar a cadeia inteira.
        let ombro = Point::new(a.x + dx * 0.2, a.y + dy * 0.2);
        cheios.move_to(a);
        cheios.line_to(Point::new(ombro.x + nx, ombro.y + ny));
        cheios.line_to(b);
        cheios.line_to(Point::new(ombro.x - nx, ombro.y - ny));
        cheios.close_path();
    }
    for (i, p) in g.pontos.iter().enumerate() {
        anel(tracos, pt(*p), glifo_px(JUNTA_PX, g.escala_em(i)));
    }
}

/// A polilinha dos consecutivos, com uma marca em cada nó.
fn desenha_corda(g: &Grupo, pt: &dyn Fn([f32; 2]) -> Point, tracos: &mut BezPath) {
    let mut aberto = false;
    for [de, para] in &g.segmentos {
        if !aberto {
            tracos.move_to(pt(g.pontos[*de]));
            aberto = true;
        }
        tracos.line_to(pt(g.pontos[*para]));
    }
    for (i, p) in g.pontos.iter().enumerate() {
        anel(tracos, pt(*p), glifo_px(JUNTA_PX * 0.7, g.escala_em(i)));
    }
}

/// Um anel em cada posição e — **se o grafo der direcção** — a agulha que a mostra.
///
/// ⚠️ **Um anel e não uma cruz:** uma cruz rodada `90°` é a MESMA cruz, logo um oscilador a girar
/// de `0` a `360` leria-se como saltos de um quarto de volta. Um anel com agulha tem uma direcção
/// só, e ela roda de verdade.
fn desenha_pontos(g: &Grupo, pt: &dyn Fn([f32; 2]) -> Point, tracos: &mut BezPath) {
    for (i, p) in g.pontos.iter().enumerate() {
        let c = pt(*p);
        let esc = g.escala_em(i);
        anel(tracos, c, glifo_px(PONTO_PX, esc));
        if let Some(graus) = g.rot_em(i) {
            // ⚠️ A coluna é em GRAUS — a unidade de ângulo autorada desta casa (a mesma conversão
            // que o lowering faz, e no mesmo sítio: a borda onde a base é construída).
            let (sin, cos) = f64::from(graus).to_radians().sin_cos();
            let r = glifo_px(AGULHA_PX, esc);
            // ⚠️ O `y` da TELA cresce para baixo; o sinal aqui é o mesmo que a base do lowering
            // escreve (`[cos, sin, -sin, cos]`), senão a agulha giraria ao contrário da arte.
            tracos.move_to(c);
            tracos.line_to(Point::new(c.x + cos * r, c.y + sin * r));
        }
    }
}

fn anel(caminho: &mut BezPath, c: Point, r: f64) {
    caminho.extend(Circle::new(c, r).path_elements(CIRCULO_TOL_PX));
}
