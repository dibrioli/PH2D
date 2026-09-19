//! ⛔⛔⛔ **TODO CHIP DESTA SECÇÃO ERA CORTADO A METADE, e a varredura de elisões do app NÃO PODIA
//! vê-lo.**
//!
//! Medido em 2026-09-19. A largura do chip era derivada (a secção já invertia a geometria da
//! pílula à mão) e o PINTOR gastava, por cima disso, o respiro de uma caixa de rótulo — que numa
//! pílula **já foi pago pelo `pad_x` dela**:
//!
//! | tag | mede | orçamento antigo | saía |
//! |---|---:|---:|---|
//! | `Ground` | `38,95` | `22,95` | `Grou…` |
//! | `Enemy` | `35,64` | `19,64` | `Ene…` |
//! | `Player` | `32,74` | `16,74` | `Pla…` |
//! | `Flying` | `31,86` | `15,93` | `Fly…` |
//!
//! # ⚠️⚠️ Porque a varredura do app é CEGA a isto
//!
//! Ela pinta cada painel do registo com o estado de FÁBRICA, e **um Inspector de fábrica não tem
//! objecto seleccionado** — logo não pinta um único chip. *Um censo que varre painéis vazios mede
//! o painel vazio*; quem mede um chip é quem entrega um objecto COM tags.
//!
//! # ⭐ E a lei mudou de dono
//!
//! A secção tinha uma cópia da geometria da pílula (`texto + 2,5·pad + ×`) e o pintor tinha a
//! outra. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*: hoje quem
//! dimensiona pergunta [`Tag::natural_width`] e quem pinta gasta [`Tag::label_budget`], e há gate
//! de ida-e-volta sobre o par em `ph2d-editor-core`.

use ph2d_editor_core::screens::hero::{InspectorTagRow, InspectorTagsInfo};
use ph2d_editor_core::text_elide::elisao::Medido;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_tags};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// ⚠️ Nomes de tag REAIS de um jogo, e o mais largo deles (`Ground`) é o que a medição acusou.
const TAGS: &[&str] = &["Enemy", "Flying", "Ground", "Player"];

fn linha(id: u64, path: &str, label: &str, depth: usize) -> InspectorTagRow {
    InspectorTagRow {
        id,
        path: path.into(),
        label: label.into(),
        depth,
    }
}

/// Pinta o Inspector com um objecto que tem as quatro tags e devolve o censo de elisões.
fn chips_pintados() -> Vec<Medido> {
    let on_object: Vec<InspectorTagRow> = TAGS
        .iter()
        .enumerate()
        .map(|(i, t)| linha(i as u64 + 1, t, t, 0))
        .collect();
    set_current_inspector_tags(Some(InspectorTagsInfo {
        entity_bits: 0xBEEF_0019,
        on_object: on_object.clone(),
        full: false,
        selected_count: 1,
    }));
    ph2d_panel_inspector::set_current_tag_tree(on_object);
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.medindo_a_pintura::<InspectorPanel>(&mut state, VIEWPORT)
}

/// ⭐⭐⭐ **O nome de uma tag chega INTEIRO ao chip dela.**
#[test]
fn o_nome_de_uma_tag_chega_inteiro_ao_chip() {
    let medidos = chips_pintados();
    // ⛔ **PISO DE POPULAÇÃO.** Um `for` sobre uma lista vazia passa trivialmente.
    assert!(
        TAGS.len() >= 4 && medidos.len() >= TAGS.len(),
        "a fixtura tem {} tag(s) e o censo leu {} rótulo(s) — com menos, este gate mede o vazio",
        TAGS.len(),
        medidos.len()
    );
    for tag in TAGS {
        let Some(m) = medidos.iter().find(|m| m.texto == *tag) else {
            panic!(
                "⛔ a tag {tag:?} não foi pintada — sem ela este gate mede o vazio. Vistos: {:?}",
                medidos.iter().map(|m| &m.texto).collect::<Vec<_>>()
            );
        };
        assert!(
            m.coube(),
            "⛔ a tag {tag:?} saiu {:?} num orçamento de {:.2} px — a pílula deixou de descrever \
             a palavra que ela carrega",
            m.pintado,
            m.largura
        );
    }
}

/// ⛔ **E o orçamento é EXACTAMENTE o que a palavra pede — a pílula não tem folga.**
///
/// Sem esta metade, dar a cada chip uma largura generosa passaria o gate acima e a nuvem de tags
/// caberia menos chips por linha sem ninguém ter pedido. *A pílula mede a palavra, não um número
/// confortável.*
#[test]
fn a_pilula_de_uma_tag_e_do_tamanho_da_palavra() {
    let medidos = chips_pintados();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for tag in TAGS {
        let m = medidos
            .iter()
            .find(|m| m.texto == *tag)
            .expect("a tag tem de ser pintada");
        let texto = ts.prefix_width_weighted(tag, m.fonte, m.peso);
        assert!(
            m.largura - texto < 1.0,
            "a tag {tag:?} mede {texto:.2} px e recebeu um orçamento de {:.2} — a pílula ganhou \
             folga, e uma folga aqui rouba lugar aos irmãos na mesma fileira",
            m.largura
        );
    }
}
