//! **Família `oleo_acrilica`** — J4, catálogo G (óleo, acrílica): Oil Bristle, Express Oil, Dabber, Filbert, Flat Bristle.
//! A Acrylic Round saiu na 2.ª ronda: fundida no Hard Round (`exames/oleo_acrilica.md`, G-2).
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

/// Os três óleos foram afinados a Spacing 0,01 (o pigmento por dab deles é o de 0,01): o pigmento por comprimento
/// mantém a tinta do Express Oil com os dabs mais esparsos.
const PASSO_DOS_OLEOS: f64 = 0.01;

// ⭐ O 2.º lote (11/10, depois da reavaliação pedida pelo dono — 18 §3: pigmento alto, o relevo de volta
// e o grão no lugar das cerdas). Cada um medido contra a foto do seu preset-oráculo do Rebelle, seco, à
// largura dela; o tamanho é o do preset dele. `cada_preset_parece_a_sua_midia` guarda-os.
pub const WET: &[Receita] = &[
    // Oil & Acrylic › Bristle Flat: corpo cheio, estrias das cerdas, relevo.
    OIL_BRISTLE,
    // Express Oil › Hairy: claro e liso, tufos (prompt 23, aceite do dono 12/10). A claridade é POUCO
    // pigmento — com o Spacing 0,01 cada célula leva ~100 carimbos, por isso 60 e não 250; sem água (a água
    // assentava o pouco pigmento nos poros: grão 0,065 → 0,027); cerdas 4 = tufos sem sal.
    Receita {
        meio: "ExpressOil",
        grupo: "Round",
        nome: "Express Oil",
        water: 0.0,
        opacity: 0.6,
        // J4 (`exames/oleo_acrilica.md`): os TUFOS do Hairy do Rebelle (4–5 faixas cheias com papel entre elas, contra as
        // faixas macias e juntas de antes): cerdas 7 × 300 e a faixa de 4 px; o pigmento 60 → 75 (as faixas saturam). A 90 e
        // a 100 (os revisores viam o nosso mais translúcido sobre o vermelho) o raio 7 pinta 17 px (pedido 14 ± 2,
        // `a_largura_e_a_prometida`) — fica 75, e a Tinta da foto antiga sai do `MIDIAS` (abaixo).
        pigment: 75.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 7.0,
        bristle_count: 300.0,
        hardness: 0.9,
        size_px: Some(64.0),
        // Os FIOS (W2, doc 48_medicao §3): as faixas do Hairy do Rebelle são quase CONSTANTES pelo traço (carris
        // 0,85) e a textura dele é o contraste entre elas ⇒ a pressão de cada fio quase constante (ruído 2000 px)
        // e forte, e a mão leve que levanta os fracos. Textura 3,2 → 7,9 (9,0), CV 0,009 (0,008), carris 0,88,
        // leve 11,5 (10,2), forte 7,1 (7,3); o período 47 % a 6 px do carimbo dele não se copia (03 §3.1).
        // ⛔ Sem o relevo de cada fio (W3, doc 49_medicao): o controlo 7,94 não se distingue do Rebelle 9,0 (o ruído
        // da foto, recto × rápido, é 0,9) — 1 px por ganho dava 8,79, dentro do ruído.
        extras: &[
            ("felt", 0.0),
            ("bristleStrength", 2.0),
            ("emboss", 0.2),
            ("bristlePressure", 2.5),
            ("bristleLaneWidth", 4.0),
            ("bristleLift", 0.2),
            ("bristleNoiseLength", 2000.0),
            ("pigmentSpacing", PASSO_DOS_OLEOS),
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // `dabberc_032` (prompt 53): o Oil Bristle de cerdas largas, mais relevo e a ponta espalhada (jitter) — a esponja.
    // L13 (J2, 20/10): a esponja é REDONDA (o vertical do Rebelle tem a largura do deitado: 38/38) e morde o papel; a
    // lâmina de pé do Oil Bristle espalhada pelo Jitter era o «rasgado» (rugosidade 2,97 × 1,05). Grelha flatten 0 ×
    // jitter 0–0,5 × paperGate 0–1 à largura do Rebelle: 0,4 · 1 = 390/420 (era 346), rugosidade 0 ✗ (eram 20).
    Receita {
        grupo: "Dabber",
        nome: "Dabber",
        bristle_size: 2.0,
        hardness: 0.6,
        size_px: Some(51.0),
        gate: 1.0,
        achatado: Some((0.0, 90)),
        extras: &[
            ("felt", 0.05),
            ("bristleStrength", 2.0),
            // J4 (`exames/oleo_acrilica.md`, contra a bateria do Rebelle `Dabber/Dabber`): o relevo 1,2 → 0,5 e o claro por
            // fio 0,55 → 0,1 — o nosso saía às manchas claras e escuras onde o dele é um granulado fino e igual (régua
            // 182 → 192/227 com o jitter 0,3).
            ("emboss", 0.5),
            ("bristlePressure", 1.2),
            ("bristleLaneWidth", 1.5),
            ("bristleLift", 0.4),
            ("bristleNoiseSat", 0.3),
            // O grão claro da esponja: as duas fotos do Rebelle cercam-no (a da bateria: textura do recto ≤ ~11, 0,6 dá
            // 11,3 ✗; a do 52 seca: ≥ 9,6, o vale do Wet Round) — 0,55 dá 10,9 e 9,67 (0,25: 9,9 e 8,1 ✗).
            ("bristleNoiseLight", 0.1),
            ("bristleHeight", 0.75),
            ("pigmentSpacing", 0.0),
            ("capsuleMaxPx", 2.0),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 3.0),
        ],
        pincel: &[("jitter", "0.3")],
        nasce_de: Some("Oil Bristle"),
        ..OIL_BRISTLE
    },
    FILBERT,
];

/// Os REPROVADOS da J4 (`docs/Painter/Pinceis/exames/oleo_acrilica.md`): a receita fica para quem os retomar, sem ficheiro
/// na fábrica (`os_reprovados_do_oleo_nao_estao_na_fabrica`).
/// - G-6 Flat Bristle (2.ª ronda: o grão era a capilaridade — `capillary` 0 dá a risca lisa; o 5 px esmaece pela lei
///   `thinStrokeFade`; régua 367/454, o 1.º dos 228 do Rebelle): o revisor cego achou o Rebelle nas 6 linhas (baixa) e
///   lista o vermelho da base opaco e saturado (cobertura 0,92 contra 0,72 — o vermelho do K–M a 200 de pigmento cobre
///   mais que o azul; não é do preset) e o portão `a_pressao_de_cada_preset_e_a_da_sua_origem` dá 0,202 (barra 0,18): a
///   entrada tardia do Rebelle (nada abaixo de p 0,2) só se copia com um piso, que a decisão 9 proíbe. O ficheiro gerado
///   vive em `fixtures/presets_teste/` ([`FLAT_EM_ESTUDO`]) para os portões da família. J4 3.ª ronda (`exames/aguarela.md`):
///   com a decisão 15 na lei a rampa lê 0,130 (passa); o revisor cego novo achou o Rebelle nas 6 linhas (baixa) e REPETE
///   o vermelho opaco, os nós escuros onde o azul cruza o azul e as pontas desfiadas contra as redondas — fica em estudo.
/// - G-9 Rake: régua 390/454, a 0,83 do Filbert; o 1.º revisor escolheu o nosso como original (baixa), o 2.º achou o
///   Rebelle nas 6 linhas (média) e os dois listam as PONTAS (as do Rebelle cortadas a direito, as nossas desfiadas
///   cerda a cerda) e, a 200 px, as ~15 riscas escuras espaçadas dele contra o nosso campo denso de estrias.
pub const EM_ESTUDO: &[Receita] = &[FLAT_BRISTLE, RAKE];

/// **G-9 Rake** (J4, contra o Rebelle `Oil/Rake/Rake Sharp`, 108 px): o redondo que SEGUE o traço (38 px em qualquer
/// direcção) com metade das faixas de cerdas tiradas (`bristleRemove` 0,5, faixas de 2 px) — riscas paralelas nítidas
/// com papel entre elas, sem o claro sorteado; a largura responde à pressão (Rebelle 0,7).
const RAKE: Receita = Receita {
    grupo: "Bristle",
    nome: "Rake",
    pigment: 800.0,
    size_px: Some(108.0),
    achatado: Some((0.0, 90)),
    extras: &[
        ("felt", 0.05),
        ("bristleStrength", 2.0),
        ("bristlePressure", 1.2),
        ("bristleLaneWidth", 2.0),
        ("bristleRemove", 0.5),
        ("bristleLift", 0.0),
        ("bristleNoiseSat", 0.3),
        ("bristleNoiseLight", 0.0),
        ("bristleHeight", 0.75),
        ("pigmentSpacing", 0.0),
        ("capsuleMaxPx", 2.0),
        ("pressureSize", 0.7),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        ("emboss", 0.7),
        ("embossSizeExp", 0.4),
        ("embossSizeRef", 41.0),
        ("levelingOwnStroke", 1.0),
        ("bristleTipRagged", 0.8),
    ],
    pincel: &[("dab_quina", "0.0")],
    nasce_de: Some("Oil Bristle"),
    ..OIL_BRISTLE
};

/// **G-6 Flat Bristle** (J4, contra o Rebelle `Oil/Flat/Flat`, 78 px): a MESMA lâmina fixa do Oil Bristle (Rebelle α 84°,
/// 39 × 8,5 px a 40 — `06_LEIS_DAS_LACUNAS` §1) mas a tinta RALA e translúcida (a cobertura 0,67 contra 0,91 do Bristle
/// Flat): pouco pigmento (200), os fios macios (força 1, sem o sorteio de cor), o relevo baixo, um feltro que fecha as
/// falhas; a largura não responde à pressão (Rebelle `pressure_size_influence` 0), a carga sim.
const FLAT_BRISTLE: Receita = Receita {
    grupo: "Flat",
    nome: "Flat Bristle",
    pigment: 200.0,
    size_px: Some(78.0),
    achatado: Some((0.73, 90)),
    extras: &[
        // 2.ª ronda (`exames/oleo_acrilica.md`): o GRÃO sal-e-pimenta da 1.ª era a CAPILARIDADE — o pouco pigmento que ela
        // espalha pelas fibras do papel, píxel a píxel (medido: o mesmo traço com `capillary` 0 é a risca lisa translúcida
        // do Rebelle; o feltro, as fibras, o sulco, a água por dab, o pickup e o papel visível não mexiam no grão). A tinta
        // grossa satura e esconde-o; a rala mostra-o. Sem motor novo: a composição já o exprime.
        ("capillary", 0.0),
        ("felt", 0.1),
        ("bristleStrength", 1.0),
        // A carga de cada fio quase constante pelo traço (ruído 2000 px, força 0,8): as riscas do Rebelle não mudam ao
        // longo de 800 px; com 150 px o nosso saía às manchas claras.
        ("bristlePressure", 0.8),
        ("bristleNoiseLength", 2000.0),
        ("bristleLaneWidth", 1.5),
        ("bristleLift", 0.4),
        ("bristleNoiseSat", 0.0),
        ("bristleNoiseLight", 0.0),
        ("bristleHeight", 0.75),
        ("pigmentSpacing", 0.0),
        // Sem o tecto de 2 px do sub-passo da cápsula do Oil Bristle: com ele a 200 px cada célula levava 5× os sub-dabs
        // da de 40 e a tinta rala saturava (a prancha: o 200 px do Rebelle é tão translúcido como o de 40).
        ("capsuleMaxPx", 0.0),
        ("pressureSize", 0.0),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        ("emboss", 0.2),
        ("embossSizeExp", 0.4),
        ("embossSizeRef", 41.0),
        ("levelingOwnStroke", 1.0),
        // A ponta do Rebelle é um corte quase recto com uma franja curta (~20 px a 200): 0,8 desfiava 60 px.
        ("bristleTipRagged", 0.3),
        // ⭐ 2.ª ronda (`Knob::ThinStrokeFade`): o chato pequeno do Rebelle quase não pinta — o recto de 5 px cobre 0,143
        // (0,22 do de 40); o nosso 0,683. γ 0,6 · 0,9 · 1,2 · 1,6 → 0,328 · 0,201 · 0,118 · 0,035; o de 40 px não muda.
        ("thinStrokeFade", 1.05),
        ("thinStrokeRef", 40.0),
    ],
    nasce_de: Some("Oil Bristle"),
    ..OIL_BRISTLE
};

/// **G-7 Palette Knife** (J4, contra o Rebelle `Oil/Knife/Knife Oily`, 64 px): a lâmina FIXA na tela no eixo do preset
/// (Rebelle PEN_TILT, α 172°, 33 × 10 px a 40: fina no horizontal, larga no vertical — o P53 errara o eixo, 0°) e a
/// carga em DEGRADÊ ao longo da lâmina (um lado cheio, o outro a esbater em estrias — a forma `Gradient` no referencial
/// do dab); dabs a 0,15 que acumulam; a largura pela pressão (Rebelle `pressure_size_influence` 1). No Digital: a Wet
/// não lê a forma de degradê (medido: a mesma pintura com e sem ela).
const PALETTE_KNIFE: ReceitaMeio = ReceitaMeio {
    pincel: &[
        ("hardness", "0.8"),
        ("borda_px", "1.0"),
        ("dab_flatten", "0.69"),
        ("dab_angle_deg", "172"),
        ("dab_quina", "0.9"),
        ("accumulate", "true"),
        // 0,15 → 0,11: o revisor cego r1 via o nosso «mais opaco e saturado em tudo» (a cobertura do recto 0,85 contra
        // 0,77 do Rebelle).
        ("flow", "0.11"),
        ("continuous_spacing", "true"),
        ("sub_passo_fino", "true"),
        ("shape", CARGA_DA_LAMINA),
    ],
    mao: &[("pressureSize", 1.0)],
    size_px: Some(64.0),
    opacity: Some(1.0),
    ..ReceitaMeio::digital("Oil", "Knife", "Palette Knife")
};

/// A carga da lâmina: UMA rampa linear de um lado ao outro do dab (`Gradient`, tom neutro, sem repetição; o desvio e a
/// escala 0,5 põem a rampa uma vez na pegada — a 0 / 1 o referencial −1..1 dava DUAS rampas e o vertical saía oco).
const CARGA_DA_LAMINA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Gradient", "mapping": "ViewPlane", "offset": [0.5, 0.5], "params": [0.5, 0.5, 0.0, 0.0, 0.5, 0.5], "rake": false, "size": [0.5, 0.5], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// **G-5 Filbert** (J4, refeito contra o Rebelle `Oil/Flat/Filbert`, 64 px; `exames/oleo_acrilica.md`): o chato de ponta
/// oval que SEGUE o traço (Rebelle FOLLOW_TRAJECTORY: 40 px em qualquer direcção, o vertical também) — a pegada redonda,
/// sem a quina; os fios do óleo com as pontas DESFIADAS em dedos (o ρ 1,6, o dobro do Oil Bristle) e a tinta mais rala
/// que satura menos (pigmento 250: a cobertura 0,85 do Rebelle); a mão não levanta os fios (a largura quase não muda
/// com a pressão, a carga sim — a lei do dono).
const FILBERT: Receita = Receita {
    grupo: "Flat",
    nome: "Filbert",
    pigment: 250.0,
    size_px: Some(64.0),
    achatado: Some((0.0, 90)),
    extras: &[
        ("felt", 0.05),
        ("bristleStrength", 2.0),
        ("bristlePressure", 1.2),
        // As FAIXAS do Filbert do Rebelle são largas (~6 a 40 px, ~12 a 200 px, com linhas claras entre elas): 4 px (os dois
        // revisores cegos viam o nosso «centenas de cerdas finas» contra as faixas dele).
        ("bristleLaneWidth", 4.0),
        ("bristleLift", 0.0),
        ("bristleNoiseSat", 0.3),
        ("bristleNoiseLight", 0.0),
        ("bristleHeight", 0.75),
        ("pigmentSpacing", 0.0),
        // 4 px (as faixas): a 2 px a pegada REDONDA de 200 px pagava ~2,5× o Oil Bristle (p95 71 × 28 ms, load 22).
        ("capsuleMaxPx", 4.0),
        ("pressureSize", 0.2),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        ("emboss", 0.4),
        ("embossSizeExp", 0.4),
        ("embossSizeRef", 41.0),
        ("levelingOwnStroke", 1.0),
        ("bristleTipRagged", 1.6),
        // ⛔ Sem Glaze: o revisor cego r1 via o azul do Rebelle «violeta» sobre o vermelho, mas a régua mede-o a COBRIR
        // (cor em cima (32,97,172), vermelho 0,05); o Glaze 0,4 escurecia o cruzamento a (30,66,117) e a mistura da matriz
        // ia a 1,96 (a origem 33.º → 29.º, a régua 377 → 333/454). A 0,1 o azul sobre a faixa SECA saía baço, às riscas de
        // vermelho (47,94,160; Rebelle (28,99,175) — ele cobre). Explícito a 0: o gerador parte do ficheiro.
        ("glaze", 0.0),
    ],
    pincel: &[("dab_quina", "0.0")],
    nasce_de: Some("Oil Bristle"),
    ..OIL_BRISTLE
};

pub const MEIOS: &[ReceitaMeio] = &[];

/// - G-10 Thick Impasto: o relevo GRANULOSO do meio Impasto (grumos, cabeças redondas com sombra) é outro pincel que o
///   `Flat/Thick` do Rebelle (riscas e fiapos com pouco relevo) e que o `ThickGloss` do ArtRage (o tubo liso e brilhante
///   com estrias): os revisores escolheram o nosso como «a tinta grossa de verdade» mas listaram o relevo, as pontas e o
///   cruzamento como diferenças (média). Régua 370/454 (Rebelle) · 166/227 (ArtRage).
/// - G-7 Palette Knife: a lâmina no eixo certo (α 172°) e a carga em degradê dão o Knife Oily como o 1.º dos 228 do
///   Rebelle (d 1,03), mas os dois revisores (r1 baixa, r2 média, ambos acharam o Rebelle) repetem: o dele mostra os
///   CARIMBOS (escamas em arco a 200 px, contas a 5 px, a borda mole esfiapada) e corta em bico/cunha diagonal; o
///   nosso é um degradê liso de pontas rectas.
pub const EM_ESTUDO_MEIOS: &[ReceitaMeio] = &[THICK_IMPASTO, PALETTE_KNIFE];

/// **G-10 Thick Impasto** (J4, contra o Rebelle `Oil/Flat/Thick`, 64 px; no meio IMPASTO — a tinta tem altura): a
/// pegada redonda que segue o traço, a orla sem dureza e um pouco espalhada (jitter 0,1), o relevo baixo (0,3) e
/// GRANULOSO — a altura vem do grão (`Clouds`, a fonte `Grain`), os grumos da tinta grossa em vez do tubo liso do
/// exemplo da J2; o corpo 0,3. A carga pela mão (pressão → opacidade 0,4, a do exemplo).
const THICK_IMPASTO: ReceitaMeio = ReceitaMeio {
    pincel: &[
        ("dab_flatten", "0.0"),
        ("hardness", "0.0"),
        ("jitter", "0.1"),
        ("impasto_depth", "0.3"),
        ("impasto_body", "0.3"),
        ("impasto_shine", "0.5"),
        ("impasto_source", "\"Grain\""),
        ("texture", GRUMOS),
    ],
    mao: &[("pressureOpacity", 0.4)],
    size_px: Some(64.0),
    opacity: Some(1.0),
    ..ReceitaMeio::impasto("Oil", "Flat", "Thick Impasto")
};

/// Os grumos da tinta grossa: nuvens (`Clouds`) presas à tela a 24 px, tom neutro.
const GRUMOS: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Clouds", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [24.0, 24.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

// ⚠️ Os três do óleo levam os valores que o DONO afinou a olho contra o Rebelle (11/10, «vamos deixar esses
// valores»); a régua reprovava Textura do Oil Bristle (8,46 contra 5,86) e da Acrylic Round (30,4 contra 3,13)
// e a Borda do Express Oil (0,66 contra 0,80) — saíram daqui, a régua não mede o que o olho dele vê (18 §2).
// O Chato e a Tinta (prompt 23, aceites 12/10) são os dois pontos que o dono VIU: a mutação «preset de antes»
// sangra cada um sozinho (Oil 0,950 · Express 0,906).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Oil Bristle", foto: "oil", raio: 20.5, metricas: &[(Cobertura, 0.909), (Borda, 0.872), (Chato, 0.317)] },
    // J4: sem a Tinta da foto `express` do prompt 23 (0,772): os tufos curados (`exames/oleo_acrilica.md`) dão 0,666 com o
    // pigmento 75 — o recto da bateria do EXAME (Rebelle `Round/Hairy`, três cenas) está dentro do ruído na opacidade, e o
    // pigmento que a devolvia (≥ 100) partia a largura prometida a raio 7. A identidade dos tufos é o portão da família.
    Midia { preset: "Express Oil", foto: "express", raio: 21.0, metricas: &[(Cobertura, 0.700)] },
    // J4: sem a Textura da foto `dabber` do 52 (10,7): a bateria do EXAME (Rebelle `Dabber/Dabber`, três cenas) quer o
    // granulado fino e igual — o claro por fio 0,55 → 0,1 deixa a textura em 6,6 (régua 368 → 388/454, origem 12.º → 8.º).
    Midia { preset: "Dabber", foto: "dabber", raio: 18.5, metricas: &[(Cobertura, 0.840), (Borda, 0.588), (Orla, 5.0), (Largura, 36.0)] },
];

const OIL_BRISTLE: Receita = Receita {
    meio: "Oil",
    grupo: "Bristle",
    nome: "Oil Bristle",
    water: 0.1,
    opacity: 1.0,
    pigment: 2000.0,
    pickup: 1.0,
    gate: 0.0,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 1.0,
    bristle_count: 2000.0,
    hardness: 0.9,
    size_px: Some(103.0),
    // O chato (prompt 23, aceite do dono 12/10): lâmina de pé, 0,68 = 1 − 13/41 (o traço de pé e o
    // deitado da foto); o feltro fecha as falhas que a lâmina fina abria. Régua: `mede_familia_c.py`.
    // Os FIOS (W2, doc 48_medicao §3): cada fio claro ou escuro pela cor (o óleo satura: a pressão só abriria
    // falhas), a pressão própria e a mão leve que os levanta. Textura 2,75 → 3,85 (Rebelle 5,86: o resto são
    // as sombras do relevo de cada fio, W3), carris 0,00 → 3,03 (3,32), CV 0,017, período 2 %.
    // O RELEVO de cada fio sem escada (W3b, doc 50_medicao): a altura contínua através dos fios, acesa pelo
    // MESMO emboss cinza da tinta (o de 18/10 — degrau por fio, luz do Impasto, 1,25 px + emboss 1 — saiu pelo
    // smoke do dono: «muito pixelado»). Textura 3,85 → 5,65 (Rebelle 5,86), contraste 8,7 → 13,4 (12,1), a
    // escada no inclinado 2,2 · 1,9 (2,5 · 2,6); a 103 px 1,9 · 1,4 (2,5 · 2,3).
    extras: &[
        ("felt", 0.05),
        ("bristleStrength", 2.0),
        ("bristlePressure", 1.2),
        ("bristleLaneWidth", 1.5),
        ("bristleLift", 0.4),
        ("bristleNoiseSat", 0.3),
        // J4 (`exames/oleo_acrilica.md`): o claro sorteado por fio 0,25 → 0,1 — o revisor cego via o nosso «mais
        // luminoso e manchado» e a régua a cor do vertical fora (régua 205 → 211/227 com o emboss 0,8); a 0 o através a
        // 200 px caía a 8,7 (`o_relevo_nao_cresce_com_o_pincel`: Rebelle 11,5 ± 2).
        ("bristleNoiseLight", 0.1),
        // J4: os fios que COMEÇAM e ACABAM a meio do traço (o «vão» sorteado de cada fio a cada ~40 px) — dois revisores
        // cegos (r2, r3) viam o nosso «mecânico»: riscas contínuas e paralelas onde as do Rebelle aparecem e somem.
        ("bristleSpan", 1.0),
        ("bristleSpanLength", 40.0),
        // ⭐ 2.ª ronda: o VEIO (`Knob::BristleStreak`) — o «pente regular demais» que três revisores repetiram: a troços raros
        // um fio pesa (quase preto) ou falta. Medido no recto de 200 px (`os_fios_do_oil_bristle_aparecem_e_somem`): escuros
        // 0,0144 · claros 0,0039 da área (Rebelle 0,0154 · 0,0074), corridas de 25 px (Rebelle 84: o pontilhado do relevo
        // parte-as). ⛔ A água por dab 0,5 dava o contraste do Rebelle aos troços (~42 contra ~31) mas subia a textura e o
        // relevo do recto (6,87 · emboss 14,2 contra 6,39 · 11,4) para fora do vale dos portões do relevo — fica a 1.
        ("bristleStreak", 0.9),
        ("bristleStreakLight", 0.7),
        ("bristleStreakLength", 2000.0),
        ("bristleStreakCut", 0.35),
        ("waterPerDab", 1.0),
        ("bristleHeight", 0.75),
        // O pigmento por DAB (doc 51 §3): ele satura (a tinta a 103 px 0,985 da de antes sem o peso) e o peso só lhe
        // dava degraus — a escada da curva 2,57 · 2,70 (tecto 2,50 · 2,55) contra 2,15 · 2,40 sem ele.
        ("pigmentSpacing", 0.0),
        // Os fios dele têm largura em px: a 103 px a escada lê 2,24 · 2,45 · 2,15 com a cápsula a 2 px, 3,06 a 2,06
        // e 4,71 a 3 (tecto 2,78 · 2,60 · 2,50; doc 51 §3).
        ("capsuleMaxPx", 2.0),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        // As leis de escala do Wet (J2, 03_MOTORES §7 c/d; `as_leis_da_escala`): o relevo pelo diâmetro (o Bristle Flat
        // do Rebelle 15,6 → 11,3 de 41 a 202 px; o nosso crescia 15 → 17), a tinta que não escoa para a faixa molhada
        // (a lente 19/12 → 12/12) e a ponta recta e desfiada de cada fio (6,5 px a 200 no Rebelle).
        // O expoente 0,4: o através 14 → 11 de 41 a 202 px (o Rebelle 15,4 → 11,5); a ponta ρ 0,8: desfiado ~6 px a 200.
        // J4: 0,7 → 0,8 com o claro por fio a 0,1 — o através a 200 px volta a 10 (Rebelle 11,5 ± 2; a 0,7 caía a 9,2).
        ("emboss", 0.6),
        ("embossSizeExp", 0.5),
        ("embossSizeRef", 41.0),
        ("levelingOwnStroke", 1.0),
        ("bristleTipRagged", 0.8),
    ],
    grao: None,
    achatado: Some((0.68, 90)),
    // A lâmina é um RECTÂNGULO na pegada (a quina 1: o corte recto a toda a largura); o desfiado é a lei das cerdas
    // (`bristleTipRagged`, J2 §7 c).
    pincel: &[("dab_quina", "1.0")],
    nasce_de: None,
};

/// As medidas de identidade de um óleo na bateria do EXAME (`ferramentas/bateria.json`, cena `gestos`, foto A, escala 1):
/// o que a prancha de cada pincel mostra contra o oráculo e contra o vizinho, num número (J4, `exames/oleo_acrilica.md`).
#[derive(Debug, Clone, Copy)]
struct Oleo {
    /// A largura do vertical lento ÷ a do recto lento: < 1 a lâmina de pé, ~1 o redondo, > 1 a lâmina deitada.
    de_pe: f64,
    /// A tinta (escuridão 0..1) mediana no miolo do recto lento: o corpo carregado ou ralo.
    miolo: f64,
    /// As ESTRIAS: o desvio-padrão do perfil médio através do miolo do recto (linhas y 78..103), ÷ a média — as faixas
    /// de cerdas (o redondo liso ~0).
    estria: f64,
    /// A borda RASGADA: o desvio-padrão da linha de cima do recto ao longo de x (px) — a esponja espalhada.
    rasgo: f64,
}

/// A escuridão de um píxel (1 − luminância): 0 o papel.
fn escuro(px: &[u8], w: usize, x: usize, y: usize) -> f64 {
    let p = &px[(y * w + x) * 4..];
    1.0 - (0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])) / 255.0
}

fn mede_oleo(nome: &str) -> Oleo {
    use crate::exame_bancada::{bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap()
        .clone();
    let p = pinceis(nome).remove(0);
    let w = tela[0] as usize;
    let (_, g) = pinta_cena(&p, None, &cena, tela, papel, 0);
    // O recto lento: y 90, x 60..560 (40 px).
    let mut centro: Vec<f64> = (150..470).map(|x| escuro(&g, w, x, 90)).collect();
    centro.sort_by(f64::total_cmp);
    let miolo = centro[centro.len() / 2];
    let lim = 0.25 * miolo.max(0.05);
    let largura = (100..520)
        .map(|x| (50..131).filter(|&y| escuro(&g, w, x, y) > lim).count() as f64)
        .sum::<f64>()
        / 420.0;
    // O vertical lento: x 700, y 440..610.
    let linhas: Vec<Vec<f64>> = (480..570)
        .map(|y| (640..761).map(|x| escuro(&g, w, x, y)).collect())
        .collect();
    let de_pe = linhas
        .iter()
        .map(|l| l.iter().filter(|&&v| v > lim).count() as f64)
        .sum::<f64>()
        / linhas.len() as f64
        / largura.max(1.0);
    // O perfil médio por linha através do recto (x 150..470).
    let atraves: Vec<f64> = (50..131)
        .map(|y| (150..470).map(|x| escuro(&g, w, x, y)).sum::<f64>() / 320.0)
        .collect();
    let nucleo = &atraves[28..53];
    let m = nucleo.iter().sum::<f64>() / nucleo.len() as f64;
    let estria = (nucleo.iter().map(|v| (v - m).powi(2)).sum::<f64>() / nucleo.len() as f64).sqrt()
        / m.max(1e-6);
    let topo: Vec<f64> = (100..520)
        .filter_map(|x| {
            (50..131)
                .find(|&y| escuro(&g, w, x, y) > lim)
                .map(|y| y as f64)
        })
        .collect();
    let mt = topo.iter().sum::<f64>() / topo.len().max(1) as f64;
    let rasgo =
        (topo.iter().map(|v| (v - mt).powi(2)).sum::<f64>() / topo.len().max(1) as f64).sqrt();
    Oleo {
        de_pe,
        miolo,
        estria,
        rasgo,
    }
}

/// ⭐ **Cada óleo é o SEU óleo** (J4, `docs/Painter/Pinceis/exames/oleo_acrilica.md`): o que a prancha mostrou contra o
/// oráculo do Rebelle preso num número que o separa do vizinho que a matriz (ou a família) acha mais perto. A fronteira
/// de cada regra é o meio entre o pincel e o que ela separa, medidos na corrida de calibração (abaixo, por regra).
#[test]
fn cada_oleo_tem_a_sua_identidade() {
    let nomes = [
        "Oil Bristle",
        "Filbert",
        "Express Oil",
        "Dabber",
        "Hard Round",
    ];
    let medidas: Vec<(&str, Oleo)> = std::thread::scope(|s| {
        let h: Vec<_> = nomes
            .iter()
            .map(|&n| (n, s.spawn(move || mede_oleo(n))))
            .collect();
        h.into_iter().map(|(n, h)| (n, h.join().unwrap())).collect()
    });
    for (n, o) in &medidas {
        println!("[oleo] {n:>13}: {o:?}");
    }
    let m = |n: &str| medidas.iter().find(|(k, _)| *k == n).unwrap().1;
    let mut falhas = Vec::new();
    let mut regra = |ok: bool, o_que: String| {
        if !ok {
            falhas.push(o_que);
        }
    };
    let (oil, filbert, express, dabber) = (
        m("Oil Bristle"),
        m("Filbert"),
        m("Express Oil"),
        m("Dabber"),
    );
    // A lâmina FIXA de pé do Bristle Flat (o vertical 0,30 do recto) contra os redondos que seguem o traço (≥ 1,0).
    regra(
        oil.de_pe < 0.65,
        format!(
            "Oil Bristle: de pé {:.2} — a lâmina deixou de ser fixa",
            oil.de_pe
        ),
    );
    // O Filbert SEGUE o traço (1,0) — não é a lâmina do Oil Bristle —, tem as faixas de cerdas (0,036 contra o redondo
    // liso do Hard Round, 0) e a borda limpa (0,05 contra a esponja rasgada do Dabber, 1,75).
    regra(
        (0.85..1.15).contains(&filbert.de_pe),
        format!("Filbert: de pé {:.2} — não segue o traço", filbert.de_pe),
    );
    regra(
        filbert.estria > 0.018,
        format!(
            "Filbert: estria {:.3} — sem as faixas de cerdas (é o Hard Round)",
            filbert.estria
        ),
    );
    regra(
        filbert.rasgo < 0.9,
        format!(
            "Filbert: rasgo {:.2} — a borda rasgou (é o Dabber)",
            filbert.rasgo
        ),
    );
    // Os TUFOS do Express Oil: as faixas separadas por papel (estria 0,175 contra 0,036 do Filbert e 0,079 do Oil Bristle)
    // e a tinta clara (miolo 0,57 contra 0,72 do Oil Bristle).
    regra(
        express.estria > 0.12,
        format!(
            "Express Oil: estria {:.3} — os tufos fundiram-se",
            express.estria
        ),
    );
    regra(
        express.miolo < 0.645,
        format!(
            "Express Oil: miolo {:.2} — a tinta é a carregada do Oil Bristle",
            express.miolo
        ),
    );
    // A esponja do Dabber: a borda rasgada (1,75 contra ≤ 0,07 de todos os outros).
    regra(
        dabber.rasgo > 0.9,
        format!(
            "Dabber: rasgo {:.2} — a borda da esponja ficou lisa",
            dabber.rasgo
        ),
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Os reprovados não estão na fábrica: nenhum ficheiro com o nome de uma receita de [`EM_ESTUDO`] ou [`EM_ESTUDO_MEIOS`].
#[test]
fn os_reprovados_do_oleo_nao_estao_na_fabrica() {
    let b = ph2d_brush_presets::Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let na_fabrica: Vec<&str> = b
        .entradas
        .iter()
        .map(|e| e.preset.parametros.name.as_str())
        .filter(|n| {
            EM_ESTUDO.iter().any(|r| r.nome == *n) || EM_ESTUDO_MEIOS.iter().any(|r| r.nome == *n)
        })
        .collect();
    assert!(
        na_fabrica.is_empty(),
        "reprovados na fábrica: {na_fabrica:?}"
    );
}

/// Os VEIOS de um traço (J4 2.ª ronda, `exames/oleo_acrilica.md`): no recto de 200 px da cena `tamanhos` (foto A, linhas
/// 355..530, x 150..850), a luminância de cada linha alisada ao longo (caixa de 7 px — o pontilhado fino do relevo não
/// parte um troço) e o desvio à mediana da linha; os troços ESCUROS e CLAROS a mais de 25 níveis: a fracção da área e o
/// comprimento médio das corridas ao longo de x. O Rebelle `Bristle Flat` (a mesma conta, `veios.py` da 2.ª ronda): escuros
/// 0,0154 em corridas de 84 px, claros 0,0074 de 56 px.
#[derive(Debug, Clone, Copy)]
struct Veios {
    escuro: f64,
    corrida_escura: f64,
    claro: f64,
}

fn mede_veios(p: &crate::exame_bancada::Pincel) -> Veios {
    use crate::exame_bancada::{bateria, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "tamanhos")
        .unwrap()
        .clone();
    let (_, g) = pinta_cena(p, None, &cena, tela, papel, 0);
    let w = tela[0] as usize;
    const K: usize = 7;
    let (mut escuros, mut claros, mut n) = (0usize, 0usize, 0usize);
    let mut corridas: Vec<usize> = Vec::new();
    for y in 355..530 {
        let l: Vec<f64> = (150..850)
            .map(|x| 255.0 * (1.0 - escuro(&g, w, x, y)))
            .collect();
        let m: Vec<f64> = (K..l.len() - K)
            .map(|i| l[i - K / 2..=i + K / 2].iter().sum::<f64>() / K as f64)
            .collect();
        let mut o = m.clone();
        o.sort_by(f64::total_cmp);
        let med = o[o.len() / 2];
        let mut corrida = 0;
        for v in &m {
            n += 1;
            if v - med < -25.0 {
                escuros += 1;
                corrida += 1;
            } else {
                if corrida > 0 {
                    corridas.push(corrida);
                }
                corrida = 0;
                claros += usize::from(v - med > 25.0);
            }
        }
        if corrida > 0 {
            corridas.push(corrida);
        }
    }
    Veios {
        escuro: escuros as f64 / n as f64,
        corrida_escura: corridas.iter().sum::<usize>() as f64 / corridas.len().max(1) as f64,
        claro: claros as f64 / n as f64,
    }
}

/// ⭐ **Os fios do Oil Bristle aparecem e somem** (J4 2.ª ronda, `Knob::BristleStreak`): três revisores cegos da 1.ª ronda
/// repetiram «um pente regular demais» — no Rebelle, a troços raros, um fio fica quase preto ou falta. O traço de 200 px
/// tem os troços escuros e claros na fracção do Rebelle (a metade a duas vezes e meia) e as corridas escuras longas;
/// CONTROLO: o mesmo preset sem o veio não os tem (a régua mede).
#[test]
fn os_fios_do_oil_bristle_aparecem_e_somem() {
    use crate::exame_bancada::{Fonte, Pincel, pinceis};
    let p = pinceis("Oil Bristle").remove(0);
    let Fonte::Preset { preset, relativo } = &p.fonte else {
        unreachable!()
    };
    let mut sem = preset.clone();
    for k in [
        "bristleStreak",
        "bristleStreakLight",
        "bristleStreakLength",
        "bristleStreakCut",
    ] {
        sem.parametros.engine.remove(k);
    }
    let controlo = Pincel {
        nome: "sem veio".into(),
        fonte: Fonte::Preset {
            preset: sem,
            relativo: relativo.clone(),
        },
    };
    let (v, c) = std::thread::scope(|s| {
        let a = s.spawn(|| mede_veios(&p));
        let b = s.spawn(|| mede_veios(&controlo));
        (a.join().unwrap(), b.join().unwrap())
    });
    println!(
        "[veios] Oil Bristle {v:?} · sem o veio {c:?} · Rebelle escuro 0,0154 (84 px) claro 0,0074"
    );
    let mut falhas = Vec::new();
    if c.escuro > 0.003 || c.claro > 0.003 {
        falhas.push(format!(
            "o controlo já tem veios ({c:?}): a régua não mede o veio"
        ));
    }
    if !(0.0077..0.039).contains(&v.escuro) {
        falhas.push(format!("troços escuros {:.4} (Rebelle 0,0154)", v.escuro));
    }
    if !(0.0037..0.0185).contains(&v.claro) {
        falhas.push(format!("troços claros {:.4} (Rebelle 0,0074)", v.claro));
    }
    if v.corrida_escura < 15.0 {
        falhas.push(format!(
            "corridas escuras de {:.1} px: o fio pisca em vez de pesar a troços",
            v.corrida_escura
        ));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// O Flat Bristle em estudo (fora da fábrica): o ficheiro gerado pela receita [`FLAT_BRISTLE`], guardado com os presets de
/// teste da bancada para os portões da família o lerem.
const FLAT_EM_ESTUDO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/Painter/Pinceis/fixtures/presets_teste/Flat Bristle.ph2d-brush.png"
);

/// A tinta do recto de 5 px ÷ a do de 40 (cena `tamanhos`, foto A): a escuridão de PICO de cada coluna x 200..800 na
/// faixa do traço, a mediana. O Rebelle (a mesma conta): `Flat` 0,228 · `Bristle Flat` 0,952.
fn fino_sobre_largo(p: &crate::exame_bancada::Pincel) -> f64 {
    use crate::exame_bancada::{bateria, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "tamanhos")
        .unwrap()
        .clone();
    let (_, g) = pinta_cena(p, None, &cena, tela, papel, 0);
    let w = tela[0] as usize;
    let pico = |ys: std::ops::Range<usize>| {
        let mut v: Vec<f64> = (200..800)
            .map(|x| ys.clone().map(|y| escuro(&g, w, x, y)).fold(0.0, f64::max))
            .collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    pico(55..86) / pico(160..221).max(1e-6)
}

/// ⭐ **O chato pequeno esmaece** (J4 2.ª ronda, `Knob::ThinStrokeFade`): o `Flat` do Rebelle a 5 px quase não pinta (0,228
/// da tinta do de 40 px) — o revisor cego da 1.ª ronda viu «o 5 px firme contra quase invisível». O Flat Bristle fica
/// perto dele; o Oil Bristle (sem a lei) não esmaece (Rebelle `Bristle Flat` 0,952). CONTROLO: o Flat Bristle sem a lei
/// pinta o 5 px cheio (a régua mede).
#[test]
fn o_chato_pequeno_esmaece() {
    use crate::exame_bancada::{Fonte, Pincel, pinceis};
    let flat = pinceis(FLAT_EM_ESTUDO).remove(0);
    let oil = pinceis("Oil Bristle").remove(0);
    let Fonte::Preset { preset, relativo } = &flat.fonte else {
        unreachable!()
    };
    let mut sem = preset.clone();
    sem.parametros.engine.remove("thinStrokeFade");
    let controlo = Pincel {
        nome: "sem a lei".into(),
        fonte: Fonte::Preset {
            preset: sem,
            relativo: relativo.clone(),
        },
    };
    let [f, o, c] = std::thread::scope(|s| {
        let h = [&flat, &oil, &controlo].map(|p| s.spawn(move || fino_sobre_largo(p)));
        h.map(|h| h.join().unwrap())
    });
    println!(
        "[fino] Flat Bristle {f:.3} (Rebelle 0,228) · Oil Bristle {o:.3} (0,952) · Flat sem a lei {c:.3}"
    );
    let mut falhas = Vec::new();
    if c < 0.7 {
        falhas.push(format!(
            "o controlo {c:.3} já esmaece: a régua não mede a lei"
        ));
    }
    if !(0.13..0.35).contains(&f) {
        falhas.push(format!("Flat Bristle: 5 px ÷ 40 px {f:.3} (Rebelle 0,228)"));
    }
    if o < 0.75 {
        falhas.push(format!(
            "Oil Bristle: 5 px ÷ 40 px {o:.3} — a lei vazou para quem não a tem (Rebelle 0,952)"
        ));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
