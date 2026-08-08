//! Ver o `sculpt3d_preview.rs` — este arquivo é o `mod tests` dele.
//!
//! ⚠️ Eles não perguntam *que valor cada vértice tem* (isso é do kernel, e tem
//! gates lá): perguntam **em que frame o preview é recalculado**, que é a única
//! coisa que este módulo decide.

use super::*;
use ph2d_sculpt3d::{Alpha, Verb};

fn mesh() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(24, 32, 1.0)
}

fn textured() -> Brush {
    Brush {
        alpha: Some(Alpha::Strata),
        alpha_scale: 0.2,
        ..Brush::default()
    }
}

/// **DESARMADO NÃO DESENHA, e a limpeza SOBE.**
///
/// ⚠️ A segunda metade é a que importa: um preview que ficasse no vetor
/// continuaria no device, o artista desmarcaria a caixa e o barro seguiria
/// tingido — que se lê como *"o botão não faz nada"*, não como um bug.
#[test]
fn disarming_clears_the_channel_and_the_clear_is_uploaded() {
    let (m, b) = (mesh(), textured());
    let mut st = PreviewState::default();
    st.refresh(&m, &b, true, &[]);
    assert_eq!(st.values.len(), m.vert_count());

    st.whole_dirty = false;
    st.refresh(&m, &b, false, &[]);
    assert!(st.values.is_empty(), "o preview sobreviveu ao desarme");
    assert!(st.whole_dirty, "a limpeza não foi marcada para subir");

    // E desarmado de novo não re-marca: nada mudou, nada sobe.
    st.whole_dirty = false;
    st.refresh(&m, &b, false, &[]);
    assert!(!st.whole_dirty, "um no-op pediu upload");
}

/// **ESCOLHER UM PADRÃO JÁ O MOSTRA NO BARRO — sem procurar interruptor.**
///
/// ⚠️ **Este gate existe porque uma mutação sobreviveu:** trocar o default para
/// `false` não derrubou nenhum dos outros, e a razão é que TODOS declaram o
/// `alpha_preview` que querem. *Um default só é testado por um teste que não o
/// menciona* — então aqui o valor vem do estado de fábrica, e o oráculo é a
/// CONSEQUÊNCIA (há o que desenhar), não o literal.
///
/// ⚠️ **O que ele NÃO prova, dito em vez de insinuado:** o `Sculpt3dScene`
/// carrega o próprio literal (o construtor dele exige um `wgpu::Device`, então
/// nenhum gate de CPU o alcança), e são dois literais em dois crates. Se
/// divergirem, o primeiro snapshot do frame reescreve o do painel — o artista
/// veria o barro mudar sozinho uma vez e nunca mais. É a mesma exposição que
/// `cavity`/`ao`/`ssao` já carregam, e a convenção daquele construtor.
#[test]
fn picking_a_pattern_already_shows_it_on_the_clay() {
    let armed = ph2d_panel_sculpt3d::Sculpt3dUi::default().alpha_preview;
    let mut st = PreviewState::default();
    st.refresh(&mesh(), &textured(), armed, &[]);
    assert!(
        !st.values.is_empty(),
        "o padrão foi escolhido e o barro não mostra nada: o preview nasce \
         desligado, e uma feature que se tem de procurar não existe para a maioria"
    );
}

/// **UM PINCEL LISO É O MESMO QUE DESARMADO** — o preview de padrão nenhum não
/// é um vetor de zeros, é a ausência.
#[test]
fn a_plain_brush_draws_nothing() {
    let (m, b) = (mesh(), Brush::default());
    assert!(b.alpha.is_none());
    let mut st = PreviewState::default();
    st.refresh(&m, &b, true, &[]);
    assert!(st.values.is_empty());
}

/// **CADA ENTRADA DA CHAVE PEDE O CAMPO INTEIRO, uma por uma.**
///
/// ⚠️ A varredura muda **um campo de cada vez** de propósito: uma que mudasse
/// dois não distinguiria *"a chave tem este"* de *"a chave tem o outro"*, e o
/// modo de falha de uma chave incompleta não é um erro — é um preview VELHO que
/// ninguém vê que é velho.
#[test]
fn every_input_of_the_key_asks_for_the_whole_field() {
    let m = mesh();
    let base = textured();

    let mut alpha = base;
    alpha.alpha = Some(Alpha::Scales);
    let mut scale = base;
    scale.alpha_scale = 0.05;
    let mut az = base;
    az.alpha_az_deg = 30;
    let mut elev = base;
    elev.alpha_elev_deg = 45;
    // O verbo de máscara não é freado pela máscara — o campo muda com ele.
    let mut verb = base;
    verb.verb = Verb::Mask;

    for (name, b) in [
        ("o padrão", alpha),
        ("a escala", scale),
        ("o azimute", az),
        ("a elevação", elev),
        ("o freio da máscara", verb),
    ] {
        let mut st = PreviewState::default();
        st.refresh(&m, &base, true, &[]);
        st.whole_dirty = false;
        st.refresh(&m, &b, true, &[]);
        assert!(
            st.whole_dirty,
            "mudar {name} não pediu o campo inteiro — a chave não o carrega"
        );
    }

    // E o CONTROLE: a mesma entrada não pede nada.
    let mut st = PreviewState::default();
    st.refresh(&m, &base, true, &[]);
    st.whole_dirty = false;
    st.refresh(&m, &base, true, &[]);
    assert!(!st.whole_dirty, "um estado parado pediu o campo inteiro");
}

/// **UMA TOPOLOGIA NOVA PEDE O CAMPO INTEIRO** — mesmo com o pincel intocado.
///
/// Um preview do comprimento antigo não é *um pouco* velho: ele descreve
/// vértices que não existem mais.
#[test]
fn a_new_topology_asks_for_the_whole_field() {
    let b = textured();
    let mut st = PreviewState::default();
    st.refresh(&mesh(), &b, true, &[]);
    st.whole_dirty = false;

    let denser = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    st.refresh(&denser, &b, true, &[]);
    assert!(st.whole_dirty, "a malha nova não pediu o campo inteiro");
    assert_eq!(st.values.len(), denser.vert_count());
}

/// **UM DAB ANDA PELA JANELA, NÃO PELO CAMPO INTEIRO** — a lei do módulo, e o
/// que a torna barata.
///
/// ⚠️ O oráculo tem DUAS metades e nenhuma sozinha basta: *o vértice movido foi
/// reescrito* (senão a janela é decorativa) e *o campo inteiro NÃO foi pedido*
/// (senão o custo do gesto voltou a ser função do documento).
#[test]
fn a_dab_walks_the_window_not_the_whole_field() {
    let mut m = mesh();
    let b = textured();
    let mut st = PreviewState::default();
    st.refresh(&m, &b, true, &[]);
    st.whole_dirty = false;

    // Move UM vértice para longe: o padrão é lido na posição, então o valor
    // dele tem de mudar.
    let before = st.values[0];
    let p = m.positions()[0];
    m.positions_mut()[0] = [p[0] + 3.7, p[1] - 2.9, p[2] + 1.3];

    st.refresh(&m, &b, true, &[0]);
    assert!(
        !st.whole_dirty,
        "um dab pediu o campo inteiro — o custo virou função do documento"
    );
    assert!(
        (st.values[0] - before).abs() > 1e-6,
        "o vértice movido não foi reescrito: a janela é decorativa"
    );
}
