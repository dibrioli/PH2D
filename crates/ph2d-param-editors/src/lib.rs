//! ⭐⭐⭐ **OS EDITORES RICOS DE UM PARAM, COM DOIS HOSPEDEIROS** — a curva, o gradiente e a
//! paleta, desenhados e conduzidos aqui, hospedados pela row do painel lateral **e** pelo cartão
//! do nó no grafo ([doc 101](../../../docs/Motion%20Nodes/101_pesquisa_cartoes_ricos_2026-09-04.md),
//! plano das 7 waves).
//!
//! ⚠️ **Ela existe porque o painel lateral SAI** (doc 103, ordem do Enio de 2026-09-05) e estes
//! três editores são a última coisa que só ele sabe abrir — o censo do
//! `panel_exit_probe` conta **5** controlos nessa situação. Escrever uma segunda versão do lado
//! do cartão seria a mesma lei em dois sítios, e o repo já pagou esse defeito o suficiente para
//! o ter escrito: *uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
//!
//! ⚠️ **O que ela NÃO sabe:** de que row / de que nó o valor vem, e por que porta a edição sai.
//! Ela recebe um **texto** (a serialização do param) e devolve **texto**; quem o guarda, quem o
//! desfaz e quem o coze é o hospedeiro. É isso que a mantém folha.

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;

pub mod curve;
pub mod gradient;
pub mod palette;

/// **Quantos pontos um editor de curva oferece.** O teto vem do texto do param (`field.remap` e
/// irmãos): um punhado de pontos molda qualquer transferência, e os widgets por-ponto são
/// agrupados posicionalmente como as opções de um enum.
pub const MAX_CURVE_POINTS: usize = 8;

/// **Máximo de paradas que o editor de gradiente oferece — MEDIDO** (doc 85; bloco Z, doc 91).
///
/// O modelo (`ph2d_color::MAX_RAMP_STOPS`) admite **32**; o `+` recusa acima daqui.
///
/// ⚠️ **O número estava certo e a RAZÃO não existia**, que é o defeito que a folha 09 da
/// conferência acusou: dizia-se *"o painel é estreito e a faixa tem de ficar legível"* — uma
/// frase, não um recurso (`CLAUDE.md` §0.0). A derivação é esta, e o gate
/// `the_gradient_stop_ceiling_is_the_narrowest_panel_divided_by_a_pointer_target` refá-la a cada
/// corrida:
///
/// | grandeza | de onde vem | px |
/// |---|---|---|
/// | superfície mais estreita | `ph2d_tokens::PANEL_MIN_W_PX` (o piso do arrasto de redimensionar) | 220 |
/// | recuo, dos dois lados | `ph2d_tokens::PANEL_HEAD_PAD_PX` × 2 | 36 |
/// | **faixa útil** | | **184** |
/// | alvo de ponteiro | `GRAB_R × 2` — a caixa de agarrar que este mesmo editor declara | 18 |
/// | folga da célula | `pad × 2` do strip de amostras | 4 |
/// | **por parada** | | **22** |
///
/// `184 / 22 = 8,36` ⇒ **8**.
///
/// ⚠️ **O recurso não é a legibilidade, é o ALVO DE PONTEIRO** — e a distinção decide o número.
/// Uma amostra de 14 px lê-se perfeitamente; o que ela deixa de ser é *clicável*, e cada amostra
/// abre o seletor de cor. A régua é a própria caixa de agarrar que este editor já declara para
/// os marcadores: uma amostra mais estreita que o alvo dos marcadores ao lado dela é um alvo
/// que a lei da casa já chama de pequeno demais.
///
/// ⚠️ **Contra o painel MAIS ESTREITO, não contra o de hoje**: um teto que só vale na largura
/// confortável parte-se quando o artista aperta a janela — a lei do pior caso que o
/// `motion.spring` já aplica ao relógio.
///
/// Os marcadores por-parada são registados a cada pintura, como as alças do Curve.
pub const MAX_GRADIENT_STOPS: usize = 8;

/// ⭐⭐⭐ **A CHAVE DE UM EDITOR ABERTO** — o prefixo de que todos os ids dele derivam.
///
/// ⚠️ **Ela existe porque os dois hospedeiros contam coisas diferentes.** No painel há **um** nó
/// selecionado e a row identifica-se pela POSIÇÃO dela na lista (`slot`); no grafo há vinte
/// cartões visíveis ao mesmo tempo, e dois `motion.color_ramp` na tela pediriam exactamente os
/// mesmos widgets. *Um id que não carrega o nó é como se escreve no objecto errado, em silêncio*
/// — foi o defeito medido na amostra de cor, e a cura é a mesma: **quem chama diz a chave**.
///
/// ⚠️ **As AMOSTRAS de cor têm prefixo próprio, e não é arrumação:** a shell também as deriva
/// (ela semeia a cor do selector e lê a escolha de volta), logo elas não podem depender de um
/// índice de row, que a shell não conhece.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EditorKey<'a> {
    /// Prefixo de tudo o que só o editor regista — marcadores, pontos, botões, chips.
    pub own: &'a str,
    /// Prefixo das amostras de cor, o único que a shell também deriva.
    pub swatch: &'a str,
}

impl EditorKey<'_> {
    /// O id do **EDITOR** — o `parent` que cada alça arrastável carrega, e por onde o despacho
    /// devolve o arrasto ao editor certo.
    #[must_use]
    pub fn root(&self) -> NodeId {
        fnv_id(self.own)
    }

    /// O id de uma peça do editor (`"pt/3"`, `"add"`, `"interp"`).
    #[must_use]
    pub fn sub(&self, role: &str) -> NodeId {
        fnv_id(&format!("{}/{role}", self.own))
    }

    /// O id da `i`-ésima amostra de cor — o prefixo separado que a shell também deriva.
    #[must_use]
    pub fn swatch_id(&self, i: usize) -> NodeId {
        fnv_id(&format!("{}/{i}", self.swatch))
    }
}

/// FNV-1a-64 de `key` — o mesmo esquema dos ids dinâmicos do painel do grafo e do de params, e
/// é por ser o mesmo que a mudança de casa destes editores não move um único id.
#[must_use]
pub fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔⛔ **A MUDANÇA DE CASA NÃO PODE MOVER UM ID.**
    ///
    /// Os ids destes editores são FNV de uma string, e a string era escrita no painel de params.
    /// Se ela mudasse, tudo continuaria a compilar e a desenhar — e o arrasto de um ponto, a
    /// semente de uma amostra e a leitura de volta do selector passariam a falar de widgets
    /// diferentes, **em silêncio**. Este gate prende as strings antigas às novas.
    #[test]
    fn moving_house_moves_no_id() {
        let k = EditorKey {
            own: "motion_param/curve/3",
            swatch: "motion_param/grad_swatch/ramp",
        };
        assert_eq!(k.root(), fnv_id("motion_param/curve/3"));
        assert_eq!(k.sub("pt/2"), fnv_id("motion_param/curve/3/pt/2"));
        assert_eq!(k.sub("add"), fnv_id("motion_param/curve/3/add"));
        assert_eq!(k.swatch_id(1), fnv_id("motion_param/grad_swatch/ramp/1"));
    }

    /// **Duas chaves diferentes não colidem** — a razão de a chave existir.
    #[test]
    fn two_keys_never_share_a_widget() {
        let a = EditorKey {
            own: "card/curve/7/curve",
            swatch: "card/grad_swatch/7/curve",
        };
        let b = EditorKey {
            own: "card/curve/8/curve",
            swatch: "card/grad_swatch/8/curve",
        };
        assert_ne!(a.root(), b.root());
        assert_ne!(a.sub("pt/0"), b.sub("pt/0"));
        assert_ne!(a.swatch_id(0), b.swatch_id(0));
    }
}
