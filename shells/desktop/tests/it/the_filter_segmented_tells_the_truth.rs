//! **O SEGMENTADO DE TEXTURE FILTER TEM DE DIZER O QUE O ECRÃ DESENHA — E OFERECER EXACTAMENTE O
//! QUE O MOTOR SABE DISTINGUIR.**
//!
//! # O defeito que este gate existe para impedir
//!
//! Auditoria de 2026-08-21 ([`docs/Sprite_projeto/20`](../../../docs/Sprite_projeto/20_auditoria_do_inspector_2026-08-21.md) §2.2):
//! o painel pintava três abas e escolhia a acesa com `.selected((filter_tag as usize).min(2))`.
//! Como o renderer manda as tags **`1 | 3 | 5`** para `Nearest`, o `.min(2)` acendia **«Linear»**
//! para as tags 3 e 5 — ou seja, **o painel afirmava o oposto do que a sprite desenhava**, e só
//! naquelas sprites cuja aparência (pixel duro) é a razão de o controlo existir.
//!
//! Duas coisas o tornavam invisível:
//!
//! 1. **A lei vivia dentro do `sampler_from_tags`, que precisa de um `wgpu::Device`** — medi-la
//!    exigia GPU, por isso ninguém a media. Hoje é a `const fn`
//!    [`filter_tag_magnifies_by_point`], e este gate corre sem adapter nenhum.
//! 2. **Nenhuma crate via os dois lados.** O painel é chrome e não depende do `ph2d-ecs` (as
//!    etiquetas são hardcoded de propósito, como o `BLEND_LABELS` da §10); o `ph2d-render` não
//!    conhece o painel. O shell vê os três — é aqui que a conta fecha, o mesmo sítio e a mesma
//!    razão do `the_emissive_ceiling_is_one_law`.
//!
//! # E o teto (CLAUDE.md §0.0)
//!
//! ⚠️ O motor entrega mipmap trilinear real e anisotropia 16× desde 2026-06-18, e o painel
//! oferecia **três** modos. As outras eram inexprimíveis por gesto nenhum, embora o componente
//! `TextureFilter` esteja no registry de cena (uma tag ≥3 escrita por script sobrevivia ao
//! save/load e o painel não a sabia mostrar). *O teto era o do painel, não o do hardware.*
//!
//! # ⛔⛔ E a correcção do outro lado: um item que NÃO PODE EXISTIR
//!
//! A tag `5` — *«Near+Aniso»* — pedia duas coisas contraditórias. O wgpu (e o Metal por baixo)
//! exige `mag`+`min`+`mipmap` os três `Linear` para `anisotropy_clamp > 1`, e *ampliar por ponto*
//! é precisamente o que aquele nome promete ao artista. O sampler que ela produzia era **campo a
//! campo idêntico** ao da `3 Near+Mip`: dois nomes diferentes, o mesmo desenho.
//!
//! ⚠️ **Por isso a lista deste gate não é escrita à mão.** Ela é DERIVADA das três leis puras do
//! renderer: *uma tag é oferecida se, e só se, nenhuma tag mais baixa produzir o mesmo descritor*.
//! Se um dia o wgpu passar a aceitar anisotropia com ampliação por ponto, a `5` volta a ser
//! distinta, este gate fica **VERMELHO** e diz que a opção pode voltar ao selector. *Uma recusa
//! medida com endereço, e não uma lista que envelhece em silêncio.*

use ph2d_ecs::FilterMode;
use ph2d_panel_inspector::FILTER_LABELS;
use ph2d_render::image_filter::{
    FILTER_TAG_MAX, filter_tag_anisotropy, filter_tag_blends_mips, filter_tag_magnifies_by_point,
};

/// **O descritor que uma tag de filtro produz**, reduzido às três leis que ela de facto decide.
///
/// ⚠️ O endereçamento vem do `repeat_tag` e o resto é `Default`, então estas três **são** o
/// `wgpu::SamplerDescriptor` — comparar isto é comparar o sampler, sem adapter nenhum (um
/// `wgpu::Sampler` a sério faria deste gate um `#[ignore]` que o CI nunca corre).
fn sampler_law(tag: u8) -> (bool, bool, u16) {
    (
        filter_tag_magnifies_by_point(tag),
        filter_tag_blends_mips(tag),
        filter_tag_anisotropy(tag),
    )
}

/// **As tags que o motor sabe DISTINGUIR**, derivadas do renderer — nunca uma lista à mão.
///
/// A tag `0 Inherit` entra sempre e sem consultar a lei: ela não é um sampler, é uma **delegação**
/// (o descritor dela é o do fallback linear, que a `2 Linear` também produz — dedup por descritor
/// apagaria uma das duas e o artista perderia a única forma de dizer *«herda»*).
fn distinguishable_tags() -> Vec<u8> {
    let mut out = vec![0u8];
    for tag in 1..=FILTER_TAG_MAX {
        if (1..tag).all(|lower| sampler_law(lower) != sampler_law(tag)) {
            out.push(tag);
        }
    }
    out
}

/// **(1) O painel oferece um segmento por modo DISTINGUÍVEL — nem menos, nem mais.**
///
/// ⚠️ Falsificável nos dois sentidos: um modo novo no motor sem rótulo aqui deixa-o inalcançável
/// por gesto nenhum (o teto do painel abaixo do teto do hardware, CLAUDE.md §0.0), e um rótulo
/// sobre uma tag indistinguível põe no menu um item que desenha o mesmo que o vizinho.
#[test]
fn the_filter_segmented_offers_exactly_the_modes_that_render_differently() {
    let esperado = distinguishable_tags();
    let oferecidos: Vec<u8> = FILTER_LABELS
        .iter()
        .enumerate()
        .filter(|(_, l)| l.is_some())
        .map(|(i, _)| u8::try_from(i).expect("as tags cabem num u8"))
        .collect();
    assert_eq!(
        oferecidos, esperado,
        "o Inspector oferece as tags {oferecidos:?} e o motor distingue {esperado:?}.\n\
         · Sobra uma? Ela desenha EXACTAMENTE o que outra ja' desenha — dois nomes, um so' \
         resultado (foi o caso da 5 «Near+Aniso», que o wgpu nao pode entregar).\n\
         · Falta uma? Ela e' INALCANCAVEL por gesto nenhum, embora o componente `TextureFilter` \
         viaje no save. ⚠️ CLAUDE.md §0.0: o teto e' o do HARDWARE, nunca o do painel.\n\
         Acrescente/retire o rotulo em `sections/sampling.rs::FILTER_LABELS` NA POSICAO DA TAG \
         (o buraco fica `None`: a posicao e' o contrato com `ids::INSP_SAMPLE_FILTER`)."
    );
}

/// **(2) A posição é a tag** — o despacho conta com isso, e sem esta conta o segmento aceso e a
/// escrita divergem em silêncio.
///
/// ⚠️ **Aqui é a posição no ARRAY, não no que se pinta.** O `event_ordering.rs` deriva o que
/// escrever de `INSP_SAMPLE_FILTER.position(|&o| o == id)`, e aquele array tem as sete entradas —
/// é por isso que o rótulo aposentado é um `None` no meio da lista e não uma lista mais curta:
/// encurtá-la faria o rótulo `n+1` casar com o id `n`.
#[test]
fn the_position_in_the_segmented_is_the_tag_itself() {
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        let Some(label) = label else { continue };
        let tag = u8::try_from(i).expect("as tags cabem num u8");
        assert_eq!(
            FilterMode::from_tag(tag).tag() as usize,
            i,
            "o rotulo «{label}» esta' na posicao {i}, mas a tag {tag} le' de volta como {:?} \
             (tag {}) — o despacho (`event_ordering.rs`) deriva a tag da POSICAO do id em \
             `ids::INSP_SAMPLE_FILTER`, entao esta divergencia faz o clique escrever outro modo",
            FilterMode::from_tag(tag),
            FilterMode::from_tag(tag).tag()
        );
    }
}

/// **(3) O rótulo não mente sobre o que o ecrã desenha** — o defeito exato que shipou.
///
/// ⚠️ A regra é de **ampliação**: um rótulo que começa por `Near` promete pixel duro, e só pode
/// estar sobre uma tag que o renderer amplia por ponto. `Inherit` fica de fora porque não promete
/// nada — ele delega.
#[test]
fn the_filter_segmented_tells_the_truth_about_what_renders() {
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        let Some(label) = label else { continue };
        if *label == "Inherit" {
            continue;
        }
        let tag = u8::try_from(i).expect("as tags cabem num u8");
        let label_promises_point = label.starts_with("Near");
        assert_eq!(
            label_promises_point,
            filter_tag_magnifies_by_point(tag),
            "o rotulo «{label}» (tag {tag}) promete {} mas o renderer amplia por {}.\n\
             Foi exatamente esta divergencia que shipou: `.min(2)` acendia «Linear» sobre as tags \
             3 e 5, que desenham com pixel duro.\n\
             ⚠️ A lei do renderer e' `ph2d_render::image_filter::filter_tag_magnifies_by_point`; \
             o rotulo tem de a seguir, nunca o contrario.",
            if label_promises_point {
                "pixel duro"
            } else {
                "interpolacao"
            },
            if filter_tag_magnifies_by_point(tag) {
                "ponto"
            } else {
                "interpolacao"
            }
        );
    }
}

/// **(4) Nenhum rótulo se repete.** Dois segmentos com o mesmo nome são indistinguíveis para quem
/// clica — e foi assim que as tags 3 e 5 se disfarçaram de «Linear» durante meses.
#[test]
fn no_two_filter_segments_carry_the_same_label() {
    let mut seen: Vec<&str> = Vec::new();
    for label in FILTER_LABELS.iter().flatten() {
        assert!(
            !seen.contains(label),
            "o rotulo «{label}» aparece duas vezes no segmentado de Texture Filter — o artista \
             nao consegue distinguir os dois modos, e o que ele escolhe passa a ser sorte"
        );
        seen.push(label);
    }
}

/// ⛔⛔ **(5) O QUE SAIU DO MENU CONTINUA A LER DO DISCO.**
///
/// Retirar um item de menu é uma operação sobre o **painel**; o `.ph2dproj` gravado ontem não sabe
/// disso. Este gate é a costura das duas metades no único sítio que vê as duas: toda tag que o
/// painel **não** oferece tem de chegar do arquivo a um modo que o painel **oferece** — senão o
/// Inspector abre um projecto velho com nenhum segmento aceso e o artista não tem como voltar.
///
/// ⚠️ A tag `5` é o caso vivo: ela lê como `Near+Mip`, que é o sampler que a máquina sempre lhe
/// deu. *Um modo que sai do selector não pode levar consigo os ficheiros que o usaram.*
#[test]
fn a_tag_that_left_the_menu_still_reads_back_into_one_that_is_offered() {
    for tag in 0..=FILTER_TAG_MAX {
        let offered = FILTER_LABELS
            .get(usize::from(tag))
            .copied()
            .flatten()
            .is_some();
        if offered {
            continue;
        }
        let lido = FilterMode::from_tag(tag);
        assert_eq!(
            sampler_law(lido.tag()),
            sampler_law(tag),
            "a tag {tag} saiu do menu e le' de volta como {lido:?} (tag {}), que desenha OUTRA \
             coisa — abrir um projecto gravado com ela muda o que o artista ve'",
            lido.tag()
        );
        assert!(
            FILTER_LABELS
                .get(usize::from(lido.tag()))
                .copied()
                .flatten()
                .is_some(),
            "a tag {tag} le' de volta como {lido:?}, que TAMBEM nao esta' no menu — o projecto \
             abre sem segmento aceso e o modo fica sem gesto que o troque"
        );
    }
    // ⚠️ **A metade JUSTA:** existe de facto uma tag fora do menu. Sem ela este gate ficaria verde
    // por vacuidade no dia em que alguém repusesse todos os rótulos.
    assert!(
        FILTER_LABELS.iter().any(Option::is_none),
        "nenhuma tag esta' fora do menu — se o `Near+Aniso` voltou, a nota de \
         `filter_tag_anisotropy` caducou e este gate tem de ser relido, nao apagado"
    );
}
