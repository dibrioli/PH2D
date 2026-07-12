//! **As duas alças das pontas do conector** — os círculos que reposicionam onde a linha
//! encosta na forma.
//!
//! Selecione um conector e cada ponta ganha um círculo. Arrastá-lo diz **onde** a linha toca a
//! forma; largá-lo sobre outra forma **religa** a linha; e — o pedido que define o módulo —
//! puxá-lo para longe da forma **não solta o vínculo**: a ponta continua presa, só que num
//! ponto afastado, e continua andando quando a forma anda.
//!
//! # A ideia que faz tudo isso caber num só campo
//!
//! A ponta afastada parece exigir um conceito novo (uma "folga", um "offset"). Não exige. O
//! [`Anchor::Port`] já guarda um ponto **normalizado na caixa LOCAL da forma** — e nada obriga
//! `u`/`v` a ficarem dentro de `[0, 1]`. Um `u = 1.8` é simplesmente um ponto além da borda
//! direita, medido na régua da própria forma.
//!
//! Isso não é economia de campo: é o que faz o afastamento **girar e escalar junto com a
//! forma** de graça (ADR-0111). Uma folga em unidades de mundo teria de ser rotacionada à mão,
//! e escalaria errado no dia em que alguém escalasse a caixa. Aqui não há nada a manter — a
//! coordenada já vive no referencial certo.
//!
//! O preço: quem lê um `Port` **não pode clampar** `u`/`v` (e o `port_world` do
//! `connector_live` deixou de fazê-lo).
//!
//! # A zona do centro
//!
//! Largar a alça no MEIO da forma devolve a ponta ao modo **automático** (o lado de saída volta
//! a ser re-escolhido a cada frame conforme a outra forma se move — a âncora "verde" do
//! draw.io). É a única maneira de desfazer um port fixo, e é onde o olho a procura: o centro é
//! o "sem preferência".

use ph2d_ecs::{Anchor, ConnectorEnd};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, Xform, xform_of};

/// Qual das duas pontas.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EndSide {
    Start,
    End,
}

/// Meia-largura da zona do centro, em fração da caixa. Largar dentro dela = voltar ao
/// automático. `0.2` ⇒ os 40% centrais de cada eixo — grande o bastante para ser fácil de
/// acertar, pequeno o bastante para não roubar a borda, que é onde os ports úteis vivem.
const CENTRE_ZONE: f64 = 0.2;

/// A folga (em múltiplos do raio da alça) com que a pressão pega o CORPO da linha para fincar um
/// waypoint. Mais generosa que a alça: agarrar a linha para dobrá-la é um gesto grosso, e errá-lo
/// só custa um clique — enquanto uma alça agarrada por engano move uma ponta.
pub(crate) const BODY_GRAB_K: f64 = 1.4;

/// Amostras por segmento na busca do ponto mais próximo do corpo da linha.
pub(crate) const BODY_SAMPLES: u32 = 24;

/// A forma sob o cursor no momento de largar.
#[derive(Copy, Clone, Debug)]
pub(crate) struct DropTarget {
    pub id: VecPathId,
    /// A caixa LOCAL da forma (é nela que `u`/`v` são medidos).
    pub lo: [f64; 2],
    pub hi: [f64; 2],
    /// O afim local→mundo da forma.
    pub xform: Xform,
}

/// As coordenadas normalizadas de `world` na caixa local da forma. **Sem clamp** — é o que
/// permite a `u`/`v` saírem de `[0, 1]` e a ponta ficar afastada, ainda vinculada.
///
/// `None` se o afim é singular (forma escalada a zero) ou a caixa é degenerada: aí não há
/// régua em que medir, e o chamador cai no ponto solto.
pub(crate) fn uv_of(t: &DropTarget, world: [f64; 2]) -> Option<(f64, f64)> {
    let local = t.xform.inverse()?.apply(world);
    let (w, h) = (t.hi[0] - t.lo[0], t.hi[1] - t.lo[1]);
    if w.abs() < 1e-9 || h.abs() < 1e-9 {
        return None;
    }
    Some(((local[0] - t.lo[0]) / w, (local[1] - t.lo[1]) / h))
}

/// A âncora que um `(u, v)` descreve: o **centro** devolve a ponta ao automático; qualquer
/// outro ponto a fixa ali.
fn anchor_for(u: f64, v: f64) -> Anchor {
    if (u - 0.5).abs() < CENTRE_ZONE && (v - 0.5).abs() < CENTRE_ZONE {
        return Anchor::Floating;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o port e uma coordenada de UI; f32 sobra"
    )]
    Anchor::Port {
        u: u as f32,
        v: v as f32,
    }
}

/// **A resolução do largar.** Puro de propósito: é aqui que mora a regra, e é aqui que ela é
/// testada — sem montar mundo, câmera nem ponteiro.
///
/// | onde a alça caiu | o que acontece |
/// |---|---|
/// | sobre uma forma, no **centro** dela | prende nela, em modo **automático** |
/// | sobre uma forma, fora do centro | prende nela, **naquele ponto** |
/// | no vazio, com a ponta já **presa** | **continua presa** — o ponto vira um `u`/`v` fora de `[0,1]`, e a linha se afasta sem soltar |
/// | no vazio, com a ponta **solta** | fica solta, no lugar novo |
///
/// A terceira linha é a que o Enio pediu, e a razão de esta função existir em vez de um `if`
/// espalhado pelo handler do ponteiro.
pub(crate) fn resolve_drop(
    current: &ConnectorEnd,
    drop_world: [f64; 2],
    over: Option<DropTarget>,
    bound_target: Option<DropTarget>,
) -> ConnectorEnd {
    // 1. Largou sobre uma forma: prende nela (religando, se for outra).
    if let Some(t) = over
        && let Some((u, v)) = uv_of(&t, drop_world)
    {
        return ConnectorEnd::Bound {
            target: t.id,
            anchor: anchor_for(u, v),
        };
    }
    // 2. Largou no vazio, mas a ponta JÁ ESTAVA PRESA: ela continua presa. O ponto é medido na
    //    régua da forma — e sai de `[0, 1]`, que é exatamente o que "afastar sem soltar"
    //    significa.
    if let (ConnectorEnd::Bound { .. }, Some(t)) = (current, bound_target)
        && let Some((u, v)) = uv_of(&t, drop_world)
    {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o port e uma coordenada de UI; f32 sobra"
        )]
        return ConnectorEnd::Bound {
            target: t.id,
            anchor: Anchor::Port {
                u: u as f32,
                v: v as f32,
            },
        };
    }
    // 3. Solta, e continua solta.
    ConnectorEnd::Free { at: drop_world }
}

/// O alvo de uma ponta presa, resolvido na cena (para a regra 2 acima).
pub(crate) fn target_of(
    scene: &VecScene,
    xforms: &VecXforms,
    end: &ConnectorEnd,
) -> Option<DropTarget> {
    let ConnectorEnd::Bound { target, .. } = end else {
        return None;
    };
    let (lo, hi) = scene.path_curve_bbox(*target)?;
    Some(DropTarget {
        id: *target,
        lo,
        hi,
        xform: xform_of(xforms, *target),
    })
}

/// As duas alças, em MUNDO: as pontas da polilinha cozida do conector.
pub(crate) fn handle_points(scene: &VecScene, id: VecPathId) -> Option<([f64; 2], [f64; 2])> {
    let p = scene.paths().iter().find(|p| p.id == id)?;
    Some((p.verts.first()?.anchor, p.verts.last()?.anchor))
}

/// Qual alça o cursor pegou. A ponta do FIM ganha o desempate: é a que tem a seta, e é a que o
/// usuário mira quando as duas coincidem (num conector degenerado, colapsado num ponto).
pub(crate) fn hit(
    scene: &VecScene,
    id: VecPathId,
    world: [f64; 2],
    r: f64,
) -> Option<(EndSide, [f64; 2])> {
    let (a, b) = handle_points(scene, id)?;
    let near = |p: [f64; 2]| (p[0] - world[0]).hypot(p[1] - world[1]) <= r;
    if near(b) {
        return Some((EndSide::End, b));
    }
    if near(a) {
        return Some((EndSide::Start, a));
    }
    None
}

#[cfg(test)]
#[path = "connector_handles_tests.rs"]
mod tests;

// ── O gesto ──────────────────────────────────────────────────────────────────────────────

use ph2d_ecs::{Entity, VecConnector};

/// O GESTO (os `impl App`) mora no módulo irmão — teto de 600 LOC por arquivo da shell. Aqui
/// ficam a REGRA (pura, e por isso testável sem montar um `App`) e os dados que ela usa.
#[path = "connector_handles_gesture.rs"]
mod gesture;
pub(crate) use gesture::view;

/// O que a pressão agarrou.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Grab {
    /// Uma das duas pontas (o círculo).
    End(EndSide),
    /// Um ponto de passagem (o quadradinho), pelo índice.
    Waypoint(usize),
    /// **O CORPO da linha, ainda sem nada criado** — o waypoint só nasce se o gesto virar um
    /// ARRASTO. `usize` = o índice em que ele entrará.
    ///
    /// Isto existe por causa de um conflito real: o **duplo-clique** num conector abre o rótulo
    /// dele, e o primeiro clique do par caía no corpo da linha. Criando o waypoint já no `Down`,
    /// todo duplo-clique fincava um ponto de passagem antes de abrir o texto. Um clique e um
    /// arrasto **não são o mesmo gesto**, e a fronteira entre eles é o movimento — não a
    /// pressão.
    Body(usize),
}

/// O quanto o cursor precisa andar (em múltiplos do raio da alça) para uma pressão no corpo da
/// linha virar um waypoint. Abaixo disso é um CLIQUE, e o clique pertence ao duplo-clique.
const BODY_DRAG_MIN_K: f64 = 0.6;

/// **A pressão no corpo da linha virou um ARRASTO?**
///
/// Função pura porque é ela que resolve o conflito de gestos, e um conflito de gestos merece um
/// gate. O duplo-clique num conector abre o rótulo dele; o arrasto do corpo finca um waypoint. Os
/// dois começam com a MESMA pressão, no MESMO lugar — o que os separa é o movimento, e nada mais.
/// Criar o waypoint já na pressão (que era o que este módulo fazia) fincava um ponto de passagem
/// debaixo de todo duplo-clique, antes de o texto sequer abrir.
#[must_use]
pub(crate) fn body_became_a_drag(start: [f64; 2], now: [f64; 2], handle_r: f64) -> bool {
    (now[0] - start[0]).hypot(now[1] - start[1]) >= handle_r * BODY_DRAG_MIN_K
}

/// O arrasto de uma alça (Down..Up).
#[derive(Clone, Debug)]
pub(crate) struct HandleDrag {
    pub(crate) path: VecPathId,
    grab: Grab,
    /// A ponta como ela **estava** (só para [`Grab::End`]). Durante o arrasto a ponta vira
    /// `Free` (para a linha seguir o cursor ao vivo, que é o preview), e o largar resolve a
    /// partir DAQUI — senão o vínculo original se perderia no meio do caminho e "afastar sem
    /// soltar" viraria "soltar".
    original: ConnectorEnd,
    /// O waypoint NASCEU neste gesto — se o gesto for cancelado, ele é desfeito.
    born_now: bool,
    /// Onde a pressão começou (mundo) — a régua do limiar que separa o clique do arrasto.
    start: [f64; 2],
}

/// A menor distância do ponto `p` ao segmento `a`–`b`.
fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 < 1e-12 {
        0.0
    } else {
        (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / len2).clamp(0.0, 1.0)
    };
    let q = [a[0] + ab[0] * t, a[1] + ab[1] * t];
    (p[0] - q[0]).hypot(p[1] - q[1])
}

/// **Onde enfiar um waypoint NOVO na lista.**
///
/// A ordem dos waypoints é a ordem em que a linha passa por eles, então inserir no fim seria um
/// bug: fincar um ponto perto da origem faria a linha ir até o fim, voltar, e seguir. O índice
/// certo é o que **menos alonga** a rota — o critério clássico de inserção (o desvio que o ponto
/// novo acrescenta ao trecho que ele parte).
///
/// `stations` são os pontos das estações **em ordem**: a saída, os waypoints, a chegada.
pub(crate) fn insert_index(stations: &[[f64; 2]], p: [f64; 2]) -> usize {
    let mut best = (f64::INFINITY, 0usize);
    for i in 0..stations.len().saturating_sub(1) {
        let (a, b) = (stations[i], stations[i + 1]);
        let detour = (p[0] - a[0]).hypot(p[1] - a[1]) + (b[0] - p[0]).hypot(b[1] - p[1])
            - (b[0] - a[0]).hypot(b[1] - a[1]);
        if detour < best.0 {
            best = (detour, i);
        }
    }
    best.1
}

/// Um waypoint largado **em cima da reta entre os vizinhos dele** não diz nada — ele nao dobra a
/// rota. Removê-lo ali é o gesto natural de desfazer um ponto de passagem (arraste-o de volta
/// para a linha), e de quebra apaga o que um clique perdido criaria.
///
/// `tol` em unidades de MUNDO (o chamador converte dos pixels da alça).
pub(crate) fn is_redundant(stations: &[[f64; 2]], i: usize, tol: f64) -> bool {
    // `i` é o índice do waypoint; nas estações ele está em `i + 1` (a saída vem antes).
    let k = i + 1;
    if k == 0 || k + 1 >= stations.len() {
        return false;
    }
    dist_to_segment(stations[k], stations[k - 1], stations[k + 1]) <= tol
}

/// Os WAYPOINTS a desenhar neste frame (em MUNDO). Vazio quando nenhum conector está
/// selecionado — como as alças de ponta, eles são controles da seleção, não enfeite da tela.
pub(crate) fn waypoint_view(
    sim: &ph2d_ecs::SimWorld,
    map: &crate::vec_entities::VecEntityMap,
    selected: &[VecPathId],
) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    for &id in selected {
        let Some(&bits) = map.get(&id) else { continue };
        if let Some(c) = sim.world().get::<VecConnector>(Entity::from_bits(bits)) {
            out.extend(c.waypoints.iter().copied());
        }
    }
    out
}
