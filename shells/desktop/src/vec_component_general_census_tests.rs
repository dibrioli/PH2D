//! ⭐⭐⭐ **NÃO HÁ SEGUNDO MOTOR DE INSTÂNCIA** — o censo que o prova (F4.6c, 2026-09-07).
//!
//! # Porque é um ficheiro irmão
//!
//! O tecto de 600 LOC do shell (HR-18), e o corte é por RESPONSABILIDADE: o [`super::tests`]
//! mede *o que a secção FAZ*, e isto mede *o que a árvore CONTÉM*. São perguntas de espécies
//! diferentes — uma corre sobre um mundo montado, a outra sobre o fonte.
//!
//! # O que ele substitui
//!
//! Um gate que media *«as três portas leem o mesmo interruptor»* — e o interruptor
//! (`PH2D_VEC_COMPONENT_GENERAL`) morreu com o motor que ele escolhia. ⇒ a pergunta muda de
//! *«qual motor?»* para **«ainda há dois?»**.

/// ⭐⭐⭐ **NÃO HÁ SEGUNDO MOTOR — o censo que o prova** (F4.6c wave 3, 2026-09-07).
///
/// O gate que aqui esteve media *«a porta está aberta por omissão»* sobre um `armed()` que
/// escolhia entre dois motores. Com o `VecInstance` apagado não há escolha, e um gate sobre um
/// interruptor que não existe seria uma asserção sobre nada.
///
/// ⇒ a pergunta muda de *«qual motor?»* para *«ainda há dois?»*, e a régua é o FONTE da shell:
/// nenhum ficheiro do produto volta a nomear o componente do motor velho.
///
/// ⚠️ **Varre o `src/` inteiro e não uma lista** — uma lista escrita à mão fica verde no dia em
/// que alguém ressuscita o tipo num ficheiro que ela não conhece. ⛔ E descasca comentários antes
/// de varrer: esta wave deixou o nome escrito em vários docs, de propósito, para que o próximo
/// leitor saiba o que foi apagado.
#[test]
fn no_file_of_the_shell_names_the_old_instance_motor() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut acusados = Vec::new();
    let mut pilha = vec![raiz.clone()];
    while let Some(dir) = pilha.pop() {
        for entrada in std::fs::read_dir(&dir).expect("ler o src") {
            let caminho = entrada.expect("entrada").path();
            if caminho.is_dir() {
                pilha.push(caminho);
                continue;
            }
            if caminho.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let corpo = std::fs::read_to_string(&caminho).expect("ler o ficheiro");
            let sem_comentario: String = corpo
                .lines()
                .map(|l| match l.find("//") {
                    Some(i) => &l[..i],
                    None => l,
                })
                .collect::<Vec<_>>()
                .join("\n");
            // ⚠️ **A agulha é MONTADA, e isso não é esperteza:** escrita inteira, ela apareceria
            // no fonte deste próprio ficheiro e o censo acusar-se-ia a si mesmo — e a cura óbvia
            // (saltar o ficheiro do gate) abriria o único ponto cego que ele não pode ter.
            // *Excluir um ficheiro de um censo é escolher onde não olhar.*
            if sem_comentario.contains(concat!("Vec", "Instance")) {
                acusados.push(
                    caminho
                        .strip_prefix(&raiz)
                        .unwrap_or(&caminho)
                        .to_path_buf(),
                );
            }
        }
    }
    assert!(
        acusados.is_empty(),
        "o motor de instancia do vetor voltou ao codigo da shell: {acusados:?}"
    );
}
