//! Os gates da tabela de matcaps — ver [`super`].

use super::{Credit, Encoding, MATCAP_NAMES, MATCAPS, decode};

/// **Todo matcap decodifica, e sai no tamanho que a textura tem.**
///
/// ⚠️ É o gate que torna o `include_bytes!` uma promessa CONFERIDA em vez de uma
/// declarada: o compilador garante que o arquivo existe, e só isto garante que
/// ele é um PNG de 512² que o nosso decoder lê. Um asset trocado por engano —
/// um JPEG com extensão errada, um 256² — passa pelo compilador e morre aqui.
#[test]
fn every_matcap_decodes_to_the_texture_size() {
    for (i, m) in MATCAPS.iter().enumerate() {
        // RGBA de meio-float = 8 bytes por texel.
        let want = (m.side as usize) * (m.side as usize) * 8;
        let px = decode(i);
        assert_eq!(
            px.len(),
            want,
            "o matcap `{}` decodificou {} bytes",
            m.name,
            px.len()
        );
    }
}

/// **Os nove são NOVE imagens diferentes** — `decode` é função do `id`.
///
/// ⚠️ Nasceu de uma mutação: fazer o `decode` devolver sempre a primeira linha
/// deixava sete dos oito gates VERDES, e o único que sangrava o fazia por
/// acidente (ele pede o `Basic Side` pelo nome e recebia outro). Sem esta
/// afirmação, *"o artista escolhe o Clay Brown e vê o Studio"* — a fileira
/// inteira de chips fazendo a mesma coisa — passaria pela suíte.
#[test]
fn the_nine_are_nine_different_images() {
    let all: Vec<Vec<u8>> = (0..MATCAPS.len()).map(decode).collect();
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(
                all[i], all[j],
                "`{}` e `{}` decodificam para os MESMOS pixels",
                MATCAPS[i].name, MATCAPS[j].name
            );
        }
    }
}

/// **Um índice fora da tabela é PRESO no último, e não panica.**
///
/// A mesma política do [`crate::ShadeRaw::pack`], e pela mesma razão: o número
/// vem de uma escolha de UI que atravessou um `u8`, não de um asset corrompido.
#[test]
fn an_index_past_the_end_is_clamped_not_a_panic() {
    let last = decode(MATCAPS.len() - 1);
    for id in [MATCAPS.len(), MATCAPS.len() + 7, usize::MAX] {
        assert_eq!(decode(id), last, "o índice {id} tinha de cair no último");
    }
}

/// **Os nomes são DERIVADOS da tabela, na ordem dela.**
///
/// ⚠️ Um gate que comparasse `MATCAP_NAMES` com uma lista escrita à mão aqui
/// seria a terceira cópia do que a wave existe para colapsar. O que ele afirma é
/// a RELAÇÃO: cada nome é o nome da linha de mesmo índice, e nenhum é vazio ou
/// repetido — as duas formas de uma fileira de chips mentir sem que a contagem
/// acuse.
#[test]
fn the_names_are_the_table_read_in_order() {
    assert_eq!(MATCAP_NAMES.len(), MATCAPS.len());
    for (i, m) in MATCAPS.iter().enumerate() {
        assert_eq!(MATCAP_NAMES[i], m.name);
        assert!(!m.name.trim().is_empty(), "o matcap {i} não tem nome");
        assert!(
            !MATCAP_NAMES[..i].contains(&m.name),
            "o matcap {i} repete o nome `{}`",
            m.name
        );
    }
}

/// **O índice 0 é o do SculptGL, porque ele É o default do app.**
///
/// ⚠️ O [`crate::DEFAULT_MATCAP`] aponta para `0`, então *qual chip nasce
/// marcado* e *qual é a primeira linha da tabela* são o MESMO fato. Este gate é
/// o que impede alguém de reordenar a lista por gosto e mudar o default do app
/// sem perceber — o que na tela é o barro abrindo com outra luz.
#[test]
fn the_default_is_the_sculptgl_matcap_and_it_leads_the_table() {
    assert_eq!(crate::DEFAULT_MATCAP, Some(0));
    assert_eq!(MATCAPS[0].credit, Credit::HazardousArts);
    assert_eq!(MATCAPS[0].name, "Skin Haz 2");
    assert_eq!(
        crate::Shade::default().matcap,
        crate::DEFAULT_MATCAP,
        "o `Shade::default` tem de ARMAR o default, não repetir um número"
    );
}

/// **A procedência de cada linha está declarada, e as duas licenças batem com o
/// arquivo que as documenta.**
///
/// ⚠️ Um matcap sem `credit` seria redistribuir um asset sem saber sob que
/// licença. A contagem por fonte é afirmada porque é ela que o
/// `assets/matcaps/LICENSES.md` narra: **um** do SculptGL (MIT) e **oito** do
/// Blender (CC0).
#[test]
fn every_matcap_declares_where_it_came_from() {
    let haz = MATCAPS
        .iter()
        .filter(|m| m.credit == Credit::HazardousArts)
        .count();
    let blender = MATCAPS
        .iter()
        .filter(|m| m.credit == Credit::Blender)
        .count();
    assert_eq!(haz, 2, "o LICENSES.md declara DOIS do HazardousArts");
    assert_eq!(blender, 8, "o LICENSES.md declara OITO do Blender");
    assert_eq!(haz + blender, MATCAPS.len());

    // ⚠️ **A precisão segue a FONTE, e este é o gate que o afirma.** Um do
    // Blender guardado como PNG seria a quantização de ~1 nível de 255 que esta
    // wave mediu e removeu; um do SculptGL guardado como EXR seria um arquivo
    // maior dizendo exatamente a mesma coisa que o JPEG já dizia.
    for m in &MATCAPS {
        let want = match m.credit {
            Credit::Blender => Encoding::ExrHalfLinear,
            Credit::HazardousArts => Encoding::PngSrgb8,
        };
        assert_eq!(m.encoding, want, "o matcap `{}` mudou de precisão", m.name);
    }
}

/// **A IMAGEM cozida está com o topo para cima** — a metade da lei de espaço que
/// mora no ASSET.
///
/// ⚠️ **Este gate NÃO pega um flip no shader, e a primeira versão deste
/// doc-comment afirmava que sim.** A frase era *"é o gate que substitui um
/// render"*, e uma mutação a derrubou na hora: invertendo o `v` do `matcap_uv`
/// os oito testes desta crate ficam **VERDES**, porque aqui só se leem os bytes
/// do PNG decodificado — e o topo de um PNG é claro quer o shader o leia de
/// cabeça para baixo, quer não. *Um gate sobre o ASSET é cego ao CONSUMIDOR.*
///
/// O que ele de fato defende continua valendo a pena: um re-cozimento que saia
/// invertido (uma linha `flipud` no script, uma fonte trocada) morre aqui, sem
/// precisar de adapter. Quem defende a **lei de uv** é o irmão de GPU
/// `the_matcap_lights_the_sculpture_from_the_top_of_its_image`, que renderiza e
/// lê de volta — e essa mutação sangra lá, com topo 48 contra base 140.
///
/// O oráculo é o `Basic Side`, escolhido porque a fonte dele é a mais
/// desequilibrada das nove: ele é lit de cima e o fundo é preto, então
/// *"o topo é mais claro que a base"* é uma afirmação com fosso, e não uma
/// diferença de um nível.
#[test]
fn the_cooked_image_has_its_lit_side_up() {
    let id = MATCAPS
        .iter()
        .position(|m| m.name == "Basic Side")
        .expect("o `Basic Side` é o oráculo desta lei");
    let px = decode(id);
    let side = MATCAPS[id].side as usize;
    // A luminância de uma faixa a meio raio ACIMA e ABAIXO do centro, na
    // coluna central — os dois pontos que um flip em `v` troca de lugar.
    // ⚠️ Meio-float agora: 8 bytes por texel, e cada canal são dois.
    let lum = |x: usize, y: usize| -> f32 {
        let i = (y * side + x) * 8;
        let ch = |k: usize| half::f16::from_le_bytes([px[i + k * 2], px[i + k * 2 + 1]]).to_f32();
        ch(0) + ch(1) + ch(2)
    };
    let cx = side / 2;
    let top = lum(cx, side / 4);
    let bottom = lum(cx, side * 3 / 4);
    assert!(
        top > bottom * 2.0,
        "o topo ({top:.4}) tinha de ser MUITO mais claro que a base ({bottom:.4}) — \
         se estão trocados, a imagem (ou o `v` do shader) está de cabeça para baixo"
    );
}
