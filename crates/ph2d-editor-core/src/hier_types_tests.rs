use super::*;
use ph2d_tokens::type_colour::TYPE_ICON_MIN_CONTRAST;
use ph2d_tokens::{Theme, srgb_to_oklch};

/// ⭐ **Cada tipo tem o SEU glifo** — e o caminho de volta (ícone → tipo) acerta todos. (Mutação: dar
/// ao `Body` o glifo do `Collider` ⇒ dois tipos com o mesmo ícone — RED.)
#[test]
fn cada_tipo_tem_icone_unico() {
    for (i, a) in ObjectType::ALL.iter().enumerate() {
        for b in &ObjectType::ALL[i + 1..] {
            assert_ne!(a.icon(), b.icon(), "{a:?} e {b:?} partilham o glifo");
        }
        assert_eq!(ObjectType::of_icon(a.icon()), Some(*a));
    }
    assert_eq!(
        ObjectType::of_icon(IconId::Layers),
        None,
        "o conteúdo não tem tipo"
    );
}

fn oklab(t: ColorToken, theme: Theme) -> [f64; 3] {
    let c = t.resolve(theme);
    let (l, ch, h) = srgb_to_oklch(c.r, c.g, c.b);
    let h = h.to_radians();
    [l, ch * h.cos(), ch * h.sin()]
}

/// ⭐ **Cada tipo tem a SUA cor, e lê-se** em todos os temas: contraste ≥ 4,5 sobre os três fundos
/// nos quatro clássicos, ≥ 3 (WCAG 1.4.11) nos modernos, e nenhum par mais perto que ΔE OKLab 0,05. (Medido na F42: escuro ≥ 0,071, claro ≥ 0,063.
/// Mutação: `obj-collider` = `obj-player` no `tokens.json` ⇒ distância 0 — RED.)
#[test]
fn cada_tipo_tem_cor_propria_e_legivel() {
    for theme in Theme::ALL {
        for t in ObjectType::ALL {
            let fg = t.color().resolve(theme);
            for bg in [ColorToken::Bg0, ColorToken::Bg1, ColorToken::Bg2] {
                let r = bg.resolve(theme).contrast_ratio(&fg);
                let min = if Theme::CLASSIC.contains(&theme) {
                    4.5
                } else {
                    TYPE_ICON_MIN_CONTRAST
                };
                assert!(r >= min, "{t:?} sobre {bg:?} em {theme:?}: {r:.2}");
            }
        }
        for (i, a) in ObjectType::ALL.iter().enumerate() {
            for b in &ObjectType::ALL[i + 1..] {
                let (pa, pb) = (oklab(a.color(), theme), oklab(b.color(), theme));
                let d = pa
                    .iter()
                    .zip(pb)
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f64>()
                    .sqrt();
                assert!(d >= 0.05, "{a:?} e {b:?} em {theme:?}: ΔE {d:.3}");
            }
        }
    }
}

/// ⭐ **O tipo vem do marcador**: o jogador ganha ao tipo do desenho; num vazio, a precedência é a
/// de `ALL`; o componente definidor NÃO muda o tipo de um objecto com arte. (Mutação: tirar o ramo
/// do `Player` ⇒ o sprite do jogador lê-se como imagem — RED.)
#[test]
fn o_tipo_vem_do_marcador() {
    fn com<'a>(nomes: &'a [&'static str]) -> impl Fn(&'static str) -> bool + 'a {
        move |n| nomes.contains(&n)
    }
    let player = "ph2d::physics::PlatformPlayer";
    let body = "ph2d::physics::RigidBody";
    let collider = "ph2d::physics::Collider";
    let camera = "ph2d::ecs::GameCamera";
    assert_eq!(
        ObjectType::of(ObjectKind::Image, com(&[player, body])),
        ObjectType::Player
    );
    assert_eq!(
        ObjectType::of(ObjectKind::Image, com(&[body, collider])),
        ObjectType::Image
    );
    assert_eq!(
        ObjectType::of(ObjectKind::Empty, com(&[collider, body])),
        ObjectType::Body
    );
    assert_eq!(
        ObjectType::of(ObjectKind::Empty, com(&[collider])),
        ObjectType::Collider
    );
    assert_eq!(
        ObjectType::of(ObjectKind::Empty, com(&[body, camera])),
        ObjectType::Camera
    );
    assert_eq!(
        ObjectType::of(ObjectKind::Empty, com(&[])),
        ObjectType::Empty
    );
    assert_eq!(
        ObjectType::of(ObjectKind::PixelSprite, com(&[])),
        ObjectType::PixelSprite
    );
    // Só o vazio é Empty.
    for k in ObjectKind::ALL {
        let t = ObjectType::of(k, com(&[]));
        assert_eq!(t == ObjectType::Empty, k == ObjectKind::Empty, "{k:?}");
        // A câmera é o único ObjectKind cujo marcador é também o do tipo da hierarquia (GEF 25).
        assert_eq!(t.marker().is_none(), k != ObjectKind::Camera, "{k:?}");
    }
}

/// ⭐ Dono 06/11: dois objectos de pintura, dois ícones. A imagem pintada é a imagem, com o ícone da tela; o do
/// Painter é da ferramenta, nunca de um objecto. *Mutação: `ObjectKind::Painted` ler-se como outro tipo ⇒ RED.*
#[test]
fn a_imagem_pintada_e_uma_imagem_e_o_pixel_sprite_e_outro() {
    let nada = |_: &'static str| false;
    assert_eq!(ObjectType::of(ObjectKind::Painted, nada), ObjectType::Image);
    assert_eq!(ObjectType::Image.icon(), IconId::Sprite);
    assert_eq!(ObjectType::PixelSprite.icon(), IconId::PixelSprite);
    assert_eq!(ObjectType::of_icon(IconId::Painter), None);
}
