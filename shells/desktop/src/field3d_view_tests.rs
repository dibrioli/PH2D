//! Os gates do **estado de vista** — ver [`super`].

use super::*;
use crate::field3d_gizmo::{Frame, Mode};
use crate::field3d_smoke::{set_armed_by_panel, with_smoke};

/// Põe o módulo de volta ao estado de repouso, para que os gates seguintes não herdem nada.
///
/// ⚠️ **Isto só é possível desde a W42.** Enquanto a bandeira do pill travava ligada, armar o
/// módulo num gate era irreversível: o `Smoke` nascia e ficava, e todo gate que corresse depois
/// (no mesmo processo, com `--test-threads=1`) via um módulo armado que ninguém tinha pedido. *Era
/// esse o motivo escrito ao lado da `next_isolation` para a lei ser testada sem estado.*
fn disarm() {
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
    forget();
}

/// ⭐⭐ **FECHAR O PAINEL NÃO PERDE A VISTA** — o ⏸️ que a W42 deixou escrito.
///
/// A W42 fez desarmar desarmar de verdade, e o preço foi nomeado na hora: *"fica: fechar o painel
/// larga a **câmera** (a peça não)"*. Este gate é a cobrança desse preço.
///
/// ⚠️ **Ele mede o CAMINHO DO ARTISTA, e não uma função pura** — abrir o painel, pousar a peça num
/// ângulo, pegar noutra ferramenta (o painel fecha), voltar. É a única forma de a costura
/// painel↔smoke ficar provada; a W38 tinha-a como buraco declarado (*"sem gate comportamental da
/// costura painel↔smoke"*), e o que a destrancou foi a lei nova: **desarmar agora limpa-se a si
/// mesmo**, então um gate pode armar o módulo sem contaminar os vizinhos.
#[test]
fn closing_the_panel_keeps_the_view_it_had() {
    disarm();
    set_armed_by_panel(true);

    let start = with_smoke(|s| s.vp().cam).expect("o pill arma o módulo");
    let posed = with_smoke(|s| {
        crate::field3d_input::law::orbit(&mut s.vp_mut().cam, 40.0, 15.0);
        s.vp_mut().manual = true;
        s.gizmo_mode = Mode::Rotate;
        s.gizmo_frame = Frame::Local;
        (s.vp().cam, s.vp().manual, s.gizmo_mode, s.gizmo_frame)
    })
    .expect("armado");
    assert_ne!(
        posed.0, start,
        "o controle do gate: a câmera de facto mexeu-se"
    );

    // O artista pega noutra ferramenta — a W40 fecha o painel, a W42 desarma o módulo.
    set_armed_by_panel(false);
    assert!(
        with_smoke(|_| ()).is_none(),
        "desarmado, todo gancho de entrada é inerte — é a lei da W42"
    );

    // E volta.
    set_armed_by_panel(true);
    let back = with_smoke(|s| (s.vp().cam, s.vp().manual, s.gizmo_mode, s.gizmo_frame))
        .expect("o pill rearma o módulo");
    assert_eq!(
        back, posed,
        "reabrir o MODEL devolveu uma vista NOVA — a câmera, o prato parado e os dois seletores do \
         gizmo morreram com o cache do quadro. O que fecha é o painel, não a vista do artista."
    );

    disarm();
}

/// ⭐ **E o PRATO não volta a girar** — a metade sem a qual restaurar a câmera não vale nada.
///
/// ⚠️ O `manual` é *"o prato para de girar assim que o artista toca nele"*. Se ele voltasse a
/// `false` no reabrir, a câmera seria restaurada e **imediatamente afastada** do ângulo restaurado,
/// um grau por quadro — o defeito leria como *"restaurar a câmera não funciona"*, e o número certo
/// estaria lá durante um quadro. *Duas metades de um mesmo fato, e provar só uma engana.*
#[test]
fn the_turntable_stays_stopped_across_a_close() {
    disarm();
    set_armed_by_panel(true);
    with_smoke(|s| s.vp_mut().manual = true).expect("armado");
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
    set_armed_by_panel(true);
    assert!(
        with_smoke(|s| s.vp().manual).expect("rearmado"),
        "o prato voltou a girar sozinho por cima do ângulo que o artista pousou"
    );
    disarm();
}

/// ⭐ **Sem nada lembrado, a vista é a PADRÃO** — a primeira abertura do app não herda nada.
#[test]
fn the_first_open_gets_the_default_view() {
    disarm();
    assert_eq!(
        recall(),
        View::default(),
        "esquecida, a memória da vista tem de ler exatamente como a primeira abertura"
    );
    set_armed_by_panel(true);
    let fresh =
        with_smoke(|s| (s.vp().cam, s.vp().manual, s.gizmo_mode, s.gizmo_frame)).expect("armado");
    let d = View::default();
    assert_eq!(
        fresh,
        (d.cam, d.manual, d.gizmo_mode, d.gizmo_frame),
        "sem memória, um módulo novo tem de nascer no padrão — senão o `View::default` e o `boot` \
         são duas ideias diferentes do que é uma vista nova"
    );
    disarm();
}

/// ⭐⭐ **UM DOCUMENTO NOVO NÃO HERDA O ISOLAMENTO — e herda a CÂMERA.**
///
/// ⚠️ Este é o buraco que a W43 **abriu** ao fazer a vista sobreviver: enquanto ela morria no
/// fecho, um isolamento nunca podia atravessar um Ctrl+O com o painel fechado. O `isolated` guarda
/// **bits de entidade**, e o mundo novo realoca-os — no melhor caso aponta para nada, no pior
/// acerta noutro nó e a peça nova abre quase toda escondida.
///
/// ⚠️ **As duas metades no mesmo gate, de propósito.** Um gate que só provasse o esquecimento
/// passaria com um `LAST.set(None)` — a cura preguiçosa, que deitaria fora a câmera que esta wave
/// inteira existe para guardar. *Uma cura que apaga tudo passa em qualquer gate que só peça
/// ausência.*
#[test]
fn a_new_document_forgets_the_isolation_and_keeps_the_camera() {
    disarm();
    set_armed_by_panel(true);
    let posed = with_smoke(|s| {
        crate::field3d_input::law::orbit(&mut s.vp_mut().cam, 25.0, -10.0);
        s.isolated = Some(11);
        s.vp().cam
    })
    .expect("armado");
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());

    forget_isolation_across_documents();

    let v = recall();
    assert_eq!(
        v.isolated, None,
        "o isolamento do projeto ANTERIOR atravessou o Ctrl+O — os bits são de outro mundo"
    );
    assert_eq!(
        v.cam, posed,
        "a câmera foi esquecida junto: é a cura preguiçosa (largar a vista inteira), e ela desfaz \
         exatamente o que esta wave comprou"
    );
    disarm();
}

/// ⚠️ **O ISOLAMENTO é vista, e viaja com ela** — a lei foi **lida** no módulo irmão, não decidida
/// aqui: lá o `isolated` vive na cena (`sculpt3d_objects`), que sobrevive a sair do modo.
///
/// ⛔ A tentação era largá-lo *"por segurança"*, já que nada na Hierarquia mostra que há um
/// isolamento em curso (⏸️ aberto da W38). Mas isso faria a mesma bandeira ter **duas** leis de
/// tempo de vida em dois módulos irmãos, e a cura do buraco é o indicador — não amputar o estado.
#[test]
fn an_isolation_survives_the_close_like_its_sibling_does() {
    disarm();
    set_armed_by_panel(true);
    with_smoke(|s| s.isolated = Some(7)).expect("armado");
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
    set_armed_by_panel(true);
    assert_eq!(
        with_smoke(|s| s.isolated).expect("rearmado"),
        Some(7),
        "o isolamento é estado de VISTA (o próprio doc do campo o diz) e o módulo irmão guarda-o \
         na cena, que sobrevive a sair do modo"
    );
    disarm();
}

/// ⭐⭐⭐ **NADA PODE ESVAZIAR A LISTA DE VIEWPORTS** (W90) — a invariante que torna
/// [`crate::field3d_smoke::Smoke::vp`] infalível.
///
/// # Porque ela é um censo, e não um `assert` num sítio
///
/// `vp()` prende o índice ao alcance e devolve `&Viewport` **sem `Option`** — é isso que mantém os
/// ~30 sítios que perguntam pela câmera livres de responder *«e se não houver vista nenhuma?»*, uma
/// pergunta que o produto não tem. O preço dessa comodidade é uma invariante: a lista **nunca** é
/// vazia. Com ela partida, `self.vps.len() - 1` faz *underflow* e o módulo entra em pânico no
/// caminho mais quente que tem.
///
/// ⚠️ **Uma invariante mantida por construção não tem um sítio onde a afirmar** — ela afirma-se
/// sobre a AUSÊNCIA dos verbos que a partiriam. Quem acrescentar a divisão do canvas tem de vir
/// aqui **nomear** onde encolhe a lista, e é essa a conversa que este gate força.
#[test]
fn nothing_can_empty_the_viewport_list() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    // Os verbos que ENCOLHEM uma `Vec`. ⚠️ `remove`/`drain`/`retain`/`truncate` também encolhem, e
    // um deles com a conta errada deixa a lista vazia tão bem como um `clear`.
    const ENCOLHEM: [&str; 6] = [
        "vps.clear(",
        "vps.pop(",
        "vps.remove(",
        "vps.drain(",
        "vps.truncate(",
        // ⛔⛔ **A ATRIBUIÇÃO estava fora da lista, e é o modo mais óbvio de todos.** A 1.ª versão
        // deste censo listava cinco verbos de `Vec` e deixou passar `vps = <o que for>` — que é
        // exactamente o que a divisão do canvas veio a fazer. *Um censo por texto só apanha o que
        // alguém se lembrou de escrever, e o que se esquece é o caso NORMAL.*
        "vps = ",
    ];
    /// ⭐ **O único sítio autorizado, com o motivo.** Ver `field3d_smoke::ensure_viewports`: ele
    /// reconstrói a lista quando a divisão muda (a vista do artista muda de quadrante, então não é
    /// um `push`), e **prende o `n` a `≥ 1` na primeira linha** — é lá que a invariante vive.
    const AUTORIZADOS: [(&str, &str); 1] = [("field3d_viewports.rs", "smoke.vps = novos;")];
    let mut achados: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("src existe") {
        let path = entry.expect("entrada").path();
        let nome = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // ⚠️ **Só o PRODUTO.** A varredura tem de se excluir a si própria — a primeira corrida
        // deste gate apanhou a lista de verbos que ele define, que é o modo de falha clássico de um
        // censo por texto. E a fronteira certa não é *«este ficheiro»*: é *«código que corre no
        // app»*, porque a invariante é sobre ele.
        if !nome.starts_with("field3d_") || !nome.ends_with(".rs") || nome.ends_with("_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).expect("lê");
        for (i, linha) in src.lines().enumerate() {
            if linha.trim_start().starts_with("//") {
                continue;
            }
            for verbo in ENCOLHEM {
                if linha.contains(verbo)
                    && !AUTORIZADOS
                        .iter()
                        .any(|(f, l)| *f == nome && linha.trim() == *l)
                {
                    achados.push(format!("{nome}:{}: {}", i + 1, linha.trim()));
                }
            }
        }
    }
    assert!(
        achados.is_empty(),
        "alguém encolhe a lista de viewports, e ela não pode ficar vazia — o `Smoke::vp` faz \
         `len() - 1`. Se este sítio é legítimo (a divisão do canvas a fechar uma vista), NOMEIE-O \
         aqui com o motivo em vez de apagar o gate:\n{}",
        achados.join("\n")
    );
}

/// ⭐⭐⭐ **A DIVISÃO SOBREVIVE A FECHAR O PAINEL** (W95).
///
/// ⛔⛔ **A W90 deixou-a de fora com uma razão ERRADA:** *«restaurar a divisão obrigaria a restaurar
/// as quatro câmeras»*. É falso — três delas são **derivadas** (nascem da orientação que o nome
/// promete), e a única autorada é a do artista, que a [`crate::field3d_view::View`] já guardava.
/// *Uma dependência afirmada sem a desmontar é uma feature adiada com cara de arquitectura.*
///
/// ⚠️ E a divisão **pertence** à vista pela mesma razão que a câmera: é uma preferência de bancada.
/// Um artista que trabalha em quatro vistas, pega no editor vetorial e volta não quer encontrar uma.
#[test]
fn the_split_survives_closing_the_panel() {
    use crate::field3d_layout::Split;
    use crate::field3d_smoke::{set_armed_by_panel, with_smoke};
    set_armed_by_panel(true);
    with_smoke(|s| {
        crate::field3d_smoke::toggle_split(s);
        assert!(matches!(s.split, Split::Quad { .. }), "abriu a divisão");
        // Uma costura fora do meio: o que se lembra é a divisão, e não a posição dela.
        s.split = s.split.with_t(0.3, 0.7);
    });
    // Fechar e reabrir.
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
    set_armed_by_panel(true);
    with_smoke(|s| {
        assert!(
            matches!(s.split, Split::Quad { .. }),
            "reabrir devolveu a vista única — a divisão é preferência de bancada, como a câmera"
        );
        assert_eq!(
            s.vps.len(),
            Split::quad().count(),
            "a lista nasceu com uma vista só: o `split` diria «quatro» num quadro em que a lista \
             tem uma, e alguém leria esse quadro"
        );
        // Volta ao estado de repouso para não deixar herança ao teste seguinte.
        crate::field3d_smoke::toggle_split(s);
    });
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
}
