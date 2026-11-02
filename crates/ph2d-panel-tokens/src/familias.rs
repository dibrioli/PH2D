//! **As famílias de cor de DADO dobram-se, e nascem dobradas** (integ 02/11).
//!
//! A tabela é `ColorToken::ALL` inteira, e as cores que pintam DADOS (categorias de nó, tipos de
//! objecto, portas, timeline, pessoas, código, Kanban) são famílias contíguas dela. A F42 (UIUX,
//! `obj-*`) somou 18 linhas abertas ao painel: `+450 px` ao abrir
//! (`a_altura_de_abertura_de_um_painel_so_encolhe`). ⇒ cada família de dado é uma secção FIXA (a
//! ordem é a da tabela, não se arrasta) semeada recolhida; o cromo do app fica à vista. ⚠️ A
//! família sai da CHAVE (o prefixo), então uma cor nova de uma família já listada entra dobrada
//! sem tocar aqui.

use ph2d_a11y::NodeId;
use ph2d_tokens::ColorToken;

/// Uma família dobrável: o prefixo da chave, o id do cabeçalho e a chave do rótulo.
pub(crate) struct Familia {
    pub(crate) prefixo: &'static str,
    pub(crate) id: NodeId,
    pub(crate) rotulo: &'static str,
}

/// As famílias de DADO, pela ordem em que aparecem na tabela.
pub(crate) const FAMILIAS: [Familia; 7] = [
    Familia {
        prefixo: "node-cat-",
        id: crate::ids::TOKENS_SEC_NODE_CAT,
        rotulo: "panel.tokens.family.node_cat",
    },
    Familia {
        prefixo: "obj-",
        id: crate::ids::TOKENS_SEC_OBJ,
        rotulo: "panel.tokens.family.obj",
    },
    Familia {
        prefixo: "port-",
        id: crate::ids::TOKENS_SEC_PORT,
        rotulo: "panel.tokens.family.port",
    },
    Familia {
        prefixo: "timeline-",
        id: crate::ids::TOKENS_SEC_TIMELINE,
        rotulo: "panel.tokens.family.timeline",
    },
    Familia {
        prefixo: "person-",
        id: crate::ids::TOKENS_SEC_PERSON,
        rotulo: "panel.tokens.family.person",
    },
    Familia {
        prefixo: "code-",
        id: crate::ids::TOKENS_SEC_CODE,
        rotulo: "panel.tokens.family.code",
    },
    Familia {
        prefixo: "kanban-",
        id: crate::ids::TOKENS_SEC_KANBAN,
        rotulo: "panel.tokens.family.kanban",
    },
];

/// Os cabeçalhos de todas as famílias, pela ordem da tabela — quem mede TODAS as linhas (os gates do
/// painel) abre-as por aqui, sem uma lista própria que esqueceria a família seguinte.
pub fn ids_das_familias() -> impl Iterator<Item = NodeId> {
    FAMILIAS.iter().map(|f| f.id)
}

/// A família dobrável de `token` (`None` = cromo do app, sempre à vista).
#[must_use]
pub(crate) fn familia_de(token: ColorToken) -> Option<&'static Familia> {
    let k = token.key();
    FAMILIAS.iter().find(|f| k.starts_with(f.prefixo))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada família é UM bloco contíguo da tabela — senão o cabeçalho repetir-se-ia a meio.
    #[test]
    fn cada_familia_e_um_bloco_contiguo_e_nao_vazio() {
        for f in &FAMILIAS {
            let rows: Vec<usize> = ColorToken::ALL
                .iter()
                .enumerate()
                .filter(|(_, t)| familia_de(**t).is_some_and(|g| g.id == f.id))
                .map(|(i, _)| i)
                .collect();
            assert!(!rows.is_empty(), "`{}` não casa nenhuma cor", f.prefixo);
            assert_eq!(
                rows.last().unwrap() - rows[0] + 1,
                rows.len(),
                "`{}` está partida na tabela",
                f.prefixo
            );
        }
    }
}
