//! **As RECEITAS da fábrica do Wet Painter** — cada preset de `assets/brushes/WetPainter/` numa linha (o
//! gerador e os portões vivem em `presets_de_fabrica.rs`; o portão por mídia em `midias_de_fabrica.rs`).

use ph2d_brush_presets::Tilt;
use serde_json::{Value, json};

/// Os knobs do 8B que os lápis do 53 herdam (a caneta, a ponta deitada e o arrasto: comentário do 8B) + a PRESSÃO de
/// cada um (J2, `docs/Painter/Pinceis/03_MOTORES.md` §7 a): a curva do tamanho e a pressão → carga ajustadas às
/// rampas da origem no Rebelle (`leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`).
macro_rules! extras_do_lapis {
    ($curva_do_tamanho:expr, $curva_da_carga:expr) => {
        &[
            ("felt", 0.0),
            ("drag", 0.05),
            ("pressureSize", 0.9),
            ("pressureSizeCurve", $curva_do_tamanho),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", $curva_da_carga),
            ("tiltStart", 39.0),
            ("tiltFull", 45.0),
            ("tiltWidth", 1.05),
            ("tiltSmooth", 0.0),
            ("tiltLength", 3.3),
            ("tiltLag", 170.0),
            MAIS_PERTO,
        ]
    };
}

/// Quantos presets a fábrica tem — CONTADO (`the_factory_presets_are_counted`), nunca escolhido.
pub const PRESETS_DE_FABRICA: usize = 28;

/// Os pincéis que nascem do ficheiro de um VIZINHO de fábrica (prompt 53): a receita medida no doc 52 é o vizinho
/// inteiro + os overrides (a sonda partiu do ficheiro dele, com o estado que a `Receita` não lista — o `drag` da
/// acrílica, os fios do Oil Bristle). O Filbert e as lâminas saíram pelo smoke do dono (doc 53 §8).
pub const NASCE_DE: [(&str, &str); 8] = [
    ("Graphite 2H", "Graphite 8B"),
    ("Graphite HB", "Graphite 8B"),
    ("Graphite 2B", "Graphite 8B"),
    ("Colored 4B", "Graphite 8B"),
    ("Scratch", "Charcoal"),
    ("Square Hard", "Soft Pastel"),
    ("Wax Scrape", "Soft Pastel"),
    ("Dabber", "Oil Bristle"),
];

/// O grão `kind` de lado `lado` no slot Grain do pincel (o JSON do `BrushSpec::texture`).
pub fn grao(kind: &str, lado: f64) -> Value {
    json!({"angle_deg": 0, "flow": false, "kind": kind, "mapping": "Tiled", "offset": [0.0, 0.0],
           "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [lado, lado],
           "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]})
}

/// Uma linha do 02 §6.9 — ⚠️ o pigmento e a orla CALIBRADOS contra o Rebelle (dono 10/10: «aproxime do
/// Rebelle» e «o correto para clarear não seria usar pigment?»; `sonda_presets_varre` contra
/// `fixtures/rebelle_presets/`, molhado a 2 s e seco a 30 s, 05 §4b). A claridade é POUCO pigmento — a cor
/// acumula até à escolhida; a Opacity fica a 100 % (é a quantidade relativa do artista). `pigment`/`gate`/`evaporation`/cerdas/`water` estão nas unidades do motor;
/// `pickup` é RAZÃO ao padrão do dono (05 §1.2); `edge` é o valor do motor (desde 10/10: o padrão mudou). A gravidade é a MESMA em
/// todos (W1e, F5: o Rebelle não a tem por pincel — a velocidade é inclinação × água × cor); o `tilt` é o
/// do dial (W1e, F4: Round, Wash e Dry com a tela direita, como o Rebelle; o Drip, o único que promete
/// escorrer, inclinado no máximo, para baixo).
pub struct Receita {
    pub nome: &'static str,
    /// A pasta: `WetPainter/<meio>/<grupo>/<nome>` (o meio pelo nome da ferramenta do Rebelle, 17 C).
    pub meio: &'static str,
    pub grupo: &'static str,
    pub water: f64,
    /// A Opacity do Wet Paint: a quantidade de pigmento, em fracção do `pigment` (`1` = a receita).
    pub opacity: f64,
    pub pigment: f64,
    pub pickup: f64,
    pub gate: f64,
    pub evaporation: f64,
    pub edge: f64,
    pub tilt: Tilt,
    pub bristle_size: f64,
    pub bristle_count: f64,
    /// A dureza do carimbo (`BrushSpec::hardness`): 0 nos quatro de 07/10.
    pub hardness: f64,
    /// O tamanho (diâmetro, px) que o preset FIXA — só onde o tamanho É a mídia (o Liner é fino); nos
    /// outros fica o do artista.
    pub size_px: Option<f64>,
    /// Os knobs do motor além dos de cima, pela chave do registo (o corpo do óleo: as cerdas e o relevo).
    pub extras: &'static [(&'static str, f64)],
    /// O grão do pincel (`BrushSpec::texture`, que toma o lugar das cerdas — W2.4): o tipo e o lado.
    pub grao: Option<(&'static str, f64)>,
    /// O pincel CHATO (`BrushSpec::dab_flatten`, `dab_angle_deg`): a pegada achatada e o ângulo dela, FIXO
    /// na tela (não segue o traço) — o Bristle Flat do Rebelle é largo deitado e fino de pé.
    pub achatado: Option<(f64, u16)>,
    /// Outros campos do `BrushSpec` que o preset FIXA (`CAMPOS_DO_PINCEL`), pelo nome e em JSON: o perfil
    /// (`falloff`) e o espalhamento (`jitter`) da família seca (prompt 35).
    pub pincel: &'static [(&'static str, &'static str)],
}

/// ⭐ **A REGRA da fábrica** (dono, 18/10, prompt 51): todo pincel com o Spacing 0,1 e o Continuous — a cápsula põe
/// os sub-dabs a 0,025 do diâmetro e o pigmento é por comprimento (`Knob::PigmentSpacing`), então o traço é o do
/// Spacing denso a uma fracção dos dabs do óleo. Censo: `presets_de_fabrica::a_regra_do_spacing_da_fabrica`.
pub const SPACING_DA_FABRICA: f64 = 0.1;

/// Os três óleos foram afinados a Spacing 0,01 (o pigmento por dab deles é o de 0,01): o pigmento por comprimento
/// mantém a tinta do Express Oil e da Acrylic Round com os dabs mais esparsos.
const PASSO_DOS_OLEOS: f64 = 0.01;

/// Os secos de GRÃO (Graphite, Charcoal, Soft Pastel, Oil Pastel) e o Airbrush arredondam o sub-passo da cápsula ao
/// inteiro mais perto (`capsula::sub_passo_mais_perto`): para baixo, a 40 px os de grão passavam dos 16,7 ms por
/// quadro (o Oil Pastel 10,3 → 20,2 ms) e o grão não o pede (Graphite 8,69 nos dois); o PÓ do Airbrush sorteia-se por
/// dab, e o dobro de dabs dilui-o (a 31 px a textura 22,3 → 16,4, Rebelle 23,5) — doc 51 §4.
const MAIS_PERTO: (&str, f64) = ("capsuleRoundNearest", 1.0);

/// A tela direita, com o dial guardado no meio e para baixo (para quando o artista a ligar).
pub const DIREITA: Tilt = Tilt {
    on: false,
    ring: 4,
    spoke: 3,
};

// ⭐ A caneta (W6, doc 47_medicao): ⛔ o `pressureSize` que punha as PONTAS da rampa nas da foto do Rebelle SAIU por
// ordem do DONO no smoke de 18/10 («apertar muito e pouco variam pouco a espessura; antes funcionava melhor») — a mão
// dele vai de 0,24 a 0,86 e o piso alto comia a resposta (§2). O 8B fica com o de sempre (0,9, explícito: o gerador
// mantém uma chave antiga) e leva a
// ponta DEITADA medida no Rebelle a 0–63° (`fixtures/rebelle_tilt/`): de 39° a 45° a elipse 3,3 × 1,05 ao longo do
// azimute (a sonda `sonda_o_lapis_deitado_por_variante`: o grão alisa-se SÓ pela sobreposição da elipse — o
// `tiltSmooth` alargava o traço e ficava fora do vale), a inclinação lida com 170 px de atraso.
pub const RECEITAS: [Receita; PRESETS_DE_FABRICA] = [
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Round",
        water: 1.0,
        opacity: 1.0,
        pigment: 78.0,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: None,
        // A pressão do Wet Round fica a de sempre (J2 §7 a): ele é o pincel SEM preset (`painter_default_knobs`) e o
        // CONTROLO das mídias — a lei do Rebelle dele (0,089 contra 0,155) tirava ao Wet Flat a RampaCobertura.
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Wash",
        water: 0.4,
        opacity: 1.0,
        pigment: 21.0,
        pickup: 0.6,
        gate: 0.3,
        evaporation: 0.7,
        edge: 20.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[("pressureSizeCurve", 0.35), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Dry Brush",
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
    },
    // ⭐ O 1.º da Fase C (prompt 17, 05 §4b «Presets de todas as mídias»): o Liner do Rebelle
    // (`Watercolor/Liner`: duro, carregado, papel 0,10 — o grão quase não fura, 28,86 px). Medido contra
    // `fixtures/rebelle/wcliner.png` (cobertura 0,65 · textura 1,7 %) à largura 38; o Wet Round dá 0,19 · 8,1.
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Liner",
        water: 0.5,
        opacity: 1.0,
        pigment: 100.0,
        pickup: 1.0,
        gate: 0.05,
        evaporation: 1.0,
        edge: 100.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: Some(29.0),
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // ⭐ O 2.º lote (11/10, depois da reavaliação pedida pelo dono — 18 §3: pigmento alto, o relevo de volta
    // e o grão no lugar das cerdas). Cada um medido contra a foto do seu preset-oráculo do Rebelle, seco, à
    // largura dela; o tamanho é o do preset dele. `cada_preset_parece_a_sua_midia` guarda-os.
    // Oil & Acrylic › Bristle Flat: corpo cheio, estrias das cerdas, relevo.
    OIL_BRISTLE,
    // Oil & Acrylic › Round Hard (a acrílica do Rebelle é esta ferramenta).
    ACRYLIC_ROUND,
    // Express Oil › Hairy: claro e liso, tufos (prompt 23, aceite do dono 12/10). A claridade é POUCO
    // pigmento — com o Spacing 0,01 cada célula leva ~100 carimbos, por isso 60 e não 250; sem água (a água
    // assentava o pouco pigmento nos poros: grão 0,065 → 0,027); cerdas 4 = tufos sem sal.
    Receita {
        meio: "ExpressOil",
        grupo: "Round",
        nome: "Express Oil",
        water: 0.0,
        opacity: 0.6,
        pigment: 60.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 4.0,
        bristle_count: 900.0,
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
            ("bristleLaneWidth", 2.0),
            ("bristleLift", 0.2),
            ("bristleNoiseLength", 2000.0),
            ("pigmentSpacing", PASSO_DOS_OLEOS),
            ("pressureSizeCurve", 0.5), ("pressureOpacity", 0.75), ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Pencil › Graphite 8B: sem água, o grão do papel a abrir na orla. Prompt 35 (14/10), contra o Rebelle
    // «Graphite Pencil/8B» a 20 px vistos (`pastel_regua.py`, nota 4,21 → 1,15): o Noise 0,4 era a mancha de
    // ~640 px do Soft Pastel (5 px a y = 40, 8 a 450) — Noise 32; papel 0,6 e carga 125 (o miolo cheio do 8B).
    // Com o arrasto na ponta seca (o motor deixou de o zerar): o portão das mídias reprovava a Textura (7,65
    // contra 9,09 ± 0,66) e o arrasto só a BAIXA (0,16 → 3,25) — não são as estrias. Varridos 1 368 + 594 na
    // bancada: perfil Linear, jitter 0,1 (a borda ondulada do 8B), arrasto 0,05, papel 0,65 — nota 1,25 → 0,64.
    GRAPHITE_8B,
    // Pencil › Charcoal: pouca carga, orla larga. Prompt 35 (14/10), contra o Rebelle «Charcoal/Charcoal» a
    // 37 px vistos (nota 12,03 → 3,35): o Noise 0,15 era uma mancha (25 px a y = 40, 36 a 450) — Noise 48; o
    // carvão do Rebelle é RALO (miolo 0,64). A orla do Rebelle (15 px a 21 de largura, 25 px de subida a 38)
    // não cabe na dureza: a 0,5 o portão das mídias lia 7, e nenhuma dureza/papel/carga/arrasto passa de 9
    // (1 755 variantes). O perfil Sharp (p², a cauda longa) e o jitter 0,2 dão 10 e a poeira do carvão:
    // dureza 0,1, papel 0, carga 24, arrasto 0,02 — nota 3,31 → 0,79.
    CHARCOAL,
    // Pastel › Pastel Dry › Dry Thin: grão fino e regular. Prompt 35 (14/10), contra o Rebelle NO TEMPO e à
    // mesma largura vista (`ferramentas/pastel_regua.py`, nota 4,76 → 1,38): o grão Noise 0,4 era uma mancha
    // de ~640 px (a largura ia de 15 a 34 px conforme a ALTURA na tela) — Noise 100 é o grão de 2–4 px; o
    // papel 0,8 ondulava a borda; a carga 400 enchia o miolo (0,98 contra 0,91).
    SOFT_PASTEL,
    // Pastel › Pastel Oil › Oil Pastel 2 (o giz de cera): grão mais fino e mais carregado que o seco (o
    // Voronoi passava as métricas, mas a largura variava até 42 % com a posição — medido 11/10). Prompt 35
    // (14/10), contra o Rebelle «Pastel Oil/Oil Pastel 2» a 77 px vistos (nota 5,06 → 1,41): o Noise 0,15
    // também era mancha (42 px a y = 40, 68 a 450) — Noise 100. Com o arrasto na ponta seca: o portão das
    // mídias reprovava Textura (9,7 contra > 12,6) e Orla (4 contra > 6); o arrasto as BAIXA (0,3: Textura
    // 12,9 só com papel 1,2) e não faz as estrias do Rebelle (a prancha: ele alisa). Carga 100, jitter 0,1,
    // arrasto 0,05 — nota 1,45 → 1,05. ABERTO: as estrias ao longo do traço não vêm de nenhum destes botões.
    Receita {
        meio: "Pastel",
        grupo: "Oil",
        nome: "Oil Pastel",
        water: 0.0,
        opacity: 1.0,
        pigment: 100.0,
        pickup: 1.0,
        gate: 0.8,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: Some(77.0),
        extras: &[("felt", 0.0), ("drag", 0.05), MAIS_PERTO, ("pressureSizeCurve", 0.5), ("pressureOpacity", 0.9), ("pressureOpacityCurve", 2.0)],
        grao: Some(("Noise", 100.0)),
        achatado: None,
        pincel: &[("jitter", "0.1")],
    },
    // ⭐ A lei da tinta POR PINCEL (prompt 42, doc 42): tinta e marcador — o Glaze do pincel escurece o
    // cruzamento como o do Rebelle (molhado: Ink 0,117 · Dip Pen 0,223 · Ink Brush 0,239 · marcadores 0,016).
    // As receitas da sonda `sonda_midias` (b7: `z_IB2_g100`, `z_DP_g80`, `z_IK_g90`, `z_MB_h75`,
    // `z_ML_h90`). As cerdas são as do Shape de fábrica: o feltro/cerdas que a sonda punha no mapa do motor
    // não pintam (medido: a cena sem eles é a mesma ao byte).
    // Ink › Brush: corpo ralo e orla de nanquim, a água que se espalha devagar.
    Receita {
        meio: "Ink",
        grupo: "Brush",
        nome: "Ink Brush",
        water: 0.5,
        opacity: 1.0,
        pigment: 150.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 3.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.1,
        size_px: None,
        // Glaze 0,8: a fracção multiplicada que o oráculo MEDIU nesta tinta (doc 42 §3: 0,76 · 0,82); o 1,0 de antes
        // compensava o halo e o azul arrastado para baixo (doc 42 §6).
        extras: &[("extDiffusion", 0.01), ("glaze", 0.8), ("pressureSizeCurve", 1.0), ("pressureOpacity", 0.9), ("pressureOpacityCurve", 0.5)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Ink › Pen › Dip Pen: duro e cheio, a tinta de cima ainda tapa um pouco (Glaze 0,8).
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Dip Pen",
        water: 0.3,
        opacity: 1.0,
        pigment: 1500.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[("glaze", 0.8), ("pressureSizeCurve", 1.0), ("pressureOpacity", 0.9), ("pressureOpacityCurve", 1.5)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Ink › Pen › Ink Bullet: o mais carregado e seco da tinta.
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Ink Bullet",
        water: 0.15,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[("glaze", 0.9), ("pressureSizeCurve", 0.25), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 3.0)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Marker › Marker Bullet: chapado, quase sem água, multiplica (Glaze 1).
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Bullet",
        water: 0.1,
        opacity: 1.0,
        pigment: 800.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.75,
        size_px: None,
        extras: &[("glaze", 1.0), ("pressureSizeCurve", 0.25), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 2.0)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Marker › Marker Liner: o mesmo marcador, a foto é fina (18 px): a dureza
    // medida à lei de largura do PRÓPRIO preset (raio 9; doc 42 §6) — 0,9 dava a borda 0,874 (Rebelle 0,716).
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Liner",
        water: 0.1,
        opacity: 1.0,
        pigment: 800.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.55,
        size_px: None,
        extras: &[("glaze", 1.0), ("pressureSizeCurve", 0.35), ("pressureOpacity", 0.9), ("pressureOpacityCurve", 3.0)],
        grao: None,
        achatado: None,
        pincel: &[],
    },
    // Airbrush › Spray › Airbrush: o PÓ (prompt 43) — gotas de 2 px, uma em quatro por dab, seco, sem cerdas;
    // contra o Rebelle «Airbrush/Default/Spray 2» (doc 43 §4: a mais perto do Rebelle das 7/7).
    Receita {
        meio: "Airbrush",
        grupo: "Spray",
        nome: "Airbrush",
        water: 0.0,
        opacity: 1.0,
        pigment: 9.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.3,
        size_px: None,
        // A pressão do Spray 2 (dono, smoke do P44: «como no Rebelle»): rampa 0,63/0,34. Com a opacidade que
        // POUSA (prompt 45) e a curva 1,5: 0,60/0,35. Os números LITERAIS dele (tamanho 0) esmaecem igual mas
        // não afinam (largura 1,00) — doc 45 §2.
        extras: &[
            ("drag", 0.0),
            ("dust", 0.8),
            MAIS_PERTO,
            ("pressureSize", 0.5),
            // As PONTAS do Rebelle (ordem do dono 19/10, doc 51 §4): a curva côncava alarga o traço leve mais cedo —
            // pela mediana de 21 tiragens do pó, pontas 0,580 → 0,680 (Rebelle 0,80), a rampa 0,640 → 0,704 (0,63).
            // Nenhuma intensidade com a curva recta casa as duas: as pontas seguem a rampa a −0,06.
            ("pressureSizeCurve", 0.7),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[("falloff", "\"Smooth\""), ("shape", SEM_FORMA)],
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
    },
    // A guache FLAT do Rebelle (`gouacheflat`, Gouache › Gouache Flat): a mesma ponta, mais alta que larga, e o
    // DOBRO da tinta (cobertura 0,50 × 0,32). A Gouache Round não entra: no Rebelle ela é o nosso Round (18 §4.3).
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DA_GUACHE,
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
            ("pressureOpacityCurve", CURVA_DA_GUACHE),
        ],
        grao: None,
        achatado: Some((0.2, 90)),
        pincel: &[("dab_quina", "1.0")],
    },
    // ⭐ Os 8 que ficaram do prompt 53 (doc 53 §8): as receitas que passaram a foto do Rebelle no doc 52 §4 (`52_regua/receitas_19*.json`,
    // pelo nome no comentário), cada uma o vizinho de [`NASCE_DE`] inteiro + os overrides. O tamanho é o do preset
    // dele (`volume_params.size` do `.rebelle-brush.png`, `ferramentas/tamanho_dos_presets.py`).
    // `lapisb_027` com a carga 50: o lápis duro, o grão fino. A 40 cobria 0,560 — a do HB (Rebelle 2H 0,595, HB 0,561) e
    // ficava mais perto da foto do irmão (5,3 × 4,2 ruídos); a 50, 0,617 e 3,7 × 11,2 (doc 53, `IRMAOS`).
    lapis("Graphite", "Graphite 2H", 50.0, 0.3, 16.0, extras_do_lapis!(0.5, 2.0)),
    // `lapisb_028`.
    lapis("Graphite", "Graphite HB", 40.0, 0.3, 32.0, extras_do_lapis!(0.5, 2.0)),
    // `lapisb_032`.
    lapis("Graphite", "Graphite 2B", 40.0, 0.65, 64.0, extras_do_lapis!(0.25, 2.0)),
    // `lapisb_041`: o de cor é o 2B com o dobro da carga.
    lapis("Colored", "Colored 4B", 80.0, 0.65, 64.0, extras_do_lapis!(0.5, 3.0)),
    // `lapisc_053`: o carvão mais ralo, o papel a meio.
    Receita {
        meio: "Pencil",
        grupo: "Scratch",
        nome: "Scratch",
        pigment: 20.0,
        gate: 0.5,
        size_px: Some(248.0),
        extras: &[("felt", 0.0), ("drag", 0.02), MAIS_PERTO, ("pressureSizeCurve", 0.35), ("pressureOpacity", 0.75), ("pressureOpacityCurve", 0.75)],
        ..CHARCOAL
    },
    // `sq_007`: o pastel de ponta RETA, o papel todo aberto.
    Receita {
        nome: "Square Hard",
        pigment: 60.0,
        gate: 1.0,
        size_px: Some(35.0),
        extras: &[("felt", 0.0), ("drag", 0.0), MAIS_PERTO, ("pressureSizeCurve", 0.25), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 2.0)],
        pincel: &[("dab_quina", "1.0")],
        ..SOFT_PASTEL
    },
    // `wx_007`: a cera raspada — muita carga sobre o papel aberto.
    Receita {
        grupo: "Scrape",
        nome: "Wax Scrape",
        pigment: 150.0,
        gate: 1.0,
        size_px: Some(141.0),
        extras: &[("felt", 0.0), ("drag", 0.0), MAIS_PERTO, ("pressureSizeCurve", 0.35), ("pressureOpacity", 0.75), ("pressureOpacityCurve", 1.5)],
        pincel: &[("dab_quina", "0.0")],
        ..SOFT_PASTEL
    },
    // `dabberc_032`: o Oil Bristle de cerdas largas, mais relevo e a ponta espalhada (jitter) — a esponja.
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
            ("emboss", 1.2),
            ("bristlePressure", 1.2),
            ("bristleLaneWidth", 1.5),
            ("bristleLift", 0.4),
            ("bristleNoiseSat", 0.3),
            // O grão claro da esponja: as duas fotos do Rebelle cercam-no (a da bateria: textura do recto ≤ ~11, 0,6 dá
            // 11,3 ✗; a do 52 seca: ≥ 9,6, o vale do Wet Round) — 0,55 dá 10,9 e 9,67 (0,25: 9,9 e 8,1 ✗).
            ("bristleNoiseLight", 0.55),
            ("bristleHeight", 0.75),
            ("pigmentSpacing", 0.0),
            ("capsuleMaxPx", 2.0),
            ("pressureSizeCurve", 0.25), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 3.0),
        ],
        pincel: &[("jitter", "0.4")],
        ..OIL_BRISTLE
    },
];

// As bases dos 8 do prompt 53, com o nome de cada uma (a fábrica e o `..` dos novos leem a MESMA).
const GRAPHITE_8B: Receita = Receita {
    meio: "Pencil",
    grupo: "Graphite",
    nome: "Graphite 8B",
    water: 0.0,
    opacity: 1.0,
    pigment: 125.0,
    pickup: 1.0,
    gate: 0.65,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.0,
    size_px: Some(6.5),
    extras: extras_do_lapis!(0.25, 3.0),
    grao: Some(("Noise", 32.0)),
    achatado: None,
    pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1")],
};

const CHARCOAL: Receita = Receita {
    meio: "Pencil",
    grupo: "Charcoal",
    nome: "Charcoal",
    water: 0.0,
    opacity: 1.0,
    pigment: 24.0,
    pickup: 1.0,
    gate: 0.0,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.1,
    size_px: Some(37.0),
    extras: &[("felt", 0.0), ("drag", 0.02), MAIS_PERTO],
    grao: Some(("Noise", 48.0)),
    achatado: None,
    pincel: &[("falloff", "\"Sharp\""), ("jitter", "0.2")],
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
    extras: &[("felt", 0.0), ("drag", 0.0), MAIS_PERTO, ("pressureSizeCurve", 0.5), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 0.5)],
    grao: Some(("Noise", 100.0)),
    achatado: None,
    pincel: &[],
};

const ACRYLIC_ROUND: Receita = Receita {
    meio: "Oil",
    grupo: "Round",
    nome: "Acrylic Round",
    water: 0.05,
    opacity: 0.6,
    pigment: 500.0,
    pickup: 1.0,
    gate: 0.0,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 0.5,
    bristle_count: 4000.0,
    hardness: 0.5,
    size_px: Some(78.0),
    extras: &[
        ("felt", 0.0),
        ("bristleStrength", 2.0),
        ("pigmentSpacing", PASSO_DOS_OLEOS),
        ("pressureSizeCurve", 1.4),
        // A textura das cerdas dela anda com o dab: a 0,025 a escada a 103 px dobrava (6,2 → 12,5); à densidade
        // dela 6,1, a diferença de imagem 0,04 (doc 51 §3).
        ("capsuleSpacing", PASSO_DOS_OLEOS),
    ],
    grao: None,
    achatado: None,
    pincel: &[],
};

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
        ("emboss", 0.7),
        ("bristlePressure", 1.2),
        ("bristleLaneWidth", 1.5),
        ("bristleLift", 0.4),
        ("bristleNoiseSat", 0.3),
        ("bristleNoiseLight", 0.25),
        ("bristleHeight", 0.75),
        // O pigmento por DAB (doc 51 §3): ele satura (a tinta a 103 px 0,985 da de antes sem o peso) e o peso só lhe
        // dava degraus — a escada da curva 2,57 · 2,70 (tecto 2,50 · 2,55) contra 2,15 · 2,40 sem ele.
        ("pigmentSpacing", 0.0),
        // Os fios dele têm largura em px: a 103 px a escada lê 2,24 · 2,45 · 2,15 com a cápsula a 2 px, 3,06 a 2,06
        // e 4,71 a 3 (tecto 2,78 · 2,60 · 2,50; doc 51 §3).
        ("capsuleMaxPx", 2.0),
        ("pressureSizeCurve", 0.25), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 2.0),
    ],
    grao: None,
    achatado: Some((0.68, 90)),
    pincel: &[],
};


const fn lapis(
    grupo: &'static str,
    nome: &'static str,
    pigment: f64,
    gate: f64,
    grao: f64,
    extras: &'static [(&'static str, f64)],
) -> Receita {
    Receita {
        grupo,
        nome,
        pigment,
        gate,
        grao: Some(("Noise", grao)),
        extras,
        ..GRAPHITE_8B
    }
}

/// O pigmento do Wet Flat e da Gouache Flat (doc 44 §3, lote 3).
const PIGMENTO_DO_FLAT: f64 = 50.0;
const PIGMENTO_DA_GUACHE: f64 = 100.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — Flat 0,41
/// (0,42), Gouache Flat 0,42 (0,43).
const CURVA_DO_FLAT: f64 = 1.75;
const CURVA_DA_GUACHE: f64 = 2.0;

/// O Shape sem forma (nem cerdas): a ponta é só a queda — o aerógrafo.
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
