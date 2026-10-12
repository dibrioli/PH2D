//! W15 — **«Send to board…»** da Hierarquia: a metade do QUADRO. A shell (composição) lê o conteúdo do
//! objecto na família dele — os píxeis da sprite, o SVG do vector, o quadro actual do Flip — e chama
//! aqui: [`ask`] abre o modal que escolhe o quadro, [`deliver`] põe o conteúdo nele.
//!
//! ⚠️ Tudo entra pela porta de entrada do quadro ([`crate::images::ingest`]): o que o soltar aceita,
//! isto aceita, com o mesmo tamanho natural.

pub use ph2d_board_edit::Pose;
use ph2d_board_model::BoardId;
use ph2d_board_render::Pixels;
pub use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::Toast;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::board_images::insert_posed;
use ph2d_editor_core::screens::hero::document_tabs::register_board;
use ph2d_editor_core::screens::hero::send_to_board::open;
use ph2d_i18n::{tr, tr_with};

/// O conteúdo de um objecto da cena, lido pela família dele.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    /// RGBA de 8 bits com alfa RETO (a sprite, o quadro do Flip) — entra como PNG.
    Pixels(Pixels),
    /// Um documento SVG (o vector) — entra como SVG, nítido a qualquer zoom.
    Svg(String),
}

impl Content {
    /// Os píxeis RGBA de 8 bits, alfa reto, `size` em píxeis.
    #[must_use]
    pub fn rgba(size: [u32; 2], rgba: Vec<u8>) -> Self {
        Self::Pixels(Pixels { size, rgba })
    }

    /// Os bytes do ficheiro que o quadro guarda.
    fn bytes(self) -> Option<Vec<u8>> {
        match self {
            Self::Pixels(px) => crate::images::encode_png(&px),
            Self::Svg(s) => Some(s.into_bytes()),
        }
    }
}

/// O pedido da linha (`HierRequest::SendToBoard`): abre o modal se o objecto é de um tipo que um
/// quadro recebe (`goes`, pela lei `send_to_board::goes_to_board`); senão, o aviso que o diz.
pub fn ask(hero: &mut HeroScreen, row: ph2d_editor_core::NodeId, goes: bool) -> Option<Toast> {
    if !goes {
        return Some(Toast::warning(tr("board.send.wrong_kind")));
    }
    open(&mut hero.store, row);
    None
}

/// W16 — um objecto a enviar: o nome dele (para os avisos), o conteúdo lido pela família dele — ou a
/// chave do aviso de porque não há (`board.send.empty`, `.unreadable`, `.wrong_kind`) — e a pose com
/// que nasce no quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct Sent {
    pub name: String,
    pub content: Result<Content, &'static str>,
    pub pose: Pose,
}

impl Sent {
    /// Um objecto sem pose (o vector e o Flip já chegam posados).
    #[must_use]
    pub fn new(name: impl Into<String>, content: Option<Content>) -> Self {
        Self {
            name: name.into(),
            content: content.ok_or("board.send.empty"),
            pose: Pose::default(),
        }
    }

    /// Um objecto que não vai, e a chave do aviso que diz porquê.
    #[must_use]
    pub fn missing(name: impl Into<String>, why: &'static str) -> Self {
        Self {
            name: name.into(),
            content: Err(why),
            pose: Pose::default(),
        }
    }

    /// W17 — um objecto apagado entre o pedido e a escolha: conta no aviso como «Deleted object».
    #[must_use]
    pub fn gone() -> Self {
        Self::missing(tr("board.send.deleted"), "board.send.gone")
    }

    /// W17 — o nome com que um objecto aparece nos avisos: o dele; sem nenhum (ou só espaços), o do
    /// TIPO («Unnamed picture») — nunca um vazio («Left out: , Flip»).
    #[must_use]
    pub fn name_of(name: Option<&str>, kind: ObjectKind) -> String {
        if let Some(n) = name.filter(|n| !n.trim().is_empty()) {
            return n.to_owned();
        }
        tr(match kind {
            ObjectKind::Image => "board.send.unnamed.image",
            ObjectKind::Vector => "board.send.unnamed.vector",
            ObjectKind::Flip => "board.send.unnamed.flip",
            _ => "board.send.unnamed.other",
        })
        .to_owned()
    }
}

/// Os nomes de uma lista de aviso, pela ordem, os iguais uma vez com a conta («Deleted object ×2»).
fn listed(names: &[&str]) -> String {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    for n in names {
        match seen.iter_mut().find(|(m, _)| m == n) {
            Some((_, k)) => *k += 1,
            None => seen.push((n, 1)),
        }
    }
    seen.iter()
        .map(|(n, k)| match k {
            1 => (*n).to_owned(),
            _ => tr_with("board.send.repeated", &[("name", n), ("count", k)]),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// ⭐ A escolha do modal (`HierRequest::SendToBoardChosen`): `board` = um quadro aberto, `None` = um
/// quadro NOVO (o nome por omissão das abas, como o «+»). O quadro fica ACTIVO (a aba dele abre-se) e
/// os conteúdos nascem no meio da vista dele (W16: vários numa grelha, cada um com a pose dele),
/// seleccionados, num passo de desfazer. Os que não têm conteúdo ou que o quadro recusa ficam de fora
/// e o aviso devolvido diz porquê (um só) ou nomeia-os (vários).
///
/// ⚠️ Nada muda se NENHUM é aceite: não se cria um quadro vazio nem se troca a aba.
pub fn deliver(
    hero: &mut HeroScreen,
    board: Option<u64>,
    items: Vec<Sent>,
) -> Result<(BoardId, Option<Toast>), Toast> {
    let many = items.len() > 1;
    let (mut left, mut files) = (Vec::new(), Vec::new());
    for s in items {
        let file = s.content.and_then(|c| {
            c.bytes()
                .and_then(|b| crate::images::ingest(&b))
                .ok_or("board.send.refused")
        });
        match file {
            Ok(f) => files.push((f, s.pose)),
            Err(why) => left.push((s.name, why)),
        }
    }
    let left_out = match left.as_slice() {
        [] => None,
        [(name, why)] if !many => Some(Toast::warning(tr_with(why, &[("name", name)]))),
        _ => {
            let names: Vec<&str> = left.iter().map(|(n, _)| n.as_str()).collect();
            Some(Toast::warning(tr_with(
                "board.send.left_out",
                &[("names", &listed(&names))],
            )))
        }
    };
    if files.is_empty() {
        // W17 — todos sumiram: o aviso diz isso, não a lista dos apagados.
        if many && left.iter().all(|(_, why)| *why == "board.send.gone") {
            return Err(Toast::warning(tr("board.send.all_gone")));
        }
        return Err(left_out.unwrap_or_else(|| Toast::warning(tr("board.send.wrong_kind"))));
    }
    let target = match board.map(BoardId) {
        Some(b) if hero.documents.boards().get(b).is_some() => b,
        Some(_) => return Err(Toast::warning(tr("board.send.no_board"))),
        None => {
            let n = hero.documents.boards().boards().len() + 1;
            let fresh = tr_with("board.tab.default_name", &[("n", &n)]);
            let b = hero.documents.create(fresh);
            register_board(&mut hero.store, b);
            b
        }
    };
    hero.documents.activate(Some(target));
    if insert_posed(hero, files, None) == 0 {
        return Err(Toast::warning(tr("board.send.no_board")));
    }
    Ok((target, left_out))
}

/// W16 — quem vai: a linha clicada; se ela está na selecção `selected` (os bits das entidades, pela
/// ordem dela), a selecção inteira — o menu de contexto de um item seleccionado age sobre todos.
#[must_use]
pub fn targets(clicked: u64, selected: &[u64]) -> Vec<u64> {
    if selected.contains(&clicked) {
        selected.to_vec()
    } else {
        vec![clicked]
    }
}

/// ⭐ W16 — a sprite com a POSE da cena: `m` = a parte linear do afim de MUNDO dela (`[a, b, c, d]`,
/// colunas `x = (a, b)`, `y = (c, d)`; o mundo tem y para CIMA), `size` = o tamanho dela no mundo e
/// `ppm` = os píxeis da régua do editor (a escala do vector e do Flip), `flip` = o espelho da PRÓPRIA
/// sprite (o `flip_x`/`flip_y` da Inspector: vira os UV, não está no afim). O quadro tem y para BAIXO.
///
/// Sem inclinação a pose vai no ELEMENTO (escala, ângulo, espelho — os píxeis inteiros, editável no
/// quadro); um afim que INCLINA (o skew da Inspector) o quadro não exprime: os píxeis redesenham-se
/// já inclinados ([`bake`]). `None` = a sprite não tem área (escala 0).
#[must_use]
pub fn posed_sprite(
    content: Content,
    m: [f64; 4],
    size: [f64; 2],
    ppm: f64,
    flip: [bool; 2],
) -> Option<(Content, Pose)> {
    let Content::Pixels(px) = content else {
        return Some((content, Pose::default()));
    };
    let [a, b, c, d] = m;
    let [nw, nh] = px.size.map(|v| f64::from(v.max(1)));
    let sign = |f: bool| if f { -1.0 } else { 1.0 };
    let (kx, ky) = (
        sign(flip[0]) * ppm * size[0] / nw,
        sign(flip[1]) * ppm * size[1] / nh,
    );
    // Do píxel da imagem (linha 0 em cima) ao quadro: F·A·F·diag(kx, ky), F = diag(1, -1).
    let k = [a * kx, -b * kx, -c * ky, d * ky];
    let r = k[0].hypot(k[1]);
    let (cs, sn) = (k[0] / r, k[1] / r);
    let shear = cs * k[2] + sn * k[3];
    let e = -sn * k[2] + cs * k[3];
    if !(r > 1e-9 && e.abs() > 1e-9) {
        return None;
    }
    if shear.abs() > 1e-6 * r.max(e.abs()) {
        // Redesenha-se à densidade NATURAL (o linear sem a escala média, |det| = 1): nenhum píxel
        // da sprite se perde; a escala vai na pose.
        let sigma = (r * e).abs().sqrt();
        let (out, scale) = bake(&px, k.map(|v| v / sigma));
        let scale = scale * sigma;
        let pose = Pose {
            scale: [scale; 2],
            ..Pose::default()
        };
        return Some((Content::Pixels(out), pose));
    }
    let mut angle = sn.atan2(cs);
    let mut flip = [false, e < 0.0];
    // R(φ)·diag(1, −1) = R(φ − π)·diag(−1, 1): o espelho horizontal com o ângulo mais pequeno.
    if e < 0.0 && cs < 0.0 {
        angle -= std::f64::consts::PI.copysign(angle);
        flip = [true, false];
    }
    let pose = Pose {
        scale: [r, e.abs()],
        angle,
        flip,
    };
    Some((Content::Pixels(px), pose))
}

/// O maior lado de uma sprite redesenhada: o atlas do desenho (`ATLAS_SIDE` do `ph2d-board-render`);
/// acima disto o quadro desenharia um nível mais pequeno de qualquer forma.
const BAKE_SIDE: f64 = 8192.0;

/// Os píxeis `px` pelo linear `k` (do píxel da imagem ao quadro, colunas `(k0, k1)` e `(k2, k3)`) na
/// caixa alinhada aos eixos que ocupam — bilinear, PESADO pelo alfa (a orla transparente não
/// escurece) — e a escala do natural da imagem ao quadro (`1`, ou mais se a caixa passava do atlas).
fn bake(px: &Pixels, k: [f64; 4]) -> (Pixels, f64) {
    let [nw, nh] = px.size.map(f64::from);
    let corners = [[0.0, 0.0], [nw, 0.0], [0.0, nh], [nw, nh]]
        .map(|[u, v]| [k[0] * u + k[2] * v, k[1] * u + k[3] * v]);
    let lo = |i: usize| corners.iter().map(|p| p[i]).fold(f64::INFINITY, f64::min);
    let hi = |i: usize| {
        corners
            .iter()
            .map(|p| p[i])
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let (x0, y0) = (lo(0), lo(1));
    let (bw, bh) = (hi(0) - x0, hi(1) - y0);
    let fit = (BAKE_SIDE / bw.max(bh)).min(1.0);
    let size = [bw, bh].map(|v| (v * fit).ceil().max(1.0) as u32);
    let det = k[0] * k[3] - k[2] * k[1];
    let mut out = vec![0u8; size[0] as usize * size[1] as usize * 4];
    let (w, h) = (px.size[0] as usize, px.size[1] as usize);
    let at = |x: isize, y: isize| -> [f64; 4] {
        if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
            return [0.0; 4];
        }
        let p = &px.rgba[(y as usize * w + x as usize) * 4..][..4];
        let a = f64::from(p[3]) / 255.0;
        [
            f64::from(p[0]) * a,
            f64::from(p[1]) * a,
            f64::from(p[2]) * a,
            a,
        ]
    };
    for oy in 0..size[1] as usize {
        for ox in 0..size[0] as usize {
            let qx = (ox as f64 + 0.5) / fit + x0;
            let qy = (oy as f64 + 0.5) / fit + y0;
            // O inverso de `k`: o ponto da imagem que cai aqui (centros dos píxeis em +0,5).
            let u = (k[3] * qx - k[2] * qy) / det - 0.5;
            let v = (-k[1] * qx + k[0] * qy) / det - 0.5;
            let (fx, fy) = (u.floor(), v.floor());
            let (tx, ty) = (u - fx, v - fy);
            let (ix, iy) = (fx as isize, fy as isize);
            let mut acc = [0.0; 4];
            for (dx, dy, wgt) in [
                (0, 0, (1.0 - tx) * (1.0 - ty)),
                (1, 0, tx * (1.0 - ty)),
                (0, 1, (1.0 - tx) * ty),
                (1, 1, tx * ty),
            ] {
                let p = at(ix + dx, iy + dy);
                for (a, q) in acc.iter_mut().zip(p) {
                    *a += q * wgt;
                }
            }
            let o = &mut out[(oy * size[0] as usize + ox) * 4..][..4];
            if acc[3] > 0.0 {
                for i in 0..3 {
                    o[i] = (acc[i] / acc[3]).round().clamp(0.0, 255.0) as u8;
                }
                o[3] = (acc[3] * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    (Pixels { size, rgba: out }, 1.0 / fit)
}

#[cfg(test)]
#[path = "send_to_board_tests.rs"]
mod tests;
