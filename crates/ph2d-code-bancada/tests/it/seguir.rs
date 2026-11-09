//! O que a aba pede à cena (item 10): os contornos do mapa e o objecto espiado, por recursos ECS;
//! fechada a aba, os pedidos apagam-se.

use crate::{SCRIPT, TEXTO, decls, objecto};
use ph2d_code_bancada::{Bancada, Open, SceneOutlines, ScenePeek};
use ph2d_code_buffer::Text;
use ph2d_code_syntax::Syntax;
use ph2d_code_viewmodel::{CellKind, Who};
use ph2d_ecs::World;
use ph2d_script::ScriptDrafts;

/// ⭐ Com o mapa ligado, a aba pede o contorno dos objectos do script (pela ordem da grelha); a
/// resposta da shell para um deles enche o mapa com o ÍNDICE dele; o espiar de um índice escreve os
/// bits e a cor dele; um índice fora da grelha não espia ninguém; sem aba, os dois apagam-se.
#[test]
fn o_follow_pede_os_contornos_espia_com_a_cor_e_apaga_sem_aba() {
    let mut w = World::new();
    w.insert_resource(ScriptDrafts([(SCRIPT, decls())].into()));
    let c = objecto(&mut w, 3, Some("C"), SCRIPT, &[]);
    let a = objecto(&mut w, 1, Some("A"), SCRIPT, &[]);
    let b = objecto(&mut w, 2, Some("B"), SCRIPT, &[]);
    let text = Text::new(TEXTO);
    let syntax = Syntax::new(&text);
    let open = || {
        Some(Open {
            script: SCRIPT,
            syntax: &syntax,
            text: &text,
        })
    };
    let mut banc = Bancada::default();

    banc.follow(&mut w, None, open(), (true, None));
    assert_eq!(
        w.resource::<SceneOutlines>().wanted,
        [a.to_bits(), b.to_bits(), c.to_bits()]
    );
    assert!(banc.who.map.is_empty(), "a shell ainda não respondeu");
    assert_eq!(*w.resource::<ScenePeek>(), ScenePeek(None));
    assert_eq!(
        banc.who.columns[0].kind,
        Some(CellKind::Number),
        "as declarações vêm do rascunho da aba (ScriptDrafts)"
    );

    let anel = vec![vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]]];
    w.resource_mut::<SceneOutlines>().got = vec![(b.to_bits(), anel.clone())];
    banc.follow(&mut w, None, open(), (true, Some(1)));
    assert_eq!(
        banc.who.map,
        [(1, anel)],
        "o contorno de B, pelo índice de B"
    );
    assert_eq!(
        *w.resource::<ScenePeek>(),
        ScenePeek(Some((b.to_bits(), 1)))
    );

    banc.follow(&mut w, None, open(), (false, Some(9)));
    assert!(
        w.resource::<SceneOutlines>().wanted.is_empty(),
        "mapa desligado: nada a pedir"
    );
    assert_eq!(*w.resource::<ScenePeek>(), ScenePeek(None), "índice fora");

    banc.follow(&mut w, None, open(), (true, Some(2)));
    assert_eq!(
        *w.resource::<ScenePeek>(),
        ScenePeek(Some((c.to_bits(), 2)))
    );
    banc.follow(&mut w, None, None, (true, Some(2)));
    assert_eq!(*w.resource::<ScenePeek>(), ScenePeek(None));
    assert_eq!(*w.resource::<SceneOutlines>(), SceneOutlines::default());
    assert_eq!(banc.who, Who::default());
}
