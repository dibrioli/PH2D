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

    let start = with_smoke(|s| s.cam).expect("o pill arma o módulo");
    let posed = with_smoke(|s| {
        crate::field3d_input::law::orbit(&mut s.cam, 40.0, 15.0);
        s.manual = true;
        s.gizmo_mode = Mode::Rotate;
        s.gizmo_frame = Frame::Local;
        (s.cam, s.manual, s.gizmo_mode, s.gizmo_frame)
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
    let back = with_smoke(|s| (s.cam, s.manual, s.gizmo_mode, s.gizmo_frame))
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
    with_smoke(|s| s.manual = true).expect("armado");
    set_armed_by_panel(false);
    let _ = with_smoke(|_| ());
    set_armed_by_panel(true);
    assert!(
        with_smoke(|s| s.manual).expect("rearmado"),
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
    let fresh = with_smoke(|s| (s.cam, s.manual, s.gizmo_mode, s.gizmo_frame)).expect("armado");
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
        crate::field3d_input::law::orbit(&mut s.cam, 25.0, -10.0);
        s.isolated = Some(11);
        s.cam
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
