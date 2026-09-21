//! Os gates da tabela de matcaps — ver [`super`].

use super::{Credit, Encoding, MATCAP_NAME_KEYS, MATCAPS, decode};

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
            m.name_key,
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
                MATCAPS[i].name_key, MATCAPS[j].name_key
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
/// ⚠️ Um gate que comparasse `MATCAP_NAME_KEYS` com uma lista escrita à mão aqui
/// seria a terceira cópia do que a wave existe para colapsar. O que ele afirma é
/// a RELAÇÃO: cada nome é o nome da linha de mesmo índice, e nenhum é vazio ou
/// repetido — as duas formas de uma fileira de chips mentir sem que a contagem
/// acuse.
#[test]
fn the_names_are_the_table_read_in_order() {
    assert_eq!(MATCAP_NAME_KEYS.len(), MATCAPS.len());
    for (i, m) in MATCAPS.iter().enumerate() {
        assert_eq!(MATCAP_NAME_KEYS[i], m.name_key);
        assert!(!m.name_key.trim().is_empty(), "o matcap {i} não tem nome");
        assert!(
            !MATCAP_NAME_KEYS[..i].contains(&m.name_key),
            "o matcap {i} repete o nome `{}`",
            m.name_key
        );
    }
}

/// **O índice 0 é o do SculptGL, e ele LIDERA a tabela.**
///
/// ⚠️⚠️ **Este gate chamava-se `the_default_is_the_sculptgl_matcap_and_it_leads_the_table` e
/// afirmava MAIS uma coisa: que o app abria nele.** Essa metade MORREU em 2026-09-21, quando o
/// [`crate::DEFAULT_LIGHTING`] passou a ser a lei que assa — o dono reportou **duas vezes** que
/// *«o bake não é idêntico ao que se vê em 3d»*, e a causa era o visor abrir noutra lei que não a do
/// bake (medido: `0,347` de desvio por canal contra `0,000215`). *Um gate cuja premissa morre é o
/// gate a funcionar; apagá-lo em silêncio seria perder a razão.*
///
/// ⭐ **O que fica é a ordem do dono de 2026-08-09** — *«SculptGL: só tem um tipo; busque e coloque
/// como o padrão do app»* — na metade dela que continua a ser lei: de todos os matcaps, **o do
/// SculptGL é o primeiro**, logo é o primeiro chip da fileira e o que um artista alcança primeiro.
/// Isto impede alguém de reordenar a lista por gosto e mudar essa escolha sem perceber.
///
/// ⛔ **Quem afirma que o app abre na lei que assa é outro gate, e ele mede o BARRO em vez de
/// comparar constantes:** `ph2d_app_sculpt3d` ::
/// `o_que_o_app_mostra_de_fabrica_e_a_lei_que_assa`.
#[test]
fn o_matcap_do_sculptgl_lidera_a_tabela() {
    assert_eq!(MATCAPS[0].credit, Credit::HazardousArts);
    assert_eq!(MATCAPS[0].name_key, "sculpt3d.matcap.skin_haz_2");
}

/// **O [`crate::Shade::default`] ARMA o default, e não repete um número.**
///
/// ⚠️ Ele era a 4.ª asserção do gate acima, e é **independente de QUAL** default está escrito: por
/// isso sobrevive à troca e por isso mora sozinho. Sem ele, o dia em que o
/// [`crate::DEFAULT_LIGHTING`] mudasse, o `Shade::default` continuaria a entregar o valor antigo —
/// e metade do app abriria numa luz e a outra metade noutra.
#[test]
fn o_shade_default_arma_o_default_em_vez_de_repetir_um_numero() {
    assert_eq!(crate::Shade::default().lighting, crate::DEFAULT_LIGHTING);
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
        assert_eq!(
            m.encoding, want,
            "o matcap `{}` mudou de precisão",
            m.name_key
        );
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
        // ⚠️ Pela CHAVE, que é o que a tabela guarda desde 2026-09-17. Escrito como o nome
        //    inglês, o `position` devolve `None` e o `expect` estoura — o modo de falha ALTO,
        //    que é o bom: um oráculo escolhido por um valor esperado que deixou de existir.
        .position(|m| m.name_key == "sculpt3d.matcap.basic_side")
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
