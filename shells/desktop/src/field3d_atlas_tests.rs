//! ⭐⭐⭐ **A IMAGEM DE UMA VISTA MUDA DE IDENTIDADE QUANDO OS PIXELS MUDAM — e em mais nenhuma
//! altura** (2026-08-30).
//!
//! # A lei, e porque ela nasceu de uma subida de stack e não de um defeito nosso
//!
//! O atlas de imagem da `vello` passou a ser **persistente** na 0.10: uma imagem entra pelo `id`
//! da `Blob`, fica residente, e só sai por despejo. Até à 0.8 o atlas era limpo a cada render, e
//! por isso a porta crua — que cunha uma `Blob` nova, logo um **id novo**, a cada chamada — era
//! barata *por construção*.
//!
//! ⇒ Um produtor que redesenhe a mesma imagem todo quadro deixou de pagar só o envio: ele passa a
//! encher um recurso **partilhado com o app inteiro**, com uma cauda de 2–3 quadros.
//!
//! Medido antes da cura (`crates/ph2d-vector/src/atlas_probe_tests.rs`), uma vista `2560×1440` com
//! a peça **parada**, 60 quadros:
//!
//! | | atlas | envios | bytes | despejos |
//! |---|---|---|---|---|
//! | ⛔ porta crua | **`8192²`** — o TECTO | 60 | **843,8 MB** | 52 |
//! | ⭐ handle estável | `4096²` | **1** | 14,1 MB | **0** |
//!
//! ⚠️ **Não era um defeito visível, e a sonda diz isso com número:** nem com um vizinho de
//! `4096²` a competir alguma imagem chegou a ser descartada (`NAO COUBE = 0` em toda a varredura).
//! *Era desperdício* — 843 MB/s de pixels que não mudaram, e um recurso partilhado no tecto.
//! Registar isto importa: a próxima leitura deste ficheiro não deve procurar um bug de tela.
//!
//! # As duas metades
//!
//! Uma lei sobre *«não muda»* é meia lei. Sozinha, ela fica verde num módulo que **nunca** produza
//! imagem nenhuma — que é precisamente o modo de falha que ela devia apanhar. Por isso o controlo
//! é obrigatório: **um traçado novo TEM de mudar o id**. Juntas, as duas dizem *«muda se e só
//! se»*, que é a lei inteira.

use crate::field3d_scene::lasso_tests::{AREA, armed_with};
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

/// Uma peça qualquer — o que se mede aqui é a identidade da imagem, não a forma dela.
fn peca() -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.4, 0.3, 0.2],
                round: 0.05,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a peça")
}

/// Desenha um quadro e devolve os ids de **todas** as imagens que a cena emitiu.
///
/// ⚠️⚠️ **A pergunta é feita à CENA, e não ao estado do módulo — e isso foi pago por uma mutação
/// que SOBREVIVEU.** A 1.ª versão deste arnês lia `Viewport::frame` (*«o handle que guardaste
/// mudou?»*) e ficava **verde** com uma chamada à porta crua acrescentada ao lado: o handle
/// guardado continuava o mesmo, e a cena cunhava uma residente nova por quadro na mesma.
/// ⇒ *o que conta é o que a cena EMITE, não o que o produtor GUARDA.*
fn tique(text: &mut ph2d_text::TextSystem) -> Vec<u64> {
    let mut scene = ph2d_vector::VectorScene::new();
    crate::field3d_smoke::draw(AREA, ph2d_tokens::Theme::default(), text, &mut scene);
    scene.probe_image_ids()
}

/// Gira até a cena emitir a primeira imagem, e devolve os ids desse quadro.
fn primeira_imagem(text: &mut ph2d_text::TextSystem) -> Vec<u64> {
    for _ in 0..600 {
        let ids = tique(text);
        if !ids.is_empty() {
            return ids;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    panic!("a cena nunca chegou a emitir uma imagem");
}

/// ⭐⭐⭐ **A LEI:** com a peça e a câmera paradas, redesenhar N quadros não cunha imagem nova.
///
/// ⚠️ **O prato tem de estar PARADO.** O módulo gira sozinho por omissão (*feature nova =
/// auto-play*), e uma câmera que se mexe pede traçado novo — o que faria este gate medir a
/// rotação em vez da identidade. `manual = true` é o estado em que o artista de facto trabalha:
/// ele tocou na peça, ela parou, e ele **olha** para ela. É aí que a porta crua desperdiçava 100%
/// do que gastava.
#[test]
fn a_still_viewport_does_not_mint_a_new_image_every_frame() {
    let doc = peca();
    armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        // Para o prato em TODAS as vistas: uma vista a girar ao lado continuaria a pedir traçado,
        // e o passe de uma vista pode roubar a vez à outra.
        crate::field3d_smoke::with_smoke(|s| {
            for v in &mut s.vps {
                v.manual = true;
            }
        });

        let _ = primeira_imagem(&mut text);

        // Deixa o assentar terminar: os dois degraus (movimento -> fino) são traçados NOVOS, e são
        // legítimos. O que a lei proíbe é um id novo depois de tudo assentar.
        for _ in 0..400 {
            let _ = tique(&mut text);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        const QUADROS: usize = 120;
        let mut ids = std::collections::BTreeSet::new();
        let mut por_quadro = 0usize;
        for _ in 0..QUADROS {
            let q = tique(&mut text);
            por_quadro = por_quadro.max(q.len());
            ids.extend(q);
        }
        // ⛔ O controlo interno: uma cena que não emite imagem nenhuma teria `ids` vazio e passaria
        // a desigualdade abaixo por vacuidade — *um balde que ninguém enche lê-se como perfeito*.
        assert_eq!(
            por_quadro, 1,
            "com uma vista só, cada quadro devia emitir exactamente UMA imagem; emitiu {por_quadro}"
        );
        assert_eq!(
            ids.len(),
            1,
            "{QUADROS} quadros de uma peça PARADA cunharam {} imagens distintas — cada uma é uma \
             residente nova no atlas persistente do vello, e a {}x{} de uma vista cheia isso foram \
             843,8 MB/s de pixels que nao mudaram",
            ids.len(),
            AREA.w as u32,
            AREA.h as u32
        );
    });
}

/// ⛔⛔ **O CONTROLO, e sem ele o gate acima é verde num módulo que não desenha nada.**
///
/// Um traçado novo **tem** de cunhar um id novo: é assim que o `ImageCache` sabe que os pixels
/// mudaram. ⚠️ Reusar o handle e chamar `mark_image_dirty` seria a outra saída — e é a errada
/// aqui: ela exigiria que o produtor e o `Renderer` combinassem por fora, e o produtor deste
/// módulo corre **noutra thread**.
#[test]
fn a_fresh_trace_does_mint_a_new_image() {
    let doc = peca();
    armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        crate::field3d_smoke::with_smoke(|s| {
            for v in &mut s.vps {
                v.manual = true;
            }
        });
        let inicial = primeira_imagem(&mut text);
        assert_eq!(inicial.len(), 1, "uma vista, uma imagem por quadro");

        // Esquecer o quadro força um traçado novo do frio — é o mesmo gesto que o gate da ordem
        // das vistas usa, e é a única forma de pedir «pixels novos» sem depender do relógio.
        crate::field3d_smoke::with_smoke(|s| {
            let a = s.active.min(s.vps.len() - 1);
            s.vps[a].probe_forget_frame();
        });

        for _ in 0..600 {
            let ids = tique(&mut text);
            if let Some(id) = ids.first() {
                assert_ne!(
                    *id, inicial[0],
                    "um traçado NOVO reusou a identidade do anterior — o atlas ficaria com os \
                     pixels velhos e a peça congelava na tela"
                );
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        panic!("o traçado novo nunca chegou");
    });
}
