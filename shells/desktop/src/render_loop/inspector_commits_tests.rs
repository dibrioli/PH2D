//! **Os gates dos CANTOS por-índice** — irmão de [`super::inspector_commits`] pelo cap HR-18 (600):
//! o módulo de teste levava aquele ficheiro a 622. *Cortar é a cura.*

#[cfg(test)]
mod tests {
    use crate::render_loop::inspector_commits_sprite::apply_sprite_field;
    use ph2d_editor::SpriteFieldEdit;
    use ph2d_render::Sprite;

    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    fn sprite_with_corners(c: [[f32; 4]; 4]) -> Sprite {
        let mut s = Sprite::individual(1, [1.0, 1.0], [1.0; 4]);
        s.per_corner_tint = c;
        s
    }

    /// **Editar UM canto preserva os outros três** — a lei que já governava `OffsetX`/`OffsetY` e
    /// `RegionX/Y/W/H`, e que faltava aqui.
    ///
    /// ⚠️ Este teste é o irmão exato do defeito: numa multi-seleção, o edit carregava o array
    /// INTEIRO da primária, então mexer no TL de uma sprite reescrevia os quatro cantos de todas
    /// as outras. E o painel *pintava* «Mixed» para esse estado — a promessa e o verbo discordavam
    /// (auditoria `docs/Sprite_projeto/20` §3.2).
    #[test]
    fn editing_one_corner_leaves_the_other_three_alone() {
        // Uma sprite «vizinha» com cantos DIVERGENTES da primária: é ela que o bug atropelava.
        let mut neighbour = sprite_with_corners([GREEN, BLUE, RED, GREEN]);
        let before = neighbour.per_corner_tint;
        apply_sprite_field(&mut neighbour, SpriteFieldEdit::PerCornerTintAt(0, RED));
        assert_eq!(
            neighbour.per_corner_tint[0], RED,
            "o canto pedido nao mudou"
        );
        assert_eq!(
            &neighbour.per_corner_tint[1..],
            &before[1..],
            "editar o TL reescreveu os outros cantos — e' exatamente o atropelo que o variante \
             por-indice existe para impedir"
        );
    }

    /// **Um índice impossível não escreve nada** — em vez de indexar fora e entrar em pânico.
    #[test]
    fn a_corner_index_the_ui_cannot_produce_is_ignored() {
        let mut s = sprite_with_corners([GREEN; 4]);
        apply_sprite_field(&mut s, SpriteFieldEdit::PerCornerTintAt(9, RED));
        assert_eq!(s.per_corner_tint, [GREEN; 4]);
    }

    /// **«Igualar» usa o TL DE CADA SPRITE, não o da primária.**
    ///
    /// ⚠️ Antes o botão emitia `PerCornerTint([tl_da_primária; 4])`, que numa multi-seleção
    /// pintava o TL da primária nas quatro pontas de todas as outras. *Igualar é uma operação
    /// sobre cada sprite, não a difusão de um valor* — daí o variante sem carga.
    #[test]
    fn equalize_uses_each_sprites_own_top_left() {
        let mut primary = sprite_with_corners([RED, GREEN, BLUE, GREEN]);
        let mut neighbour = sprite_with_corners([BLUE, RED, GREEN, RED]);
        apply_sprite_field(&mut primary, SpriteFieldEdit::EqualizeCorners);
        apply_sprite_field(&mut neighbour, SpriteFieldEdit::EqualizeCorners);
        assert_eq!(primary.per_corner_tint, [RED; 4]);
        assert_eq!(
            neighbour.per_corner_tint, [BLUE; 4],
            "a vizinha igualou pelo TL da PRIMARIA — o verbo esta' a difundir um valor em vez de \
             executar uma operacao"
        );
    }
}
