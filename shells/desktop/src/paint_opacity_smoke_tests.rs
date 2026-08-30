//! Os gates da cena da OPACIDADE (`PH2D_BUILD_SMOKE=79`, plano 36 W6).
//!
//! ⚠️ **Eles medem a cena que o produto EMPILHA**, não uma reconstrução — o `populate` é o mesmo
//! que a `App` chama. *Um gate que refaz a fixtura mede o que ele próprio escreveu*, e foi assim
//! que a régua da `=78` teve de ser deitada fora a meio da wave.

use super::{ALFAS, ART, art_rgba, hero_of, populate};
use ph2d_vec_scene::{Paint, PatternSource, StrokePaint, VecPath, VecScene};

/// A cena montada com uma fonte de imagem qualquer — o `AssetId` não é o sujeito de nenhum destes
/// gates, e pedi-lo a um `AssetDb` real arrastaria a `App` para dentro deles.
fn cena() -> VecScene {
    let mut s = VecScene::new();
    populate(
        &mut s,
        PatternSource::Image(ph2d_asset::AssetId::from_bytes(b"xadrez")),
    );
    s
}

fn estampas(s: &VecScene) -> Vec<f32> {
    s.paths()
        .iter()
        .filter_map(|p| match p.fill.as_ref() {
            Some(Paint::Pattern(f)) => Some(f.alpha),
            _ => None,
        })
        .collect()
}

fn pinceis(s: &VecScene) -> Vec<&VecPath> {
    s.paths()
        .iter()
        .filter(|p| {
            p.stroke
                .as_ref()
                .is_some_and(|k| matches!(k.paint, StrokePaint::Brush(_)))
        })
        .collect()
}

/// ⭐⭐⭐ **A CENA TEM AS DUAS TINTAS, e cada uma em TRÊS opacidades distintas.**
///
/// ⚠️ **O controlo é a coluna OPACA**, e ele é metade do gate: uma cura que escurecesse tudo por
/// sistema daria três valores distintos na mesma ordem e passaria sem esta linha.
#[test]
fn the_scene_shows_both_paints_at_three_distinct_opacities() {
    let s = cena();
    let pat = estampas(&s);
    assert_eq!(
        pat.len(),
        ALFAS.len(),
        "a fileira da estampa nao esta' toda"
    );
    assert!(
        (pat[0] - 1.0).abs() < 1e-6,
        "a coluna de CONTROLO nasceu ja' desvanecida ({}) - com ela translucida a cena nao \
         distingue «obedece a barra» de «escureceu tudo»",
        pat[0]
    );
    for j in 1..pat.len() {
        assert!(
            pat[j] < pat[j - 1] - 0.05,
            "duas colunas da estampa desenham a MESMA opacidade ({} e {})",
            pat[j - 1],
            pat[j]
        );
    }

    let br = pinceis(&s);
    assert_eq!(br.len(), ALFAS.len(), "a fileira do pincel nao esta' toda");
    let alfas: Vec<u8> = br
        .iter()
        .map(|p| p.stroke.as_ref().unwrap().color().a)
        .collect();
    assert_eq!(alfas[0], 255, "a coluna de CONTROLO do pincel nao e' opaca");
    for j in 1..alfas.len() {
        assert!(alfas[j] < alfas[j - 1], "duas colunas do pincel sao iguais");
    }
}

/// ⭐⭐⭐ **AS CÓPIAS QUE O PRODUTO EMITE desvanecem** — a régua sobre a SAÍDA, e não sobre o campo.
///
/// ⚠️ **É a metade que nenhum gate de campo dá.** Afirmar que o `fallback.a` desce é afirmar o que a
/// cena escreveu; o que o artista vê é a **arte**, e entre um e outro está o consumidor que durante
/// toda a W1..W5 **descartava o valor**. Esta régua chama a mesma função que o renderer chama.
#[test]
fn the_copies_the_product_emits_actually_fade() {
    let s = cena();
    let art = s
        .paths()
        .iter()
        .find(|p| {
            matches!(p.fill, Some(Paint::Solid(_))) && p.verts.len() == 4 && p.stroke.is_some()
        })
        .cloned();
    // A arte do pincel é a única forma sólida COM contorno de 4 vértices que não é uma faixa (as
    // faixas nascem sem `stroke`). Se ela sumir, o gate tem de reprovar em vez de medir zero.
    let art = art.expect("a arte do pincel existe na cena");
    let mut visto = Vec::new();
    for p in pinceis(&s) {
        let copias = ph2d_vec_scene::brush_along_path(p, &art, p.stroke.as_ref().unwrap());
        assert!(
            !copias.is_empty(),
            "uma coluna do pincel nao emitiu copia nenhuma"
        );
        let a = match copias[0].fill.as_ref() {
            Some(Paint::Solid(c)) => c.a,
            _ => panic!("a copia perdeu o preenchimento da arte"),
        };
        visto.push(a);
    }
    assert_eq!(
        visto[0], 255,
        "a coluna de controlo emitiu copias JA' desvanecidas"
    );
    for j in 1..visto.len() {
        assert!(
            visto[j] < visto[j - 1],
            "as copias emitidas nao desvanecem ({:?}) - a barra anda e nao muda um pixel",
            visto
        );
    }
}

/// ⭐⭐ **AS FAIXAS CLARAS FICAM POR BAIXO** — sem isto a cena prova o defeito tão bem quanto a cura.
///
/// ⚠️ Sobre um fundo neutro, *mais transparente* e *mais escuro* desenham-se quase igual. A faixa é
/// o que torna a transparência **legível**, e ela só serve se for desenhada ANTES. ⛔ O gate mede a
/// relação (a faixa que cobre o centro de uma forma vem antes dela), e não o índice `0` — um índice
/// literal voltaria a estar errado no dia em que a cena ganhasse mais uma forma.
#[test]
fn the_light_band_is_drawn_under_every_shape_it_makes_legible() {
    let s = cena();
    let caixa = |p: &VecPath| {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in &p.verts {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        (lo, hi)
    };
    let paths = s.paths();
    /// Uma faixa: o índice de empilhamento e a caixa dela.
    type Faixa = (usize, ([f64; 2], [f64; 2]));
    // As faixas: sólidas e SEM contorno (toda forma-sujeito da cena leva um).
    let faixas: Vec<Faixa> = paths
        .iter()
        .enumerate()
        .filter(|(_, p)| p.stroke.is_none() && matches!(p.fill, Some(Paint::Solid(_))))
        .map(|(i, p)| (i, caixa(p)))
        .collect();
    assert_eq!(faixas.len(), 2, "a cena tem de ter UMA faixa por fileira");

    for (i, p) in paths.iter().enumerate() {
        let sujeito = matches!(p.fill, Some(Paint::Pattern(_)))
            || p.stroke
                .as_ref()
                .is_some_and(|k| matches!(k.paint, StrokePaint::Brush(_)));
        if !sujeito {
            continue;
        }
        let (lo, hi) = caixa(p);
        let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        let cobre = faixas.iter().any(|(j, (blo, bhi))| {
            *j < i && c[0] >= blo[0] && c[0] <= bhi[0] && c[1] >= blo[1] && c[1] <= bhi[1]
        });
        assert!(
            cobre,
            "a forma {i} nao tem faixa clara POR BAIXO - sobre o fundo neutro, transparente e \
             escuro desenham-se igual e a cena aprova o defeito"
        );
    }
}

/// ⚠️ **O HERÓI é a primeira ESTAMPA, e derivado.** A cena abre com o painel na secção que a
/// mensagem manda usar; um índice literal poria o painel na secção errada no dia em que alguém
/// acrescentasse uma faixa, **sem erro nenhum**.
#[test]
fn the_hero_is_a_patterned_shape_not_a_band() {
    let s = cena();
    let id = hero_of(&s).expect("a cena tem heroi");
    let p = s.path(id).expect("o heroi esta' na cena");
    assert!(
        matches!(p.fill, Some(Paint::Pattern(_))),
        "o heroi nao e' uma estampa - o painel abre na seccao errada"
    );
    // CONTROLO: numa cena sem estampa nenhuma a resposta é `None`, e não a 1.ª forma que houver.
    let mut vazia = VecScene::new();
    vazia.push_path(VecPath::default());
    assert!(hero_of(&vazia).is_none());
}

/// ⚠️ **A CENA CABE NO QUADRO** — uma cena que nasce meio fora lê-se como *"faltam formas"*.
///
/// ⚠️⚠️ **A barra é uma CERCA derivada de uma irmã shipada, e NÃO uma medição do viewport.** Eu não
/// medi a caixa da câmera de omissão; medi a extensão da [`crate::texture_pattern_smoke`] (`=76`),
/// que o Enio já smokou e cujo conteúdo vive em `y ∈ [−4,1 ; 1,1]` e `x ∈ [−7,6 ; 7,6]`. *Uma cerca
/// que se declara medida sem o ser é pior que nenhuma*, então fica declarado o que ela é: se esta
/// cena ficar dentro daquela caixa, ela é vista; se sair, isto pergunta antes de o Enio abrir.
#[test]
fn the_whole_scene_fits_inside_the_box_a_shipped_sibling_proves_visible() {
    // A extensão da `=76`, contada dos constantes dela: `BOX = 2,2` (⇒ meio `1,1`), fileira de
    // baixo em `y = −3,0`, arte em `y ∈ [−3,5 ; −2,6]`, colunas em `x = ±6,5`.
    const LO: [f64; 2] = [-7.6, -4.1];
    const HI: [f64; 2] = [7.6, 1.1];
    let s = cena();
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for p in s.paths() {
        for v in &p.verts {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
    }
    assert!(
        lo[0] >= LO[0] && lo[1] >= LO[1] && hi[0] <= HI[0] && hi[1] <= HI[1],
        "a cena ocupa x {:?}..{:?} y {:?}..{:?} e sai da caixa que a =76 prova visivel - parte \
         dela nasce fora do quadro, e isso le-se como «faltam formas»",
        lo[0],
        hi[0],
        lo[1],
        hi[1]
    );
}

/// ⚠️ **A arte da estampa é OPACA por inteiro** — um quadrante transparente seria uma **segunda**
/// fonte de transparência a somar-se à que a cena mede, e nenhuma das duas se leria.
#[test]
fn the_pattern_art_carries_no_transparency_of_its_own() {
    let px = art_rgba();
    assert_eq!(px.len(), (ART * ART * 4) as usize);
    assert!(
        px.as_chunks::<4>().0.iter().all(|c| c[3] == 255),
        "a arte da estampa tem pixels translucidos - a cena passa a medir duas transparencias"
    );
}

/// ⚠️ A opacidade de uma estampa vive em DOIS campos que têm de descer juntos: o `alpha` (o que se
/// desenha) e a `fallback.a` (o instante pré-resolução). Fora de sincronia, a forma **salta** no
/// quadro em que o ladrilho carrega.
#[test]
fn the_scenes_patterns_keep_alpha_and_fallback_in_step() {
    for p in cena().paths() {
        if let Some(Paint::Pattern(f)) = p.fill.as_ref() {
            let esperado = f32::from(f.fallback.a) / 255.0;
            assert!(
                (f.alpha - esperado).abs() < 1e-6,
                "alpha={} contra fallback.a={} - a forma salta quando o ladrilho carrega",
                f.alpha,
                f.fallback.a
            );
        }
    }
}
