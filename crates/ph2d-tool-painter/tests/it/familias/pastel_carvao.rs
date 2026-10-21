//! **Família `pastel_carvao`** — J4, catálogo F (carvão, giz, pastel): Charcoal, Soft Pastel, Oil Pastel, Square Hard,
//! Wax Scrape. Escreva aqui as receitas e os testes da família; nada fora deste ficheiro
//! (`docs/Painter/Pinceis/04_BANCADA.md` §9). O `CHARCOAL` é base PARTILHADA (o Scratch da família `lapis` nasce dele).

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Pencil › Charcoal: pouca carga, orla larga. Prompt 35 (14/10), contra o Rebelle «Charcoal/Charcoal» a
    // 37 px vistos (nota 12,03 → 3,35): o Noise 0,15 era uma mancha (25 px a y = 40, 36 a 450) — Noise 48; o
    // carvão do Rebelle é RALO (miolo 0,64). A orla do Rebelle (15 px a 21 de largura, 25 px de subida a 38)
    // não cabe na dureza: a 0,5 o portão das mídias lia 7, e nenhuma dureza/papel/carga/arrasto passa de 9
    // (1 755 variantes). O perfil Sharp (p², a cauda longa) e o jitter 0,2 dão 10 e a poeira do carvão:
    // dureza 0,1, papel 0, carga 24, arrasto 0,02 — nota 3,31 → 0,79.
    // J4 (`exames/pastel_carvao.md`): o TECTO 76 (o Rebelle pinta 73–76 px a 200; o nosso 151, a nuvem), o jitter 0
    // (o 0,2 fazia manchas de ~100 px ao longo do recto de 200, que o Rebelle não tem) e as VETAS finas ao longo do
    // traço — a Shape um Noise `ViewPlane` (o padrão anda com o dab, o traço arrasta-o em fios: [`VETAS_DO_CARVAO`]),
    // carga 34 (a Shape tira tinta). A base `CHARCOAL` fica como estava (o Scratch dos lápis lê-a).
    // A revisão cega r1: o Rebelle quase não pinta nos primeiros 40 % da rampa e escurece no fim (miolo escuro) — a
    // pressão vai à opacidade (0,6, curva 1: a lei `leis_sistemicas` 0,163; a curva 2 dava 0,193); carga 38. (O `pressureGrain` não morde sem o portão do papel: papel 0.)
    // J4 r3: jitter 0,15 (a orla de pó; a 0 a mídia lia Orla 8 contra 15) e a Shape um Noise `Random` de lado 8 a 0,6 —
    // o miolo MARMOREADO (a revisão cega r1 via no nosso riscas horizontais regulares, a 200 px «como cerdas»); carga 42.
    Receita {
        pigment: 42.0,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.02),
            MAIS_PERTO,
            ("pressureOpacity", 0.6),
            ("pressureOpacityCurve", 1.0),
            ("pressureGrain", 0.0),
        ],
        pincel: &[
            ("falloff", "\"Sharp\""),
            ("jitter", "0.15"),
            ("shape", MARMORE_DO_CARVAO),
            ("size_max_px", TECTO_DO_CARVAO),
        ],
        ..CHARCOAL
    },
    // Pastel › Pastel Dry › Dry Thin: grão fino e regular. Prompt 35 (14/10), contra o Rebelle NO TEMPO e à
    // mesma largura vista (`ferramentas/pastel_regua.py`, nota 4,76 → 1,38): o grão Noise 0,4 era uma mancha
    // de ~640 px (a largura ia de 15 a 34 px conforme a ALTURA na tela) — Noise 100 é o grão de 2–4 px; o
    // papel 0,8 ondulava a borda; a carga 400 enchia o miolo (0,98 contra 0,91).
    // J4 r3 (duas revisões cegas repetem: o Rebelle mais aberto e translúcido, a rampa nasce em névoa e não em agulha, o
    // de 5 px pontilhado): papel 0,6, carga 85 e a pressão leve em grão (`pressureGrain` 0,5). A base `SOFT_PASTEL`
    // fica como estava (o Square Hard, o Wax Scrape e o Pastel Side nascem dela).
    Receita {
        pigment: 85.0,
        gate: 0.6,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 0.5),
            ("pressureGrain", 0.5),
        ],
        ..SOFT_PASTEL
    },
    // Pastel › Pastel Oil › Oil Pastel 2 (o giz de cera): grão mais fino e mais carregado que o seco (o
    // Voronoi passava as métricas, mas a largura variava até 42 % com a posição — medido 11/10). Prompt 35
    // (14/10), contra o Rebelle «Pastel Oil/Oil Pastel 2» a 77 px vistos (nota 5,06 → 1,41): o Noise 0,15
    // também era mancha (42 px a y = 40, 68 a 450) — Noise 100. Com o arrasto na ponta seca: o portão das
    // mídias reprovava Textura (9,7 contra > 12,6) e Orla (4 contra > 6); o arrasto as BAIXA (0,3: Textura
    // 12,9 só com papel 1,2) e não faz as estrias do Rebelle (a prancha: ele alisa). Carga 100, jitter 0,1,
    // arrasto 0,05 — nota 1,45 → 1,05.
    // J4 (`exames/pastel_carvao.md`): as ESTRIAS ao longo do traço são a Shape em Flow ([`ESTRIAS_DO_OLEO`]); com
    // elas o papel 0,6 e a carga 160 (o miolo das estrias tira tinta).
    OIL_PASTEL,
    // `sq_007` (prompt 53): o pastel de ponta RETA, o papel todo aberto. J4 (`exames/pastel_carvao.md`): o Rebelle
    // «Square Hard» é um RECTÂNGULO de aresta dura — a largura não muda com a pressão (rampa 26 → 33 px; a nossa ia
    // de 0 a 45) e a pressão só dá a opacidade (0,07 → 0,63, quase em recta): `pressureSize` 0, dureza 0,7, curva 2, carga 75
    // (a 90 o azul tapava o vermelho: Cruzamento 0,82 contra 0,58), e o grão GROSSO e manchado (Noise 100 a 0,6: a revisão
    // cega r1 via o nosso «fino e uniforme, como ruído»; a Textura da mídia 14 contra 21 com o 160). A pressão leve DESFAZ-SE em
    // grão (`pressureGrain` 0,25: a revisão cega r1 viu o Rebelle em pontos soltos onde o nosso esmaecia liso; a 0,5 a
    // largura da rampa leve caía a 0 e a lei da pressão `leis_sistemicas` ia a 0,24 — a 0,25, 0,10).
    Receita {
        nome: "Square Hard",
        pigment: 75.0,
        gate: 1.0,
        hardness: 0.7,
        size_px: Some(35.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSize", 0.0),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.0),
            ("pressureGrain", 0.25),
        ],
        grao: None,
        pincel: &[("dab_quina", "1.0"), ("texture", GRAO_DO_SQUARE), ("shape", MANCHAS_DO_SQUARE)],
        nasce_de: Some("Soft Pastel"),
        ..SOFT_PASTEL
    },
    // `wx_007` (prompt 53): a cera raspada — muita carga sobre o papel aberto. J4: os TRAÇOS curtos da cera
    // ([`TRACOS_DA_CERA`]) — sem eles era o Soft Pastel (d 0,484); carga 200, papel 0,8, dureza 0 (a borda macia da
    // mídia do Rebelle, 0,59; a 0,2 lia 0,82, e a orla ondula 0,051 a 40 px contra 0,045).
    Receita {
        grupo: "Scrape",
        nome: "Wax Scrape",
        pigment: 200.0,
        gate: 0.8,
        hardness: 0.0,
        size_px: Some(141.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 1.5),
        ],
        pincel: &[("dab_quina", "0.0"), ("shape", TRACOS_DA_CERA)],
        nasce_de: Some("Soft Pastel"),
        ..SOFT_PASTEL
    },
];

/// A orla aos caroços do giz de cera (J4): a 2.ª ponta (Multiply) a 0,8 do diâmetro, 2 marcas, espalhadas 0,6 e 0,5
/// através, dureza 0,4 — a do 8B (`familias::grafite`) mais larga: os caroços grandes do Rebelle a 200 px.
const ORLA_DA_CERA: &str = r#"{"angulo": 0, "atraves": 0.5, "contagem": 2, "dureza": 0.4, "escala": 0.8, "espalhar": 0.6, "forma": {"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}, "modo": 0, "passo": 0.25, "rotacao": 0.0}"#;

/// O miolo manchado do giz de cera (J4): a Shape um Noise `Random` de lado 6 a 0,8 — com ela e a pressão em curva (3,5 · 1)
/// o vizinho mais perto passa de Soft Pastel 0,528 a Graphite 8B 0,699 (a matriz das variantes, `exames/pastel_carvao.md`).
const MANCHAS_DA_CERA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.8, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [6.0, 6.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// ⭐ **As VETAS do carvão** (J4): a Shape um Noise `ViewPlane` de lado 16 a 0,6 (viés 0,55) — o padrão anda com o
/// dab, e o traço contínuo arrasta-o em fios finos ao longo (só com o jitter 0: o jitter baralha-os). Rebelle Charcoal
/// a 40 px: contraste 0,18; o nosso 0,17.
const VETAS_DO_CARVAO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.6, 0.55, 0.3, 0.5, 0.5, 0.5], "rake": false, "size": [16.0, 16.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// As vetas da barra do Compressed Charcoal (J4 r3): o Noise `ViewPlane` de lado 24 a 0,8 — riscas ao comprido mais finas
/// e fortes que as do lápis (o Rebelle «Charcoal Flat» de pé: translúcido e riscado).
const VETAS_DA_BARRA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.8, 0.5, 0.3, 0.5, 0.5, 0.5], "rake": false, "size": [24.0, 24.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O miolo marmoreado do Charcoal (J4 r3): a Shape um Noise `Random` de lado 8, contraste 0,6, viés 0,55.
const MARMORE_DO_CARVAO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.6, 0.55, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [8.0, 8.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O tecto do Charcoal (decisão 8): o recto de 200 px do Rebelle pinta 73–76 px (`fixtures/leis_escala_rebelle.json`).
const TECTO_DO_CARVAO: &str = "76";

/// Sem Shape (o `None` do pincel): os que nascem de um preset com Shape e não a querem.
const SEM_SHAPE: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// ⭐ **As ESTRIAS do pastel de óleo** (J4): a Shape um Noise em FLOW (o padrão no referencial do traço: ao longo o
/// arco, através a normal), quase constante ao longo (Size 0,02) e fino através (20 células por raio), contraste
/// 1,0 e viés 0,6 — faixas paralelas contínuas que seguem a curva e crescem com a ponta. Rebelle «Oil Pastel 2»: a
/// ½-autocorrelação no recto de 200 px 600 px ao longo × 6 através, contraste 0,23; sem a Shape 1 × 1, 0,17.
const ESTRIAS_DO_OLEO: &str = r#"{"angle_deg": 0, "flow": true, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [1.0, 0.6, 0.0, 0.5, 0.5, 0.5], "rake": false, "size": [0.02, 20.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O grão do Square Hard (J4): Noise `Tiled` Size 100 (2,56 px), contraste 0,6.
const GRAO_DO_SQUARE: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.6, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [100.0, 100.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O miolo MANCHADO do Square Hard (J4 r3): a Shape um Noise `Random` de lado 6 a 0,5 — a mídia do Rebelle varia ao longo
/// do traço (Cv 0,077; o nosso só com o grão 0,042) e a revisão cega r1 via-o «grosso e manchado».
const MANCHAS_DO_SQUARE: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [6.0, 6.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// ⭐ **Os TRAÇOS da cera** (J4): o mesmo Noise em Flow, mas curto ao longo (Size 8) e fino através (50) com o viés 0,78
/// — falhas brancas curtas deitadas ao longo do traço, o «Wax Scrape» do Rebelle a 200 px (½-autocorrelação 4 × 2 px,
/// contraste 0,29; a 40 px liso, 0,13). A 6 × 40 as falhas eram o dobro das dele (contraste 0,39 a 40 px).
const TRACOS_DA_CERA: &str = r#"{"angle_deg": 0, "flow": true, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.75, 0.78, 0.0, 0.5, 0.5, 0.5], "rake": false, "size": [8.0, 50.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// ⛔ **Os REPROVADOS da família** (J4, `docs/Painter/Pinceis/exames/pastel_carvao.md`): a receita fica (o próximo exame parte
/// dela), o ficheiro não — nenhum entra na fábrica sem a revisão cega (`os_reprovados_nao_estao_na_fabrica`).
pub const EM_ESTUDO: &[Receita] = &[
    // REPROVADO r2 e r3: os DOIS revisores cegos juntam-no ao Pastel Side (o nosso e o vizinho são o par) e dão o
    // Rebelle como «outro» — riscas paralelas ao comprido, translúcido, fiapos; o nosso tem o grão do papel por cima.
    // ⭐ F-2 (J4) `Pencil › Charcoal › Charcoal Flat`: o carvão PRENSADO, a barra de lado — a lâmina FIXA na tela
    // (Rebelle 16 × 37 px a 40: 2,31; o deitado denso 0,63–0,71, o de pé mais claro 0,41) com as vetas do carvão; o
    // tecto 40 (o do LADO FINO: a largura que o slider promete é a do recto deitado; a 200 px o Rebelle pinta 40
    // deitado). r3 (a revisão cega r2 juntava-o ao Pastel Side e via o Rebelle «translúcido, com riscas ao comprido»):
    // carga 80 e as vetas da barra mais fortes ([`VETAS_DA_BARRA`]); a pressão dá 0,3 do tamanho e a opacidade (curva 1:
    // a rampa dele já tem largura a 0,1 da pressão).
    Receita {
        nome: "Compressed Charcoal",
        pigment: 80.0,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.02),
            MAIS_PERTO,
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
        achatado: Some((0.55, 0)),
        pincel: &[
            ("falloff", "\"Sharp\""),
            ("jitter", "0.0"),
            ("shape", VETAS_DA_BARRA),
            ("size_max_px", "40"),
        ],
        nasce_de: Some("Charcoal"),
        ..CHARCOAL
    },
    // REPROVADO r2 e r3 (duas revisões, confiança média, a mesma diferença): a 200 px o Rebelle é um bloco de borda
    // definida com nódoas e furos; o nosso um halo gaussiano homogéneo. Falta a gota do pó GROSSA (a fila).
    // ⭐ F-4 (J4) `Pencil › Charcoal › Powder`: o PÓ de carvão — sem linha, ralo (0,16–0,19) e salpicado (o `dust`),
    // a borda macia; a pressão leve não pinta nada (a rampa do Rebelle só aparece a meio) e depois abre logo.
    Receita {
        nome: "Charcoal Powder",
        pigment: 11.0,
        hardness: 0.0,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.02),
            MAIS_PERTO,
            ("dust", 0.7),
            ("pressureSizeCurve", 0.8),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.5),
        ],
        pincel: &[("falloff", "\"Sharp\""), ("jitter", "0.2"), ("shape", SEM_SHAPE), ("size_max_px", "null")],
        nasce_de: Some("Charcoal"),
        ..CHARCOAL
    },
    // REPROVADO r2 e r3 (confiança média): falta o PÓ SOLTO à volta do traço (a dispersão de partículas é da família
    // `texturas`) e as faixas do carimbo do Rebelle a 200 px; a pressão já muda a densidade e não a largura.
    // ⭐ F-7 (J4) `Pastel › Pastel Dry › Flat Soft`: o LADO do bastão — a lâmina FIXA na tela (L3: o deitado fino, o
    // de pé largo; Rebelle 19 × 36 px a 40, 1,89), as pontas rectas (`dab_quina`), denso, o tecto 50 (o do LADO FINO: a
    // largura que o slider promete é a do recto deitado; a 200 px o Rebelle pinta 50 deitado). O Soft Pastel com a lâmina 0,47 a 0° e a curva da opacidade 1,2 (a rampa dele 0,05 → 0,62).
    // r3 (a revisão cega r2): a pressão muda a DENSIDADE e não a largura (`pressureSize` 0,2) e a leve é névoa de grão
    // (`pressureGrain` 0,5); o lado largo (os de pé) translúcido e riscado — as vetas `ViewPlane` (de pé 0,62, Rebelle 0,64).
    Receita {
        nome: "Pastel Side",
        size_px: None,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSize", 0.2),
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.2),
            ("pressureGrain", 0.5),
        ],
        achatado: Some((0.47, 0)),
        pincel: &[("dab_quina", "1.0"), ("size_max_px", "50"), ("shape", VETAS_DO_CARVAO)],
        nasce_de: Some("Soft Pastel"),
        ..SOFT_PASTEL
    },
    // REPROVADO r3: o revisor junta-o ao Graphite 8B (o nosso como original, o 8B como imitação) e dá o Rebelle como
    // «outro» — carimbos rectangulares e festões. Distinto do Soft Pastel só com a Shape e a pressão em curva (0,699).
    // ⭐ F-10 (J4) `Pastel › Pastel Oil › Wax Pencil`: o giz de CERA — denso (0,75–0,86), a borda aos caroços (a orla
    // do dual, onda 0,045–0,049 no Rebelle) e o miolo manchado; sem estrias. O Oil Pastel sem a Shape em Flow, carga
    // 300, papel 0,4, o tecto 140.
    Receita {
        nome: "Wax Crayon",
        pigment: 300.0,
        gate: 0.4,
        size_px: None,
        extras: &[
            ("felt", 0.0),
            ("drag", 0.05),
            MAIS_PERTO,
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 3.5),
        ],
        pincel: &[
            ("jitter", "0.1"),
            ("shape", MANCHAS_DA_CERA),
            ("dual", ORLA_DA_CERA),
            ("size_max_px", "140"),
        ],
        nasce_de: Some("Oil Pastel"),
        ..OIL_PASTEL
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// O Cruzamento do Wax Scrape não entra (0,166 do controlo, abaixo de 3× o ruído desta régua); o raio do Wax por grelha.
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Charcoal", foto: "charcoal", raio: 10.5, metricas: &[(Cobertura, 0.384), (Borda, 0.378), (Orla, 15.0)] },
    Midia { preset: "Soft Pastel", foto: "pastel", raio: 8.5, metricas: &[(Cobertura, 0.752), (Borda, 0.443), (Orla, 6.0)] },
    Midia { preset: "Oil Pastel", foto: "oilpastel2", raio: 15.0, metricas: &[(Cobertura, 0.654), (Textura, 17.354), (Borda, 0.417), (Orla, 11.0)] },
    Midia { preset: "Square Hard", foto: "squarehard", raio: 15.50, metricas: &[(Cobertura, 0.705), (Textura, 20.884), (Borda, 0.856), (Cv, 0.077), (Cruzamento, 0.579), (Largura, 31.0)] },
    Midia { preset: "Wax Scrape", foto: "waxscrape", raio: 19.5, metricas: &[(Cobertura, 0.763), (Textura, 16.691), (Borda, 0.586), (Largura, 36.0)] },
];

const OIL_PASTEL: Receita = Receita {
    meio: "Pastel",
    grupo: "Oil",
    nome: "Oil Pastel",
    water: 0.0,
    opacity: 1.0,
    pigment: 160.0,
    pickup: 1.0,
    gate: 0.6,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.0,
    size_px: Some(77.0),
    extras: &[
        ("felt", 0.0),
        ("drag", 0.05),
        MAIS_PERTO,
        ("pressureSizeCurve", 0.5),
        ("pressureOpacity", 0.9),
        ("pressureOpacityCurve", 2.0),
    ],
    grao: Some(("Noise", 100.0)),
    achatado: None,
    pincel: &[("jitter", "0.1"), ("shape", ESTRIAS_DO_OLEO)],
    nasce_de: None,
};

const SOFT_PASTEL: Receita = Receita {
    meio: "Pastel",
    grupo: "Dry",
    nome: "Soft Pastel",
    water: 0.0,
    opacity: 1.0,
    pigment: 100.0,
    pickup: 1.0,
    gate: 0.4,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.2,
    size_px: Some(29.0),
    extras: &[
        ("felt", 0.0),
        ("drag", 0.0),
        MAIS_PERTO,
        ("pressureSizeCurve", 0.5),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 0.5),
    ],
    grao: Some(("Noise", 100.0)),
    achatado: None,
    pincel: &[],
    nasce_de: None,
};

// ---- Os portões da família (J4, `docs/Painter/Pinceis/exames/pastel_carvao.md`) ----

#[cfg(test)]
mod portoes {
    use crate::as_leis_da_escala::{cena, tamanhos};
    use crate::exame_bancada::{Fonte, Pincel, pinceis};

    /// A cobertura (0..1) de cada píxel da tela 1000 × 640 sobre o papel branco (`ferramentas/leis_escala.py`).
    fn cobertura(px: &[u8]) -> Vec<f64> {
        px.chunks_exact(4)
            .map(|c| (0..3).map(|k| 255.0 - f64::from(c[k])).fold(0.0, f64::max) / 255.0)
            .collect()
    }

    /// O pincel de fábrica `nome` com o campo `campo` do pincel (`brush.<campo>`) ou o knob (`engine`) trocado.
    fn mutado(nome: &str, campo: &str, valor: serde_json::Value) -> Pincel {
        let mut p = pinceis(nome).remove(0);
        if let Fonte::Preset { preset, .. } = &mut p.fonte {
            match campo.strip_prefix("brush.") {
                Some(c) => {
                    preset.parametros.brush.insert(c.to_owned(), valor);
                }
                None => {
                    preset.parametros.engine.insert(campo.to_owned(), valor.as_f64().unwrap());
                }
            }
        }
        p
    }

    /// As bordas (topo, fundo) de cada coluna de `x0..x1` no recto horizontal em `y` (±`h`), ao limiar ¼ do p90 dos picos.
    fn bordas(c: &[f64], y: usize, h: usize, x0: usize, x1: usize) -> Vec<Option<(usize, usize)>> {
        let col = |x: usize| -> Vec<f64> { (y - h..=y + h).map(|yy| c[yy * 1000 + x]).collect() };
        let mut p: Vec<f64> = (x0..x1).map(|x| col(x).into_iter().fold(0.0, f64::max)).collect();
        p.sort_by(f64::total_cmp);
        let limiar = 0.25 * p[(0.9 * (p.len() - 1) as f64) as usize];
        (x0..x1)
            .map(|x| {
                let v = col(x);
                Some((v.iter().position(|&q| q >= limiar)?, v.iter().rposition(|&q| q >= limiar)?))
            })
            .collect()
    }

    fn mediana(mut v: Vec<f64>) -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    /// ⭐ **As ESTRIAS** do recto horizontal em `y` (±`h`, colunas 200..800; `ferramentas/…/estrias.py` da família, ao
    /// mesmo método): a largura mediana, o contraste do miolo (desvio ÷ média, na faixa de ±0,3 da largura) e a
    /// ½-autocorrelação (circular, a do FFT do script) AO LONGO e ATRAVÉS — em px, o 1.º desfasamento abaixo de ½.
    pub(super) fn estrias(c: &[f64], y: usize, h: usize) -> (f64, f64, usize, usize) {
        let (x0, x1) = (200, 800);
        let b = bordas(c, y, h, x0, x1);
        let w = mediana(b.iter().flatten().map(|(t, f)| (f - t + 1) as f64).collect());
        let centro = mediana(b.iter().flatten().map(|(t, f)| (t + f) as f64 / 2.0).collect()) as usize + y - h;
        let hh = ((w * 0.3) as usize).max(1);
        let (n, m) = (x1 - x0, 2 * hh + 1);
        let mut z: Vec<f64> = (0..m)
            .flat_map(|r| (x0..x1).map(move |x| (centro - hh + r) * 1000 + x))
            .map(|i| c[i])
            .collect();
        let media = z.iter().sum::<f64>() / z.len() as f64;
        z.iter_mut().for_each(|v| *v -= media);
        let var: f64 = z.iter().map(|v| v * v).sum();
        let contraste = (var / z.len() as f64).sqrt() / media.max(1e-6);
        let ac = |dx: usize, dy: usize| -> f64 {
            let mut s = 0.0;
            for r in 0..m {
                for k in 0..n {
                    s += z[r * n + k] * z[((r + dy) % m) * n + (k + dx) % n];
                }
            }
            s / var.max(1e-12)
        };
        let ao_longo = (1..n).find(|&k| ac(k, 0) < 0.5).unwrap_or(n);
        let atraves = (1..m).find(|&k| ac(0, k) < 0.5).unwrap_or(m);
        (w, contraste, ao_longo, atraves)
    }

    /// A largura mediana e a cobertura média de cada uma das 10 faixas de 50 px da rampa 0 → 1 (cena `gestos`, y 400 ± 45,
    /// x 60..560, limiar 0,08 — `rampa.py` da família).
    fn rampa(c: &[f64]) -> Vec<(f64, f64)> {
        (0..10)
            .map(|b| {
                let (mut ws, mut cs) = (Vec::new(), Vec::new());
                for x in 60 + 50 * b..60 + 50 * (b + 1) {
                    let col: Vec<f64> = (355..=445).map(|y| c[y * 1000 + x]).collect();
                    match (col.iter().position(|&q| q >= 0.08), col.iter().rposition(|&q| q >= 0.08)) {
                        (Some(t), Some(f)) => {
                            ws.push((f - t + 1) as f64);
                            cs.push(col[t..=f].iter().sum::<f64>() / (f - t + 1) as f64);
                        }
                        _ => {
                            ws.push(0.0);
                            cs.push(0.0);
                        }
                    }
                }
                (mediana(ws), cs.iter().sum::<f64>() / cs.len() as f64)
            })
            .collect()
    }

    /// A LÂMINA: a largura do recto horizontal lento (y 40..141, x 80..560) e a da vertical lenta (x 640..761, y 470..591)
    /// da cena `gestos` (`lamina.py` da família) — a de pé ÷ a deitada.
    fn lamina(c: &[f64]) -> f64 {
        let largura = |amostras: Vec<Vec<f64>>| -> f64 {
            let mut p: Vec<f64> = amostras.iter().map(|v| v.iter().copied().fold(0.0, f64::max)).collect();
            p.sort_by(f64::total_cmp);
            let limiar = 0.25 * p[(0.9 * (p.len() - 1) as f64) as usize];
            mediana(
                amostras
                    .iter()
                    .filter_map(|v| {
                        Some((v.iter().rposition(|&q| q >= limiar)? - v.iter().position(|&q| q >= limiar)? + 1) as f64)
                    })
                    .collect(),
            )
        };
        let h = largura((80..560).map(|x| (40..141).map(|y| c[y * 1000 + x]).collect()).collect());
        let v = largura((470..591).map(|y| (640..761).map(|x| c[y * 1000 + x]).collect()).collect());
        v / h
    }

    /// A «onda» da borda do recto de 200 px (y 440 ± 170, colunas 200..800): o desvio das duas bordas ÷ a largura.
    fn onda(c: &[f64]) -> f64 {
        let b: Vec<(usize, usize)> = bordas(c, 440, 170, 200, 800).into_iter().flatten().collect();
        let w = mediana(b.iter().map(|(t, f)| (f - t + 1) as f64).collect());
        let sd = |v: Vec<f64>| {
            let m = v.iter().sum::<f64>() / v.len() as f64;
            (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
        };
        (sd(b.iter().map(|p| p.0 as f64).collect()) + sd(b.iter().map(|p| p.1 as f64).collect())) / 2.0 / w
    }

    /// As medidas de um pincel nas cenas `tamanhos` e `gestos`: as estrias a 40 e a 200, a rampa, a lâmina e a onda.
    struct Medidas {
        e40: (f64, f64, usize, usize),
        e200: (f64, f64, usize, usize),
        rampa: Vec<(f64, f64)>,
        lamina: f64,
        onda: f64,
    }

    fn mede(p: &Pincel) -> Medidas {
        let t = cobertura(&tamanhos(p));
        let g = cobertura(&cena(p, "gestos"));
        Medidas { e40: estrias(&t, 190, 60), e200: estrias(&t, 440, 170), rampa: rampa(&g), lamina: lamina(&g), onda: onda(&t) }
    }

    /// Mede vários pincéis em paralelo (cada um num fio).
    fn mede_todos(ps: Vec<Pincel>) -> Vec<Medidas> {
        std::thread::scope(|s| {
            let hs: Vec<_> = ps.iter().map(|p| s.spawn(move || mede(p))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        })
    }

    fn imprime(n: &str, m: &Medidas) {
        let f = |e: &(f64, f64, usize, usize)| format!("w {:.0} contr {:.2} ao longo {} através {}", e.0, e.1, e.2, e.3);
        let r: Vec<String> = m.rampa.iter().map(|(w, c)| format!("{w:.0}/{c:.2}")).collect();
        println!("[pastel_carvao] {n:<20} 40: {} | 200: {} | lâmina {:.2} · onda {:.3}", f(&m.e40), f(&m.e200), m.lamina, m.onda);
        println!("[pastel_carvao] {n:<20} rampa {}", r.join(" "));
    }

    /// As leis da IDENTIDADE de cada pincel da família, contra a foto do Rebelle (os números do Rebelle medidos pelos
    /// scripts da família, ao mesmo método, nas fotos `fixtures/rebelle_bateria/…_A.png`) — devolve as falhas.
    fn falhas(n: &str, m: &Medidas) -> Vec<String> {
        let mut f = Vec::new();
        let mut exige = |ok: bool, o: String| {
            if !ok {
                f.push(format!("{n}: {o}"));
            }
        };
        match n {
            // Oil Pastel 2: a 200 px ½-autocorrelação 600 px ao longo × 6 através, contraste 0,23 (o Soft Pastel 1 × 1).
            "Oil Pastel" => {
                exige(m.e200.2 >= 30 && m.e200.3 <= 8, format!("as estrias: {} × {} px (Rebelle 600 × 6)", m.e200.2, m.e200.3));
                exige((0.12..=0.35).contains(&m.e200.1), format!("o contraste das estrias {:.2} (Rebelle 0,23)", m.e200.1));
            }
            // Wax Scrape: os traços curtos a 200 px — 4 × 2 px, contraste 0,29 (o Soft Pastel 0,15 e liso).
            "Wax Scrape" => {
                exige((3..=40).contains(&m.e200.2) && m.e200.3 <= 4, format!("os traços: {} × {} px (Rebelle 4 × 2)", m.e200.2, m.e200.3));
                exige(m.e200.1 >= 0.22, format!("o contraste dos traços {:.2} (Rebelle 0,29)", m.e200.1));
            }
            // Charcoal: o tecto (73–76 px a 200) e o contraste do miolo 0,17–0,18.
            "Charcoal" => {
                exige((m.e200.0 - 75.0).abs() <= 8.0, format!("o tecto: {:.0} px a 200 (Rebelle 74–76)", m.e200.0));
                exige((0.10..=0.26).contains(&m.e40.1), format!("o contraste {:.2} (Rebelle 0,18)", m.e40.1));
            }
            // Square Hard: a largura não muda com a pressão (do 6.º décimo ao fim 31 → 33 px no Rebelle; antes, a pressão
            // leve desfaz-se em grão e a largura medida cai) e só escurece (cobertura 0,07 → 0,63).
            "Square Hard" => {
                let (w_fim, c_fim) = m.rampa[9];
                let varia = m.rampa[5..].iter().map(|(w, _)| (w - w_fim).abs() / w_fim).fold(0.0, f64::max);
                exige(varia <= 0.10, format!("a largura varia {:.0} % na rampa (Rebelle ≤ 6 % desde o 6.º décimo)", 100.0 * varia));
                exige(m.rampa[3].1 <= 0.5 * c_fim, format!("a pressão não escurece: {:.2} → {c_fim:.2}", m.rampa[3].1));
            }
            _ => {}
        }
        f
    }

    const FAMILIA: [&str; 5] = ["Charcoal", "Soft Pastel", "Square Hard", "Oil Pastel", "Wax Scrape"];

    /// ⭐⭐ **Cada pastel e carvão se reconhece pela foto** (J4): a IDENTIDADE de cada um contra o seu Rebelle — as estrias
    /// do óleo, os traços da cera, o tecto do carvão, o rectângulo do Square Hard — e o controlo (o Soft Pastel: sem
    /// estrias nem lâmina).
    #[test]
    fn cada_pastel_e_carvao_se_reconhece_pela_foto() {
        let presentes: Vec<&str> = FAMILIA.to_vec();
        let m = mede_todos(presentes.iter().map(|n| pinceis(n).remove(0)).collect());
        let mut f = Vec::new();
        for (n, m) in presentes.iter().zip(&m) {
            imprime(n, m);
            f.extend(falhas(n, m));
        }
        let sp = &m[presentes.iter().position(|&n| n == "Soft Pastel").unwrap()];
        if sp.e200.2 > 5 || (sp.lamina - 1.0).abs() > 0.15 {
            f.push(format!("o controlo Soft Pastel tem estrias ({}) ou lâmina ({:.2})", sp.e200.2, sp.lamina));
        }
        assert!(f.is_empty(), "{f:#?}");
    }

    /// A mutação do portão de cima, por dentro: cada identidade sem o valor que a faz TEM de reprovar — o óleo sem a
    /// Shape, a cera sem os traços, o carvão sem o tecto, o Square Hard com a pressão no tamanho.
    #[test]
    fn sem_o_que_faz_cada_identidade_o_portao_reprova() {
        let nada = serde_json::json!({"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane",
            "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0],
            "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]});
        let casos: Vec<(&str, &str, serde_json::Value)> = vec![
            ("Oil Pastel", "brush.shape", nada.clone()),
            ("Wax Scrape", "brush.shape", nada.clone()),
            ("Charcoal", "brush.size_max_px", serde_json::Value::Null),
            ("Square Hard", "pressureSize", serde_json::json!(0.9)),
        ];
        let m = mede_todos(casos.iter().map(|(n, c, v)| mutado(n, c, v.clone())).collect());
        let mut sobreviveu = Vec::new();
        for ((n, c, _), m) in casos.iter().zip(&m) {
            imprime(&format!("{n} sem {c}"), m);
            if falhas(n, m).is_empty() {
                sobreviveu.push(format!("{n} sem {c}"));
            }
        }
        assert!(sobreviveu.is_empty(), "a mutação sobreviveu (o portão não mede a identidade): {sobreviveu:?}");
    }

    /// O GRÃO da pressão leve: no 4.º–6.º décimo da rampa 0 → 1 (x 210..360, y 380..420 — a pressão 0,3–0,6), o p90 da
    /// cobertura ÷ a média, só onde há tinta (> 0,02). Esmaecer liso dá ~1,5; os pontos soltos do grão, bem mais.
    fn grao_da_rampa(c: &[f64]) -> f64 {
        let mut v: Vec<f64> = (380..420)
            .flat_map(|y| (210..360).map(move |x| (x, y)))
            .map(|(x, y)| c[y * 1000 + x])
            .filter(|&q| q > 0.02)
            .collect();
        v.sort_by(f64::total_cmp);
        let media = v.iter().sum::<f64>() / v.len().max(1) as f64;
        v.get((0.9 * v.len() as f64) as usize).copied().unwrap_or(0.0) / media.max(1e-9)
    }

    /// ⭐⭐ **A pressão leve DESFAZ o traço em grão** (`Knob::PressureGrain`, J4): a parte da opacidade da pressão que
    /// entra ANTES do portão do papel — com pouca tinta só os picos do dente a vencem. No Square Hard (0,25) o grão da
    /// rampa leve sobe sobre o MESMO preset sem a lei (`pressureGrain` 0 = a lei de sempre, ao bit): a revisão cega r1
    /// viu o Rebelle em pontos soltos onde o nosso esmaecia liso. (Sem portão do papel — `paperGate` 0, o Charcoal — a
    /// lei não tem dente onde morder: medido 1,72 · 1,72.)
    #[test]
    fn a_pressao_leve_desfaz_o_traco_em_grao() {
        let nomes = ["Square Hard"];
        let mut ps: Vec<Pincel> = nomes.iter().map(|n| pinceis(n).remove(0)).collect();
        ps.extend(nomes.iter().map(|n| mutado(n, "pressureGrain", serde_json::json!(0.0))));
        let g: Vec<f64> = std::thread::scope(|s| {
            let hs: Vec<_> = ps.iter().map(|p| s.spawn(move || grao_da_rampa(&cobertura(&cena(p, "gestos"))))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut f = Vec::new();
        for (i, n) in nomes.iter().enumerate() {
            let (com, sem) = (g[i], g[i + nomes.len()]);
            println!("[pastel_carvao] {n:<12} grão da rampa leve: com a lei {com:.2} · sem {sem:.2}");
            // Medido: 1,81 com a lei (0,25) contra 1,53 sem ela (0,5 dava 1,97); sem a lei as duas são iguais ao bit.
            if com < 1.12 * sem {
                f.push(format!("{n}: a pressão leve não se desfaz em grão ({com:.2} contra {sem:.2} sem a lei)"));
            }
        }
        assert!(f.is_empty(), "{f:?}");
    }

    /// Os reprovados não estão na fábrica: nenhum ficheiro com o nome de uma receita [`super::EM_ESTUDO`].
    #[test]
    fn os_reprovados_nao_estao_na_fabrica() {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
        let mut achados = Vec::new();
        let mut pilha = vec![raiz];
        while let Some(d) = pilha.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    pilha.push(p);
                } else if let Some(n) = p.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".ph2d-brush.png")) {
                    if super::EM_ESTUDO.iter().any(|r| r.nome == n) {
                        achados.push(p.display().to_string());
                    }
                }
            }
        }
        assert!(achados.is_empty(), "um reprovado está na fábrica: {achados:?}");
    }
}
