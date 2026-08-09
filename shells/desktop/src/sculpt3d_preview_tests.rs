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

/// Um carimbo de bandas diagonais 32².
///
/// ⚠️ **UMA imagem, compartilhada — e isto é load-bearing.** O `PartialEq` do
/// [`Alpha`] compara duas imagens por **IDENTIDADE** (é o que mantém a chave
/// O(1) em vez de um `memcmp` de megabytes por quadro), então uma fixture que
/// construísse uma imagem por pincel faria as duas chaves diferirem pelo
/// PADRÃO. A mutação *"a chave esqueceu o deslocamento"* sobreviveria — e o
/// gate ficaria verde afirmando outra coisa. Foi o que aconteceu na primeira
/// versão deste arquivo, e quem pegou foi a mutação.
fn stamp() -> Alpha {
    let n = 32u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let v = u8::from((x + y) % 8 < 4) * 255;
            let i = ((y * n + x) * 4) as usize;
            rgba[i..i + 3].fill(v);
            rgba[i + 3] = 255;
        }
    }
    Alpha::Image(std::sync::Arc::new(
        ph2d_sculpt3d::AlphaImage::from_rgba(n, n, &rgba).expect("imagem válida"),
    ))
}

/// Um pincel com o CARIMBO dado armado — a única família em que o deslocamento
/// está vivo.
fn stamped(stamp: &Alpha, offset: [f32; 2]) -> Brush {
    Brush {
        alpha: Some(stamp.clone()),
        alpha_scale: 0.2,
        // O eixo que o produto semeia para uma imagem: encarando a vista.
        alpha_elev_deg: ph2d_sculpt3d::MAX_AXIS_ELEV_DEG,
        alpha_offset: offset,
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

    let mut alpha = base.clone();
    alpha.alpha = Some(Alpha::Scales);
    let mut scale = base.clone();
    scale.alpha_scale = 0.05;
    let mut az = base.clone();
    az.alpha_az_deg = 30;
    let mut elev = base.clone();
    elev.alpha_elev_deg = 45;
    // O verbo de máscara não é freado pela máscara — o campo muda com ele.
    let mut verb = base.clone();
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
    // ⚠️ **O DESLOCAMENTO tem fixture PRÓPRIA, e a assimetria é a razão de ele
    // ter escapado desta varredura.** A lista acima roda sobre um procedural, e
    // para um procedural ele é NEUTRO por construção — uma linha a mais ali teria
    // REPROVADO produto correto. O gate dele é o
    // `placing_the_stamp_asks_for_the_whole_field`, com um carimbo armado.

    // E o CONTROLE: a mesma entrada não pede nada.
    let mut st = PreviewState::default();
    st.refresh(&m, &base, true, &[]);
    st.whole_dirty = false;
    st.refresh(&m, &base, true, &[]);
    assert!(!st.whole_dirty, "um estado parado pediu o campo inteiro");
}

/// **COLOCAR O CARIMBO PEDE O CAMPO INTEIRO — e o barro passa a mostrar outro.**
///
/// ⚠️ **É o gate do report** *"Pattern Offset parece sem efeito"* (Enio,
/// 2026-08-09), e ele mora aqui porque o deslocamento **não move um vértice**:
/// o `moved` de um arrasto de slider está VAZIO, então a chave era a única coisa
/// capaz de pedir o recálculo — e ela guardava `az`/`elev` e mais nada do frame.
/// O barro seguia com o carimbo no lugar de antes, enquanto um dab de verdade já
/// o depositava no lugar novo (medido: um passo do slider muda 10.159 dos 13.682
/// vértices).
///
/// **É o modo de falha que o `whole_dirty` já nomeia** — *"um giro de eixo não
/// move um vértice"* —, e um deslocamento é um giro de eixo por outro nome.
///
/// ⚠️ **O oráculo tem DUAS metades:** *pediu o campo inteiro* (senão nada sobe ao
/// device) **e** *os valores mudaram* (senão ele pediu um recálculo que devolve a
/// mesma coisa, e o pedido seria decorativo).
#[test]
fn placing_the_stamp_asks_for_the_whole_field() {
    let m = mesh();
    // ⚠️ **O MESMO carimbo nos dois** — ver o doc de [`stamp`]: dois `Arc`
    // distintos fariam a chave diferir pelo padrão, e o gate ficaria verde sem
    // nunca perguntar pelo deslocamento.
    let s = stamp();
    let here = stamped(&s, [0.0, 0.0]);
    let there = stamped(&s, [0.31, -0.17]);

    let mut st = PreviewState::default();
    st.refresh(&m, &here, true, &[]);
    let before = st.values.clone();
    st.whole_dirty = false;

    // ⚠️ **`moved` VAZIO é a premissa, não um detalhe da fixture:** é exatamente
    // o que o produto entrega quando o artista arrasta um slider sem esculpir.
    st.refresh(&m, &there, true, &[]);
    assert!(
        st.whole_dirty,
        "colocar o carimbo não pediu o campo inteiro — a chave não carrega o \
         deslocamento, e o barro fica com o padrão de antes"
    );
    assert!(
        st.values
            .iter()
            .zip(&before)
            .any(|(a, b)| (a - b).abs() > 1e-6),
        "o campo foi recalculado e saiu igual: o deslocamento não chega ao valor"
    );
}

/// **UM PADRÃO PROCEDURAL NÃO SE MEXE COM O DESLOCAMENTO** — o controle do gate
/// acima, e ele defende uma decisão, não uma coincidência.
///
/// A neutralidade mora dentro do [`ph2d_sculpt3d::Brush::alpha_frame`]: os nove
/// procedurais são campos infinitos e homogêneos, não têm posição — só fase. Se
/// ela se perdesse, a row que o painel ESCONDE para eles passaria a agir sem como
/// ser desfeita, e o preview pediria o campo inteiro a cada arrasto de um controle
/// que não está na tela.
#[test]
fn a_procedural_pattern_does_not_move_with_the_stamp_offset() {
    let m = mesh();
    let mut moved = textured();
    moved.alpha_offset = [0.31, -0.17];

    let mut st = PreviewState::default();
    st.refresh(&m, &textured(), true, &[]);
    let before = st.values.clone();
    st.whole_dirty = false;

    st.refresh(&m, &moved, true, &[]);
    assert!(
        !st.whole_dirty,
        "o deslocamento vazou para um padrão procedural e pediu o campo inteiro"
    );
    assert_eq!(st.values, before, "o campo de um procedural se moveu");
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
