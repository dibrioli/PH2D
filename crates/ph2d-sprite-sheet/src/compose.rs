//! **COMPOR** — a folha nasce dos retângulos que o CHAMADOR deu.
//!
//! Irmão do [`crate::pack`], e a divisão entre os dois é a pergunta que cada um responde: o `pack`
//! decide **onde** cada peça fica e depois compõe; este só honra o que já foi decidido.
//!
//! ⚠️ **É a metade que o BAKE precisa.** As peças de uma folha no canvas estão onde o **artista**
//! as arrastou; re-arranjá-las na hora de assar desfaria o trabalho dele em silêncio. *Quem
//! escolhe as posições e quem as honra são duas perguntas, e só a primeira tem opinião.*
//!
//! O `pack` chama-o: um laço de `blit` no projeto, não dois — o segundo é o que ganharia a
//! correção de borda que o primeiro não teria.
//!
//! ⚠️ Saiu do `pack.rs` por medição (2026-08-19): com esta função e os seus testes, aquele ficheiro
//! media 774 linhas contra um teto de 700, e a cura de um teto é o **corte para o irmão**
//! (`feedback_loc_cap_split_not_allowlist_and_fmt_reexpands`).

use crate::AuthoredSheet;
use crate::pack::{PackError, PackInput, blit};

/// **Compõe** uma folha `size × size` com cada entrada no canto que o chamador mandou.
///
/// ⚠️ **É a metade que o BAKE precisa, e a razão de ela existir separada do [`pack`]:** o `pack`
/// decide *onde* e depois compõe; o bake não pode decidir — as peças estão onde o **artista** as
/// arrastou no canvas, e re-arranjá-las na hora de assar desfaria o trabalho dele em silêncio.
/// *Quem escolhe as posições e quem as honra são duas perguntas, e só a primeira tem opinião.*
///
/// O `pack` passou a chamá-la: um laço de `blit` no projeto, não dois — o segundo é o que ganharia
/// a correção de borda que o primeiro não teria.
///
/// A folha nasce **transparente**, e é isso que faz o vão entre regiões ser vão em vez de lixo.
///
/// ⚠️ Recusa (em vez de cortar) uma peça que caia fora: um `blit` que corta produz uma folha em
/// que a região declarada no `.json` **não contém** o que o `.png` mostra, e esse é o defeito que
/// só aparece no consumidor, meses depois. O [`AuthoredSheet::validate`] recusaria na mesma no
/// save; recusar aqui nomeia a peça.
pub fn compose(
    id: u32,
    sheet_name: String,
    size: u32,
    inputs: Vec<PackInput>,
    at: &[[u32; 2]],
) -> Result<AuthoredSheet, PackError> {
    if inputs.is_empty() {
        return Err(PackError::Empty);
    }
    let mut places: Vec<(String, [u32; 4])> = Vec::with_capacity(inputs.len());
    for (i, input) in inputs.iter().enumerate() {
        let expected = (input.width as usize)
            .saturating_mul(input.height as usize)
            .saturating_mul(4);
        if input.rgba.len() != expected {
            return Err(PackError::PixelCountMismatch {
                name: input.name.clone(),
                expected,
                found: input.rgba.len(),
            });
        }
        // Sem posição para esta peça, a origem: o chamador enganou-se na contagem, e assar no
        // canto é mais fácil de ver do que ignorar a peça.
        let [x, y] = at.get(i).copied().unwrap_or([0, 0]);
        let rect = [x, y, input.width, input.height];
        // Soma em `u64`: `x + w` em `u32` daria a volta e um retângulo absurdo passaria a "caber".
        if u64::from(x) + u64::from(input.width) > u64::from(size)
            || u64::from(y) + u64::from(input.height) > u64::from(size)
        {
            return Err(PackError::OutsideSheet {
                name: input.name.clone(),
                rect,
                size,
            });
        }
        // ⚠️ **DUAS PEÇAS NO MESMO PIXEL RECUSAM, e é uma recusa que se paga a si própria.** O
        // `blit` copia — não mistura —, então a segunda apagaria a borda da primeira em silêncio,
        // e o `.json` continuaria a declarar as duas regiões inteiras. O artista veria uma peça
        // com um lado comido e nada a explicá-lo.
        //
        // ⚠️ **E o caso REAL não é o arrasto grosseiro, é o ARREDONDAMENTO:** o chamador mede as
        // caixas em unidades contínuas e converte para pixels aqui; duas peças que apenas se
        // tocam podem, depois de arredondadas, partilhar uma coluna. Por isso a verificação vive
        // em PIXELS — que é a unidade em que a folha existe — e não em quem lhe passou os rects.
        for (name, other) in &places {
            let [ox, oy, ow, oh] = *other;
            if x < ox + ow && ox < x + input.width && y < oy + oh && oy < y + input.height {
                return Err(PackError::Overlap {
                    a: name.clone(),
                    b: input.name.clone(),
                });
            }
        }
        places.push((input.name.clone(), rect));
    }
    let mut rgba = vec![0u8; (size as usize) * (size as usize) * 4];
    for (i, input) in inputs.iter().enumerate() {
        let [x, y] = at.get(i).copied().unwrap_or([0, 0]);
        blit(&mut rgba, size, input, x, y);
    }
    Ok(AuthoredSheet::new(id, sheet_name, size, size, rgba, places))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::{PackOptions, pack};

    fn solid(name: &str, w: u32, h: u32, v: u8) -> PackInput {
        PackInput {
            name: name.to_string(),
            width: w,
            height: h,
            rgba: vec![v; (w as usize) * (h as usize) * 4],
        }
    }

    /// O píxel `(x, y)` da folha, como quatro bytes.
    fn px(sheet: &AuthoredSheet, x: u32, y: u32) -> [u8; 4] {
        let i = ((y as usize) * (sheet.width as usize) + x as usize) * 4;
        [
            sheet.rgba[i],
            sheet.rgba[i + 1],
            sheet.rgba[i + 2],
            sheet.rgba[i + 3],
        ]
    }

    /// **A posição é a do CHAMADOR** — é isto que separa o `compose` do `pack`, e o que faz o bake
    /// honrar o arranjo que o artista fez à mão em vez de o refazer.
    #[test]
    fn compose_honours_the_given_corners() {
        let sheet = compose(
            7,
            "hero".into(),
            16,
            vec![solid("a", 4, 4, 0x11), solid("b", 2, 2, 0x22)],
            &[[0, 0], [10, 12]],
        )
        .expect("cabe");
        assert_eq!(sheet.width, 16);
        assert_eq!(px(&sheet, 0, 0), [0x11; 4]);
        assert_eq!(px(&sheet, 3, 3), [0x11; 4]);
        assert_eq!(px(&sheet, 10, 12), [0x22; 4]);
        assert_eq!(px(&sheet, 11, 13), [0x22; 4]);
    }

    /// O vão nasce **transparente**, e é isso que faz o padding ser padding em vez de lixo.
    #[test]
    fn the_gap_is_transparent() {
        let sheet = compose(7, "s".into(), 8, vec![solid("a", 2, 2, 0xFF)], &[[0, 0]]).unwrap();
        assert_eq!(px(&sheet, 5, 5), [0, 0, 0, 0]);
    }

    /// As regiões saem ORDENADAS POR NOME — o índice é a referência durável que o `Sprite`
    /// guarda, e a ordem de entrada não pode decidi-lo.
    #[test]
    fn the_regions_are_sorted_by_name_whatever_the_input_order() {
        let sheet = compose(
            7,
            "s".into(),
            16,
            vec![solid("zebra", 2, 2, 1), solid("alpha", 2, 2, 2)],
            &[[0, 0], [8, 8]],
        )
        .unwrap();
        let names: Vec<&str> = sheet.regions.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zebra"]);
        // E o retângulo acompanhou o NOME, não a posição na lista de entrada.
        assert_eq!(sheet.region(0).unwrap().rect, [8, 8, 2, 2]);
        assert_eq!(sheet.region(1).unwrap().rect, [0, 0, 2, 2]);
    }

    /// ⚠️ Uma peça fora da folha é RECUSADA, não cortada: cortar produziria um `.json` cuja região
    /// não contém o que o `.png` mostra, e esse defeito só aparece no consumidor.
    #[test]
    fn a_piece_outside_the_sheet_is_refused_by_name() {
        let err = compose(7, "s".into(), 8, vec![solid("late", 4, 4, 1)], &[[6, 0]]).unwrap_err();
        assert_eq!(
            err,
            PackError::OutsideSheet {
                name: "late".into(),
                rect: [6, 0, 4, 4],
                size: 8,
            }
        );
    }

    /// A soma `x + w` é feita em `u64`: em `u32` daria a volta e o retângulo absurdo "caberia".
    #[test]
    fn a_rect_that_would_wrap_u32_does_not_pass() {
        let err = compose(
            7,
            "s".into(),
            8,
            vec![solid("wrap", 4, 4, 1)],
            &[[u32::MAX - 1, 0]],
        )
        .unwrap_err();
        assert!(matches!(err, PackError::OutsideSheet { .. }));
    }

    /// ⚠️ **Duas peças no mesmo pixel RECUSAM.** Medido em 2026-08-19: antes desta guarda, o
    /// segundo `blit` apagava a última coluna do primeiro (`0xAA` → `0x00`) **em silêncio**, e o
    /// `.json` continuava a declarar as duas regiões inteiras — uma peça com um lado comido e nada
    /// a explicá-lo.
    #[test]
    fn two_pieces_on_the_same_pixel_are_refused_by_name() {
        let err = compose(
            7,
            "s".into(),
            16,
            vec![solid("a", 4, 4, 0xAA), solid("b", 4, 4, 0x00)],
            &[[0, 0], [3, 0]],
        )
        .unwrap_err();
        assert_eq!(
            err,
            PackError::Overlap {
                a: "a".into(),
                b: "b".into()
            }
        );
    }

    /// **Encostar NÃO é sobrepor** — e sem este controle o teste acima passaria com uma guarda que
    /// recusasse qualquer vizinhança. O empacotador põe peças a um `padding` de distância, e com
    /// `padding: 0` elas encostam-se de propósito.
    #[test]
    fn pieces_that_merely_touch_are_fine() {
        let sheet = compose(
            7,
            "s".into(),
            16,
            vec![solid("a", 4, 4, 0xAA), solid("b", 4, 4, 0xBB)],
            &[[0, 0], [4, 0]],
        )
        .expect("encostadas cabem");
        assert_eq!(sheet.rgba[3 * 4], 0xAA, "a ultima coluna de `a` sobreviveu");
        assert_eq!(sheet.rgba[4 * 4], 0xBB, "a primeira de `b` esta' la'");
    }

    /// **O `pack` passou a compor por esta porta** — o mesmo laço de blit. Se algum dia divergirem,
    /// é porque alguém escreveu o segundo.
    #[test]
    fn pack_and_compose_agree_on_the_pixels() {
        let inputs = vec![solid("a", 4, 4, 0x11), solid("b", 2, 2, 0x22)];
        let packed = pack(
            1,
            "s".into(),
            vec![solid("a", 4, 4, 0x11), solid("b", 2, 2, 0x22)],
            PackOptions {
                padding: 0,
                max_size: 64,
            },
        )
        .expect("empacota");
        let at: Vec<[u32; 2]> = packed
            .regions
            .iter()
            .map(|r| [r.rect[0], r.rect[1]])
            .collect();
        // As regiões do `pack` já vêm ordenadas por nome, e `inputs` também está — a
        // correspondência é posicional de propósito.
        let composed = compose(1, "s".into(), packed.width, inputs, &at).expect("compõe");
        assert_eq!(composed.rgba, packed.rgba);
        assert_eq!(composed.regions, packed.regions);
    }
}
