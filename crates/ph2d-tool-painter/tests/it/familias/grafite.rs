//! **Família `grafite`** — J3, catálogo B (a série de grafite): Graphite 8B · 2B · HB · 2H, Mechanical 0.5.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! O 8B é base PARTILHADA (`receitas_de_fabrica::GRAPHITE_8B`: os lápis nascem dele): mudá-lo muda a família `lapis`.

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Pencil › Graphite 8B: sem água, o grão do papel a abrir na orla. Prompt 35 (14/10), contra o Rebelle
    // «Graphite Pencil/8B» a 20 px vistos (`pastel_regua.py`, nota 4,21 → 1,15): o Noise 0,4 era a mancha de
    // ~640 px do Soft Pastel (5 px a y = 40, 8 a 450) — Noise 32; papel 0,6 e carga 125 (o miolo cheio do 8B).
    // Com o arrasto na ponta seca (o motor deixou de o zerar): o portão das mídias reprovava a Textura (7,65
    // contra 9,09 ± 0,66) e o arrasto só a BAIXA (0,16 → 3,25) — não são as estrias. Varridos 1 368 + 594 na
    // bancada: perfil Linear, jitter 0,1 (a borda ondulada do 8B), arrasto 0,05, papel 0,65 — nota 1,25 → 0,64.
    GRAPHITE_8B,
    // ⭐ Os do prompt 53 (doc 53 §8): as receitas que passaram a foto do Rebelle no doc 52 §4 (`52_regua/receitas_19*.json`,
    // pelo nome no comentário), cada uma o vizinho (`nasce_de`) inteiro + os overrides. O tamanho é o do preset
    // dele (`volume_params.size` do `.rebelle-brush.png`, `ferramentas/tamanho_dos_presets.py`).
    // `lapisb_027` com a carga 50: o lápis duro, o grão fino. A 40 cobria 0,560 — a do HB (Rebelle 2H 0,595, HB 0,561) e
    // ficava mais perto da foto do irmão (5,3 × 4,2 ruídos); a 50, 0,617 e 3,7 × 11,2 (doc 53, `IRMAOS`).
    lapis(
        "Graphite",
        "Graphite 2H",
        50.0,
        0.18,
        GRAO_DURO,
        extras_do_lapis!(0.5, 2.0),
    ),
    // `lapisb_028`.
    lapis(
        "Graphite",
        "Graphite HB",
        40.0,
        0.18,
        GRAO_DURO,
        extras_do_lapis!(0.5, 2.0),
    ),
    // `lapisb_032`.
    lapis(
        "Graphite",
        "Graphite 2B",
        40.0,
        0.52,
        GRAO_MACIO,
        extras_do_lapis!(0.25, 2.0),
    ),
];

pub const MEIOS: &[ReceitaMeio] = &[];

// O raio é o do preset à largura SECA da foto pela lei PRÓPRIA dele (`ferramentas/avalia_53.py`).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Graphite 8B", foto: "pencil", raio: 15.0, metricas: &[(Cobertura, 0.798), (Textura, 9.093), (Borda, 0.320), (Orla, 7.0)] },
    Midia { preset: "Graphite 2H", foto: "pencil2h", raio: 14.40, metricas: &[(Cobertura, 0.595), (Textura, 4.983), (Borda, 0.321), (Orla, 10.0), (Cruzamento, 0.599), (Largura, 24.0)] },
    Midia { preset: "Graphite HB", foto: "pencilhb", raio: 15.31, metricas: &[(Cobertura, 0.561), (Textura, 6.153), (Borda, 0.325), (Orla, 11.0), (Cruzamento, 0.583), (Largura, 24.0)] },
    Midia { preset: "Graphite 2B", foto: "pencil2b", raio: 14.40, metricas: &[(Cobertura, 0.544), (Textura, 13.901), (Borda, 0.350), (Orla, 11.0), (Cruzamento, 0.589), (Largura, 24.0)] },
];
