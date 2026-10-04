//! ⭐⭐⭐ **OS EFEITOS NA PELE** — `PH2D_VEC_BONE_SMOKE=5` (F50, ordem do dono de 2026-10-02:
//! formas vetoriais presas a ossos COM efeitos).
//!
//! # O que a cena monta
//!
//! Seis barras iguais, cada uma com um esqueleto igual de [`OSSOS`] ossos, todas dobradas em S a
//! [`DOBRA`] por junta. A de cima à esquerda NÃO tem efeito (o controlo); cada uma das outras tem
//! um ([`EFEITOS`]). ⭐⭐ Desde 2026-10-03 o Bind COZE os efeitos no desenho (ordem do dono: *«ao
//! aplicar os bones, os efeitos são cozidos antes»*, ver `skin_live::bind`) — o efeito desenhado em
//! repouso dobra com a barra, e a barra presa já não recebe efeitos (a secção Effects diz porquê).
//!
//! ⚠️ **A dobra é aplicada DEPOIS de prender** — prender fotografa a pose de repouso (ver o
//! [`crate::smoke_bone_par`]).

use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// Ossos por barra — os do PAR.
pub(crate) const OSSOS: u32 = 3;

/// ⭐ **A dobra por junta, em graus** — a mais forte em que o desenho fiel ainda não se cruza (o
/// contacto começa a `~110°`, ver o `DOBRA_FORTE` do par): a cena pergunta pelo EFEITO, não pelo
/// contacto.
pub(crate) const DOBRA: f32 = 60.0;

/// `(comprimento, espessura)` de uma barra, em metros — seis cabem na câmera de omissão (gate).
pub(crate) const PECA: (f64, f64) = (4.5, 0.75);

/// O vão entre duas barras dobradas, em metros.
const VAO: f64 = 0.5;

/// A cor das barras e a do contorno — as do par.
const COR: [u8; 3] = crate::smoke_bone_par::COR;
const CONTORNO: [u8; 3] = crate::smoke_bone_par::CONTORNO;

/// Um efeito da grelha: o nome e a fábrica dele (`None` = o controlo).
pub(crate) type Barra = (&'static str, Option<fn() -> PathEffect>);

/// ⭐⭐ **Os efeitos, pela ordem da grelha** (linha a linha, esquerda → direita) — `None` é o
/// controlo. O nome é o que o painel de efeitos mostra.
pub(crate) const EFEITOS: [Barra; 6] = [
    ("sem efeito", None),
    ("Zig Zag", Some(zig_zag)),
    ("Twist", Some(twist)),
    ("Warp", Some(warp)),
    ("Bloat", Some(bloat)),
    ("Hatch", Some(hatch)),
];

fn zig_zag() -> PathEffect {
    PathEffect::ZigZag(ph2d_vec_scene::fx_zigzag::ZigZagSpec {
        amplitude: 6.0,
        ridges: 24.0,
        ..Default::default()
    })
}

fn twist() -> PathEffect {
    PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle: 10.0 })
}

fn warp() -> PathEffect {
    let mut s = ph2d_vec_scene::fx_warp_presets::WarpSpec::new(
        ph2d_vec_scene::fx_warp_presets::WarpStyle::Arc,
    );
    s.bend = 25.0;
    PathEffect::Warp(s)
}

fn bloat() -> PathEffect {
    PathEffect::Bloat(ph2d_vec_scene::fx_warp::BloatSpec { amount: -20.0 })
}

fn hatch() -> PathEffect {
    PathEffect::Hatch(ph2d_vec_scene::fx_hatch::HatchSpec {
        angle: 45.0,
        spacing: 8.0,
        cross: false,
    })
}

/// ⭐ **A caixa da barra DOBRADA** centrada (recta) na origem — `[x0, y0, x1, y1]`, a pose que o
/// [`crate::smoke_bone_par::dobra_duas`] produz (o 2.º osso sobe, o 3.º volta a deitar).
#[must_use]
pub(crate) fn caixa_dobrada() -> [f64; 4] {
    let (l, t) = PECA;
    let passo = (l - t) / f64::from(OSSOS);
    let mut p = [-l / 2.0 + t / 2.0, 0.0];
    let mut ang = 0.0_f64;
    let mut c = [p[0], p[1], p[0], p[1]];
    for k in 0..OSSOS {
        if k > 0 {
            ang += if k % 2 == 1 { 1.0 } else { -1.0 } * f64::from(DOBRA).to_radians();
        }
        p = [p[0] + passo * ang.cos(), p[1] + passo * ang.sin()];
        c = [
            c[0].min(p[0]),
            c[1].min(p[1]),
            c[2].max(p[0]),
            c[3].max(p[1]),
        ];
    }
    let r = t / 2.0;
    [c[0] - r, c[1] - r, c[2] + r, c[3] + r]
}

/// ⭐⭐ **O centro (recto) de cada barra** — duas colunas, três linhas, com a caixa DOBRADA de cada
/// uma centrada na sua célula.
#[must_use]
pub(crate) fn origens() -> Vec<[f64; 2]> {
    let [x0, y0, x1, y1] = caixa_dobrada();
    let (w, h) = (x1 - x0, y1 - y0);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    (0..EFEITOS.len())
        .map(|i| {
            let (col, lin) = ((i % 2) as f64, (i / 2) as f64);
            [(col - 0.5) * (w + VAO) - cx, (1.0 - lin) * (h + VAO) - cy]
        })
        .collect()
}

/// Um esqueleto de [`OSSOS`] ossos pelo EIXO da barra centrada em `centro`, recto.
pub(crate) fn esqueleto(sim: &mut SimWorld, centro: [f64; 2], nome: &str) -> Option<Entity> {
    let (l, t) = PECA;
    let x0 = centro[0] - l / 2.0 + t / 2.0;
    let passo = (l - t) / f64::from(OSSOS);
    let (mut pai, mut raiz) = (None, None);
    for k in 0..OSSOS {
        let a = [x0 + passo * f64::from(k), centro[1]];
        let b = [x0 + passo * f64::from(k + 1), centro[1]];
        let e = Entity::try_from_bits(ph2d_skeleton_live::bone::create(sim, pai, a, b)?)?;
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::Name::new(format!("{nome} bone {}", k + 1)));
        raiz.get_or_insert(e);
        pai = Some(e);
    }
    raiz
}

/// O 1.º tempo: as seis barras e os seis esqueletos, rectos.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let (l, t) = PECA;
    let mut pend = Vec::new();
    for ((nome, efeito), o) in EFEITOS.iter().zip(origens()) {
        let mut p = shape(
            ShapeKind::RoundRect,
            [o[0] - l / 2.0, o[1] - t / 2.0],
            [o[0] + l / 2.0, o[1] + t / 2.0],
            &[t / 2.0],
            COR,
        );
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(CONTORNO[0], CONTORNO[1], CONTORNO[2], 255),
            t * crate::smoke_bone_par::ESPESSURA_DO_CONTORNO,
        ));
        if let Some(f) = efeito {
            p.effects.push(FxEntry::new(f()));
        }
        let id = scene.push_path(p);
        pend.push((id, esqueleto(sim, o, nome)));
    }
    st.bone_smoke_pend = Some(pend);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende cada barra ao seu esqueleto RECTO, dá-lhe o nome do efeito e dobra.
pub(crate) fn bind(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let mut presas = 0;
    for ((id, raiz), (nome, _)) in pecas.iter().zip(EFEITOS) {
        presas += ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[*id], *raiz);
        if let Some(e) = st.entities.get(id).and_then(|b| Entity::try_from_bits(*b)) {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new(nome));
        }
        if let Some(r) = raiz {
            crate::smoke_bone_par::dobra_duas(sim, *r, DOBRA, DOBRA);
        }
    }
    if presas != EFEITOS.len() {
        eprintln!(
            "[vec-bone-smoke] PARE: so' {presas} de {} barras prenderam -- a cena nao montou",
            EFEITOS.len()
        );
    }
    println!(
        "[vec-bone-smoke] OS EFEITOS NA PELE: seis barras iguais, cada uma presa a um esqueleto \
         igual e dobrada em S ({DOBRA}° por junta). Da esquerda para a direita, de cima para baixo: \
         SEM EFEITO, Zig Zag, Twist, Warp, Bloat, Hatch.\n\
         [vec-bone-smoke] 1) Olhe cada barra: o efeito deve DOBRAR junto com ela, como um desenho \
         feito na barra -- as riscas do Hatch acompanham a curva, o Zig Zag mantem os dentes do \
         mesmo tamanho dos dois lados de cada junta.\n\
         [vec-bone-smoke] 2) Clique numa barra e abra o painel Vector: a seccao Effects diz \
         \"Bound to bones: effects are baked into the drawing\" -- ao prender, o efeito passou a \
         fazer parte do desenho, e uma forma presa nao recebe efeitos."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐⭐ **AS SEIS CABEM NA CÂMERA DE OMISSÃO e nenhuma toca outra** — a mesma régua do par
    /// (largura visível `height × 16/9 × 0,63`, `90 %` de folga).
    ///
    /// ⛔ **A altura é a FAIXA MEDIDA, `±4,0 m`, e não `90 %`:** a `±4,5 m` este gate aprovou uma
    /// grelha de quatro linhas (2026-10-03) e a FOTO (`1930 × 1040`) cortava-lhe o topo — a barra
    /// de ferramentas tapa `~0,95 m` em cima e a de estado `~0,6 m` em baixo.
    #[test]
    fn as_seis_cabem_na_camera_de_omissao_e_nao_se_tocam() {
        let cam = ph2d_render::Camera2d::default();
        let h = f64::from(cam.height_world);
        let meia_largura = h * 16.0 / 9.0 * 0.63 * 0.9 / 2.0;
        let meia_altura = h * 0.8 / 2.0;
        let [x0, y0, x1, y1] = caixa_dobrada();
        let caixas: Vec<[f64; 4]> = origens()
            .iter()
            .map(|o| [x0 + o[0], y0 + o[1], x1 + o[0], y1 + o[1]])
            .collect();
        for c in &caixas {
            assert!(
                c[0].abs().max(c[2].abs()) <= meia_largura
                    && c[1].abs().max(c[3].abs()) <= meia_altura,
                "uma barra sai da camera de omissao: {c:?} (meia largura {meia_largura:.2}, meia \
                 altura {meia_altura:.2})"
            );
        }
        for (i, a) in caixas.iter().enumerate() {
            for b in &caixas[i + 1..] {
                let separadas = a[2] < b[0] || b[2] < a[0] || a[3] < b[1] || b[3] < a[1];
                assert!(separadas, "duas barras tocam-se: {a:?} {b:?}");
            }
        }
    }

    /// ⭐⭐ **A CENA ENSINA O QUE DIZ:** o controlo não tem efeito e cada uma das outras tem um
    /// ACTIVO (um efeito neutro seria uma barra igual ao controlo com um nome a mentir).
    #[test]
    fn cada_barra_com_efeito_tem_um_efeito_activo() {
        assert!(EFEITOS[0].1.is_none(), "a 1.ª barra é o CONTROLO");
        for (nome, f) in &EFEITOS[1..] {
            let f = f.expect("efeito");
            assert!(
                FxEntry::new(f()).is_active(),
                "a barra «{nome}» tem um efeito NEUTRO — a cena mostraria o controlo"
            );
        }
    }

    /// ⭐⭐⭐ **PRESAS, AS BARRAS JÁ NÃO TÊM EFEITOS VIVOS** — o Bind coze-os no desenho (ordem do
    /// dono, 2026-10-03), e o painel recebe «presa» pela mesma pergunta que a shell faz
    /// ([`crate::fx_bridge::is_bound`]). ⛔ O CONTROLO: antes de prender, cada barra com efeito tem-no.
    #[test]
    fn presas_as_barras_ja_nao_tem_efeitos_vivos() {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut st = crate::state::VecState::default();
        build(&mut scene, &mut sim, &mut st);
        ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
        let ids: Vec<_> = st
            .bone_smoke_pend
            .as_ref()
            .expect("o 1.º tempo deixa as barras pendentes")
            .iter()
            .map(|(id, _)| *id)
            .collect();
        let com_efeito = |scene: &VecScene| {
            ids.iter()
                .map(|id| !scene.path(*id).expect("path").effects.is_empty())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            com_efeito(&scene),
            [false, true, true, true, true, true],
            "o CONTROLO"
        );
        assert!(
            ids.iter()
                .all(|id| !crate::fx_bridge::is_bound(&sim, &st.entities, *id))
        );
        bind(&mut scene, &mut sim, &mut st);
        assert_eq!(
            com_efeito(&scene),
            [false; 6],
            "presas, a pilha tem de sair vazia"
        );
        assert!(
            ids.iter()
                .all(|id| crate::fx_bridge::is_bound(&sim, &st.entities, *id)),
            "as seis prenderam e o painel tem de as ver presas"
        );
    }
}

#[cfg(test)]
#[path = "smoke_bone_efeitos_sondas.rs"]
mod sondas;
