//! Dobras: cada `function … end` dobra; só blocos de >= 2 linhas; uma por linha de início.

use crate::{BOB, line_of};
use ph2d_code_buffer::Text;
use ph2d_code_syntax::{Fold, Syntax};

#[test]
fn dobras_de_bob() {
    let text = Text::new(BOB);
    let folds = Syntax::new(&text).folds(&text);
    let l = |s| line_of(BOB, s);
    let init = l("function init(self)");
    let update = l("function update(self, dt)");
    let se = l("if self.wave == \"square\"");
    let senao = BOB
        .lines()
        .position(|x| x == "  else")
        .unwrap_or(usize::MAX);
    let topo = l("if up and not self.was_up");
    // Linhas desta cópia (cabeçalho de 2 linhas): init 23..28, update 30..52, if 35..41 com
    // elseif 37..38 e else 39..40 (um ramo dobra o seu corpo; o `end` é do `if`), o if do topo 48..50. O `if … then return end` (31) não dobra.
    assert_eq!((init, update, se, senao, topo), (23, 30, 35, 39, 48));
    let quero = vec![
        Fold {
            start_line: init,
            end_line: 28,
        },
        Fold {
            start_line: update,
            end_line: 52,
        },
        Fold {
            start_line: se,
            end_line: 41,
        },
        Fold {
            start_line: 37,
            end_line: 38,
        },
        Fold {
            start_line: senao,
            end_line: 40,
        },
        Fold {
            start_line: topo,
            end_line: 50,
        },
    ];
    assert_eq!(folds, quero);
    assert!(folds.iter().all(|f| f.end_line > f.start_line));
}

#[test]
fn bloco_de_uma_linha_nao_dobra_e_tabela_de_varias_dobra() {
    let text = Text::new("local t = { 1, 2 }\nlocal u = {\n  1,\n}\n--[[ a\nb ]]\n");
    let folds = Syntax::new(&text).folds(&text);
    assert_eq!(
        folds,
        vec![
            Fold {
                start_line: 1,
                end_line: 3
            },
            Fold {
                start_line: 4,
                end_line: 5
            }
        ]
    );
}
