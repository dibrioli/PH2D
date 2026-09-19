//! ⭐⭐⭐ **O BOTÃO *BIND* PERGUNTA À PORTA ANTES DE PRENDER** — o fio que torna a lei alcançável.
//!
//! ⛔⛔⛔ **O defeito está MEDIDO na crate** (`sonda_do_rig_partilhado_tests.rs`): sem osso escolhido
//! a semente é `None`, o `skeleton_of` responde com **todos os ossos da cena**, e com dois
//! esqueletos a forma fica presa aos **seis** — em silêncio, com o log a dizer *«1 imagem presa»*.
//!
//! ⚠️ **Este gate é da SHELL porque a lei já tem os dela:** a porta é medida por quatro gates puros
//! em `ph2d-skeleton-live`. *Uma lei viva que nenhum gesto consulta é uma lei órfã* — e este repo
//! tem uma nomeada (o `dock_columns::close`, com zero chamadores de produto).

const FASE: &str = include_str!("../../src/render_loop/fase_skeleton_verbs.rs");

/// O despacho do ponteiro — a SEGUNDA superfície que recusa em nome do osso (o pincel de peso,
/// 2026-09-19). ⚠️ Ela é outra porque o gesto dela acontece no press, e não num botão de painel.
const DESPACHO: &str = include_str!("../../src/input_dispatch/despacho_peso_do_osso.rs");

/// A fonte que DECLARA a população das recusas — o oráculo do censo abaixo.
const DECLARACAO: &str =
    include_str!("../../../../crates/ph2d-skeleton-live/src/recusa_do_osso.rs");

/// **Os nomes das variantes, lidos do `TODAS` que a crate declara.**
///
/// ⚠️ **Derivado e não escrito à mão:** a lista à mão foi a 1.ª redacção deste censo, e ela pediu
/// para ser SUBIDA no dia em que uma recusa nova nasceu — que é como uma catraca vira licença.
fn variantes() -> Vec<String> {
    let bloco = DECLARACAO
        .split_once("pub const TODAS:")
        .expect("a crate deixou de declarar a populacao das recusas")
        .1;
    let bloco = bloco.split_once("];").expect("o `TODAS` nao fecha").0;
    let mut v: Vec<String> = bloco
        .match_indices("Self::")
        .map(|(i, _)| {
            bloco[i + 6..]
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect::<String>()
        })
        .filter(|n: &String| !n.is_empty())
        .collect();
    v.sort();
    v.dedup();
    v
}

/// ⭐ **A pergunta vem ANTES das duas rotas de prender**, e as duas contam.
///
/// ⚠️ Perguntar depois de prender não é perguntar: a forma já ficou com os seis ossos.
#[test]
fn a_fase_consulta_a_porta_antes_de_qualquer_bind() {
    let porta = FASE
        .find("recusa_do_osso::recusa_do_bind(")
        .expect("a fase do bind deixou de consultar a porta da recusa");
    for rota in ["skeleton_live::bind(", "skeleton_live::bind_image("] {
        let i = FASE
            .find(rota)
            .unwrap_or_else(|| panic!("a rota {rota} saiu da fase — este gate ficou sem sujeito"));
        assert!(
            porta < i,
            "a fase chama {rota} ANTES de perguntar a porta: quando a recusa chega, a forma ja' \
             esta' presa aos ossos de todos os esqueletos da cena"
        );
    }
}

/// ⛔⛔ **A recusa é do BIND e não do quadro** — ela não pode levar com ela os outros verbos.
///
/// ⚠️ A 1.ª redacção usava `return None` e cancelava o **soltar** e os **knobs** do mesmo quadro,
/// que são verbos independentes: o artista carregava em *Bind* sem osso e o painel inteiro ficava
/// inerte nesse quadro. *Uma recusa que desliga o vizinho é pior do que a que ela cura.*
#[test]
fn a_recusa_do_bind_nao_cancela_os_outros_verbos() {
    let porta = FASE
        .find("recusa_do_osso::recusa_do_bind(")
        .expect("a porta saiu da fase");
    let resto = &FASE[porta..];
    let fim = resto
        .find("if let Some(keep) = pending_bone_release")
        .expect("o verbo de soltar saiu da fase — este gate ficou sem sujeito");
    assert!(
        !resto[..fim].contains("return None"),
        "a recusa do bind devolve cedo da FASE: ela leva com ela o soltar e os knobs do mesmo \
         quadro. O idioma e' um bloco rotulado (`break 'bind`), que sai so' do bind"
    );
    assert!(
        resto[..fim].contains("break 'bind"),
        "a recusa deixou de sair do bloco do bind: sem isso ela imprime a queixa e PRENDE na \
         mesma, que e' a unica coisa pior do que nao avisar"
    );
}

/// ⭐⭐⭐ **E TODA RECUSA DO OSSO CHEGA À TELA, por UMA porta** (2026-09-18).
///
/// ⛔⛔ *Uma recusa que só o terminal vê é um botão mudo* — as três viveram assim, e o dono aprovou
/// dois smokes em que foi preciso dizer-lhe *«olhe na janela preta»*. A superfície **não é nova**: a
/// `ToastQueue` já servia a irmã desta família (o *solta-se-sozinho* de uma ferramenta de moldura).
///
/// ⚠️ **As metades são três defeitos:** não avisar · avisar por três `push` espalhados (a quarta
/// recusa nasce muda) · e avisar com texto CRU, que na tela viola o HR-15.
#[test]
fn as_recusas_do_osso_chegam_a_tela_por_uma_porta() {
    // ⚠️ **UMA porta POR SUPERFÍCIE**, e não uma no repo inteiro: o botão de painel e o gesto de
    // ponteiro acontecem em sítios que não se alcançam. O que a lei proíbe é uma superfície com
    // `push` espalhados — *com vários, a próxima recusa daquele ficheiro nasce muda*.
    for (nome, src) in [
        ("a fase dos verbos", FASE),
        ("o traco do pincel de peso", DESPACHO),
    ] {
        assert_eq!(
            src.matches("Toast::warning(").count(),
            1,
            "{nome}: a recusa chega a' tela por mais (ou menos) de UMA porta"
        );
    }
    // ⭐⭐⭐ **O CENSO: toda recusa da população tem de ser NOMEADA numa superfície.**
    //
    // ⛔⛔ **A 1.ª redacção contava CHAMADAS num ficheiro só** (`avisa(` na fase), e ela reprovou
    // no dia em que o pincel de peso trouxe três recusas que saem do DESPACHO — *sobre produto
    // certo*. ⚠️ E contar chamadas era a régua errada por uma segunda razão, que só a cura
    // revelou: uma mesma recusa pode ter DOIS sítios que a levantam (o `ForaDaArte` sai da guarda
    // do pen-down **e** do resultado da lei), logo o número de chamadas nunca foi o número de
    // recusas. ⇒ a pergunta certa é *este nome aparece nalguma superfície?*
    //
    // ⛔⛔ **E há DUAS formas de ter voz, o que a 2.ª redacção não viu:** uma recusa ou é NOMEADA
    // na superfície (as três do pincel) ou viaja como VALOR devolvido por uma porta que a
    // superfície reencaminha (a `VariosEsqueletos`, que só o `recusa_do_bind` constrói). *Um censo
    // que só procurasse nomes na shell acusaria de muda uma recusa que fala há uma wave.*
    // ⇒ a pergunta é *alguém consegue EMITIR isto?*, e o universo são as superfícies mais os
    // produtores da crate (tudo o que vem depois do `impl`, onde o `chave`/`quantos` nomeiam todas
    // por construção e não provariam nada).
    let produtores = DECLARACAO
        .split_once("\npub fn recusa_")
        .expect("a crate deixou de ter produtores de recusa")
        .1;
    let universo = format!("{FASE}{DESPACHO}{produtores}");
    let mudas: Vec<String> = variantes()
        .into_iter()
        .filter(|v| !universo.contains(v.as_str()))
        .collect();
    assert!(
        mudas.is_empty(),
        "estas recusas nao sao emitidas por ninguem — declaradas e mudas: {mudas:?}"
    );
    assert!(
        variantes().len() >= 8,
        "o censo leu {} variantes e a populacao ja' foi 8 — a extraccao do `TODAS` partiu-se e \
         este gate passou a medir o vazio",
        variantes().len()
    );
    assert!(
        FASE.contains("ph2d_i18n::tr_with(r.chave()") && FASE.contains("ph2d_i18n::tr(r.chave())"),
        "o texto do aviso deixou de vir do i18n: na TELA nao ha' texto cru (HR-15), e a chave e' \
         derivada da propria recusa"
    );
}
