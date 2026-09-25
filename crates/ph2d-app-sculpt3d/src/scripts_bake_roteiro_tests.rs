//! ⭐⭐⭐⭐ **TODO NOME QUE O ROTEIRO DA `=11` PÕE ENTRE CRASES É UM NOME QUE O DONO LÊ NA TELA.**
//!
//! Irmão (`#[path]`) do [`super`], e o molde é o
//! [`crate::scenes::pintura`]`::tests::todo_nome_entre_crases_do_roteiro_existe_na_tela`, escrito
//! um dia antes por exactamente este defeito: *«um passo que manda clicar numa linha AFIRMA que
//! ela está na tela, e o dono aprova o smoke com o passo impossível dentro»*.
//!
//! # ⛔⛔ Por que ele é POR BLOCO e não sobre o ficheiro inteiro
//!
//! Medido antes de o escrever: o [`super`] tem **368** linhas de roteiro e **12** trechos entre
//! crases, e **dois deles têm várias linhas de prosa lá dentro** — porque dois roteiros nomeiam a
//! **tecla da crase** com uma crase solta (*«abra o painel com a CRASE (`)»*). Um censo sobre o
//! ficheiro inteiro leria essa crase como o início de um nome, colheria meia dúzia de passos como
//! se fossem o rótulo de um controlo, e acusaria prosa.
//!
//! ⚠️ *A alternativa — uma lista de excepções larga — é a catraca a virar licença*, que é a razão
//! pela qual o gate irmão mantém a dele em **duas** entradas. Fica então o bloco desta cena, que é
//! a população que esta wave tocou, com a **paridade das crases** a ser verificada nele.

use ph2d_i18n::tr;
use ph2d_panel_sculpt3d::rows::rows;
use ph2d_panel_sculpt3d::state::{Sculpt3dUi, UiLevel};

/// O fonte que declara o roteiro.
///
/// ⚠️ **Lido por `include_str!`**: se o roteiro mudar de ficheiro isto deixa de **COMPILAR**, em vez
/// de ficar verde a medir menos — a cegueira §2.7 do HOWTO, cujo modo de falha é MUDO.
const FONTE: &str = include_str!("scripts.rs");

/// A primeira linha do bloco da `=11`, que é como o roteiro se apresenta ao dono.
const ABRE: &str = "[sculpt3d] =11 O OBJETO MISTO";

/// ⚠️ **O que o roteiro nomeia e NÃO é um rótulo de painel.** Lista NOMEADA e curta de propósito:
/// ela é a única saída do censo, e o CONTROLO abaixo proíbe que alguém estacione aqui um rótulo
/// que o painel de facto pinta.
const TECLAS: &[&str] = &["Numpad5"];

/// O bloco do roteiro da `=11`, colhido do fonte.
///
/// ⚠️ A régua da colheita é o prefixo `[sculpt3d]`, que **só** as linhas do roteiro trazem — e
/// nunca um doc-comment, que é a prosa que EXPLICA a cura e já enganou uma régua desta família.
fn bloco() -> String {
    let linhas: Vec<&str> = FONTE
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && l.contains("[sculpt3d]"))
        .collect();
    let inicio = linhas
        .iter()
        .position(|l| l.contains(ABRE))
        .unwrap_or_else(|| panic!("o roteiro da =11 nao abre com {ABRE:?} — a cena mudou de nome"));
    // ⛔ O bloco acaba onde o `eprintln!` acaba: a primeira linha do roteiro que FECHA a string
    // (aspas sem a continuação `\n\`). Sem este corte o censo colheria os roteiros vizinhos, onde
    // vive a crase solta que o `//!` acima mede.
    let fim = linhas[inicio..]
        .iter()
        .position(|l| l.trim_end().ends_with('"'))
        .map_or(linhas.len(), |i| inicio + i + 1);
    linhas[inicio..fim].join("\n")
}

/// ⭐⭐⭐ **O veredito, nome a nome.**
#[test]
fn todo_nome_entre_crases_do_roteiro_da_bake_existe_na_tela() {
    let roteiro = bloco();

    // ⛔ **O PISO da colheita:** uma extracção partida devolve um bloco curto, e todo laço abaixo
    // fica trivialmente verdadeiro sobre o vazio.
    let linhas = roteiro.lines().count();
    assert!(
        linhas > 30,
        "a colheita do bloco da =11 deu {linhas} linhas: a extraccao partiu-se"
    );

    // ── A paridade das crases vem PRIMEIRO ──────────────────────────────────
    // ⛔ Uma crase solta faz o emparelhamento colher meia frase como se fosse o nome de um
    // controlo, e o censo passa a medir prosa **em silêncio** — uma frase nunca casa com rótulo
    // nenhum, e o erro lê-se como «o roteiro nomeia algo que não existe».
    let crases = roteiro.matches('`').count();
    assert_eq!(
        crases % 2,
        0,
        "o bloco da =11 tem {crases} crases, um numero IMPAR: ha' uma solta, e o censo abaixo \
         deixa de saber onde acaba um nome"
    );

    // ── As populações do que o painel MOSTRA ────────────────────────────────
    // ⚠️ **No nível em que o painel NASCE** (`Basic`): um rótulo que só o `Pro` pinta é um passo
    // impossível para quem abre a cena e segue o roteiro — o defeito do `Auto-Smooth` da `=41`.
    let ui = Sculpt3dUi {
        ui_level: UiLevel::Basic,
        ..Sculpt3dUi::default()
    };
    let mut na_tela: Vec<String> = rows()
        .filter(|r| r.visible(&ui))
        .map(|r| tr(r.label).to_string())
        .collect();
    // ⚠️ E tudo o que os PINTORES do corpo do painel traduzem — a população que a tabela de rows
    // **não** contém: a fileira da lente vive num deles e a da lei do assado no outro, as duas
    // pintadas à mão.
    //
    // ⚠️⚠️ **São DOIS ficheiros, e o segundo entrou por um gate VERMELHO:** o tecto de LOC do
    // painel obrigou a partir o corpo em *como o barro é moldado* e *o que se faz com a peça*, e o
    // censo — que lia só o primeiro — passou a colher `39` rótulos. ⭐ *Foi o PISO de população que
    // o apanhou*: sem ele a varredura teria ficado verde a medir menos, e o roteiro passaria a
    // poder nomear um controlo da secção Bake que ninguém pinta.
    const PINTORES: [&str; 2] = [
        include_str!("../../ph2d-panel-sculpt3d/src/paint/body.rs"),
        include_str!("../../ph2d-panel-sculpt3d/src/paint/body_peca.rs"),
    ];
    for pintor in PINTORES {
        for pedaco in pintor.split("tr(\"").skip(1) {
            if let Some(chave) = pedaco.split('"').next() {
                na_tela.push(tr(chave).to_string());
            }
        }
    }
    // ⛔⛔ **E uma TERCEIRA população, que a varredura acima NÃO PODE ver — o achado deste gate.**
    //
    // A colheita de cima procura `tr("literal")`, e os chips da LENTE são pintados por
    // `tr(l.label_key())` sobre [`LensMode::ALL`]: *uma fileira derivada de uma tabela é invisível
    // a um scanner de literais*, e o gate reprovou na 1.ª corrida a dizer que `Perspective` não
    // existe na tela — sobre um painel correcto.
    //
    // ⚠️ **Isto NÃO é o oráculo a comparar-se consigo mesmo**, e a distinção é o que o mantém
    // honesto: o que este censo julga é a PROSA do roteiro, escrita à mão, contra a tabela; que a
    // tabela chega a pixel é outro gate (o `populate_censo_tests` da crate do painel), e que ela
    // resolve para a lente da câmera é o `panel_lente_tests`. ⇒ renomear `Orthographic` no i18n
    // **acusa o roteiro**, que é exactamente o defeito que esta família pagou em 2026-09-20.
    //
    // ⚠️⚠️ **E ela é uma FAMÍLIA, não uma excepção — foi o gate a acusar dívida PRÉ-EXISTENTE que
    // o mostrou:** a fileira da LEI do assado (`Form (PBR)` / `Paint`, no roteiro desde a wave
    // dela) pinta os rótulos que vêm do RETRATO, e estava igualmente invisível. *Toda fileira cujos
    // chips saem de uma tabela é invisível a um scanner de literais*, e a lista abaixo é a das
    // tabelas que o corpo do painel lê.
    na_tela.extend(
        ph2d_panel_sculpt3d::state::LensMode::ALL
            .iter()
            .map(|l| tr(l.label_key()).to_string()),
    );
    na_tela.extend(
        ph2d_form_donation::lei_da_luz::CHAVES_DOS_ROTULOS
            .iter()
            .map(|k| tr(k).to_string()),
    );
    // ⛔ **O PISO de população**: uma extracção partida devolveria zero, e um censo que não acha
    // nada acusaria tudo.
    assert!(
        na_tela.len() > 40,
        "o censo colheu so' {} rotulos de tela: a extraccao partiu-se",
        na_tela.len()
    );
    // ⛔ **O CONTROLO da saída**: nenhuma tecla pode ser também um rótulo pintado — senão a lista
    // de excepções vira o sítio onde um controlo morto se esconde do censo.
    for t in TECLAS {
        assert!(
            !na_tela.iter().any(|p| p.as_str() == *t),
            "`{t}` esta' na lista de teclas E e' um rotulo que o painel pinta: a excepcao deixou \
             de ser uma excepcao"
        );
    }
    na_tela.extend(TECLAS.iter().map(|s| (*s).to_string()));

    let mut nomeados = 0usize;
    for (i, pedaco) in roteiro.split('`').enumerate() {
        if i % 2 == 0 {
            continue; // fora das crases
        }
        nomeados += 1;
        assert!(
            !pedaco.contains('\n'),
            "o roteiro parte um nome em duas linhas ({pedaco:?}): um rotulo tem de caber numa \
             linha, senao o dono nao o reconhece na tela"
        );
        assert!(
            na_tela.iter().any(|n| n == pedaco),
            "o roteiro da =11 manda o dono procurar `{pedaco}` e NADA na tela se chama assim — \
             nem fileira da tabela, nem rotulo do corpo do painel, nem tecla. Ele vai procurar \
             uma linha que nao existe."
        );
    }
    // ⛔ E o piso do próprio sujeito: um roteiro que deixasse de nomear controlos passaria este
    // gate por vácuo.
    assert!(
        nomeados >= 5,
        "o bloco da =11 nomeia so' {nomeados} controlos: este censo ficou sem sujeito"
    );
}

/// ⭐⭐ **E o roteiro ENSINA as duas coisas que esta wave trouxe** — a lente e o enquadramento.
///
/// ⛔⛔ *Uma cena que ensina o contrário do que acontece é pior que uma cena ausente* (CLAUDE.md
/// §5.0), e o report do dono de 2026-09-21 tinha **duas** metades: a lente que faltava e o assado
/// que não caía onde ele o pôs. Um roteiro que só falasse de uma delas mandaria o dono aprovar
/// metade da wave sem nunca olhar para a outra.
///
/// ⚠️ **A agulha é a FRASE do passo e não o número dele**: um `(2-bis)` é a numeração, que um corte
/// futuro pode mudar sem que nada se perca; o que não pode desaparecer é o dono ser mandado
/// comparar o enquadramento.
#[test]
fn o_roteiro_ensina_a_lente_e_o_enquadramento() {
    let roteiro = bloco();
    for (assunto, agulha) in [
        ("a LENTE", "`Lens`"),
        ("a lente ORTOGRAFICA", "`Orthographic`"),
        ("a tecla da lente", "Numpad5"),
        ("o ENQUADRAMENTO", "MESMO lugar"),
        ("o tamanho do assado", "MESMO tamanho"),
    ] {
        assert!(
            roteiro.contains(agulha),
            "o roteiro da =11 deixou de ensinar {assunto} (agulha {agulha:?}) — o dono aprovaria \
             a wave sem nunca julgar essa metade do report"
        );
    }
}
