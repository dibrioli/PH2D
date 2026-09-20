//! ⭐⭐⭐ **O BALÃO DE UMA FILEIRA** — a chave ausente é o caso normal, e a pergunta custa uma vez.

use super::{SUFIXO, dica_da_fileira, pendura, perguntas_feitas};
use crate::state::ParamRow;
use ph2d_editor_core::interaction::WidgetStore;

fn fileira(key: &'static str) -> ParamRow {
    ParamRow {
        entity: 0,
        param: ph2d_field::Param::Dim(0),
        key,
        value: 0.0,
        lo: 0.0,
        bound: ph2d_field::Bound::Soft(1.0),
        inert: None,
        integral: false,
        choices: &[],
        swatch: None,
        section: None,
        subject: None,
    }
}

/// ⚠️⚠️ **As chaves forjadas por este ficheiro vivem FORA do espaço de nomes do painel, e isso é uma
/// lei.** O censo de chaves (`every_key_of_this_panel_exists_on_both_sides`) varre o repo INTEIRO —
/// testes incluídos — à procura de `panel.model3d.*` **usadas e não declaradas**, porque uma dessas
/// pinta o identificador cru na tela. Uma chave de mentira escrita naquele prefixo é indistinguível
/// de uma gralha de produção. *Medido em 2026-09-19: o gate nasceu vermelho por isto, e por uma
/// gralha igual num teste de outra crate.*
const FORJADA: &str = "gate.dica.chave.que.nao.existe";

/// ⭐⭐⭐ **A CHAVE AUSENTE NÃO PINTA NADA** — e é ela o caso normal.
///
/// ⛔ **Sem esta metade o mecanismo seria pior que a ausência dele:** com o `tr` a devolver a chave
/// crua, um balão apareceria em toda fileira a dizer o identificador dela — *um identificador cru no
/// ecrã lê-se como uma avaria, e ele apareceria em cima de cada controlo.*
///
/// **Mutação que deve sangrar:** devolver `Some(texto)` sem comparar com a chave.
#[test]
fn uma_chave_sem_dica_nao_pinta_balao() {
    assert_eq!(
        dica_da_fileira(FORJADA),
        None,
        "uma fileira sem dica recebeu o identificador cru como balão"
    );
    let mut store = WidgetStore::default();
    let id = crate::ids::model3d_radius_slider(0);
    pendura(&mut store, &fileira("gate.dica.outra.forjada"), &[id]);
    assert_eq!(
        store.tooltip_for(id),
        None,
        "o `pendura` registou um balão sobre uma chave que a tabela não tem"
    );
}

/// ⭐⭐⭐ **CONTROLO POSITIVO: uma chave QUE EXISTE chega ao widget.**
///
/// ⚠️ **Sem ele, uma porta que devolvesse `None` a tudo passaria o gate de cima** — e o mecanismo
/// inteiro seria um no-op que ninguém veria. *A metade negativa sozinha aprova o nada.*
///
/// ⚠️ **A chave usada aqui é uma que a tabela JÁ declara**, e não uma dica: o que este gate mede é o
/// fio (`tr` → `set_tooltip` → `tooltip_for`), não o conteúdo do texto.
#[test]
fn uma_chave_que_existe_chega_ao_widget() {
    // ⚠️ **A dica propriamente dita pode ainda não existir** — as chaves `.tip` são escritas pelo
    // dono da tabela de textos, e a ausência delas é o estado NORMAL. ⇒ o que este gate mede é o
    // FIO (o que a porta entrega chega ao widget e volta de lá), com um texto que existe de certeza.
    // ⚠️ **A chave vem do `Panel::TITLE` e não de um `tr` escrito à mão**, e isto não é estilo: o
    // gate `no_panel_paints_its_own_name_beside_the_key` conta as SEGUNDAS portas para o nome de um
    // painel, e uma delas é o que deu ao artista `"Tokens"` no cabeçalho e `"Design Tokens"` na aba.
    // ⇒ *uma chave, duas superfícies* — e de graça a fixtura deixa de poder derivar da tabela.
    let com_dica = <crate::Model3dPanel as ph2d_editor_core::panel::Panel>::TITLE.tr();
    assert_ne!(
        com_dica,
        <crate::Model3dPanel as ph2d_editor_core::panel::Panel>::TITLE.key(),
        "a fixtura deste gate deixou de existir na tabela"
    );
    let viva = format!("uma.chave.qualquer{SUFIXO}");
    let mut store = WidgetStore::default();
    let id = crate::ids::model3d_radius_slider(1);
    store.set_tooltip(id, com_dica);
    assert_eq!(
        store.tooltip_for(id),
        Some(com_dica),
        "o `WidgetStore` deixou de guardar a dica que lhe entregam — o mecanismo do balão morreu \
         debaixo desta porta"
    );
    assert!(
        viva.ends_with(SUFIXO),
        "a convenção do sufixo mudou e este gate deixou de a descrever"
    );
}

/// ⭐⭐⭐ **A PERGUNTA CHEGA À TABELA UMA VEZ POR CHAVE, e nunca por quadro.**
///
/// ⛔⛔ **É esta a propriedade que torna a porta aceitável.** A `ph2d_i18n::tr` responde a uma chave
/// desconhecida com um `Box::leak` da própria chave — perguntar por fileira, por quadro, vazaria uma
/// `String` a `60 Hz`. Medido aqui: `1 000` perguntas sobre a mesma chave ⇒ **UMA** ida à tabela.
///
/// **Mutação que deve sangrar:** tirar o memo (perguntar ao `tr` sempre).
#[test]
fn a_pergunta_chega_uma_vez_por_chave() {
    const CHAVE: &str = "gate.dica.uma.chave.so.para.este.gate";
    let antes = perguntas_feitas();
    for _ in 0..1000 {
        let _ = dica_da_fileira(CHAVE);
    }
    let feitas = perguntas_feitas() - antes;
    assert_eq!(
        feitas, 1,
        "mil perguntas sobre a MESMA chave foram à tabela {feitas} vezes — cada ida a uma chave \
         desconhecida deixa uma `String` na heap (o `leak_key` da `ph2d-i18n`), logo isto é uma \
         fuga POR QUADRO"
    );
}
