//! **As RECEITAS da fábrica de pincéis** — o formato (`Receita` do Wet Paint, `ReceitaMeio` dos outros três meios),
//! as BASES partilhadas entre famílias e a CONTAGEM. As receitas moram cada uma na sua família
//! (`familias/<família>.rs`, J3); aqui só fica o Wet Round, que é de todas. O gerador e os portões vivem em
//! `presets_de_fabrica.rs`; o portão por foto em `midias_de_fabrica.rs`.

use super::familias::{FAMILIAS, FERRAMENTAS_WET};
use ph2d_brush_presets::Tilt;
use ph2d_tool_painter::PaintMedia;
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
            $crate::receitas_de_fabrica::MAIS_PERTO,
        ]
    };
}
pub(crate) use extras_do_lapis;

/// O Wet Round: o pincel SEM preset (`painter_default_knobs`) e o CONTROLO dos portões por foto — não é de nenhuma
/// família. As receitas das famílias somam-se a esta em [`receitas`].
pub const RECEITAS: &[Receita] = &[Receita {
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
    nasce_de: None,
}];

/// Quantos presets a fábrica tem — CONTADO das receitas (a base + cada família, o Wet e os outros meios), nunca
/// escolhido; `the_factory_presets_are_counted` confere-o com os ficheiros de `assets/brushes/`.
pub const PRESETS_DE_FABRICA: usize = {
    let mut n = RECEITAS.len();
    let mut i = 0;
    while i < FAMILIAS.len() {
        n += FAMILIAS[i].wet.len() + FAMILIAS[i].meios.len();
        i += 1;
    }
    let mut j = 0;
    while j < FERRAMENTAS_WET.len() {
        n += FERRAMENTAS_WET[j].len();
        j += 1;
    }
    n
};

/// **Uma FERRAMENTA do Wet Paint** (J4 `mistura`): uma [`Receita`] e o MODO dela — a ferramenta molhada que o preset
/// arma (`Blend`, `Wet`, `Dry`, `Smear`, `Blow`: o `paint_mode` do ficheiro, `WetTool` no tool). Ao lado da
/// `Receita` e não um campo dela: os literais das outras famílias não mudam. Uma ferramenta não pinta — a lei da
/// largura e a miniatura medem-na SOBRE a base do seu meio ([`base_da_ferramenta`]).
pub struct ReceitaFerramenta {
    pub receita: Receita,
    pub modo: &'static str,
}

/// O modo de uma receita do Wet que é uma ferramenta (`None` = um pincel que pinta).
pub fn modo_da_ferramenta(nome: &str) -> Option<&'static str> {
    FERRAMENTAS_WET
        .iter()
        .flat_map(|l| l.iter())
        .find(|f| f.receita.nome == nome)
        .map(|f| f.modo)
}

/// A tinta de fábrica sobre a qual uma ferramenta do Wet se mede (a mesma base da bancada,
/// `exame_bancada::base_padrao`): a aguarela do Wet Round para a água e o secador (o Rebelle fotografou WATER e DRY
/// sobre o Watercolor Round); para o resto (BLEND/SMUDGE sobre o Round Hard do óleo, uma faixa LISA e cheia) o Ink
/// Brush — a Acrylic Round riscava a faixa com as cerdas dela (e a ferramenta arrastava as riscas), e o Marker Bullet
/// (liso) tem o tecto de tamanho de 101 px: a lei da largura da ferramenta perdia a faixa acima do raio 72.
pub fn base_da_ferramenta(modo: &str) -> &'static str {
    match modo {
        "Wet" | "Dry" => "Wet Round",
        _ => "Ink Brush",
    }
}

/// Todas as receitas do Wet Paint: a base e as das famílias, pela ordem do agregador (a ordem não muda nenhum ficheiro).
pub fn receitas() -> impl Iterator<Item = &'static Receita> {
    RECEITAS
        .iter()
        .chain(FAMILIAS.iter().flat_map(|f| f.wet))
        .chain(FERRAMENTAS_WET.iter().flat_map(|l| l.iter().map(|f| &f.receita)))
}

/// Todas as receitas de Digital, Watercolor e Impasto (as das famílias).
pub fn receitas_dos_meios() -> impl Iterator<Item = &'static ReceitaMeio> {
    FAMILIAS.iter().flat_map(|f| f.meios)
}

/// Os presets de fábrica por meio, pela ordem de `PaintMedia` (Digital · Watercolor · Impasto · Wet Paint).
pub fn contagem_por_meio() -> [usize; 4] {
    let mut n = [0; 4];
    n[usize::from(PaintMedia::WetPaint.to_u8())] = receitas().count();
    for r in receitas_dos_meios() {
        n[usize::from(r.meio.to_u8())] += 1;
    }
    n
}

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
    /// O preset de fábrica VIZINHO de onde nasce um pincel ainda sem ficheiro (prompt 53): o vizinho inteiro + os
    /// overrides (o estado que a receita não lista — o `drag` da acrílica, os fios do Oil Bristle). Com o ficheiro
    /// gravado, a fonte é ele.
    pub nasce_de: Option<&'static str>,
}

/// **Um preset de Digital, Watercolor ou Impasto** (J3): parte do pincel por omissão do meio (`PainterTool::default`
/// + `set_paint_media` + `preset_capturar`) — ou do ficheiro de [`Self::nasce_de`] — e fixa o que lista. Grava em
/// `assets/brushes/<motor_do_meio>/<pasta>/<grupo>/<nome>.ph2d-brush.png`. ⚠️ Ao contrário do Wet, NÃO parte do
/// próprio ficheiro: a receita é a verdade inteira (`cada_preset_dos_meios_e_a_sua_receita`).
pub struct ReceitaMeio {
    pub meio: PaintMedia,
    /// A pasta de cima, a categoria («Pencil», «Ink»…): tem de ter rótulo (`rotulo_da_pasta`, ver 04_BANCADA §9).
    pub pasta: &'static str,
    pub grupo: &'static str,
    pub nome: &'static str,
    /// O modo pelo nome: «Paint»/«Smear»/«Blur» (Digital, Watercolor) ou a ferramenta do Impasto («Knife»…).
    pub modo: Option<&'static str>,
    /// Os campos do `BrushSpec` que o preset fixa, pelo nome e em JSON — só os de `campos_do_meio(meio)`.
    pub pincel: &'static [(&'static str, &'static str)],
    /// A mão, pelas chaves `engine` dos knobs da caneta (`LINHAS_DA_MAO`) — só as linhas que o meio lê.
    pub mao: &'static [(&'static str, f64)],
    /// O diâmetro (px) que o preset fixa; `None` = o do artista.
    pub size_px: Option<f64>,
    /// A Strength que o preset fixa (a Watercolor não a lê); `None` = a do artista.
    pub opacity: Option<f64>,
    /// Um preset de fábrica DO MESMO MEIO de onde este nasce (no lugar do pincel por omissão).
    pub nasce_de: Option<&'static str>,
    /// As IMAGENS do preset (J4 `texturas`, a ponta de imagem e a mangueira L19): o id no ficheiro (`shape`,
    /// `shape_2`…, `texture`) e a função que desenha a imagem — NOSSA, procedural — já no formato do ficheiro
    /// (`ph2d_brush_presets::escrever_textura`). Vazio = nenhuma (o de sempre).
    pub texturas: &'static [(&'static str, fn() -> Vec<u8>)],
}

impl ReceitaMeio {
    /// A receita que não fixa nada além do caminho: o resto com `..ReceitaMeio::novo(…)`.
    pub const fn novo(
        meio: PaintMedia,
        pasta: &'static str,
        grupo: &'static str,
        nome: &'static str,
    ) -> Self {
        Self {
            meio,
            pasta,
            grupo,
            nome,
            modo: None,
            pincel: &[],
            mao: &[],
            size_px: None,
            opacity: None,
            nasce_de: None,
            texturas: &[],
        }
    }

    /// [`Self::novo`] no Digital.
    pub const fn digital(pasta: &'static str, grupo: &'static str, nome: &'static str) -> Self {
        Self::novo(PaintMedia::Digital, pasta, grupo, nome)
    }

    /// [`Self::novo`] na Watercolor.
    pub const fn aquarela(pasta: &'static str, grupo: &'static str, nome: &'static str) -> Self {
        Self::novo(PaintMedia::Watercolor, pasta, grupo, nome)
    }

    /// [`Self::novo`] no Impasto.
    pub const fn impasto(pasta: &'static str, grupo: &'static str, nome: &'static str) -> Self {
        Self::novo(PaintMedia::Impasto, pasta, grupo, nome)
    }
}

/// ⭐ **A REGRA da fábrica** (dono, 18/10, prompt 51): todo pincel com o Spacing 0,1 e o Continuous — a cápsula põe
/// os sub-dabs a 0,025 do diâmetro e o pigmento é por comprimento (`Knob::PigmentSpacing`), então o traço é o do
/// Spacing denso a uma fracção dos dabs do óleo. Censo: `presets_de_fabrica::a_regra_do_spacing_da_fabrica`.
pub const SPACING_DA_FABRICA: f64 = 0.1;

/// Os secos de GRÃO (Graphite, Charcoal, Soft Pastel, Oil Pastel) e o Airbrush arredondam o sub-passo da cápsula ao
/// inteiro mais perto (`capsula::sub_passo_mais_perto`): para baixo, a 40 px os de grão passavam dos 16,7 ms por
/// quadro (o Oil Pastel 10,3 → 20,2 ms) e o grão não o pede (Graphite 8,69 nos dois); o PÓ do Airbrush sorteia-se por
/// dab, e o dobro de dabs dilui-o (a 31 px a textura 22,3 → 16,4, Rebelle 23,5) — doc 51 §4.
pub const MAIS_PERTO: (&str, f64) = ("capsuleRoundNearest", 1.0);

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
// As bases PARTILHADAS entre famílias (o `..` dos novos lê a MESMA): o 8B (grafite e lápis) e o Charcoal (pastel e
// carvão, e o Scratch dos lápis).
pub const GRAPHITE_8B: Receita = Receita {
    meio: "Pencil",
    grupo: "Graphite",
    nome: "Graphite 8B",
    water: 0.0,
    opacity: 1.0,
    pigment: 125.0,
    pickup: 1.0,
    gate: 0.5, // §7.2: 0,65 → 0,5 com o grão fino (o colar a 5 px 0,115 → 0,078, Rebelle 0,072; a Textura no vale)
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.0,
    size_px: Some(6.5),
    extras: extras_do_lapis!(0.25, 3.0),
    grao: Some(("Noise", GRAO_MACIO)),
    achatado: None,
    pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1")],
    nasce_de: None,
};

pub const CHARCOAL: Receita = Receita {
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
    grao: Some(("Noise", GRAO_MACIO)),
    achatado: None,
    pincel: &[("falloff", "\"Sharp\""), ("jitter", "0.2")],
    nasce_de: None,
};

/// ⭐ **O grão do lápis e do carvão é preso à tela e mais FINO que o de antes** (J2, `docs/Painter/Pinceis/03_MOTORES.md`
/// §7.2): o Size do Noise `Tiled` (período `256 ÷ Size` px) e o `paperGate`, da grelha Size 32…256 × Roughness ×
/// portão contra o Rebelle em DUAS réguas — o colar de contas dos rectos 5 · 40 · 200 px (`sonda_o_grao_fino_por_size`)
/// e o vale da mídia (`cada_preset_parece_a_sua_midia`, a Textura). Os macios e o carvão no 100 (2,56 px); os duros
/// (HB, 2H) no 64 (4 px): mais fino, o resíduo fino passava o vale da Textura do Rebelle (6,2 · 5,0). Eram 16–64
/// (4–16 px) e uma lei pelo diâmetro (`grao_do_wet`, saiu: o papel mudava com a ponta): a 5 px sem ela um vale
/// atravessava o traço (buracos de 2–3 colunas) e a 40 px o cv era 1,5–2,4× o do Rebelle.
pub const GRAO_MACIO: f64 = 100.0;
/// Ver [`GRAO_MACIO`].
pub const GRAO_DURO: f64 = 64.0;

/// Um lápis do prompt 53: o 8B inteiro (de onde nasce) com o grupo, a carga, o papel, o grão e a mão dele.
pub const fn lapis(
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
        nasce_de: Some("Graphite 8B"),
        ..GRAPHITE_8B
    }
}
