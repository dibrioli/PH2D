//! ⭐⭐⭐ **O PAINEL DO ESQUELETO SÓ ABRE ONDE TEM SUJEITO.**
//!
//! ⛔⛔ **A lei MUDOU DE DONO em 2026-09-09.** Enquanto o esqueleto era uma *secção* do painel de
//! vector, ela decidia sozinha se se pintava (`if !has_skeleton() && mode != Bone { return }`). Com
//! painel próprio, quem decide é a **shell** — a mesma porta de todos os painéis — e o gate tinha
//! de vir atrás: *uma lei que muda de dono e deixa o gate para trás é uma lei sem prova.*
//!
//! A lei é a mesma, palavra por palavra: **um painel que fala de algo que não existe é ruído**, e
//! com um esqueleto na cena ele vale em TODA ferramenta (o osso posa-se com a seta, não só com a
//! ferramenta que o cria).
//!
//! ⚠️ **E o «revelar-ao-focar» também mudou de EFEITO sem mudar de lei** (report do dono,
//! 2026-09-08): a rolagem existia porque o cabeçalho caía `1394 px` abaixo de `785 px` de outro
//! assunto. Neste painel ele é a **primeira** linha — o que sobra é o encaixe partilhado, e revelar
//! passa a ser **trazer a aba à frente** (`bump_panel_z`).

const LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **O fonte sem comentários** — sem isto, uma nota que cita a chamada conta como chamada.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A shell DECIDE a visibilidade, e decide-a com os DOIS factos.**
///
/// ⚠️⚠️ **A 1.ª redacção deste gate era VÁCUA e as três mutações SOBREVIVERAM:** ela procurava
/// `tem_esqueleto` e `ferramenta_osso` numa **janela de bytes** à volta da chamada — e as duas
/// linhas que os DECLARAM caem dentro dessa janela, então tirar um deles do ARGUMENTO não movia
/// nada. *Um gate que mede a vizinhança de um nome mede o nome, não a origem* — a mesma família que
/// mordeu o gizmo do limite em 2026-09-08.
///
/// ⇒ a âncora é a **linha do argumento**, que é onde a decisão de facto vive.
#[test]
fn the_shell_opens_it_from_the_scene_and_from_the_tool() {
    let src = code_only(LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.contains("SkeletonPanel as ph2d_editor::panel::Panel>::ID"))
        .expect(
            "a shell nunca publica a visibilidade do painel do esqueleto — ele fica DEFAULT_VISIBLE \
             = false para sempre, e o artista não tem gesto nenhum que o abra",
        );
    // O `set_panel_visible` tem três argumentos, um por linha: `hero`, o ID, e a DECISÃO.
    let decisao = linhas[i + 1];
    assert!(
        decisao.contains("tem_esqueleto"),
        "a decisão não olha para a CENA (`{decisao}`) — um painel que aparece sem osso nenhum é ruído"
    );
    assert!(
        decisao.contains("ferramenta_osso"),
        "a decisão não olha para a FERRAMENTA (`{decisao}`) — sem esta metade, a ferramenta que CRIA \
         ossos abriria sem painel, que é exactamente onde o artista está prestes a ter um"
    );
}

/// ⭐⭐⭐ **Um osso NOVO em foco traz a ABA à frente** — o sucessor do «revelar-ao-focar».
///
/// ⚠️ A ARESTA continua a ser a lei (`skeleton_reveal::on_focus`, gateada no seu módulo): pedi-la em
/// todo quadro prenderia a aba e o artista não conseguiria olhar para outra.
///
/// ⚠️ A âncora é a **linha de código seguinte** à pergunta, e não uma janela — pela lição do gate
/// acima e da irmã em `the_bone_pickers_are_modal.rs`.
#[test]
fn a_new_bone_in_focus_brings_the_tab_forward() {
    let src = code_only(LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.contains("skeleton_reveal::on_focus("))
        .expect("a aresta do foco deixou de ser perguntada — o painel nunca vem à frente");
    let corpo = linhas[i + 1];
    assert!(
        corpo.contains("bump_panel_z") && corpo.contains("SKELETON_PANEL"),
        "a aresta é perguntada e o corpo dela é `{corpo}` — se não traz a aba do esqueleto à frente, \
         é o report de 2026-09-08 de volta, com a aba no lugar da rolagem"
    );
}
