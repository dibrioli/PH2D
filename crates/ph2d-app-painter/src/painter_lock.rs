//! **A TRAVA DE SELEÇÃO DO PAINTER** — enquanto se pinta, a imagem não muda debaixo do pincel.
//!
//! Enio, 2026-08-19: *"se estou com o painter ativo numa imagem/sprite, posso escolher outra e
//! isso provoca problemas. Se o painter está ativo numa imagem não permita selecionar outra nem no
//! canvas nem na hierarchy… Não permita a seleção de múltiplas imagens se o painter está ativo.
//! Se o usuário estiver com múltiplas imagens selecionadas e entrar no painter, selecione a última
//! selecionada e desselecione as outras antes de entrar."*
//!
//! ## Por que isto é uma trava e não uma correção pontual
//!
//! O Painter é a única ferramenta do app que **possui um documento** enquanto está ativa: camadas,
//! histórico de pinceladas, prévia na GPU, tudo amarrado a UMA sprite. Trocar a seleção por baixo
//! dele não é "mudar de objeto" — é dizer-lhe que o documento que ele tem em mãos passou a ser
//! outro, a meio de um traço. As três regras que o Enio pediu são a mesma frase dita a três portas:
//! *enquanto o Painter tem um documento aberto, a seleção é dele.*
//!
//! ## Uma lei, N portas
//!
//! A seleção muda em sítios que não se parecem: o pick do canvas, a linha da hierarquia, o
//! `Shift`+clique de intervalo, o `Ctrl`+clique de alternância. Escrever a regra em cada um seria
//! como ela passa a discordar de si própria — a porta que alguém acrescentar amanhã nasceria sem
//! ela. Por isso a decisão é **uma função pura** ([`decide`]) e cada porta só lhe pergunta.
//!
//! ⚠️ **Limpar a seleção NÃO é selecionar outra**, e por isso é permitido: recusar um clique no
//! vazio faria o `Esc` e o canvas parecerem partidos, e o pedido é sobre *trocar de imagem*.
//!
//! ⭐⭐ **Desde a F2 do spec/06 a trava é do MODO, não do Painter** — a lei (`decide`) e a
//! entidade trancada moram em `ph2d_editor_core::object_mode` (o Painter abre-se pelo modo Paint
//! da imagem, e o Sculpt e o Flip herdam a mesma trava na F3). Aqui fica o colapso que o Painter
//! pede ao ENTRAR, que é dele.

use ph2d_editor_core::screens::hero::HeroScreen;

/// **Colapsa uma seleção múltipla à ÚLTIMA escolhida**, devolvendo quantas saíram.
///
/// ⚠️ *Última* é a última que o artista acrescentou — o fim do `extra_selection`, e não o primário
/// (que é a **primeira**). É a leitura que corresponde ao gesto: ele clicou numa, `Shift`+clicou em
/// mais quatro, e a que ele tem em mente ao abrir o Painter é a última em que tocou.
pub fn collapse_to_last(hero: &mut HeroScreen) -> usize {
    let extras = hero.gizmo.extra_selection.len();
    if extras == 0 {
        return 0;
    }
    let last = hero.gizmo.extra_selection.last().copied();
    hero.gizmo.replace_selection(last);
    extras
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hero_with(primary: u64, extras: &[u64]) -> HeroScreen {
        ph2d_editor_core::test_support::ensure_panel_registry();
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.replace_selection(Some(primary));
        for e in extras {
            hero.gizmo.add_to_selection(*e);
        }
        hero
    }

    /// ⚠️ **A ÚLTIMA é a última ACRESCENTADA, não o primário.** O primário é a PRIMEIRA em que o
    /// artista tocou; a que ele tem em mente ao abrir o Painter é aquela em que tocou por último.
    /// Colapsar para o primário seria escolher a errada em todo gesto de `Shift`+clique.
    #[test]
    fn collapsing_keeps_the_last_added_not_the_first() {
        let mut hero = hero_with(10, &[20, 30]);
        assert_eq!(hero.gizmo.selected_len(), 3);
        let dropped = collapse_to_last(&mut hero);
        assert_eq!(dropped, 2);
        assert_eq!(hero.gizmo.selection, Some(30));
        assert_eq!(hero.gizmo.selected_len(), 1);
    }

    /// **Controle positivo:** com uma só selecionada não há nada a colapsar, e o Painter não pode
    /// receber um toast a dizer que desselecionou coisa nenhuma.
    #[test]
    fn a_single_selection_is_left_alone() {
        let mut hero = hero_with(10, &[]);
        assert_eq!(collapse_to_last(&mut hero), 0);
        assert_eq!(hero.gizmo.selection, Some(10));
    }
}
