//! Gates do **picker de ícone** (plano UI/UX W8b §6.2).

use super::*;

/// **A primeira linha é `Drawing`, e as outras são o catálogo INTEIRO, em ordem.**
///
/// ⚠️ O `Drawing` na posição zero não é decoração: ele é **como se tira a escolha**, e um picker
/// que só ofereça glifos deixaria o artista sem porta de volta ao desenho — a rota default do
/// tipo. Um botão *"Clear"* ao lado seria um verbo que só existe depois de o primeiro ter sido
/// usado.
#[test]
fn the_first_row_is_the_drawing_and_the_rest_is_the_catalogue() {
    assert_eq!(row_count(), IconId::all().len() + 1);
    assert_eq!(row_label(0), "Drawing");
    for (n, id) in IconId::all().iter().enumerate() {
        assert_eq!(
            row_label(n + 1),
            id.slug(),
            "a linha {} saiu do lugar",
            n + 1
        );
    }
}

/// **Sem escolha, o vigente é o `Drawing`** — e com escolha é a linha DAQUELE glifo.
///
/// ⚠️ A metade do `None` é a que importa: um realce parado na primeira linha só é honesto porque
/// ela significa *o botão desenha a forma*. Se `Drawing` fosse um item a mais no fim, o estado
/// "sem escolha" não teria linha nenhuma e o picker abriria sem nada realçado.
#[test]
fn the_highlight_follows_the_choice_and_lands_on_drawing_without_one() {
    let with = |icon: Option<Option<String>>| {
        crate::state::set_widget_skin_state(
            Some(crate::state::WidgetSkinState {
                icon,
                ..Default::default()
            }),
            0,
        );
        selected_row()
    };
    assert_eq!(with(None), 0, "sem row de icone o realce nao e' o Drawing");
    assert_eq!(with(Some(None)), 0, "o Drawing nao e' a linha zero");

    let third = IconId::all()[2];
    assert_eq!(
        with(Some(Some(third.slug().to_string()))),
        3,
        "o realce nao pousou na linha do glifo escolhido"
    );
    // ⚠️ Um slug que este build não conhece cai no `Drawing` — o mesmo canal de compatibilidade
    // do `kind`: nunca um realce numa linha que não existe.
    assert_eq!(with(Some(Some("nao-existe".into()))), 0);
    crate::state::set_widget_skin_state(None, 0);
}

/// **O CHIP fechado diz a escolha**, e `Drawing` quando não há — nunca um vazio.
///
/// ⚠️ Sem isto o picker podia abrir na linha certa e o chip continuar a dizer sempre a mesma
/// coisa: o artista escolheria um glifo, veria o botão mudar no canvas, e a seção continuaria a
/// afirmar *Drawing*. Duas respostas para *qual ícone está escolhido?*, e a errada à vista.
///
/// ⚠️ E a metade do `None` é a fronteira: um tipo sem face de ícone **não tem row**, e devolver
/// `Some("Drawing")` ali pintaria um picker num `Slider`.
#[test]
fn the_chip_reads_the_choice_and_says_drawing_without_one() {
    let st = |icon| crate::state::WidgetSkinState {
        icon,
        ..Default::default()
    };
    assert_eq!(chip_label(&st(None)), None, "um tipo sem face ganhou chip");
    assert_eq!(chip_label(&st(Some(None))).as_deref(), Some(DRAWING));
    assert_eq!(
        chip_label(&st(Some(Some("trash".into())))).as_deref(),
        Some("trash")
    );
    // A palavra é a MESMA nos dois lugares — é isso que o dono único garante.
    assert_eq!(row_label(0), DRAWING);
}
