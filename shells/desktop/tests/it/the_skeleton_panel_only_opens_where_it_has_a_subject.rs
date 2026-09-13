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

/// **O QUADRO como UNIDADE**: o `render_loop/mod.rs` e toda `render_loop/fase_*.rs`, concatenados.
///
/// ⚠️ Desde a OBRA 2 da `line/render-loop` (2026-09-13) o quadro vive em FASES: a aresta do foco e a única escrita da
/// visibilidade mudaram-se para a `fase_selection_mirror_bone_focus`. O gate de baixo é um CENSO («escrita em 1
/// sítio»), e a unidade dele não pode ser o texto emendado — que repetiria o que ainda mora no `run_render_frame` —
/// nem só o `mod.rs`, que acharia ZERO e reprovaria, ou acharia 1 com uma segunda escrita escondida numa fase. É a
/// unidade do quadro inteiro, cada ficheiro uma vez — a mesma cura do censo dos sons (P4e). ⛔ Com PISO de população:
/// uma varredura que achasse só o `mod.rs` voltaria a medir metade.
static LOOP: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("ler src/render_loop")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "mod.rs" || (n.starts_with("fase_") && n.ends_with(".rs")))
        })
        .collect();
    files.sort();
    assert!(
        files.iter().any(|p| p.ends_with("mod.rs")) && files.len() >= 30,
        "o quadro achou {} ficheiros (sem o mod.rs?) — o censo mediria metade",
        files.len()
    );
    files
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("ler um ficheiro do quadro"))
        .collect::<Vec<_>>()
        .join("\n")
});

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

/// **O corpo do bloco que abre a partir da linha `i`** — do primeiro `{` até ao `}` que o fecha,
/// contando chavetas no fonte SEM comentários.
///
/// ⚠️ A aresta do foco media-se com [`codigo_apos`] (8 linhas de código), e a janela partiu-se em
/// 2026-09-12 **sem uma linha de lei mudar**: o campo passou a `self.skeleton.osso_revelado`, a
/// chamada deixou de caber numa linha, o `rustfmt` pôs a `{` sozinha — e a linha que ARMA o verbo
/// ficou na 9.ª. *Uma janela medida em linhas é uma distância, e o `rustfmt` muda distâncias*; o que
/// o gate afirma é uma relação — as três metades estão DENTRO do `if` da aresta.
fn corpo_do_bloco(linhas: &[&str], i: usize) -> String {
    let resto = linhas[i..].join("\n");
    let Some(abre) = resto.find('{') else {
        return String::new();
    };
    let mut fundo = 0i32;
    for (k, c) in resto[abre..].char_indices() {
        match c {
            '{' => fundo += 1,
            '}' => {
                fundo -= 1;
                if fundo == 0 {
                    return resto[abre..=abre + k].to_string();
                }
            }
            _ => {}
        }
    }
    String::new()
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
    let src = code_only(&LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let escritas: Vec<usize> = linhas
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("SkeletonPanel as ph2d_editor_core::panel::Panel>::ID"))
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
        codigo_apos(&linhas, aresta, 8)
            .contains("SkeletonPanel as ph2d_editor_core::panel::Panel>::ID"),
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
    let src = code_only(&LOOP);
    let linhas: Vec<&str> = src.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.contains("skeleton_reveal::on_focus("))
        .expect("a aresta do foco deixou de ser perguntada — o painel nunca vem à frente");
    let corpo = corpo_do_bloco(&linhas, i);
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

/// ⭐⭐⭐ **UM OSSO ACABADO DE NASCER NÃO ACORDA A ARESTA** (report do dono, 2026-09-09: *«cada vez
/// que se cria um osso o modo Transform é selecionado»*).
///
/// ⛔⛔ **A aresta nunca teve defeito — faltava-lhe a outra metade da história.** O osso novo fica
/// aceso (é assim que o artista vê qual é), e no quadro seguinte o `on_focus` lia essa mudança como
/// *«o artista escolheu um osso»*: abria o painel, trazia a aba e **armava *Transform***, arrancando
/// o artista do verbo em que ele estava. *«O foco mudou» tem duas causas que se leem iguais no fim
/// do quadro — o artista apontou, e o gesto produziu — e só quem produziu as distingue.*
///
/// ⇒ quem cria o osso **alimenta a memória** (`skeleton_reveal::on_birth`), e a aresta seguinte não
/// tem nada a relatar. ⚠️ A lei em si é gateada no módulo dela (`a_newborn_bone_asks_for_nothing`);
/// o que este gate mede é que ela está **no caminho de quem cria**, que é a metade que um teste de
/// unidade não alcança.
#[test]
fn the_bone_creation_site_absorbs_the_focus_edge() {
    const DISPATCH: &str = include_str!("../../src/input_dispatch.rs");
    let src = code_only(DISPATCH);
    let linhas: Vec<&str> = src.lines().collect();
    let nascimento = linhas
        .iter()
        .position(|l| l.contains("bone_gesture::create("))
        .expect("o sítio onde um osso nasce do gesto deixou de existir");
    assert!(
        codigo_apos(&linhas, nascimento, 20).contains("skeleton_reveal::on_birth("),
        "o sítio que cria o osso (linha {nascimento}) não absorve a memória do revelar-ao-focar — \
         sem isso o quadro seguinte lê o osso novo como uma ESCOLHA e arma *Transform*, que é \
         literalmente o report do dono"
    );
}

/// ⭐⭐⭐ **O PAI DE UM OSSO NOVO VEM DO PRESS, NUNCA DA SELECÇÃO** (ordem do dono, 2026-09-09:
/// *«para criar um osso como filho de outro o clique deve acontecer na ponta do osso pai»*).
///
/// ⛔⛔ **A lei antiga sobrevivia no OUTRO extremo do gesto:** o press decidia a origem e o release
/// ia buscar o pai a `hero.gizmo.selection`. Enquanto essa leitura existir, um esqueleto na cena
/// torna impossível criar uma raiz nova — que é o report — mesmo com o press já a decidir certo.
///
/// ⚠️ Este gate mede a **AUSÊNCIA**: nada no `Up` do osso pergunta à selecção quem é o pai. A metade
/// positiva (o press decide, e decide certo) é gateada no módulo do gesto.
#[test]
fn the_release_never_asks_the_selection_who_the_parent_is() {
    const DISPATCH: &str = include_str!("../../src/input_dispatch.rs");
    let src = code_only(DISPATCH);
    let linhas: Vec<&str> = src.lines().collect();
    let up = linhas
        .iter()
        .position(|l| l.contains("self.skeleton.bone_drag.take()"))
        .expect("o release do gesto de osso deixou de existir");
    let corpo = codigo_apos(&linhas, up, 30);
    assert!(
        corpo.contains("nascimento.parent") || corpo.contains(".parent"),
        "o release não lê o pai que o press decidiu — o parentesco perdeu-se entre os dois extremos \
         do gesto"
    );
    assert!(
        !corpo.contains("gizmo.selection)") && !corpo.contains("h.gizmo.selection"),
        "o release volta a perguntar à SELECÇÃO quem é o pai — é a lei que o dono mandou tirar, e \
         com ela nenhum press consegue fazer uma raiz nova enquanto houver um osso aceso"
    );
}

/// ⭐⭐⭐ **O RELEASE EMENDA PELA MESMA PORTA QUE O DESENHO CONSULTOU** (ordem do dono, 2026-09-09).
///
/// ⛔⛔ **O artista viu o osso novo SALTAR para aquela bolinha.** Se o release perguntasse outra
/// coisa — uma segunda varredura, um raio próprio, a selecção — ele receberia um osso que não é o
/// que estava desenhado. *Uma pré-visualização que promete uma emenda que o gesto não faz é a
/// espécie de cena que o `CLAUDE.md` §5.0 chama de pior que ausente.*
///
/// ⇒ este gate mede a **SEQUÊNCIA** dentro do release: perguntar a porta · encaixar a ponta ·
/// criar · adoptar. A metade do *o que a porta responde* é gateada no módulo dela.
#[test]
fn the_release_splices_through_the_same_door_the_preview_asked() {
    const DISPATCH: &str = include_str!("../../src/input_dispatch.rs");
    // ⛔⛔ **A AGULHA SEGUE O SUJEITO, E EU APONTEI-A PARA ONDE O FICHEIRO FOI** (W2 Fase C).
    //
    // A 1.ª correcção desta fatia mandou este `PICK` para
    // `crates/ph2d-app-skeleton/src/bone_pick.rs`, porque foi para lá que o ficheiro se mudou. Mas o
    // que este gate mede é **a pré-visualização** (`refresh_bone_hover`), e essa é um `impl App` —
    // ela ficou na shell, no `skeleton_app_bridge.rs`. O gate compilou (o caminho novo existe) e
    // reprovou a correr, que é a forma MENOS má desta armadilha.
    //
    // ⚠️ **É a 3.ª vez que esta linha a paga** (a 1.ª foi a agulha do marquee, a 2.ª a das setas do
    // morph). *Re-apontar uma agulha é perguntar para onde foi o SUJEITO dela, nunca para onde foi
    // o ficheiro* — e aqui o ficheiro partiu-se em dois, com o sujeito de cada metade diferente.
    const PREVIEW: &str = include_str!("../../src/skeleton_app_bridge.rs");
    let src = code_only(DISPATCH);
    let linhas: Vec<&str> = src.lines().collect();
    let up = linhas
        .iter()
        .position(|l| l.contains("self.skeleton.bone_drag.take()"))
        .expect("o release do gesto de osso deixou de existir");
    let corpo = codigo_apos(&linhas, up, 40);
    for (agulha, o_que) in [
        (
            "bone_gesture::drag_now(",
            "PERGUNTAR a porta que o desenho leu",
        ),
        ("agora.tip", "nascer na ponta ENCAIXADA"),
        ("bone_gesture::connect(", "ADOPTAR a corrente solta"),
    ] {
        assert!(
            corpo.contains(agulha),
            "o release não faz «{o_que}» — a emenda entrega meio gesto"
        );
    }
    // ⚠️ E a ADOPÇÃO vem DEPOIS da criação: o osso do meio é o pai, e ele só existe ali.
    let (i_cria, i_liga) = (
        corpo
            .find("bone_gesture::create(")
            .expect("a criação saiu do release"),
        corpo
            .find("bone_gesture::connect(")
            .expect("a adopção saiu do release"),
    );
    assert!(
        i_cria < i_liga,
        "a adopção corre ANTES da criação — não há osso novo onde pendurar a corrente"
    );
    // ⚠️⚠️ **E a pré-visualização lê a MESMA função.** Sem esta metade, o desenho e o release podem
    // divergir sem que nenhum gate de unidade o veja: cada um responde certo à sua própria pergunta.
    // ⛔ E ela lê os TRÊS campos de lá — resolver qualquer um por si reabre a divergência.
    let pick = code_only(PREVIEW);
    assert!(
        pick.contains("bone_gesture::drag_now("),
        "a pré-visualização não consulta a porta da emenda — o artista veria o osso encaixar num \
         sítio e recebê-lo-ia noutro"
    );
    for campo in ["a.tip", "a.armed", "a.splice"] {
        assert!(
            pick.contains(campo),
            "a pré-visualização resolve «{campo}» por si em vez de o ler da porta — é a divergência \
             desenho/release a voltar por dentro"
        );
    }
}
