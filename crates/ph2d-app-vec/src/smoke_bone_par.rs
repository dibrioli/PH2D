//! ⭐⭐⭐ **O PAR — a MESMA barra como DESENHO e como IMAGEM** — `PH2D_VEC_BONE_SMOKE=3` (ordem do
//! dono, 2026-09-29: *«monte a cena com apenas dois esqueletos iguais, um com vector outro com
//! img»*).
//!
//! # Porque esta cena existe
//!
//! O report anterior dele (*«o osso do meio provoca ondulações discretas, que diferem da deformação
//! de imagens»*) só se julga com as duas mídias **lado a lado e em condições iguais** — e a cena
//! `=1` não as dá: lá o braço vectorial tem `7 × 1` m e o pintado `3,2 × 0,96`, com esqueletos de
//! tamanhos diferentes e sítios diferentes do ecrã. *Duas peças que diferem em tudo não dizem qual
//! diferença é da mídia.*
//!
//! # O que a cena monta, e o que é IGUAL por construção
//!
//! | | em cima | em baixo |
//! |---|---|---|
//! | a peça | um `RoundRect` **vectorial** | uma **imagem** pintada da mesma cápsula |
//! | o tamanho | [`peca`] — UMA porta para os dois | idem |
//! | a cor | [`COR`] | idem |
//! | o esqueleto | [`OSSOS`] ossos pelo eixo da cápsula | idem, deslocado só em `y` |
//! | a pose | [`DOBRA`] em Z, aplicada DEPOIS de prender | idem |
//!
//! ⚠️ **A dobra é aplicada no 2.º tempo, depois de prender**, e é load-bearing: prender fotografa a
//! pose de REPOUSO, e com o esqueleto já dobrado as duas peças aprenderiam o dobrado como repouso e
//! abririam RECTAS.
//!
//! ⚠️ **Nada mais na cena, de propósito** — o dono pediu *«apenas»*. Sem folha solta, sem
//! bifurcação, sem barra por cima: *uma cena de comparação com uma terceira peça obriga o olho a
//! decidir o que comparar*.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// Quantos ossos cada esqueleto tem — os três do braço da cena `=1`, que é a corrente em que o
/// report nasceu.
pub(crate) const OSSOS: u32 = 3;

/// ⭐ **A dobra por junta, em graus, em Z** (o 2.º osso sobe, o 3.º volta a deitar) — a forma da
/// pose da foto do dono.
///
/// ⚠️ `40°` e não os `25°` da família das mídias: ali a pergunta é *«dobram?»*; aqui é *«dobram
/// IGUAL?»*, e a diferença entre as duas mídias (a transição entre ossos) só tem tamanho para se ver
/// com a dobra franca. ⛔ Acima de `~90°` o vinco de dentro do cotovelo nasce nas DUAS mídias
/// (`det J`, pesquisa 04 §1.3), e a cena passaria a mostrar esse, que é outra pergunta.
pub(crate) const DOBRA: f32 = 40.0;

/// A cor das duas peças — a do braço da cena `=1`.
pub(crate) const COR: [u8; 3] = [230, 170, 90];

/// A arte da imagem, em pixels. ⚠️ O tamanho no MUNDO sai daqui pelo `ppm` do projecto ([`peca`]),
/// e o desenho vectorial copia esse tamanho — logo os dois são iguais em qualquer `ppm`.
pub(crate) const IMG_W: u32 = 600;
/// Ver [`IMG_W`].
pub(crate) const IMG_H: u32 = 100;

/// ⭐ **O afastamento vertical entre os dois centros, em múltiplos da espessura da peça.**
///
/// ⚠️ A cena abre na **câmera de omissão** (o `Frame All` só conta imagens e enquadraria a de baixo
/// sozinha — ver o prólogo em [`crate::smoke_bone_envelope::prologo_do_nivel`]), e este número põe
/// as duas peças dobradas com um vão de `~1,5` espessuras entre elas, dentro dela (gate [`tests`]).
pub(crate) const AFASTAMENTO: f64 = 3.6;

/// ⭐⭐ **O tamanho de UMA peça no mundo** — `(largura, espessura)`, a porta que as DUAS leem.
#[must_use]
pub(crate) fn peca(ppm: f32) -> (f64, f64) {
    let ppm = f64::from(ppm.max(1e-3));
    (f64::from(IMG_W) / ppm, f64::from(IMG_H) / ppm)
}

/// Os dois centros (`y`) — o desenho em cima, a imagem em baixo.
#[must_use]
pub(crate) fn centros(ppm: f32) -> [f64; 2] {
    let (_, t) = peca(ppm);
    let d = AFASTAMENTO * t / 2.0;
    [d, -d]
}

/// A arte da imagem: a cápsula deitada, cor chapada, borda suave de um pixel.
///
/// ⚠️ **Cor chapada e não listras**, ao contrário do braço da `=1`: aqui o que se compara é a
/// SILHUETA com a do desenho, que também é chapado — *listras só de um lado seriam uma diferença que
/// não é da deformação*.
fn pixels() -> Vec<u8> {
    let (w, h) = (f64::from(IMG_W), f64::from(IMG_H));
    let raio = h / 2.0;
    let (ax, bx) = (raio, w - raio);
    let mut px = vec![0u8; (IMG_W * IMG_H * 4) as usize];
    for y in 0..IMG_H {
        for x in 0..IMG_W {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            let d = (p[0] - p[0].clamp(ax, bx)).hypot(p[1] - raio);
            let cobertura = (raio - d + 0.5).clamp(0.0, 1.0);
            if cobertura <= 0.0 {
                continue;
            }
            let i = ((y * IMG_W + x) * 4) as usize;
            px[i..i + 3].copy_from_slice(&COR);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "cobertura em [0, 1]"
            )]
            let a = (cobertura * 255.0).round() as u8;
            px[i + 3] = a;
        }
    }
    px
}

/// Um esqueleto de [`OSSOS`] ossos pelo EIXO da cápsula centrada em `y`, recto (o repouso), com os
/// ossos nomeados `«{prefixo} bone k»` — o nome que o roteiro manda escolher na Hierarquia.
fn esqueleto(sim: &mut SimWorld, ppm: f32, y: f64, prefixo: &str) -> Option<Entity> {
    let (l, t) = peca(ppm);
    // ⚠️ Das pontas do EIXO, e não das pontas da peça: a tampa redonda é meia espessura de cada lado.
    let x0 = -l / 2.0 + t / 2.0;
    let passo = (l - t) / f64::from(OSSOS);
    let mut pai = None;
    let mut raiz = None;
    for k in 0..OSSOS {
        let a = [x0 + passo * f64::from(k), y];
        let b = [x0 + passo * f64::from(k + 1), y];
        let e = Entity::try_from_bits(ph2d_skeleton_live::bone::create(sim, pai, a, b)?)?;
        sim.world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::Name::new(format!("{prefixo} bone {}", k + 1)));
        raiz.get_or_insert(e);
        pai = Some(e);
    }
    raiz
}

/// O 1.º tempo: as duas peças e os dois esqueletos, rectos.
pub(crate) fn build(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    assets: &mut ph2d_asset::AssetDb,
    ppm: f32,
    st: &mut crate::state::VecState,
) {
    let (l, t) = peca(ppm);
    let [y_vec, y_img] = centros(ppm);
    let desenho = scene.push_path(shape(
        ShapeKind::RoundRect,
        [-l / 2.0, y_vec - t / 2.0],
        [l / 2.0, y_vec + t / 2.0],
        &[t / 2.0],
        COR,
    ));
    let osso_vec = esqueleto(sim, ppm, y_vec, "Vector");
    let px = pixels();
    st.bone_smoke_img = match renderer.acquire_individual(IMG_W, IMG_H, &px) {
        Ok(texture_id) => {
            let pixels_id = assets.insert_image_rgba8(IMG_W, IMG_H, px);
            #[expect(clippy::cast_possible_truncation, reason = "metros de uma cena")]
            let (_, bits) = ph2d_image_import::spawn_sprite(
                sim,
                ph2d_image_import::PackedSource::Individual {
                    texture_id,
                    pixels_id,
                },
                ph2d_core::Vec2::new(0.0, y_img as f32),
                [l as f32, t as f32],
                "Image",
            );
            Some((bits, esqueleto(sim, ppm, y_img, "Image")))
        }
        Err(e) => {
            eprintln!("[vec-bone-smoke] PARE: a imagem nao subiu para a GPU: {e}");
            None
        }
    };
    st.bone_smoke_pend = Some(vec![(desenho, osso_vec)]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende as duas peças com os esqueletos RECTOS e só depois dobra-os — iguais.
pub(crate) fn bind(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    assets: &ph2d_asset::AssetDb,
    ppm: f32,
    st: &mut crate::state::VecState,
) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let mut raizes = Vec::new();
    let mut vector_preso = false;
    for (id, raiz) in &pecas {
        vector_preso |=
            ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[*id], *raiz) > 0;
        if let Some(e) = st.entities.get(id).and_then(|b| Entity::try_from_bits(*b)) {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new("Vector"));
        }
        raizes.extend(*raiz);
    }
    let mut imagem_presa = false;
    if let Some((bits, raiz)) = st.bone_smoke_img
        && let Some(e) = Entity::try_from_bits(bits)
    {
        let arte = sim
            .world()
            .get::<ph2d_ecs::SpritePixels>(e)
            .map(|p| p.0)
            .and_then(|id| assets.get(&id));
        imagem_presa = arte
            .as_ref()
            .and_then(|a| a.image_rgba8())
            .is_some_and(|(w, h, cow)| {
                ph2d_skeleton_live::skin_image_bind::bind_image(
                    sim,
                    e,
                    &cow,
                    [w, h],
                    ppm,
                    ph2d_poly2d::GridOptions::default(),
                    raiz,
                )
            });
        raizes.extend(raiz);
    }
    for raiz in &raizes {
        dobra(sim, *raiz);
    }
    anuncia(vector_preso, imagem_presa);
}

/// ⭐ **A MESMA pose nos dois**: o 2.º osso sobe [`DOBRA`], o 3.º volta a deitar — em Z.
pub(crate) fn dobra(sim: &mut SimWorld, raiz: Entity) {
    let mut e = raiz;
    let mut k = 0;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
            .copied()
    }) {
        e = f;
        k += 1;
        let sinal = if k % 2 == 1 { 1.0 } else { -1.0 };
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
            t.rotation += sinal * DOBRA.to_radians();
        }
    }
}

fn anuncia(vector_preso: bool, imagem_presa: bool) {
    if !vector_preso || !imagem_presa {
        eprintln!(
            "[vec-bone-smoke] PARE: o desenho {} e a imagem {} -- a cena nao montou",
            if vector_preso {
                "prendeu"
            } else {
                "NAO prendeu"
            },
            if imagem_presa {
                "prendeu"
            } else {
                "NAO prendeu"
            },
        );
    }
    println!(
        "[vec-bone-smoke] O PAR: a MESMA barra duas vezes, com o MESMO tamanho, a MESMA cor e um \
         esqueleto IGUAL de {OSSOS} ossos em cada. Em cima e' um DESENHO VECTORIAL («Vector»), em \
         baixo uma IMAGEM («Image»). Os dois esqueletos abrem dobrados da MESMA forma ({DOBRA}° por \
         junta, em Z).\n\
         [vec-bone-smoke] 1) Compare as duas barras: a forma das bordas, sobretudo junto das duas \
         juntas. O que for diferente entre elas e' a diferenca entre as duas midias.\n\
         [vec-bone-smoke] 2) Para outra pose, IGUAL nas duas: na Hierarquia clique em «Vector bone \
         2», e no Inspector escreva um numero no campo de rotacao. Depois clique em «Image bone 2» e \
         escreva o MESMO numero. Faca o mesmo com o osso 3 se quiser.\n\
         [vec-bone-smoke] 3) Tambem pode arrastar os ossos com a ferramenta Bone (painel Bones, \
         «Transform»), mas a mao nao repete o mesmo angulo nas duas -- para comparar, use os \
         numeros."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A caixa de uma peça DOBRADA, em mundo — os pontos da cadeia com a pose da cena, alargados
    /// pela meia espessura (a cápsula à volta do eixo).
    fn caixa_dobrada(ppm: f32, y: f64) -> [f64; 4] {
        let (l, t) = peca(ppm);
        let passo = (l - t) / f64::from(OSSOS);
        let mut p = [-l / 2.0 + t / 2.0, y];
        let mut ang = 0.0_f64;
        let mut caixa = [p[0], p[1], p[0], p[1]];
        for k in 0..OSSOS {
            if k > 0 {
                let sinal = if k % 2 == 1 { 1.0 } else { -1.0 };
                ang += sinal * f64::from(DOBRA).to_radians();
            }
            p = [p[0] + passo * ang.cos(), p[1] + passo * ang.sin()];
            caixa = [
                caixa[0].min(p[0]),
                caixa[1].min(p[1]),
                caixa[2].max(p[0]),
                caixa[3].max(p[1]),
            ];
        }
        let r = t / 2.0;
        [caixa[0] - r, caixa[1] - r, caixa[2] + r, caixa[3] + r]
    }

    /// ⭐⭐⭐ **O PAR CABE NA CÂMERA DE OMISSÃO e as duas peças não se tocam.**
    ///
    /// A câmera de omissão mostra `height_world` de alto, e a largura VISÍVEL é a do canvas entre
    /// as duas colunas laterais, que tapam `~37 %` da janela (medido pela família das mídias) — com
    /// uma janela `16:9` isso são `height × 16/9 × 0,63`. A barra exige `90 %` de cada um, para a
    /// peça não encostar à borda. ⚠️ No `ppm` de omissão do projecto, que é o da cena do dono.
    #[test]
    fn o_par_cabe_na_camera_de_omissao_e_nao_se_toca() {
        let ppm = ph2d_editor_core::DEFAULT_PIXELS_PER_METER;
        let cam = ph2d_render::Camera2d::default();
        let [y_vec, y_img] = centros(ppm);
        let (a, b) = (caixa_dobrada(ppm, y_vec), caixa_dobrada(ppm, y_img));
        let (x0, x1) = (a[0].min(b[0]), a[2].max(b[2]));
        let (y0, y1) = (a[1].min(b[1]), a[3].max(b[3]));
        let h = f64::from(cam.height_world);
        let meia_largura = h * 16.0 / 9.0 * 0.63 * 0.9 / 2.0;
        let meia_altura = h * 0.9 / 2.0;
        for (v, lim, eixo) in [
            (x0.abs().max(x1.abs()), meia_largura, "x"),
            (y0.abs().max(y1.abs()), meia_altura, "y"),
        ] {
            assert!(
                v <= lim,
                "o par vai a {v:.3} m do centro em {eixo}, e a camera de omissao mostra {lim:.3} — \
                 uma das pecas sai do ecra'"
            );
        }
        {
            assert!(
                b[3] < a[1],
                "a imagem (topo {:.3}) toca o desenho (base {:.3}) — a comparacao exige as duas \
                 SEPARADAS",
                b[3],
                a[1]
            );
        }
    }

    /// ⭐⭐ **As duas peças TÊM O MESMO TAMANHO** — o desenho copia o da imagem pela mesma porta, e
    /// a arte da imagem enche o quadro dela de ponta a ponta (sem margem, que a tornaria mais curta).
    #[test]
    fn as_duas_pecas_tem_o_mesmo_tamanho() {
        let px = pixels();
        let alfa = |x: u32, y: u32| px[((y * IMG_W + x) * 4 + 3) as usize];
        // O eixo da cápsula toca as duas bordas: o píxel do meio de cada ponta é (quase) opaco.
        assert!(alfa(0, IMG_H / 2) > 100 && alfa(IMG_W - 1, IMG_H / 2) > 100);
        // E a espessura é a altura inteira da imagem.
        assert!(alfa(IMG_W / 2, 0) > 100 && alfa(IMG_W / 2, IMG_H - 1) > 100);
        // ⛔ E fora da tampa é transparente — senão a imagem seria um rectângulo e não a cápsula.
        assert_eq!(alfa(0, 0), 0);
    }

    /// ⭐ **O prólogo desta cena fecha a timeline e abre o painel dos ossos — e NÃO enquadra.**
    ///
    /// ⛔ A metade negativa é a que a foto de 2026-09-29 escreveu: o `Frame All` conta só imagens e
    /// enquadrou a peça de baixo sozinha, com o desenho de cima fora do ecrã.
    #[test]
    fn o_prologo_do_par_fecha_a_timeline_e_nao_enquadra() {
        let p = crate::smoke_bone_envelope::prologo_do_nivel(3);
        assert!(
            p.timeline_fechada && p.painel_do_osso,
            "a cena do PAR abre sem o prologo de que precisa: {p:?}"
        );
        assert!(
            !p.enquadrar,
            "a cena do PAR voltou a pedir o Frame All, que enquadra so' a IMAGEM: {p:?}"
        );
    }
}
