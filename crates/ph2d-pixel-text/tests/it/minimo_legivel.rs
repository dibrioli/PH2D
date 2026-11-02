//! A régua de [`ph2d_pixel_text::SYSTEM_MIN_PX`]: o menor tamanho (px por em, Antialias desligado,
//! Hinting ligado — as omissões do Text) a partir do qual os 62 caracteres `A-Z a-z 0-9` saem
//! todos DIFERENTES uns dos outros (os pares que a fonte já iguala a 48 px não contam: são dela).
//! Mede as fontes de sistema comuns que estiverem instaladas; ausentes, saltam.

use ph2d_pixel_text::{FontChoice, SYSTEM_MIN_PX, TextStyle, rasterize_outline};

const FONTES: [&str; 12] = [
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
    "/usr/share/fonts/TTF/DejaVuSerif.ttf",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
    "/usr/share/fonts/noto/NotoSerif-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
    "/usr/share/fonts/cantarell/Cantarell-VF.otf",
    "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
    "/usr/share/fonts/TTF/FiraSans-Regular.ttf",
    "/usr/share/fonts/TTF/Vera.ttf",
];

fn mascaras(data: &[u8], px: u32) -> Vec<(u32, u32, i32, Vec<u8>)> {
    let style = TextStyle::new(FontChoice::Builtin(String::new()), px);
    ('A'..='Z')
        .chain('a'..='z')
        .chain('0'..='9')
        .map(|c| {
            let m = rasterize_outline(data, 0, &c.to_string(), &style).expect("rasteriza");
            (m.w, m.h, m.baseline, m.cover)
        })
        .collect()
}

fn colisoes(m: &[(u32, u32, i32, Vec<u8>)]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..m.len() {
        for j in i + 1..m.len() {
            if m[i] == m[j] {
                v.push((i, j));
            }
        }
    }
    v
}

/// O 1.º px de 4..=32 em que nenhum par (que a fonte distingue a 48) colide; e as colisões de cada
/// px abaixo dele (a tabela).
fn minimo(data: &[u8]) -> (u32, Vec<usize>) {
    let intrinsecos = colisoes(&mascaras(data, 48));
    let mut tabela = Vec::new();
    for px in 4..=32 {
        let novas = colisoes(&mascaras(data, px))
            .into_iter()
            .filter(|p| !intrinsecos.contains(p))
            .count();
        if novas == 0 {
            return (px, tabela);
        }
        tabela.push(novas);
    }
    (33, tabela)
}

#[test]
fn system_min_px_is_the_largest_measured_legibility_floor_of_the_common_fonts() {
    let mut pior = 0;
    for f in FONTES {
        let Ok(data) = std::fs::read(f) else {
            continue;
        };
        let (m, tabela) = minimo(&data);
        println!("{f}: {m} px · colisões de 4 px em diante {tabela:?}");
        pior = pior.max(m);
    }
    if pior > 0 {
        assert_eq!(SYSTEM_MIN_PX, pior, "a constante é o pior medido");
    }
}

/// A lista do dropdown: as de fábrica, as de pixel do sistema instaladas (na ordem de
/// `PIXEL_POPULAR`, casadas sem maiúsculas/espaços/hífens) e depois as restantes, na ordem dada.
#[test]
fn the_font_list_puts_the_pixel_fonts_first() {
    use ph2d_pixel_text::BUILTIN;
    use ph2d_pixel_text::carimbo::{font_list, pixel_popular_count};
    let sistema = [
        "Noto Sans",
        "VT323",
        "DejaVu Sans",
        "press-start 2p",
        "Silkscreen",
    ];
    let l = font_list(sistema.map(str::to_owned));
    let n = BUILTIN.len();
    assert_eq!(
        &l[n..],
        [
            "press-start 2p",
            "Silkscreen",
            "VT323",
            "Noto Sans",
            "DejaVu Sans"
        ]
    );
    assert_eq!(pixel_popular_count(&l), n + 3);
}
