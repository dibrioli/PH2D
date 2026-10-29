//! **Família `aguarela`** — J4, catálogo I (aguarela, sumi-e): Wet Wash, Wet Dry Brush, Wet Drip, Wet Liner, Wet Flat, Sumi-e.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! ⛔ O Wet Round NÃO é desta família: é o pincel sem preset e o controlo dos portões (`receitas_de_fabrica::RECEITAS`).

use super::prelude::*;

mod estudo;
mod lamina;
mod molhado;
mod portoes;
mod sonda;

/// O pigmento do Wet Flat (doc 44 §3, lote 3). J4: 44 casava a cobertura do recto do Rebelle (régua 383 → 399/456) mas
/// tirava a rampa da pressão da origem (`leis_sistemicas`, 0,212 > 0,18) — fica 50 (`exames/aguarela.md` I-3).
const PIGMENTO_DO_FLAT: f64 = 50.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — 0,41 (0,42).
const CURVA_DO_FLAT: f64 = 1.75;
/// ⭐ O MIOLO LISO (motor J4, `Knob::StrokeFilm`; `exames/aguarela.md` «Motor J4»): o traço deixa UM filme, sem o tapete
/// das cerdas — e o pincel de filme leva o arrasto a 0 (medido no portão do motor).
const FILME: (&str, f64) = ("strokeFilm", 1.0);
const SEM_ARRASTO: (&str, f64) = ("drag", 0.0);
/// ⭐ J4 3.ª ronda — o CRUZAMENTO MOLHADO do Rebelle (`exames/aguarela.md` «3.ª ronda»): a ponta da aguarela não se
/// suja (`pickup` 0 na receita: o azul que levava o vermelho da faixa molhada pelo vertical abaixo, R−B +33 contra −3
/// do Rebelle) e a tinta nova não escoa para a molhada de outro traço (a parede da J2): a lente pálida do Mop sai.
const PAREDE: (&str, f64) = ("levelingOwnStroke", 1.0);
/// O cruzamento SECO do Rebelle é um vidrado (azul-marinho, não o violeta-cinza da mistura): 0,2.
const VIDRADO: (&str, f64) = ("glaze", 0.2);
/// ⭐ O FIO escuro da borda: a orla nasce na borda que seca ANTES do miolo — o filme encharcava (água 3,5 por célula contra
/// ~1 sem ele) e aos 240 quadros nada secara. Água por dab 0,15 e capilaridade 0,05 (a mancha da água pouca).
const POUCA_AGUA: [(&str, f64); 2] = [("waterPerDab", 0.15), ("capillary", 0.05)];

/// ⭐ J5 — MOLHADO × MOLHADO (`Knob::WetOpen`, `exames/aguarela.md` «Motor J5»): o azul que entra no vermelho molhado abre
/// uma mancha redonda e empurra-o. Medido na cena `sobre` (A): o Rebelle `Liner` 1,42 × a largura · escuro 1,06 — com
/// 0,42 · 0,5 o nosso 1,51 · 1,06 (régua 419 → 430/454); o `Wash Round` 1,17 · 1,30 — 0,15 · 0,4 (373 → 384/454).
const MANCHA_DO_LINER: [(&str, f64); 2] = [("wetOpen", 0.42), ("wetOpenPush", 0.5)];
const MANCHA_DO_WASH: [(&str, f64); 2] = [("wetOpen", 0.15), ("wetOpenPush", 0.4)];

/// ⭐ J6 — a BORDA da mancha molhada esbate (`Knob::WetOpenEdge`) e o filme largo tem o miolo carregado
/// (`Knob::StrokeFilmDome`; `exames/aguarela.md` «Motor J6»). Medido contra o Rebelle (cena `sobre` e o recto de 200 px):
/// o degradê da mancha `Liner` 12,2 células e 1,42 × a largura (pena 28 → 12,5 · 1,46); o domo `Liner` 0,89 ·
/// 0,91 · 0,94 a 5 · 10 · 20 % da largura (0,3 → 0,88 · 0,92 · 0,96), `Wash Round` 0,79 · 0,85 · 0,92 (0,3 → 0,76 · 0,86 ·
/// 0,93), `Mop` quase chato (0,97 · 0,98 · 0,94).
const DOMO: (&str, f64) = ("strokeFilmDome", 0.3);
/// J6 2.ª ronda (revisor cego: «a ref esbate o traço grosso, a imitação tem contorno duro»): as tampas do filme largo
/// esbatem (`Knob::StrokeFilmEnds`) — a ponta do Wash a 200 px 0,94 → 0,83 do meio (Rebelle ~0,5).
const TAMPAS: (&str, f64) = ("strokeFilmEnds", 0.8);
/// As curas da J6 sobre o preset de fábrica — por aplicar até à revisão cega (o coordenador): a bancada escreve o
/// estudo com `"cura": "<nome>"` (`estudo.rs`).
pub const CURAS_J6: &[(&str, &[(&str, f64)])] = &[
    ("Wet Liner", &[("wetOpenEdge", 28.0), DOMO, TAMPAS]),
    // A pena NÃO serve ao Wash (medido J6: com a orla 100 a frente da água esticada risca a mancha de fios escuros e a
    // pena larga levanta o vermelho numa caixa branca): só o domo.
    ("Wet Wash", &[DOMO, TAMPAS]),
];
/// Os campos do PINCEL das curas da J6 (a mesma regra: por aplicar até à revisão cega). J6t: as pontas ESBATEM ao comprido
/// (`docs/Painter/Pinceis/J6t_pontas_esbatem.md`) — o Wash a 1 (RMS dos perfis das pontas ao Rebelle 0,182 → 0,096; 1,3 ·
/// 2: 0,135 · 0,214; régua 384 = 384). ⛔ O Liner NÃO: 0,5 · 1 · 1,6 → 0,086 · 0,102 · 0,196 contra 0,084 sem (a 40 px o
/// Rebelle Liner é cheio até à tampa; régua 429 → 426).
pub const CURAS_J6_PINCEL: &[(&str, &[(&str, &str)])] =
    &[("Wet Wash", &[("pontas_esbate", "1.0")])];

pub const WET: &[Receita] = &[
    // J4 3.ª ronda (`exames/aguarela.md`): CURADO, NÃO aprovado — o FILME (o miolo mosqueado e a orla oca que dois revisores
    // repetiram saem), dura 0,9, papel 0, pigmento 2,8, orla 100, largura fixa, a ponta que não se suja; régua 349 → 373/456
    // e a rampa a 0,12 da origem desde que ela pinta (decisão 15). O revisor novo ainda vê a 200 px as pontas duras (o
    // Rebelle esbate em degradê) e o vermelho molhado que o Rebelle empurra para os lados.
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Wash",
        water: 0.4,
        opacity: 1.0,
        pigment: 2.8,
        pickup: 0.0,
        gate: 0.0,
        evaporation: 0.7,
        edge: 100.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.0),
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            FILME,
            SEM_ARRASTO,
            VIDRADO,
            MANCHA_DO_WASH[0],
            MANCHA_DO_WASH[1],
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
        real_oil: false,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Dry Brush",
        // J4: NÃO aprovado (`docs/Painter/Pinceis/exames/aguarela.md` I-5). A origem (Rebelle `Watercolor/Flat Dry`) é
        // um chato cheio: curá-lo para ela deu o Wet Flat sem esmaecer (régua 220 → 308/454, e o revisor cego emparelhou-o
        // com o Wet Flat em 6/6) — a receita fica a de antes, e a fusão vai ao coordenador.
        water: 0.2,
        opacity: 1.0,
        pigment: 300.0,
        pickup: 1.6,
        gate: 0.4,
        evaporation: 1.5,
        edge: 56.25,
        tilt: DIREITA,
        bristle_size: 1.5,
        bristle_count: 1500.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
        real_oil: false,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Drip",
        water: 1.0,
        opacity: 1.0,
        pigment: 63.0,
        pickup: 0.8,
        gate: 0.2,
        evaporation: 0.6,
        edge: 20.0,
        tilt: Tilt {
            on: true,
            ring: 8,
            spoke: 3,
        },
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
        real_oil: false,
    },
    // ⭐ O 1.º da Fase C (prompt 17, 05 §4b «Presets de todas as mídias»): o Liner do Rebelle
    // (`Watercolor/Liner`: duro, carregado, papel 0,10 — o grão quase não fura, 28,86 px). Medido contra
    // `fixtures/rebelle/wcliner.png` (cobertura 0,65 · textura 1,7 %) à largura 38; o Wet Round dá 0,19 · 8,1.
    // J4 motor (`exames/aguarela.md` «Motor J4»): com o FILME (sem o tapete, sem a soma), papel 0, orla 200, pigmento 17
    // — régua 322 → 375/456, as listras e o grão saem (o revisor novo vê-o liso). J4 3.ª ronda: a ponta que não se suja,
    // a parede, o vidrado 0,2 e a água pouca (o FIO escuro da borda) — régua 419/456; curado, NÃO aprovado: a 200 px o
    // Rebelle esbate as pontas e carrega o miolo (o nosso chapado), e a espiral dele mostra as costuras das voltas.
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Liner",
        water: 0.5,
        opacity: 1.0,
        pigment: 17.0,
        pickup: 0.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 200.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: Some(29.0),
        extras: &[
            FILME,
            SEM_ARRASTO,
            PAREDE,
            VIDRADO,
            POUCA_AGUA[0],
            POUCA_AGUA[1],
            MANCHA_DO_LINER[0],
            MANCHA_DO_LINER[1],
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
        real_oil: false,
    },
    // ⭐ As FORMAS (prompt 44, doc 44): o Wet Round com a ponta RETA (quina 1, dura 0,9 — a ponta macia lê-se
    // redonda), um pouco mais alta que larga e FIXA na tela (o rato do Rebelle não roda a ponta), e a pressão
    // que ESMAECE em vez de estreitar. Watercolor › Flat do Rebelle (`wcflat`).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DO_FLAT,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", CURVA_DO_FLAT),
        ],
        grao: None,
        achatado: Some((0.12, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: None,
        real_oil: false,
    },
];

/// ⛔ Os REPROVADOS da J4 (`docs/Painter/Pinceis/exames/aguarela.md`): a receita medida fica para quando o motor que lhes
/// falta existir — sem ficheiro de preset (a fábrica são os aprovados e os pré-existentes).
pub const EM_ESTUDO: &[Receita] = &[
    // J4 I-7: a trincha CARREGADA do Rebelle (`Watercolor/Mop`): a lavagem sem o fio escuro, muita água (a espiral
    // funde-se num disco), a pressão estreita (curva 0,5) E esmaece. J4 motor: com o FILME a régua 368/445 → 405/456 e o
    // revisor cego novo leu o NOSSO como o original em 6/6 (média); reprovado ainda pelo que ele listou do nosso lado: o
    // recto de 5 px quase some, a rampa 0,15→1→0,15 de tampas redondas (o Rebelle afina), a mancha lilás com contas no
    // cruzamento molhado (`exames/aguarela.md` «Motor J4»). 3.ª ronda: a ponta que não se suja e a parede (a lente lilás
    // pálida sai: a cor do cruzamento molhado [183,170,194] contra o Rebelle [182,176,200]); régua 390/456; o revisor novo
    // tomou o NOSSO pelo original nos gestos e acha o Rebelle nos tamanhos e no «sobre»: as pontas do 200 px, a mancha
    // redonda que o azul abre no vermelho molhado (o nosso, com a parede, um quadrado) — reprovado ainda.
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Mop",
        water: 1.0,
        opacity: 1.0,
        pigment: 5.2,
        pickup: 0.0,
        gate: 0.05,
        evaporation: 0.7,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("waterPerDab", 1.5),
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.3),
            FILME,
            SEM_ARRASTO,
            PAREDE,
            VIDRADO,
            // J6: a mancha do molhado (a da J5, 0,42 · 0,8) agora de borda esbatida, e o domo leve (o Rebelle `Mop` é
            // quase chato através).
            ("wetOpen", 0.42),
            ("wetOpenPush", 0.8),
            ("wetOpenEdge", 28.0),
            ("strokeFilmDome", 0.1),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: Some("Wet Wash"),
        real_oil: false,
    },
    // J4 I-8: a trincha de CERDAS da aguarela do Rebelle (`Bristle/Bristle Flat`, 46 × 16 px): a lâmina FIXA na tela
    // (larga deitada, fina de pé; a espiral abre buracos), cheia, de borda direita — o Wet Flat achatado a 0,65. Régua
    // 282/454; reprovado: o pigmento é por COMPRIMENTO e a lâmina deitada varre 3× a área da de pé — o recto fica a 0,28
    // e o vertical a 0,43 (Rebelle 0,44 · 0,42); nenhum pigmento casa os dois (motor: o pigmento pela largura da pegada).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Bristle Watercolor",
        water: 1.0,
        opacity: 1.0,
        pigment: 60.0,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", CURVA_DO_FLAT),
        ],
        grao: None,
        // J6 L3: a lâmina do Rebelle (fixa, eixo 100°) é mais fina — w90/w0 0,35 · S±40/w0 0,89; 0,65 dava 0,44 · 0,94,
        // 0,75 dá 0,36 · 0,91 (`docs/Painter/Pinceis/J6_lamina.md`).
        achatado: Some((0.75, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: Some("Wet Flat"),
        real_oil: false,
    },
    // J4 I-13: o Sumi-e do Rebelle (`Sumi-e/Sumi-e soft`): o pincel de cerdas da tinta (o Ink Brush) com ÁGUA — sem
    // o reservatório nem o vidrado da tinta, a ponta desfiada inteira, o pigmento que se dilui (45: régua 306 → 378/454,
    // vizinho Charcoal 1,01). Reprovado: o revisor cego achou o Rebelle em 6/6 — as cerdas dele são um PENTE de riscas
    // finas com um fio escuro de um lado e falhas brancas (a espiral mostra as voltas) e bordas duras a 200 px; o nosso
    // é liso, de pontas redondas e halo esfiapado (`bristleStrength` 1,6 + `bristleNoiseLight` 0,45: riscas largas e
    // ondeadas, régua 295) — motor: as cerdas em pente do Wet.
    Receita {
        meio: "Watercolor",
        grupo: "Brush",
        nome: "Sumi-e",
        water: 0.5,
        opacity: 1.0,
        pigment: 45.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.0),
            ("tipReservoir", 0.0),
            ("leveling", 0.5),
            ("bristleTipRagged", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: Some("Ink Brush"),
        real_oil: false,
    },
];

/// ⭐ **J6 — em ESTUDO** (`docs/Painter/Pinceis/exames/aguarela_texturas_j6.md`): a aguarela que só se exprime em
/// VALORES — o grão do papel (I-9) e os salpicos (I-11). Fora da fábrica até à revisão cega (o coordenador promove);
/// a bancada escreve o estudo com `"receita": "<nome>"` (`estudo.rs`).
pub const EM_ESTUDO_J6: &[Receita] = &[
    // I-9 `Watercolor › Granulation › Granulation` do Rebelle: pálido, o pigmento só nos VALES do papel (pontos de
    // 1–2 px, sem miolo cheio), a borda macia; a rampa afina e esmaece.
    GRANULACAO,
    Receita {
        nome: "Granulation B",
        pigment: 40.0,
        gate: 1.5,
        extras: &[
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 0.6),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("extGranulation", 1.0),
            ("extValleyGran", 1.0),
            ("extGrainDither", 3.0),
            VIDRADO,
            PAREDE,
        ],
        ..GRANULACAO
    },
    // I-11 `Watercolor › Spatter › Spatter` do Rebelle: a 5–40 px um véu pálido e esfiapado; a 200 px gotas soltas de
    // 2–8 px espalhadas numa nuvem larga, algumas fundidas. Gotas = o spray do carimbo (L4) com água.
    SALPICO,
    // r2: o spray do carimbo no Wet espalha as gotas pela tela inteira (r1, prancha) e a variante a jitter 0,8 matou o
    // gerador por memória (OOM a 24 G) — as gotas passam a ser o PÓ do Wet (`dust`, a gota `dustDrop`) sobre a água.
    Receita {
        nome: "Spatter Watercolor C",
        pigment: 30.0,
        hardness: 0.0,
        edge: 0.0,
        extras: &[
            ("pressureSize", 0.5),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("dust", 0.95),
            ("dustDrop", 4.0),
            VIDRADO,
        ],
        pincel: &[],
        ..SALPICO
    },
    Receita {
        nome: "Spatter Watercolor D",
        pigment: 30.0,
        hardness: 0.0,
        edge: 0.0,
        extras: &[
            ("pressureSize", 0.5),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("dust", 0.95),
            ("dustDrop", 3.0),
            VIDRADO,
        ],
        pincel: &[("jitter", "0.5")],
        ..SALPICO
    },
    // J6s (o spray do Wet curado, `o_spray_do_wet`): a gota na nuvem do traço e um rasto só. E = o r1 tal qual; F = as
    // gotas em PÍXEIS (2–8 px a todo tamanho, como o Rebelle); G = F + o véu pálido por baixo (o traço fino do Rebelle é
    // um véu esfiapado); H = F mais cheio e mais pálido (12 marcas, pigmento 25, a nuvem mais larga).
    Receita {
        nome: "Spatter Watercolor E",
        ..SALPICO
    },
    Receita {
        nome: "Spatter Watercolor F",
        pincel: &[
            ("spray_count", "8"),
            ("spacing", "0.5"),
            ("jitter", "0.6"),
            ("jitter_scale", "0.6"),
            ("continuous_spacing", "false"),
            ("dab_jitter", GOTAS_EM_PX),
        ],
        ..SALPICO
    },
    Receita {
        nome: "Spatter Watercolor G",
        pincel: &[
            ("spray_count", "8"),
            ("spacing", "0.5"),
            ("jitter", "0.6"),
            ("jitter_scale", "0.6"),
            ("continuous_spacing", "false"),
            (
                "dab_jitter",
                r#"{"across": 0.6, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 1.0, "particle_px": 5.0, "particle_min_px": 2.0, "particle_max_px": 8.0, "roundness": 0.0, "veu": 0.35, "veu_escala": 2.5, "veu_opacidade": 0.25}"#,
            ),
        ],
        ..SALPICO
    },
    Receita {
        nome: "Spatter Watercolor H",
        pigment: 25.0,
        pincel: &[
            ("spray_count", "12"),
            ("spacing", "0.5"),
            ("jitter", "0.75"),
            ("jitter_scale", "0.6"),
            ("continuous_spacing", "false"),
            (
                "dab_jitter",
                r#"{"across": 0.8, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 1.0, "particle_px": 5.0, "particle_min_px": 2.0, "particle_max_px": 8.0, "roundness": 0.0}"#,
            ),
        ],
        ..SALPICO
    },
    // r2 da Granulation: r1 (papel 1,2–1,5) saiu pálido sem pontos e de orla escura — menos portão, mais pigmento,
    // sem orla, e os pontos pelo pontilhado do render (C) ou pelo pó (D).
    Receita {
        nome: "Granulation C",
        pigment: 120.0,
        gate: 0.8,
        edge: 0.0,
        extras: &[
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 0.6),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("extGranulation", 1.0),
            ("extValleyGran", 1.0),
            ("extGrainDither", 6.0),
            VIDRADO,
            PAREDE,
        ],
        ..GRANULACAO
    },
    Receita {
        nome: "Granulation D",
        pigment: 80.0,
        gate: 0.4,
        edge: 0.0,
        extras: &[
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 0.6),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("extGranulation", 1.0),
            ("dust", 0.8),
            ("dustDrop", 1.5),
            VIDRADO,
            PAREDE,
        ],
        ..GRANULACAO
    },
];
const GRANULACAO: Receita = Receita {
    meio: "Watercolor",
    grupo: "Grain",
    nome: "Granulation",
    water: 1.0,
    opacity: 1.0,
    pigment: 25.0,
    pickup: 0.0,
    gate: 1.2,
    evaporation: 1.0,
    edge: 12.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.3,
    size_px: None,
    extras: &[
        ("pressureSize", 0.7),
        ("pressureSizeCurve", 0.6),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 1.0),
        ("extGranulation", 1.0),
        ("extValleyGran", 1.0),
        VIDRADO,
        PAREDE,
    ],
    grao: None,
    achatado: None,
    pincel: &[],
    nasce_de: Some("Wet Round"),
    real_oil: false,
};
/// As gotas do Spatter em PÍXEIS (J6s): 5 px de diâmetro, 2–8 px com a escala, a nuvem do pincel.
const GOTAS_EM_PX: &str = r#"{"across": 0.6, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 1.0, "particle_px": 5.0, "particle_min_px": 2.0, "particle_max_px": 8.0, "roundness": 0.0}"#;
const SALPICO: Receita = Receita {
    meio: "Watercolor",
    grupo: "Splatter",
    nome: "Spatter Watercolor",
    water: 1.0,
    opacity: 1.0,
    pigment: 40.0,
    pickup: 0.0,
    gate: 0.2,
    evaporation: 1.0,
    edge: 30.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.6,
    size_px: None,
    extras: &[
        ("pressureSize", 0.5),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 1.0),
        VIDRADO,
    ],
    grao: None,
    achatado: None,
    pincel: &[
        ("spray_count", "6"),
        ("spacing", "0.5"),
        ("jitter", "0.7"),
        ("jitter_scale", "0.9"),
        ("continuous_spacing", "false"),
        (
            "dab_jitter",
            r#"{"across": 0.6, "angle_per_stroke": 0.0, "count": 0.6, "flip_x": 0.0, "flip_y": 0.0, "opacity": 0.5, "particle": 0.04, "roundness": 0.0}"#,
        ),
    ],
    nasce_de: Some("Wet Round"),
    real_oil: false,
};

/// A Aquarela básica (dono, 23/10): o pincel de fábrica do meio — os números do painel dele
/// (`state_default::aquarela_de_fabrica`), capturados sem nada por cima — o mesmo nome e o mesmo pincel
/// do preset de fábrica «Watercolor Basic».
pub const MEIOS: &[ReceitaMeio] = &[ReceitaMeio::aquarela(
    "Watercolor",
    "Basic",
    "Watercolor Basic",
)];

// Medido contra `fixtures/rebelle/wcliner.png` à largura dela; as formas (prompt 44, doc 44 §3) e a pressão que
// ESMAECE (prompt 45, doc 45).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Wet Liner", foto: "wcliner", raio: 19.0, metricas: &[(Cobertura, 0.653), (Textura, 1.708)] },
    Midia { preset: "Wet Flat", foto: "wcflat", raio: 17.0, metricas: &[(Cobertura, 0.315), (Ponta, 0.096), (RampaLargura, 0.829), (RampaCobertura, 0.42), (Largura, 34.0), (Pontas, 0.9)] },
];

/// Os reprovados da J4 ([`EM_ESTUDO`]) não têm ficheiro na fábrica, e nenhum tem o nome de um pincel dela.
#[test]
fn os_reprovados_da_aguarela_nao_estao_na_fabrica() {
    let b = ph2d_brush_presets::Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let na_fabrica: Vec<&str> = b
        .entradas
        .iter()
        .map(|e| e.preset.parametros.name.as_str())
        .filter(|n| EM_ESTUDO.iter().any(|r| r.nome == *n))
        .collect();
    assert!(
        na_fabrica.is_empty(),
        "reprovados na fábrica: {na_fabrica:?}"
    );
}
