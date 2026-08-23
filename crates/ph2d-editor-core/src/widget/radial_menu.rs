//! **O MENU RADIAL** (estudo de UI viva, E4) — *direcção em vez de posição*.
//!
//! # O que ele compra, e por que é medido em segundos
//!
//! Num menu de lista, escolher o 7º item custa mais que o 1º: os olhos percorrem, a mão persegue.
//! Num radial, cada item é uma **DIRECÇÃO** a partir de onde a mão já está — e uma direcção
//! reproduz-se **sem olhar**. O 7º custa o mesmo que o 1º, e a mão **aprende**.
//!
//! É o eixo 3 do estudo (AGÊNCIA), o único que se mede em *segundos por operação* e não em encanto.
//!
//! # ⚠️ OITO, e o número não é meu
//!
//! Uma mão reproduz **oito** direcções sem olhar — os oito pontos da bússola. É por isso que os
//! radiais do Blender, do Maya, do Krita e do Photoshop param nos 8, e não porque 9 não caiba na
//! tela: cabe, e deixa de ser reproduzível. Acima de 8 o radial perde exactamente aquilo que ele
//! existe para comprar.
//!
//! ⛔ **E um item a mais NÃO é truncado em silêncio.** Quando a lista é maior, o último sector
//! passa a ser a porta para a **paleta** (`Ctrl+K`), que segura qualquer número. *Um teto que
//! esconde o que não coube é um teto que mente*; este diz onde o resto está, e há gate.
//!
//! # A ZONA MORTA é o cancelar
//!
//! Soltar sem sair do centro **não escolhe nada**. É o que torna o gesto seguro: chamar o menu e
//! desistir é o mesmo movimento de não o chamar. Sem ela, todo toque no botão do meio escolheria
//! o que estivesse na direcção do último ruído da mão.

use ph2d_a11y::NodeId;

/// Quantos sectores um radial pode ter. Ver o doc do módulo: é o número de direcções que uma mão
/// reproduz **sem olhar**, não um limite de desenho.
pub const MAX_SECTORS: usize = 8;

/// O raio da zona morta, em px de tela. Dentro dele o gesto **cancela**.
///
/// ⚠️ Ele é maior que o ruído de uma mão parada e menor que o primeiro movimento deliberado — e a
/// régua é o próprio menu: a zona morta é **um `Xl3`** (32 px), que é exactamente um terço do raio.
/// Ela escala com o catálogo, e não com um número que eu tenha escolhido.
pub fn dead_zone_px() -> f32 {
    ph2d_tokens::Spacing::Xl3.px()
}

/// O raio em que os rótulos assentam, em px de tela — **dois `Xl4`** (96 px).
///
/// ⚠️ É o menor raio em que oito rótulos de `LABEL_W` cabem sem se tocarem, e a régua é o catálogo:
/// um raio escrito à mão deixaria de acompanhar uma mudança de densidade.
pub fn radius_px() -> f32 {
    ph2d_tokens::Spacing::Xl4.px() * 2.0
}

/// Um item do radial — o mesmo par que a paleta usa, porque é a mesma lista.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadialItem {
    pub label: String,
    /// O id que o router da paleta reconhece. ⚠️ **Nunca um verbo próprio**: o radial é uma VISTA
    /// da lista que o app já oferece, e quem executa é quem já executava.
    pub id: NodeId,
}

/// **O PIE MENU ABERTO** — onde ele nasceu, o que ele oferece, e o que está aceso.
///
/// ⚠️ O `center` é o ponto em que o menu foi **chamado**, e ele não se move: é a origem de todas as
/// direcções, e movê-lo faria a mão perder a referência a meio do gesto.
#[derive(Clone, Debug, PartialEq)]
pub struct RadialOpen {
    pub center: [f32; 2],
    pub items: Vec<RadialItem>,
    /// O sector sob o ponteiro, ou `None` na zona morta (o cancelar).
    pub hot: Option<usize>,
}

/// **QUE SECTOR ESTÁ SOB O PONTEIRO** — a lei pura do radial, sem tela e sem estado.
///
/// `None` = nenhum: ou não há itens, ou o ponteiro está na **zona morta** (o cancelar).
///
/// ⚠️ **A resposta depende só da DIRECÇÃO**, nunca da distância — passada a zona morta, dez px ou
/// duzentos escolhem o mesmo sector. É essa a propriedade inteira: ela é o que faz o 7º item
/// custar o mesmo que o 1º, e o que permite escolher **sem olhar**.
///
/// ⚠️ **O sector 0 aponta para CIMA**, e os demais seguem no sentido horário. Um radial que
/// começasse à direita (o zero trigonométrico) poria o primeiro item — o mais usado — na direcção
/// que a mão menos associa a *"o primeiro"*.
#[must_use]
pub fn sector_at(center: [f32; 2], pointer: [f32; 2], count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let (dx, dy) = (pointer[0] - center[0], pointer[1] - center[1]);
    if dx.hypot(dy) < dead_zone_px() {
        return None;
    }
    // Ângulo medido a partir de CIMA, no sentido horário. Em coordenadas de tela o `y` cresce para
    // baixo, então "para cima" é `-y` — daí o `atan2(dx, -dy)`.
    let mut a = dx.atan2(-dy);
    if a < 0.0 {
        a += std::f32::consts::TAU;
    }
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let step = std::f32::consts::TAU / count as f32;
    // ⚠️ **Meio sector de deslocamento**, e é ele que centra o item na direcção dele: sem isto o
    // sector 0 ocuparia de 0° a 45° e o item desenhado em 0° ficaria na BORDA entre dois sectores
    // — a direcção que o artista aponta seria a mais ambígua do menu.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i = (((a + step * 0.5) % std::f32::consts::TAU) / step) as usize;
    Some(i.min(count - 1))
}

/// **ONDE O ITEM `i` ASSENTA**, em px de tela relativos ao centro — a inversa de [`sector_at`], e a
/// razão de ela existir é que o desenho e o hit-test **têm de concordar sobre onde um item está**.
#[must_use]
pub fn item_offset(i: usize, count: usize) -> [f32; 2] {
    if count == 0 {
        return [0.0, 0.0];
    }
    #[allow(clippy::cast_precision_loss)]
    let a = std::f32::consts::TAU * i as f32 / count as f32;
    // A mesma convenção do `sector_at`: 0 aponta para cima, horário.
    [radius_px() * a.sin(), -radius_px() * a.cos()]
}

/// **DESENHA O PIE MENU** — o disco, os rótulos nas oito direcções, e o aceso.
///
/// ⚠️ **px de TELA, sob `Affine::IDENTITY`** — a lei do [`super::super::widget::radial_menu`] é a do
/// marquee: no Vello o transform do `stroke` multiplica a largura, e este menu vive na tela, não no
/// mundo.
///
/// ⚠️ **O rótulo aceso ganha uma CAIXA, não outra cor.** Uma cor sozinha obriga o olho a comparar
/// oito rótulos para achar o diferente; a caixa é um facto local — vê-se sem varrer.
pub fn paint_radial_menu(
    open: &RadialOpen,
    scene: &mut ph2d_vector::VectorScene,
    text_system: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
) {
    use crate::paint::{fill_rounded_rect, paint_text_centered, resolve, stroke_rounded_rect};
    use crate::zones::Rect;
    use ph2d_tokens::{ColorToken, Radius, StrokeToken, TypeToken};

    let n = open.items.len();
    // O disco do centro: a zona morta, desenhada para o artista SABER onde ela acaba.
    let d = dead_zone_px() * 2.0;
    let hub = Rect::new(
        open.center[0] - dead_zone_px(),
        open.center[1] - dead_zone_px(),
        d,
        d,
    );
    fill_rounded_rect(scene, hub, dead_zone_px(), resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        scene,
        hub,
        dead_zone_px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );

    for (i, item) in open.items.iter().enumerate() {
        let o = item_offset(i, n);
        let (w, h) = (label_w_px(), label_h_px());
        let r = Rect::new(
            open.center[0] + o[0] - w * 0.5,
            open.center[1] + o[1] - h * 0.5,
            w,
            h,
        );
        let lit = open.hot == Some(i);
        fill_rounded_rect(
            scene,
            r,
            Radius::Sm.px(),
            resolve(
                if lit {
                    ColorToken::AccentSoft
                } else {
                    ColorToken::Bg1
                },
                theme,
            ),
        );
        stroke_rounded_rect(
            scene,
            r,
            Radius::Sm.px(),
            StrokeToken::Thin.px(),
            resolve(
                if lit {
                    ColorToken::Accent
                } else {
                    ColorToken::Border
                },
                theme,
            ),
        );
        paint_text_centered(
            text_system,
            scene,
            &item.label,
            r,
            TypeToken::Sm.px(),
            resolve(
                if lit {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
    }
}

/// A caixa de um rótulo, em px de tela — do catálogo, como tudo o resto.
fn label_w_px() -> f32 {
    ph2d_tokens::Spacing::Xl4.px() * 2.0 - ph2d_tokens::Spacing::Xs.px()
}
fn label_h_px() -> f32 {
    ph2d_tokens::Spacing::Xl2.px() + ph2d_tokens::Spacing::Xxs.px()
}

#[cfg(test)]
mod tests {
    //! Os gates da **LEI PURA do radial** — direcção, zona morta, e o acordo entre desenho e hit-test.

    use super::*;

    const C: [f32; 2] = [500.0, 400.0];

    /// Um ponto a `d` px do centro, na direcção do item `i` de `count`.
    fn toward(i: usize, count: usize, d: f32) -> [f32; 2] {
        let o = item_offset(i, count);
        let len = o[0].hypot(o[1]).max(f32::EPSILON);
        [C[0] + o[0] / len * d, C[1] + o[1] / len * d]
    }

    /// ⭐ **O QUE DECIDE É A DIRECÇÃO, NUNCA A DISTÂNCIA.**
    ///
    /// É a propriedade inteira do radial: passada a zona morta, dez px ou duzentos escolhem o mesmo
    /// item. É ela que faz o 7º custar o mesmo que o 1º — e é a única coisa que um menu de lista não
    /// consegue oferecer.
    #[test]
    fn only_the_direction_decides_never_the_distance() {
        for count in [4usize, 8] {
            for i in 0..count {
                for d in [dead_zone_px() + 1.0, 60.0, 200.0, 4000.0] {
                    assert_eq!(
                        sector_at(C, toward(i, count, d), count),
                        Some(i),
                        "count={count} item={i} a {d} px escolheu outro sector — a distância entrou na \
                         resposta, e o radial deixou de ser reproduzível sem olhar"
                    );
                }
            }
        }
    }

    /// ⛔ **A ZONA MORTA CANCELA.**
    ///
    /// ⚠️ Sem ela, chamar o menu e desistir seria impossível: todo toque escolheria o que estivesse na
    /// direcção do último ruído da mão. *Cancelar tem de ser o mesmo movimento de não ter chamado.*
    #[test]
    fn the_dead_zone_chooses_nothing() {
        assert_eq!(sector_at(C, C, 8), None, "o próprio centro escolheu algo");
        for d in [0.0, dead_zone_px() * 0.5, dead_zone_px() - 0.01] {
            for i in 0..8 {
                assert_eq!(
                    sector_at(C, toward(i, 8, d), 8),
                    None,
                    "a {d} px do centro (zona morta = {}) o gesto escolheu o item {i}",
                    dead_zone_px()
                );
            }
        }
        // O CONTROLE: um passo além da zona morta já escolhe — senão o gate acima ficaria verde sobre
        // um radial que nunca escolhe nada.
        assert!(
            sector_at(C, toward(3, 8, dead_zone_px() + 1.0), 8).is_some(),
            "passada a zona morta o gesto tem de escolher"
        );
    }

    /// ⭐ **O DESENHO E O HIT-TEST CONCORDAM SOBRE ONDE UM ITEM ESTÁ.**
    ///
    /// ⚠️ É a razão de o [`item_offset`] existir em vez de cada metade calcular o seu ângulo: um menu
    /// em que o rótulo é desenhado num sítio e escolhido noutro é pior que não haver menu — a mão
    /// aprende a direcção ERRADA, e a memória muscular passa a trabalhar contra o artista.
    #[test]
    fn what_is_drawn_at_a_direction_is_what_is_chosen_there() {
        for count in 1..=MAX_SECTORS {
            for i in 0..count {
                let p = [
                    C[0] + item_offset(i, count)[0],
                    C[1] + item_offset(i, count)[1],
                ];
                assert_eq!(
                    sector_at(C, p, count),
                    Some(i),
                    "count={count}: o item {i} é desenhado onde o sector {:?} é escolhido",
                    sector_at(C, p, count)
                );
            }
        }
    }

    /// **O ITEM 0 APONTA PARA CIMA.**
    ///
    /// ⚠️ Um radial que começasse à direita (o zero trigonométrico) poria o primeiro item — o mais
    /// usado — na direcção que a mão menos associa a *"o primeiro"*. É convenção, e por isso é gate:
    /// convenções mudam por acidente.
    #[test]
    fn the_first_item_points_up() {
        let o = item_offset(0, 8);
        assert!(o[0].abs() < 1e-3, "o item 0 saiu do eixo vertical: {o:?}");
        assert!(
            o[1] < 0.0,
            "o item 0 aponta para BAIXO: {o:?} (em tela, y cresce para baixo)"
        );
        // E o seguinte vai para a direita — o sentido horário.
        assert!(
            item_offset(1, 8)[0] > 0.0,
            "o segundo item não foi para a direita"
        );
    }

    /// **Todo item é alcançável, seja qual for a contagem.**
    ///
    /// ⚠️ O laço cobre 1..=8 de propósito: um radial de UM item é o caso degenerado em que o
    /// arredondamento de sector é mais fácil de escrever errado — ele ocupa a volta inteira.
    #[test]
    fn every_item_is_reachable_at_every_count() {
        for count in 1..=MAX_SECTORS {
            let mut seen = vec![false; count];
            // Varre a volta inteira em passos finos e regista que sector responde.
            for k in 0..3600u16 {
                let a = std::f32::consts::TAU * f32::from(k) / 3600.0;
                let p = [C[0] + 80.0 * a.sin(), C[1] - 80.0 * a.cos()];
                if let Some(i) = sector_at(C, p, count) {
                    seen[i] = true;
                }
            }
            assert!(
                seen.iter().all(|&s| s),
                "count={count}: há item inalcançável — {seen:?}"
            );
        }
    }
}
