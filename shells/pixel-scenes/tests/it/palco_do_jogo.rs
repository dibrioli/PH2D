//! ⭐ **O Pixel Sprite no PALCO DO JOGO** (P14, 09/11): no palco (Fullscreen/Window) o Pixel Sprite
//! desenha a ARTE pelo pivô — sem nada do Draw (xadrez, grelha, borda, selecção) — e bloqueia a luz,
//! como um Sprite do motor. Antes a faixa Pixel ia VAZIA no palco (`sem_pixel`, GEF 19 E2) e, sem
//! faixas, quem a desenhava era o realce da selecção, que não corre no palco.
//!
//! ```text
//! bash scripts/ph2d-run.sh bash scripts/cargo-test-narrow.sh ph2d-host-pixel-scenes \
//!     o_pixel_sprite_aparece_no_palco_do_jogo -- --ignored
//! ```

use ph2d_app_pixel::Tool;
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_tokens::Theme;
use ph2d_vector::{Point, VectorScene};

use crate::comum::{Mesa, PPM, WINDOW, spawn_root};

const WHITE: [u8; 4] = [255, 255, 255, 255];

/// O texto do ficheiro sem as linhas de comentário.
fn so_codigo(texto: &str) -> String {
    texto
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// *Mutações: repor o `sem_pixel` (faixa vazia) · tirar o `art` do `inteiro` (sem faixas, nada) ·
/// passar `editing` no palco (o xadrez do Draw aparece: o controlo abaixo mede-o).*
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn o_pixel_sprite_aparece_no_palco_do_jogo() {
    // 1. O elo da shell: no palco a faixa Pixel leva os ids dela, sem faixas o documento leva a arte,
    //    e nenhum dos dois caminhos passa o objecto em Draw.
    let doc = so_codigo(include_str!(
        "../../../desktop/src/render_loop/fase_vector_bands_doc.rs"
    ));
    let bandas = so_codigo(include_str!(
        "../../../desktop/src/render_loop/fase_vector_bands.rs"
    ));
    assert!(
        !doc.contains("sem_pixel"),
        "a faixa Pixel esvazia-se no palco"
    );
    let inteiro = &doc[doc.find("fn inteiro(").expect("o `inteiro`")..];
    let inteiro = &inteiro[..inteiro.find("\n    }").expect("fecha")];
    // Sem espaços: o `rustfmt` parte a cadeia em linhas e o elo continua o mesmo.
    let compacto: String = inteiro.split_whitespace().collect();
    assert!(
        compacto.contains(concat!("if", "self.palco{self.order.pixel_ids()}"))
            && compacto.contains("ph2d_app_pixel::draw::art("),
        "sem faixas, o documento do palco não leva a arte do Pixel:\n{inteiro}"
    );
    assert!(
        bandas.contains("palco: hero.view.palco_do_jogo")
            && bandas.contains(".filter(|_| !hero.view.palco_do_jogo)"),
        "o palco passa o objecto em Draw (xadrez, Tiled, onion)"
    );

    // 2. Os píxeis: um rectângulo branco num Pixel Sprite, desenhado como o palco o desenha.
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem placa: este gate corre na faixa da GPU");
    };
    let mut m = Mesa::nova();
    m.desenha(Tool::RECTANGLE, (4, 4), (12, 12));
    m.app
        .frame(&mut m.sim, &mut m.cam, WINDOW, PPM, None, spawn_root);
    assert_eq!(m.v((4, 4)), WHITE, "a arte assentou");
    let size = (WINDOW.width, WINDOW.height);
    let mut vp = VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, size).expect("vello");
    let cam = m.cam.world_to_screen_affine(WINDOW);
    let (borda, dentro, fora) = (m.celula((4, 8)), m.celula((8, 8)), m.celula((1, 1)));
    let mut ler = |editing: Option<u64>, chrome: bool| {
        let mut cena = VectorScene::new();
        let ids = [m.bits];
        let world = m.sim.world();
        ph2d_app_pixel::draw::art(
            &mut m.app, world, cam, PPM, editing, &ids, chrome, &mut cena,
        );
        if chrome {
            ph2d_app_pixel::draw::draw(
                &mut m.app,
                world,
                &m.cam,
                WINDOW,
                PPM,
                editing,
                &ids,
                Theme::default(),
                &mut cena,
            );
        }
        let px = vp
            .render_and_readback(&gpu, cena.inner(), size)
            .expect("render");
        move |p: (f32, f32)| {
            let i = ((p.1 as usize) * size.0 as usize + p.0 as usize) * 4;
            [px[i], px[i + 1], px[i + 2], px[i + 3]]
        }
    };
    let palco = ler(None, false);
    assert_eq!(
        palco(borda),
        WHITE,
        "a arte no sítio do objecto ({borda:?})"
    );
    assert_eq!(
        palco(dentro)[3],
        0,
        "o vão do rectângulo é transparente: sem xadrez"
    );
    assert_eq!(
        palco(fora)[3],
        0,
        "fora da arte: nem xadrez, nem grelha, nem borda"
    );
    // Controlo: a régua VÊ o chrome do Draw quando ele lá está.
    let editor = ler(Some(m.bits), true);
    assert_eq!(editor(borda), WHITE);
    assert!(editor(fora)[3] > 0, "o xadrez do Draw lê-se na mesma régua");

    // 3. A luz: o Pixel Sprite bloqueia no sítio da arte (a oclusão não pergunta pelo palco).
    let oc = ph2d_app_pixel::luz::oclusao(&m.app, m.sim.world(), PPM);
    let mundo = oc.objetos[0].img_mundo;
    let (c, f) = (mundo * Point::new(4.5, 8.5), mundo * Point::new(8.5, 8.5));
    // Uma janela de 0,1 m a 64 px (640 px/m): uma célula do sprite (1/PPM m) são 6,4 px.
    let r = 0.05;
    let janela = [c.x - r, c.y - r, c.x + r, c.y + r];
    let px = oc
        .rasterizar((64, 64), janela, &[m.bits])
        .expect("desenhou");
    let alfa = |p: Point| {
        let k = 64.0 / (2.0 * r);
        let (i, j) = (
            ((p.x - janela[0]) * k) as usize,
            ((janela[3] - p.y) * k) as usize,
        );
        px[(j * 64 + i) * 4 + 3]
    };
    assert_eq!(alfa(c), 255, "a borda branca faz sombra");
    assert_eq!(alfa(f), 0, "o vão deixa a luz passar");
}
