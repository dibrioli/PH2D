//! ⭐ **A ARTE DAS TRÊS MÍDIAS** — as três fontes de pixels da cena `=1`. Filho do [`super`] para
//! herdar as constantes dela (`LADO_PX`, `QUADROS`, `BORDA_PX`).
//!
//! ⚠️ **Ele saiu do pai por TECTO DE LOC, e o corte é por RESPONSABILIDADE:** *gerar pixels* e
//! *montar uma cena* são dois assuntos, e o primeiro não conhece `SimWorld`, `Entity` nem o
//! esqueleto. ⛔ Nunca por uma entrada nova no `FILE_OVERAGE_OK` (§5.0).

use super::*;

/// Tinta LISTRADA — o controlo. As listras transversais tornam a dobra legível (uma barra chapada
/// dobrada lê-se quase igual à mesma barra rodada).
pub(super) fn listrada(w: u32, h: u32) -> Vec<u8> {
    let (wu, hu) = (w as usize, h as usize);
    let mut rgba = vec![0u8; wu * hu * 4];
    for y in 0..hu {
        for x in 0..wu {
            let i = (y * wu + x) * 4;
            let escura = (x / 12) % 2 == 0;
            rgba[i] = if escura { 40 } else { 235 };
            rgba[i + 1] = if escura { 90 } else { 235 };
            rgba[i + 2] = if escura { 180 } else { 235 };
            rgba[i + 3] = 255;
        }
    }
    rgba
}

/// A FOLHA: `QUADROS` células de `LADO_PX`, cada uma um DISCO com a mordida num lado diferente.
///
/// ⭐⭐⭐ **A união das quatro é o DISCO INTEIRO, e essa é a razão da forma.** A malha do bind é
/// traçada sobre a UNIÃO dos quadros (é o que impede um quadro de ser recortado ao dar play), então
/// a união é geometria de verdade — e quatro formas SOBREPOSTAS de famílias diferentes (disco,
/// triângulo, cruz, barra) davam uma união com **gargalos finos**, onde o traçador deixa fendas.
///
/// ⛔⛔ **A foto apanhou-o e eu quase o li como defeito do produto:** a arte desenhava-se RASGADA ao
/// meio, e ao afinar a resolução o rasgo ficou **mais** visível — *é assim que se distingue um
/// artefacto do traçador de um defeito da lei: afinar a malha piora um e cura o outro.*
///
/// ⚠️ Cada quadro tira um QUADRANTE diferente, logo nenhum ponto é tirado em mais de um ⇒ a união
/// é exactamente o disco.
pub(super) fn folha() -> Vec<u8> {
    let lado = LADO_PX as usize;
    let w = lado * QUADROS as usize;
    let mut rgba = vec![0u8; w * lado * 4];
    for c in 0..QUADROS as usize {
        let (r, g, b) = match c {
            0 => (230u8, 80, 80),
            1 => (80, 200, 120),
            2 => (90, 140, 240),
            _ => (240, 200, 70),
        };
        for y in 0..lado {
            for x in 0..lado {
                let (dx, dy) = (x as f32 / lado as f32 - 0.5, y as f32 / lado as f32 - 0.5);
                if dx.hypot(dy) >= 0.45 {
                    continue;
                }
                // O quadrante que ESTE quadro tira — `atan2` em `0..4`, a começar à direita.
                let q = ((dy.atan2(dx) / std::f32::consts::FRAC_PI_2).rem_euclid(4.0)) as usize;
                if q == c {
                    continue;
                }
                let i = ((y * w) + c * lado + x) * 4;
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
    }
    rgba
}

/// A MOLDURA do 9-slice: um anel opaco com o miolo TRANSPARENTE, e os cantos marcados.
pub(super) fn moldura() -> Vec<u8> {
    let lado = LADO_PX as usize;
    let b = BORDA_PX as usize;
    let mut rgba = vec![0u8; lado * lado * 4];
    for y in 0..lado {
        for x in 0..lado {
            let no_anel = x < b || y < b || x >= lado - b || y >= lado - b;
            if !no_anel {
                continue;
            }
            // ⭐ O canto é de outra cor: é ele que tem de ficar do MESMO tamanho quando a moldura
            // estica, e sem contraste ninguém vê se ele esticou.
            let canto = (x < b || x >= lado - b) && (y < b || y >= lado - b);
            let i = (y * lado + x) * 4;
            rgba[i] = if canto { 250 } else { 120 };
            rgba[i + 1] = if canto { 170 } else { 130 };
            rgba[i + 2] = if canto { 60 } else { 200 };
            rgba[i + 3] = 255;
        }
    }
    rgba
}
