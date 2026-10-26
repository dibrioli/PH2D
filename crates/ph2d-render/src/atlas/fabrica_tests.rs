use super::*;

fn alfa(art: &FactoryArt, x: u32, y: u32) -> u8 {
    factory_pixels(art)[((y * art.px[0] + x) * 4 + 3) as usize]
}

/// Toda a arte abre, rasteriza ao tamanho da entrada, tem desenho, e cada chave é uma só.
#[test]
fn toda_a_arte_rasteriza() {
    for art in &FACTORY_ART {
        let px = factory_pixels(art);
        assert_eq!(px.len() as u32, art.px[0] * art.px[1] * 4, "{:#x}", art.key);
        assert!(px.chunks(4).any(|p| p[3] == 255), "{:#x} vazia", art.key);
        assert_eq!(art.px[0] % art.grid[0], 0);
        assert_eq!(art.px[1] % art.grid[1], 0);
        assert_eq!(FACTORY_ART.iter().filter(|a| a.key == art.key).count(), 1);
    }
}

/// ⭐ Os azulejos REPETEM sem costura: a 1.ª coluna de texels é a vizinha da última (o desenho é periódico),
/// e os do chão são opacos de ponta a ponta — um buraco na borda vira um risco no chão a cada metro (os do
/// fundo têm céu transparente por cima).
#[test]
fn os_azulejos_repetem_sem_costura() {
    for (art, vertical, opaco) in [
        (GROUND_TILE, false, true),
        (PLATFORM_TILE, false, true),
        (DIRT_TILE, true, true),
        (CLOUDS_TILE, false, false),
        (HILLS_TILE, false, false),
    ] {
        let px = factory_pixels(&art);
        let [w, h] = art.px;
        let at = |x: u32, y: u32| &px[((y * w + x) * 4) as usize..((y * w + x) * 4 + 4) as usize];
        let perto = |a: &[u8], b: &[u8]| a.iter().zip(b).all(|(p, q)| p.abs_diff(*q) <= 40);
        for y in 0..h {
            assert!(
                !opaco || at(0, y)[3] == 255,
                "{:#x} buraco na borda esquerda em y={y}",
                art.key
            );
            assert!(
                perto(at(0, y), at(w - 1, y)),
                "{:#x} costura em y={y}",
                art.key
            );
        }
        if vertical {
            for x in 0..w {
                assert!(
                    perto(at(x, 0), at(x, h - 1)),
                    "{:#x} costura em x={x}",
                    art.key
                );
            }
        }
    }
}

/// Um azulejo é o MESMO desenho em todas as células — o vizinho da do meio, no atlas, é ele próprio.
#[test]
fn um_azulejo_repete_o_mesmo_desenho_em_cada_celula() {
    for art in FACTORY_ART.iter().filter(|a| a.tile) {
        let px = factory_pixels(art);
        let [cw, ch] = art.cell_px();
        let celula = |c: u32, r: u32| -> Vec<u8> {
            (0..ch)
                .flat_map(|y| {
                    let o = (((r * ch + y) * art.px[0] + c * cw) * 4) as usize;
                    px[o..o + (cw * 4) as usize].to_vec()
                })
                .collect()
        };
        let meio = art.middle_frame();
        let ref_ = celula(meio % art.grid[0], meio / art.grid[0]);
        for r in 0..art.grid[1] {
            for c in 0..art.grid[0] {
                assert!(celula(c, r) == ref_, "{:#x} célula ({c},{r})", art.key);
            }
        }
    }
}

/// FNV-1a das chaves e dos texels das 6 artes de antes do plano 11, pela ordem do atlas — medido ANTES da arte
/// do fundo (`docs/GameEngineFuncional/11_camara_e_parallax_plano.md`).
const HASH_DAS_ARTES_ANTIGAS: u64 = 0xea58_a1ac_470a_2c84;

/// ⭐ **Ficheiro antigo igual:** a arte nova entra no FIM e as 6 de antes ficam com a chave, o lugar e os texels.
#[test]
fn as_artes_antigas_sao_as_de_antes() {
    let antigas = [
        HERO_ART.key,
        GROUND_TILE_KEY,
        PLATFORM_TILE_KEY,
        DIRT_TILE_KEY,
        COIN_SHEET_KEY,
        FLAG_SHEET_KEY,
    ];
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (art, chave) in FACTORY_ART.iter().zip(antigas) {
        assert_eq!(art.key, chave, "a ordem do atlas mudou");
        for b in art.key.to_le_bytes().iter().chain(factory_pixels(art)) {
            h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    assert_eq!(h, HASH_DAS_ARTES_ANTIGAS, "hash medido {h:#018x}");
}

/// A moeda e a bandeira deixam 2 texels transparentes entre células vizinhas, e cada célula muda (anima).
#[test]
fn as_folhas_animadas_tem_margem_e_mexem() {
    for art in [COIN_SHEET, FLAG_SHEET] {
        let [cw, ch] = art.cell_px();
        let celula = |c: u32| -> Vec<u8> {
            (0..ch)
                .flat_map(|y| (0..cw).map(move |x| (x, y)))
                .map(|(x, y)| alfa(&art, c * cw + x, y))
                .collect()
        };
        for c in 0..art.grid[0] {
            for y in 0..ch {
                for x in [0, 1, cw - 2, cw - 1] {
                    assert_eq!(
                        alfa(&art, c * cw + x, y),
                        0,
                        "{:#x} célula {c} na margem",
                        art.key
                    );
                }
            }
            assert_ne!(
                celula(c),
                celula((c + 1) % art.grid[0]),
                "{:#x} célula {c} parada",
                art.key
            );
        }
    }
}

/// ⭐ Plano GEF 13 D1: a IMAGEM de um azulejo é uma célula, e as cópias do atlas são ela repetida — byte a byte
/// a tira de antes (o `HASH_DAS_ARTES_ANTIGAS` guarda-a); uma folha animada é a mesma nos dois.
#[test]
fn a_imagem_de_um_azulejo_e_uma_celula_e_o_atlas_a_envolve() {
    for art in &FACTORY_ART {
        let [w, h] = art.image_px();
        assert_eq!(factory_image(art).len() as u32, w * h * 4, "{:#x}", art.key);
        if art.tile {
            assert_eq!([w, h], art.cell_px());
            assert_eq!(art.frames(), [1, 1]);
            assert_eq!(
                crate::atlas::wrap_image(factory_image(art), [w, h], art.grid),
                factory_pixels(art)
            );
        } else {
            assert_eq!(art.frames(), art.grid);
            assert_eq!(factory_image(art), factory_pixels(art));
        }
    }
}
