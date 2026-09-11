//! Gates do RECORTE — a única aritmética deste módulo, e a que não precisa de GPU.

use super::crop;

/// Uma imagem `4×4` em que cada pixel guarda `(x, y)` no vermelho e no verde.
fn fixture() -> (u32, u32, Vec<u8>) {
    let (w, h) = (4u32, 4u32);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            px.extend_from_slice(&[x as u8, y as u8, 0, 255]);
        }
    }
    (w, h, px)
}

/// ⭐ **O recorte devolve a REGIÃO pedida, e nos pixels certos.**
///
/// ⚠️ A régua lê o CONTEÚDO (cada pixel diz onde estava), não só as dimensões: um recorte
/// deslocado dá o tamanho certo e a imagem errada, e um gate de dimensões não o vê.
#[test]
fn a_crop_returns_the_region_it_was_asked_for() {
    let (w, h, px) = fixture();
    // A imagem inteira.
    let (cw, ch, out) = crop(w, h, &px, [0.0, 0.0, 1.0, 1.0]).expect("inteira");
    assert_eq!((cw, ch), (4, 4));
    assert_eq!(out.as_slice(), px.as_slice());
    // O quadrante inferior direito.
    let (cw, ch, out) = crop(w, h, &px, [0.5, 0.5, 1.0, 1.0]).expect("quadrante");
    assert_eq!((cw, ch), (2, 2));
    assert_eq!(
        &out[0..4],
        &[2, 2, 0, 255],
        "o 1.o pixel do quadrante e' (2,2)"
    );
    assert_eq!(&out[4..8], &[3, 2, 0, 255]);
    assert_eq!(&out[8..12], &[2, 3, 0, 255], "a 2.a linha comeca em (2,3)");
    // ⚠️ **Invertido dá o mesmo** — um `uv` com as pontas trocadas é uma região, não um erro.
    let (cw2, ch2, out2) = crop(w, h, &px, [1.0, 1.0, 0.5, 0.5]).expect("invertido");
    assert_eq!((cw2, ch2), (2, 2));
    assert_eq!(out2, out);
}

/// ⛔ **O que não é uma região devolve `None`**, e nunca uma imagem vazia que o desenho tentaria
/// pintar: um `uv` degenerado, uma imagem sem pixels, um buffer curto de mais.
#[test]
fn a_degenerate_region_returns_nothing() {
    let (w, h, px) = fixture();
    assert!(
        crop(w, h, &px, [0.5, 0.5, 0.5, 0.5]).is_none(),
        "largura zero"
    );
    assert!(
        crop(0, 0, &px, [0.0, 0.0, 1.0, 1.0]).is_none(),
        "sem dimensoes"
    );
    assert!(
        crop(w, h, &px[..8], [0.0, 0.0, 1.0, 1.0]).is_none(),
        "um buffer curto tem de ser recusado, nao lido fora"
    );
    // ⚠️ Fora da faixa CORTA-SE, e não estoura: um `uv` de um atlas em crescimento pode passar
    // de `1` por um epsilon.
    assert!(crop(w, h, &px, [-0.5, -0.5, 1.5, 1.5]).is_some());
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// A MEMÓRIA E A INVALIDAÇÃO — auditoria de seis lentes, doc 96 §2.5.
// ─────────────────────────────────────────────────────────────────────────────────────────

use super::LeafImages;

/// ⭐⭐⭐ **O FIM DO QUADRO LARGA O ATLAS E GUARDA OS RECORTES** — as duas metades, e são
/// opostas.
///
/// ⛔⛔ A cópia em CPU do atlas é `ATLAS_DEFAULT_SIZE_PX² × 4 = 8192² × 4 = 268 MB`, e ficava
/// retida **pela vida do processo** (o doc dizia *«lê-se INTEIRO, e uma vez só»* — o que ele não
/// dizia é que «inteiro» é um quarto de gigabyte, seja qual for o tamanho da folha).
///
/// ⚠️ **E os recortes têm de FICAR**: eles são a resposta memoizada por `(textura, região)` e
/// são do tamanho de uma folha. Largá-los junto faria cada quadro voltar a ler o atlas inteiro —
/// *a cura da memória viraria uma paragem de GPU por quadro*, que é pior que o defeito.
#[test]
fn the_end_of_the_frame_drops_the_atlas_and_keeps_the_crops() {
    let mut c = LeafImages::default();
    c.seed_for_tests(7, 8192, 5);
    assert_eq!(c.cached(), (true, 5), "a fixtura tem de partir cheia");
    c.end_frame();
    assert_eq!(
        c.cached(),
        (false, 5),
        "o fim do quadro tinha de largar o ATLAS e guardar os RECORTES"
    );
}

/// ⭐⭐ **PINTAR NA FOLHA INVALIDA O CACHE** — a metade de CORRECÇÃO do mesmo achado.
///
/// ⛔ Nada era invalidado: *pintar na folha servia pixels velhos para sempre*. Um cache de bytes
/// de GPU não se invalida por tempo nem por tamanho — invalida-se por **mudança**, e só o dono da
/// textura sabe quando ela muda ([`ph2d_render::TextureAtlas::epoch`], incrementado no funil por
/// onde toda escrita de conteúdo passa).
///
/// ⚠️ **E o CONTROLE está na mesma função**: um epoch igual não pode limpar nada, senão a cura
/// da correcção apagaria o cache a cada quadro e reintroduziria a leitura de `268 MB`.
#[test]
fn a_changed_atlas_throws_the_stale_pixels_away_and_an_unchanged_one_does_not() {
    let mut c = LeafImages::default();
    c.seed_for_tests(7, 8192, 5);
    // ⚠️ **Pela PORTA (`synced`), nunca pelo `sync_to`** — a mutação que esvaziava o corpo do
    // `synced` SOBREVIVEU enquanto este gate chamava a metade de dentro. *Um gate que entra por
    // baixo da porta prova a lei e não prova a porta.*
    let _ = c.synced(7);
    assert_eq!(
        c.cached(),
        (true, 5),
        "o MESMO epoch limpou o cache — isto le' 268 MB a cada quadro"
    );
    let _ = c.synced(8);
    assert_eq!(
        c.cached(),
        (false, 0),
        "o atlas mudou e os pixels velhos ficaram — pintar na folha nao se veria"
    );
}
