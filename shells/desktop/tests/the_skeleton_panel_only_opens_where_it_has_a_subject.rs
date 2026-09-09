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

/// **As `n` linhas de CÓDIGO a seguir a `i`** — as vazias e as comentadas não contam.
///
/// ⚠️⚠️ **Ela existe porque uma janela de LINHAS mede a densidade dos comentários, não o corpo.** A
/// 1.ª redacção do gate da aresta procurava nas `12` linhas seguintes e reprovou sobre produto
/// **correcto**: a terceira metade estava lá, atrás de um bloco de nota de quatro linhas. *Num
/// ficheiro em que a nota é metade do texto, contar linhas é contar prosa.*
fn codigo_apos(linhas: &[&str], i: usize, n: usize) -> String {
    linhas[i + 1..]
        .iter()
        .filter(|l| !l.trim().is_empty())
        .take(n)
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

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

/// ⭐⭐⭐ **AS DUAS PORTAS SÃO DE ARESTA, e nenhuma escreve em todo quadro.**
///
/// ⛔⛔ **A lei mudou por ordem do dono (2026-09-09) e este gate mudou com ela.** A 1.ª redacção
/// media *«a visibilidade olha para a cena E para a ferramenta»*, que era a lei de horas antes:
/// a shell escrevia `tem_esqueleto || ferramenta_osso` em **TODO quadro**. Com a linha *Window →
/// Bones* isso passou a ser um interruptor morto — o quadro seguinte repunha a decisão da shell por
/// cima da do artista. *Duas fontes de verdade para o mesmo bool, e a que o artista toca é a que
/// perde.*
///
/// ⇒ o que este gate mede agora é a **AUSÊNCIA**: nada escreve a visibilidade deste painel fora de
/// uma aresta.
#[test]
fn nothing_writes_the_visibility_every_frame() {
    let src = code_only(LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let escritas: Vec<usize> = linhas
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("SkeletonPanel as ph2d_editor::panel::Panel>::ID"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        escritas.len(),
        1,
        "a visibilidade do painel de Bones é escrita em {} sítios — com mais de um, o menu *Window*\
         vira um interruptor que o quadro seguinte desfaz",
        escritas.len()
    );
    // ⚠️ E a única escrita é a da ARESTA: ela mora dentro do `if` do `on_focus`.
    let aresta = linhas
        .iter()
        .position(|l| l.contains("skeleton_reveal::on_focus("))
        .expect("a aresta do foco deixou de ser perguntada");
    assert!(
        codigo_apos(&linhas, aresta, 8).contains("SkeletonPanel as ph2d_editor::panel::Panel>::ID"),
        "a escrita da visibilidade (linha {}) não está dentro da aresta do foco (linha {aresta}) — \
         fora dela ela corre em todo quadro",
        escritas[0]
    );
}

/// ⭐⭐⭐ **A ARESTA faz as TRÊS coisas que o dono descreveu.**
///
/// ⛔⛔ *«Se já existe um osso no mundo, ao selecionar o osso o painel de Bones é aberto e o botão
/// Transform é selecionado»* (2026-09-09). São **três** metades — abrir · trazer à frente · armar —
/// e as três saem da MESMA aresta: abrir sem armar deixaria a fileira apagada sobre um osso
/// escolhido, e armar sem abrir armaria um verbo que ninguém vê.
///
/// ⚠️ A ARESTA continua a ser a lei (`skeleton_reveal::on_focus`, gateada no seu módulo): pedi-la em
/// todo quadro prenderia a aba e o artista não conseguiria olhar para outra.
#[test]
fn the_focus_edge_opens_raises_and_arms() {
    let src = code_only(LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.contains("skeleton_reveal::on_focus("))
        .expect("a aresta do foco deixou de ser perguntada — o painel nunca vem à frente");
    let corpo = codigo_apos(&linhas, i, 8);
    for (agulha, o_que) in [
        ("set_panel_visible", "ABRIR o painel"),
        ("bump_panel_z", "trazer a ABA à frente"),
        ("bone_arm_pending", "armar o verbo *Transform*"),
    ] {
        assert!(
            corpo.contains(agulha),
            "a aresta do foco não faz «{o_que}» — as três metades saem da MESMA aresta, e uma \
             sozinha entrega meio gesto"
        );
    }
    assert!(
        corpo.contains("SKELETON_PANEL"),
        "a aresta traz OUTRO painel à frente"
    );
}
