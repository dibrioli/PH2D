//! ⭐⭐⭐⭐ **A LENTE, e o que dá à paralela um valor ABSOLUTO** — irmão (`#[path]`) do
//! [`super`].
//!
//! ⛔⛔ **Os dois gates daqui nasceram de DUAS MUTAÇÕES SOBREVIVENTES** (2026-09-21): dobrar o
//! `view_height() * 0.5` **da projecção** e **do lançador de raios** passava os `97` testes da
//! crate. A causa é estrutural e vale para toda lente que alguém acrescente: tudo o que havia
//! sobre a paralela ou era uma **RELAÇÃO** (o recorte medido contra a vista inteira — cega a um
//! factor comum aos dois lados) ou uma propriedade de **FORMA** (*sob raios paralelos o que varia
//! com o pixel é a origem, não a direcção* — cega à escala). *Uma paridade entre duas contas que
//! partilham o erro concorda com ele.*
//!
//! ⚠️ **O corte deste ficheiro foi o tecto de LOC** (700), e a cura é CORTE — nunca uma entrada no
//! `FILE_OVERAGE_OK`. O corte é por ASSUNTO: lá o que a câmera é, aqui **com que lente ela olha**.

use super::*;

/// ⭐⭐⭐⭐ **AS DUAS LENTES COINCIDEM EXACTAMENTE NO PLANO DO ALVO** — a lei que dá um valor
/// ABSOLUTO à escala da lente paralela.
///
/// ⛔⛔ **Ela nasceu de DUAS MUTAÇÕES SOBREVIVENTES, e a causa é estrutural:** dobrar o
/// `view_height() * 0.5` da projecção **e** o do lançador de raios passava as `97` — porque tudo o
/// que havia sobre a lente paralela ou era uma **RELAÇÃO** (o recorte contra a vista inteira, que é
/// cega a um factor comum aos dois lados) ou uma propriedade de FORMA (a origem varia, a direcção
/// não). *Uma paridade entre duas contas que partilham o erro concorda com ele.*
///
/// ⭐ A régua é a que o modelador implícito desta casa já ship e a que o Blender usa: **a
/// meia-extensão da paralela é `distance · tan(fov/2)`**, logo um ponto à distância do alvo cai no
/// MESMO ponto de tela nas duas lentes. Tudo o resto — perto e longe — tem de divergir, e a segunda
/// metade afirma-o: *sem ela, «as duas coincidem» seria satisfeito por `ortho == perspectiva`.*
#[test]
fn as_duas_lentes_coincidem_no_plano_do_alvo() {
    const ASPECT: f32 = 16.0 / 9.0;
    let base = Camera3d {
        distance: 4.0,
        ..Camera3d::default()
    };
    let ortho = Camera3d {
        lens: crate::Lens::Ortho,
        ..base
    };
    // O plano do alvo: quatro pontos que vivem NELE, deslocados no plano da tela.
    let (right, up) = {
        let v = (base.target - base.eye()).normalize();
        let r = v.cross(glam::Vec3::Y).normalize();
        (r, r.cross(v).normalize())
    };
    let ndc = |c: &Camera3d, p: glam::Vec3| {
        let clip = c.proj(ASPECT) * c.view() * p.extend(1.0);
        glam::Vec2::new(clip.x / clip.w, clip.y / clip.w)
    };
    let meia = base.view_height() * 0.5;
    for (nome, du, dv) in [
        ("centro", 0.0, 0.0),
        ("borda de cima", 0.0, 0.9),
        ("canto", 0.8, -0.7),
    ] {
        let p = base.target + right * (du * meia * ASPECT) + up * (dv * meia);
        let (a, b) = (ndc(&base, p), ndc(&ortho, p));
        assert!(
            (a - b).length() < 1e-5,
            "no plano do alvo as duas lentes discordam no {nome}: {a:?} contra {b:?} — a \
             meia-extensao da paralela deixou de ser `distance * tan(fov/2)`"
        );
    }
    // ⛔ **O CONTROLO, e é ele que impede a lei de ser satisfeita por «as duas são iguais»:** fora
    // do plano do alvo elas TÊM de divergir — é isso que a lente paralela É.
    let longe = base.target + (base.target - base.eye()).normalize() * 2.0 + up * (0.9 * meia);
    let (a, b) = (ndc(&base, longe), ndc(&ortho, longe));
    assert!(
        (a - b).length() > 0.1,
        "fora do plano do alvo as duas lentes dao o MESMO ponto ({a:?} contra {b:?}): a paralela \
         esta' a convergir, e o chip nao muda a imagem"
    );
}

/// ⭐⭐⭐ **O RAIO DE UM PIXEL ACERTA NO MESMO SÍTIO DO PLANO DO ALVO NAS DUAS LENTES** — a metade
/// irmã, e ela existe porque a meia-extensão está escrita em DOIS sítios.
///
/// ⛔⛔ **A segunda mutação sobrevivente foi esta:** dobrar o `view_height() * 0.5` do
/// [`Camera3d::ray_through`] passava as `97`, porque a única propriedade que havia sobre ele era de
/// FORMA (*sob raios paralelos o que varia com o pixel é a ORIGEM*) e uma forma é cega à escala.
/// ⚠️ *Um pick que mira ao lado é o defeito de família que esta casa nomeia como «o lugar onde o
/// mouse toca não corresponde ao local na malha»* — e ele não tinha régua na lente nova.
///
/// ⭐ A régua é a mesma lei de cima: no plano do alvo as duas lentes vêem o mesmo, logo o raio de um
/// pixel tem de furar aquele plano no mesmo ponto — e é isso que faz o pincel cair onde o artista
/// aponta depois de trocar de lente.
#[test]
fn o_raio_de_um_pixel_fura_o_plano_do_alvo_no_mesmo_ponto() {
    const SIZE: (u32, u32) = (1600, 900);
    let base = Camera3d {
        distance: 4.0,
        ..Camera3d::default()
    };
    let ortho = Camera3d {
        lens: crate::Lens::Ortho,
        ..base
    };
    // A normal do plano do alvo é a direcção da vista; furá-lo é resolver um `t`.
    let n = (base.target - base.eye()).normalize();
    let fura = |c: &Camera3d, px: f32, py: f32| {
        let r = c.ray_through(px, py, SIZE);
        let o = glam::Vec3::from(r.origin());
        let d = glam::Vec3::from(r.dir()).normalize();
        let t = (base.target - o).dot(n) / d.dot(n);
        o + d * t
    };
    for (nome, px, py) in [
        ("centro", 800.0, 450.0),
        ("canto de cima", 120.0, 60.0),
        ("borda da direita", 1560.0, 450.0),
    ] {
        let (a, b) = (fura(&base, px, py), fura(&ortho, px, py));
        assert!(
            (a - b).length() < 1e-4,
            "o raio do pixel do {nome} fura o plano do alvo em sitios diferentes: {a:?} contra \
             {b:?} — o pick da lente paralela mira ao lado do que ela desenha"
        );
    }
    // ⛔ **O CONTROLO:** fora do plano do alvo os dois raios TÊM de se afastar — senão a lente
    // paralela está a convergir e o gate acima seria satisfeito por `ortho == perspectiva`.
    let (a, b) = (
        glam::Vec3::from(base.ray_through(120.0, 60.0, SIZE).origin()),
        glam::Vec3::from(ortho.ray_through(120.0, 60.0, SIZE).origin()),
    );
    assert!(
        (a - b).length() > 0.1,
        "as duas lentes lancam o raio da MESMA origem ({a:?} contra {b:?}): a paralela nao e' \
         paralela"
    );
}
