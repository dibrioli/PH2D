//! **As FAMÍLIAS da fábrica de pincéis** (missão Pincéis, J3 — `docs/Painter/Pinceis/04_BANCADA.md` §9). Cada família
//! escreve SÓ o seu ficheiro: as receitas do Wet Paint (`WET`), as de Digital/Watercolor/Impasto (`MEIOS`), as fotos
//! do portão por mídia (`MIDIAS`) e os testes dela (`#[test]` no próprio ficheiro).
//!
//! ⚠️ Este é o ÚNICO sítio que lista as famílias, e já lista as da J3 e as da J4: nenhuma família o edita.

pub mod aerografo;
pub mod aguarela;
pub mod basicos;
pub mod caligrafia_marcadores;
pub mod canetas;
pub mod efeitos;
pub mod grafite;
pub mod guache;
pub mod lapis;
pub mod mistura;
pub mod oleo_acrilica;
pub mod pastel_carvao;
pub mod texturas;
pub mod tintas;

/// O que uma família importa: `use super::prelude::*;`.
pub mod prelude {
    pub(crate) use crate::midias_de_fabrica::Metrica::{
        Borda, Chato, Cobertura, Cruzamento, Cv, Largura, Orla, Ponta, Pontas, RampaCobertura,
        RampaLargura, Textura, Tinta,
    };
    pub(crate) use crate::midias_de_fabrica::Midia;
    pub(crate) use crate::receitas_de_fabrica::{
        CHARCOAL, DIREITA, GRAO_DURO, GRAO_MACIO, GRAPHITE_8B, MAIS_PERTO, Receita, ReceitaMeio,
        extras_do_lapis, lapis,
    };
    pub(crate) use ph2d_brush_presets::Tilt;
}

/// Uma família: o nome (o do ficheiro) e as três listas dela.
pub struct Familia {
    pub nome: &'static str,
    pub wet: &'static [crate::receitas_de_fabrica::Receita],
    pub meios: &'static [crate::receitas_de_fabrica::ReceitaMeio],
    pub midias: &'static [crate::midias_de_fabrica::Midia],
}

macro_rules! familia {
    ($m:ident) => {
        Familia {
            nome: stringify!($m),
            wet: $m::WET,
            meios: $m::MEIOS,
            midias: $m::MIDIAS,
        }
    };
}

/// Todas as famílias — J3: básicos (A), grafite e lápis (B), canetas e tintas (C), caligrafia e marcadores (D, E),
/// aerógrafo (J); J4: pastel e carvão (F), óleo e acrílica (G), guache (H), aguarela (I), texturas (K), mistura (L),
/// efeitos (M). A ordem não muda nenhum ficheiro gerado.
pub const FAMILIAS: [Familia; 14] = [
    familia!(basicos),
    familia!(grafite),
    familia!(lapis),
    familia!(canetas),
    familia!(tintas),
    familia!(caligrafia_marcadores),
    familia!(aerografo),
    familia!(pastel_carvao),
    familia!(oleo_acrilica),
    familia!(guache),
    familia!(aguarela),
    familia!(texturas),
    familia!(mistura),
    familia!(efeitos),
];

/// Cada família listada é um ficheiro `familias/<nome>.rs` e cada ficheiro uma família listada (o agregador não esquece
/// nenhuma); imprime o que cada uma tem.
#[test]
fn cada_familia_e_um_ficheiro() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it/familias");
    let mut ficheiros: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(".rs").map(str::to_owned))
        .filter(|n| n != "mod")
        .collect();
    ficheiros.sort();
    let mut listadas: Vec<String> = FAMILIAS.iter().map(|f| f.nome.to_owned()).collect();
    listadas.sort();
    assert_eq!(
        ficheiros, listadas,
        "um ficheiro de família fora do agregador (ou o contrário)"
    );
    for f in &FAMILIAS {
        println!(
            "[familia] {:<22} Wet {:>2} · outros meios {:>2} · fotos {:>2}",
            f.nome,
            f.wet.len(),
            f.meios.len(),
            f.midias.len()
        );
    }
}
