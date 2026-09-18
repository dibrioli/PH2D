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

/// ⭐⭐⭐ **E AS TRÊS RECUSAS DO OSSO CHEGAM À TELA, por UMA porta** (2026-09-18).
///
/// ⛔⛔ *Uma recusa que só o terminal vê é um botão mudo* — as três viveram assim, e o dono aprovou
/// dois smokes em que foi preciso dizer-lhe *«olhe na janela preta»*. A superfície **não é nova**: a
/// `ToastQueue` já servia a irmã desta família (o *solta-se-sozinho* de uma ferramenta de moldura).
///
/// ⚠️ **As metades são três defeitos:** não avisar · avisar por três `push` espalhados (a quarta
/// recusa nasce muda) · e avisar com texto CRU, que na tela viola o HR-15.
#[test]
fn as_recusas_do_osso_chegam_a_tela_por_uma_porta() {
    assert_eq!(
        FASE.matches("Toast::warning(").count(),
        1,
        "a recusa chega a' tela por mais (ou menos) de UMA porta: com varias, a proxima recusa \
         nasce muda e ninguem ve'"
    );
    assert_eq!(
        // ⚠️ A agulha conta `avisa(` e **não** `avisa(toasts,`: o `cargo fmt` parte as chamadas
        // longas em várias linhas, e a 1.ª redacção contou `1` de `3` sobre produto CERTO. *Um
        // literal lê-se do ficheiro já formatado* — a segunda vez que esta jornada o paga.
        FASE.matches("                avisa(").count(),
        3,
        "as TRES recusas do osso nao passam todas pela porta do aviso — a que faltar so' existe \
         no terminal, que e' onde elas estavam antes desta wave"
    );
    assert!(
        FASE.contains("ph2d_i18n::tr_with(r.chave()") && FASE.contains("ph2d_i18n::tr(r.chave())"),
        "o texto do aviso deixou de vir do i18n: na TELA nao ha' texto cru (HR-15), e a chave e' \
         derivada da propria recusa"
    );
}
