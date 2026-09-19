//! ⭐⭐⭐ **O *Reset Transform* DE UM OSSO PASSA PELA PORTA DO REPOUSO** — o fio que torna a cura
//! alcançável pelo gesto que tinha o defeito.
//!
//! ⛔⛔⛔ **O defeito está MEDIDO na crate** (`ph2d_skeleton_live::sonda_do_reset_na_hierarquia_tests`):
//! a tabela daquele menu é **plana** — ela não sabe o que a linha é —, e `*t = Transform::IDENTITY`
//! sobre um osso movia a arte presa **26,48 unidades num desenho de 60**, com uma mensagem
//! **verde**.
//!
//! ⚠️ **Este gate é da SHELL porque a lei já tem os dela:** sete gates puros medem a porta em
//! `ph2d-skeleton-live`. O que só aqui se pode afirmar é que o **gesto** a consulta — *uma lei viva
//! que nenhum gesto consulta é uma lei órfã*, e este repo tem uma nomeada (o `dock_columns::close`,
//! com zero chamadores de produto).
//!
//! ⚠️ **A agulha vive NESTE ficheiro e o sujeito no outro**, de propósito: um `include_str!` cuja
//! agulha está escrita dentro do próprio ficheiro que ele lê conta-se a si mesmo e não afirma nada.

const HIER_FONTE: &str = include_str!("../../src/render_loop/hierarchy_reset.rs");

/// ⛔⛔ **A régua mede CÓDIGO, e a prosa é deitada fora ANTES** — esta é a armadilha que a Fase B
/// da física já pagou por escrito (*«uma régua textual a varrer `\bApp\b` lê o doc-comment que
/// EXPLICA a cura»*), e ela mordeu outra vez aqui: o cabeçalho deste ficheiro **cita** a linha da
/// identidade para dizer que ela era o defeito, logo a agulha aparecia **antes** da porta e o gate
/// reprovou sobre produto CERTO.
fn codigo(fonte: &str) -> String {
    fonte
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//!") && !t.starts_with("///") && !t.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
/// O ficheiro que DRENA o verbo — ele tem de continuar a delegar, senão o irmão fica órfão e a
/// identidade volta a ser escrita aqui.
const DISPATCH: &str = include_str!("../../src/render_loop/hierarchy.rs");
const FASE: &str = include_str!("../../src/render_loop/fase_skeleton_verbs.rs");
const CLICKS: &str = include_str!("../../src/render_loop/fase_bus_clicks.rs");

/// ⭐ **O menu plano pergunta à porta, e a identidade fica no braço de quem NÃO é osso.**
///
/// ⚠️ **As duas metades são dois defeitos:** não perguntar é o defeito do dono de volta; perguntar
/// e continuar a escrever a identidade fora do braço certo é o mesmo defeito com um `match` por
/// cima.
#[test]
fn o_menu_da_hierarquia_pergunta_a_porta_do_repouso() {
    let hier = codigo(HIER_FONTE);
    let hier = hier.as_str();
    let porta = hier.find("pose_de_repouso::repor_transformacao(").expect(
        "o Reset Transform da Hierarquia deixou de consultar a porta do repouso: sobre um \
             osso ele volta a mandar a arte presa 26 unidades para longe",
    );
    let arm = hier
        .find("Reposicao::NaoEOsso")
        .expect("o braço de quem nao e' osso saiu — este gate ficou sem sujeito");
    let identidade = hier.find("*t = Transform::IDENTITY").expect(
        "a lei de sempre da Hierarquia saiu: uma sprite tem de continuar a ir para a origem",
    );
    assert!(
        porta < identidade,
        "a identidade e' escrita ANTES de a porta ser consultada: quando ela responde, o osso ja' \
         foi para a origem"
    );
    assert!(
        arm < identidade,
        "a identidade nao esta' dentro do braço `NaoEOsso`: ela volta a alcançar um osso"
    );
    // ⚠️ **E o despacho tem de continuar a delegar**: o corte por tecto de função pôs a lei num
    // irmão, e um irmão que ninguém chama é uma lei órfã — que este repo já tem uma.
    assert!(
        DISPATCH.contains("hierarchy_reset::drain("),
        "o despacho da Hierarquia deixou de chamar o dreno do reset: o verbo ficou inerte"
    );
    assert!(
        !DISPATCH.contains("*t = Transform::IDENTITY"),
        "a identidade voltou ao despacho, ao lado da porta: sao duas respostas a' mesma pergunta, \
         e a que o artista ve' e' a que envelhece"
    );
}

/// ⭐⭐ **As duas respostas de um osso chegam à TELA** — a que repôs e a que recusou.
///
/// ⛔ *Uma recusa que só o terminal vê é um botão mudo*, e esta é a única saída de um rig anterior
/// a esta wave: sem repouso guardado o verbo é **inerte**, e o artista tem de saber porquê.
#[test]
fn as_duas_respostas_de_um_osso_falam_na_tela() {
    let hier = codigo(HIER_FONTE);
    assert!(
        hier.contains("shell.hierarchy.bone_back_to_rest"),
        "repor a pose de um osso deixou de dizer o que fez"
    );
    assert!(
        hier.contains("shell.hierarchy.bone_has_no_rest"),
        "um osso SEM repouso guardado voltou a ser mudo — e inerte e mudo le^-se como partido"
    );
}

/// ⭐⭐⭐ **OS DOIS BOTÕES DO PAINEL CHEGAM À LEI** — o clique, o encaminhamento e a aplicação.
///
/// ⚠️ **As três metades são três defeitos desta casa, todos já pagados:** um id que o `fase_bus_clicks`
/// não conhece morre no painel; um verbo que a fase não aplica acende e não faz nada; e sem a
/// recusa, um osso sem repouso é indistinguível de um botão partido.
#[test]
fn os_dois_verbos_do_repouso_chegam_do_botao_ate_a_lei() {
    for id in ["VECTOR_BONE_REST_APPLY", "VECTOR_BONE_REST_SET"] {
        assert!(
            CLICKS.contains(id),
            "o botao {id} nao e' encaminhado: ele pinta, acende sob o rato e o clique morre \
             dentro do painel — indistinguivel, do lado de fora, de um verbo que recusou"
        );
    }
    assert!(
        FASE.contains("pose_de_repouso::aplica("),
        "a fase dos verbos do osso deixou de aplicar o repouso: os dois botoes ficam acesos e \
         inertes"
    );
    assert!(
        FASE.contains("RecusaDoOsso::SemPoseDeRepouso"),
        "um osso sem repouso guardado deixou de falar: o botao parece partido"
    );
}
