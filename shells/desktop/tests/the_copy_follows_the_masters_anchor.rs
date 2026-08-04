//! **A cópia segue a âncora do mestre** — arch-gate sobre a costura que nenhum unit test alcança
//! (irmão de `the_scale_anchor_follows_the_live_modifier`, e pela mesma razão).
//!
//! A LEI mora onde ela é: `crate::vec_instance_follow` é puro e tem os seus gates de kernel (a
//! quina que o mestre segura a cópia segura · a outra quina também · uma escala ancorada no centro
//! não move ninguém · o passo atravessa a parte linear da cópia · uma cópia arrastada fica de fora
//! · aplicar duas vezes aterra no mesmo sítio). O que eles **não podem tocar** é a fiação:
//! `advance_gizmo_drag` precisa de `App` + `HeroScreen` + janela. E é lá que vivem as três metades
//! que decidem se o seguimento é de facto o que o artista vê:
//!
//! 1. **O instantâneo nasce sob ROTAÇÃO/ESCALA e nunca sob TRANSLAÇÃO.** *Mover* o mestre não
//!    pode mover as cópias — é a lei do Figma que o `instance_live::place_delta` existe para
//!    honrar, e um seguimento que a ignorasse faria toda cópia colar-se ao mestre.
//! 2. **A aplicação corre DEPOIS das escritas de pose.** Ela lê a translação de mundo do mestre
//!    AGORA e compara-a com o instantâneo; correr antes mediria o delta do frame ANTERIOR, e a
//!    cópia ficaria um `CursorMoved` atrás do dedo.
//! 3. **Ela é UMA, fora dos ramos.** Quem decide se há algo a seguir é o instantâneo, não uma
//!    segunda enumeração dos ramos que escrevem pose — um ramo novo (a moldura foi o último) nasce
//!    coberto em vez de esquecido.
//!
//! ⚠️ Nada aqui afirma distância em bytes nem vizinhança de linhas — a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy posicional expira na wave
//! seguinte. O que se afirma é *que pergunta é feita* e *onde a resposta pousa*.

use std::fs;

fn drag_src() -> String {
    fs::read_to_string("src/input_dispatch/gizmo_drag.rs").expect("gizmo_drag.rs")
}

/// **(1) O instantâneo é de rotação/escala — nunca de translação.**
#[test]
fn only_a_rotate_or_scale_asks_the_copies_to_follow() {
    let src = drag_src();
    let i = src
        .find("let follows = matches!(")
        .expect("o avanco do arrasto nao decide quando as copias seguem");
    let end = src[i..].find(");").map_or(src.len(), |e| i + e);
    let guard = &src[i..end];
    for kind in ["Rotate", "ScaleCorner", "ScaleEdge"] {
        assert!(
            guard.contains(kind),
            "o seguimento nao cobre `{kind}` — um gesto do gizmo ficaria com a lei do outro:\n{guard}"
        );
    }
    assert!(
        !guard.contains("Translate"),
        "o seguimento cobre a TRANSLACAO: mover o mestre passaria a mover as copias, que e' \
         exactamente a lei que o `place_delta` existe para honrar:\n{guard}"
    );
    assert!(
        src.contains("self.begin_instance_follow("),
        "o avanco do arrasto nao fotografa as copias"
    );
}

/// **(2)+(3) A aplicação é UMA, e corre depois de TODA escrita de pose.**
///
/// ⚠️ O oráculo é a relação entre duas coisas do próprio ficheiro — a última escrita de
/// `Transform` e a chamada —, não uma distância. Mover a chamada para dentro (ou para antes) do
/// encadeamento de ramos deixa-a a ler o mestre do frame anterior.
#[test]
fn the_copies_follow_after_the_pose_is_written() {
    let src = drag_src();
    let call = "crate::vec_instance_follow::apply(";
    assert_eq!(
        src.matches(call).count(),
        1,
        "a aplicacao aparece mais de uma vez: quem decide se ha' algo a seguir e' o instantaneo, \
         nao uma enumeracao dos ramos que escrevem pose"
    );
    let at = src
        .find(call)
        .expect("o avanco do arrasto nao aplica o seguimento");
    let last_write = src
        .rfind("get_mut::<Transform>(")
        .expect("o avanco do arrasto nao escreve pose nenhuma");
    assert!(
        last_write < at,
        "o seguimento corre ANTES da ultima escrita de pose: ele leria a translacao do mestre do \
         frame anterior, e a copia ficaria um `CursorMoved` atras do dedo"
    );
}

/// **O instantâneo morre com o gesto, nos DOIS sítios.**
///
/// ⚠️ O `is_for` sozinho não basta: soltar e voltar a pegar a MESMA alça sem um `CursorMoved` pelo
/// meio não passa pela limpeza do avanço, e o instantâneo antigo descreveria um gesto que já
/// acabou — as cópias saltariam para onde o mestre estava há dois gestos. É a mesma cicatriz que o
/// `frame_resize_start` carrega, e por isso os dois são limpos lado a lado.
#[test]
fn the_snapshot_dies_with_the_gesture() {
    assert!(
        drag_src().contains("self.instance_follow = None;"),
        "o avanco do arrasto nao larga o instantaneo quando nao ha' arrasto"
    );
    let up = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    let i = up
        .find("self.frame_resize_start = None;")
        .expect("o release nao larga o instantaneo da moldura");
    let tail = &up[i..(i + 400).min(up.len())];
    assert!(
        tail.contains("self.instance_follow = None;"),
        "o release larga o instantaneo da moldura e nao o das copias:\n{tail}"
    );
}
