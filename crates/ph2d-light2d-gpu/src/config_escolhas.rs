//! As ESCOLHAS da [`super::Config`] (o miolo, o canto, a cunha do vértice, a holográfica, a máscara da
//! fusão) — ficheiro irmão do `config.rs` pelo tecto de 700 LOC (W6g r9).

/// ⭐ W5h — o miolo por difusão: `Δu − u/ℓ² = 0` dentro dos opacos, com `u` = a luz do anel de
/// texels junto à borda (o pé da W5g); resolvido numa pirâmide, da grade mais grossa à fina.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Miolo {
    /// `0` = o exterior mais próximo · `1` = a difusão inteira (mistura linear).
    pub suavidade: f32,
    /// `ℓ` em metros; `f32::INFINITY` espalha até ao centro.
    pub penetracao_m: f32,
    /// As relaxações de Jacobi no nível 1 da pirâmide (o logo acima da fina, que faz uma só); cada
    /// nível acima faz o dobro do de baixo até [`Self::relaxa_max`], e o mais grosso o dobro deste.
    pub relaxa: u32,
    pub relaxa_max: u32,
    /// ⭐ W5n — os V-ciclos de CORREÇÃO depois da cascata (o defeito do nível 1 desce, o erro resolve-se
    /// no mais grosso e sobe); 0 = só a cascata (a W5h: o erro de média frequência ficava).
    pub ciclos: u32,
    /// ⭐ W5n — os pares de relaxações do erro em cada nível do V-ciclo, a subir.
    pub suaviza: u32,
    /// ⭐ W5o — os níveis mais grossos que cabem na memória de grupo resolvem-se num despacho só
    /// (`c3_miolo_base.wgsl`, a mesma conta); `false` = um despacho por relaxação (a W5n).
    pub base_no_grupo: bool,
    /// ⭐ W5q — até `bloco` relaxações por despacho (4 ou 2, o bloco 16×16 com a borda na memória de
    /// grupo, a mesma conta ao bit); ≤ 1 = uma por despacho (a W5o).
    pub bloco: u32,
}

/// ⭐ W5h — a escolha do dono (11/10: a difusão inteira, até ao centro); as relaxações MEDIDAS na
/// rodada `medida_luz2d_w5h`.
pub const MIOLO_PADRAO: Miolo = Miolo {
    suavidade: 1.0,
    penetracao_m: f32::INFINITY,
    relaxa: 4,
    relaxa_max: 32,
    ciclos: 3,
    suaviza: 2,
    base_no_grupo: true,
    bloco: 4,
};

/// ⭐ W6f — o texel do canto pelas faces ([`Config::canto`]; o `onde_calcular` do `c3_direta.wgsl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Canto {
    /// As normais «iguais» (ou não medidas) de duas faces achadas pela direção medem-se de novo pelo lado
    /// OPOSTO à outra face (a paralela centrada cruza a outra face junto ao vértice).
    pub normais: bool,
    /// O ponto remapeado em que a normal secante desiste (o canto) lê a visibilidade pela normal da face
    /// medida no `onde_calcular`, não a marcha cheia (o campo bilinear arredonda o vértice convexo).
    pub luz: bool,
    /// No texel que uma face corta, a fração da outra face conta só a parte dentro das duas.
    pub fracao: bool,
    /// ⭐ W6g — com duas faces, as normais SEMPRE pelo lado oposto (com [`Self::normais`]): a centrada sai
    /// 2,4° torta no (152, 245) do F3 sem as dar «iguais» (handoff W6f §9, W6g §2).
    pub um_lado: bool,
    /// ⭐ W6g r5 — o canto num passe PRÓPRIO sobre os texels de duas faces: a fina comum compila sem o
    /// código dele. A mesma luz, bit a bit; o passe custa ~0,5 ms de latência (a cadeia de ~1 600 leituras do
    /// pior texel) e o que a fina comum poupava era um degrau de registos que a r6 já tira da lei W6d —
    /// desligado no [`Self::LIGADO`] (handoff W6g §12, §14, §16).
    pub passe_proprio: bool,
}

impl Canto {
    pub const DESLIGADO: Self = Self {
        normais: false,
        luz: false,
        fracao: false,
        um_lado: false,
        passe_proprio: false,
    };
    pub const LIGADO: Self = Self {
        normais: true,
        luz: true,
        fracao: true,
        um_lado: true,
        passe_proprio: false,
    };
}

/// ⭐ W6e — os dois limiares da cunha no parcial do vértice ([`Config::borda_vertice`], frações da
/// área de um texel da captura).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertice {
    /// A reta própria do parcial não explica a 3×3 dele: a cobertura que ela prevê para um vizinho
    /// difere da lida em mais disto (o filtro barato: só esses procuram a cunha).
    pub pre: f32,
    /// A cunha explica o parcial: a área que ela deixa dentro dele difere da cobertura em menos disto.
    pub area: f32,
}

/// ⭐ W6e — os limiares da cunha, MEDIDOS (`medida_luz2d_w6d_canto`/`_borda`, handoff W6e §2): o texel do
/// canto sai igual (0,601) com `pre` 0,02–0,10 e `area` 0,01–0,05; `(0,02, 0,05)` erra 5 anéis do disco
/// no zoom 0,8 (o arco passa por canto), `(0,05, 0,02)` nenhum.
/// ⭐ W6g r9 — 1 slot da `V` e da `V2` por 32 blocos da captura: 7 680 a 1080p (3,1 % dos blocos), contra
/// 2 451 / 3 250 da cena 2 do smoke (a de mais vértices; handoff W6g §25) — 7,9 MB nos dois atlas.
pub const CANTOS_POR_BLOCOS_PADRAO: u32 = 32;

pub const VERTICE_PADRAO: Vertice = Vertice {
    pre: 0.05,
    area: 0.02,
};

/// ⭐ W5c — as escolhas da cascata holográfica.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Holografica {
    /// O passo dos traços curtos (T0–T2), em texels.
    pub passo: f32,
    /// O filtro em cruz da Eq. 21 (sondas de v par e ímpar não conversam).
    pub borrao: bool,
    /// O «+1» da Fig. 12 do artigo: a fluência do texel lê-se na sonda um adiante na direção do
    /// quadrante (salvo quando ela cai dentro de um opaco).
    pub mais_um: bool,
}

/// ⭐ W5d — a meia-largura da média da c0 ao longo da face (texels das cascatas) — MEDIDA (rodada
/// `medida_luz2d_w5d`, o disco e a elipse da cena 5 com o rebote): 1 e 2 alisam o interior sem custo
/// (a elipse a 0,9 do raio: excesso de variação +0,02 → −0,17 → −0,26); 2 apaga os riscos na foto. Pela
/// MEDIANA (o `Config::c0_ao_longo_mediana`): a média espalhava a luz que a cascata vaza junto a um canto
/// (o portão `junto_a_um_canto_a_face_le_a_sua_luz`: a 2 · 3 texels do canto, 0,15 · 0,02 sem nada →
/// 0,30 · 0,17 com a média → 0,11 · 0,02 com a mediana), ao preço de menos riscos tirados (a elipse
/// com o rebote 1,109 → 0,981 com a média, → 1,033 com a mediana).
pub const C0_AO_LONGO_DA_FACE_PADRAO: f32 = 2.0;

/// A cascata holográfica por omissão — ver a rodada `medida_luz2d_w5c`.
pub const HOLOGRAFICA_PADRAO: Holografica = Holografica {
    passo: 0.5,
    borrao: true,
    mais_um: true,
};

/// ⭐ W5c — que raios bilineares a fusão deixa de fora junto a um opaco (o `MASCARA_DA_FUSAO` do
/// `c2_rc.wgsl`); os pesos dos que ficam renormalizam-se.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MascaraDaFusao {
    /// A bilinear de sempre: o raio até uma sonda de cima de DENTRO de uma parede vê a parede.
    Nenhuma,
    /// Sem as sondas de cima que caem dentro de um opaco.
    PelaSonda,
    /// Sem os raios de uma sonda de cima DENTRO de um opaco cujo FIM (dentro/fora de um opaco)
    /// discorda do fim do raio verdadeiro.
    PeloFim,
}

impl MascaraDaFusao {
    pub(crate) fn codigo(self) -> f64 {
        match self {
            Self::Nenhuma => 0.0,
            Self::PelaSonda => 1.0,
            Self::PeloFim => 2.0,
        }
    }
}
