//! ⭐⭐⭐ **A ORDEM do tween dentro do quadro** (suplente #22).
//!
//! Três marcos, e **cada troca de par é um defeito distinto que nenhum teste de unidade alcança**:
//!
//! 1. o **apply da timeline** escreve a pose da cena (`fase_timeline_drain`) — o tween tem de vir
//!    depois, senão o ledger troca o autorado em vez de compor `autorado → A → B` (é a mesma lei
//!    que a `fase_sequences` já escreve, e o mecanismo está no cabeçalho dela);
//! 2. a **tabela nome → acção** resolve — um `Start Timer` publicado neste quadro tem de mexer
//!    NESTE quadro, senão o artista lê *«o sinal falhou»*;
//! 3. o **tween escreve**.
//!
//! ⚠️ **O gate lê o texto EMENDADO do quadro** (`frame_text::render_frame`), onde a posição de um
//! literal é a ordem em que ele corre — o mesmo oráculo dos gates de ordem irmãos. ⚠️ É por isso
//! que a fase-filha se chama `fase_tweens`: o texto emendado colhe só `fn fase_*`, e com outro nome
//! ela **desaparece** dali sem nada ficar vermelho.

/// Os três marcos, na ordem em que TÊM de aparecer.
const ORDER: &[(&str, &str)] = &[
    (
        "self.fase_timeline_drain(",
        "o apply da timeline escreve a pose da cena",
    ),
    (
        "ph2d_ecs::resolve_signal_actions(",
        "a tabela nome -> accao resolve (o `Start Timer` deste quadro)",
    ),
    (
        "tween_bridge::drive_tweens(",
        "o TWEEN escreve, por ultimo",
    ),
];

#[test]
fn o_tween_corre_depois_do_apply_e_de_quem_o_arranca() {
    let text = crate::frame_text::render_frame();
    let mut at = Vec::new();
    for (needle, what) in ORDER {
        let hits = text.matches(needle).count();
        // Um marco que aparece duas vezes não tem POSIÇÃO — e o `find` pegaria a primeira em
        // silêncio, que é como um gate de ordem começa a medir outra função.
        assert_eq!(
            hits, 1,
            "o marco `{needle}` ({what}) aparece {hits}x no texto do quadro. Um gate de ORDEM só \
             pode falar de um marco que existe uma vez."
        );
        at.push((text.find(needle).expect("acabou de ser contado"), *what));
    }
    for par in at.windows(2) {
        assert!(
            par[0].0 < par[1].0,
            "a ordem do quadro inverteu-se: «{}» tem de vir ANTES de «{}»",
            par[0].1,
            par[1].1
        );
    }
}

/// ⭐⭐ **E o passe do tween é o ÚNICO sítio da shell que conduz um tween.**
///
/// ⚠️ **É o gate irmão do de ordem, e ele mede outra coisa:** um segundo chamador — numa fase
/// anterior, ou dentro de um ramo de ferramenta — teria a sua própria janela de ordem, e as duas
/// discordariam no dia em que uma delas mudasse de sítio. *Uma lei escrita em dois sítios ainda não
/// é uma lei.*
#[test]
fn so_uma_fase_conduz_os_tweens() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut achados = Vec::new();
    let mut pilha = vec![raiz];
    while let Some(dir) = pilha.pop() {
        for e in std::fs::read_dir(&dir).expect("ler o directorio da shell") {
            let p = e.expect("entrada").path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && std::fs::read_to_string(&p)
                    .expect("ler o ficheiro")
                    .contains("tween_bridge::drive_tweens(")
            {
                achados.push(p.file_name().expect("nome").to_string_lossy().to_string());
            }
        }
    }
    achados.sort();
    assert_eq!(
        achados,
        vec!["fase_tweens.rs".to_string()],
        "a shell conduz tweens em mais de um sitio: {achados:?}"
    );
}
