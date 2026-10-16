//! A configuração do solver: a grade ANCORADA NO MUNDO e as escolhas que a rodada de medição fixou.

use crate::solver::ErroSolver;

/// A grade da luz. O texel `(i, j)` cobre `origem + [i, i+1)·texel × [j, j+1)·texel` em metros do
/// mundo; a sonda da c0 é o centro dele. A linha `j` cresce com o `y` do mundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    pub origem_m: [f64; 2],
    pub texel_m: f64,
    pub largura: u32,
    pub altura: u32,
}

impl Grade {
    /// O centro do texel `(i, j)`, no mundo.
    #[must_use]
    pub fn centro(&self, i: u32, j: u32) -> [f64; 2] {
        [
            self.origem_m[0] + (f64::from(i) + 0.5) * self.texel_m,
            self.origem_m[1] + (f64::from(j) + 0.5) * self.texel_m,
        ]
    }

    /// Um ponto do mundo em coordenadas de texel (o centro do texel `(i, j)` é `(i+½, j+½)`).
    #[must_use]
    pub fn em_texels(&self, p: [f32; 2]) -> [f64; 2] {
        [
            (f64::from(p[0]) - self.origem_m[0]) / self.texel_m,
            (f64::from(p[1]) - self.origem_m[1]) / self.texel_m,
        ]
    }
}

/// Como a cascata `i` funde a de cima (plano §5, K6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fusao {
    /// RC de Sannikov sem correção: um raio por direção, a radiância de cima interpolada.
    Classica,
    /// *Bilinear fix*: quatro raios por direção, cada um até à sonda de cima que ele funde.
    CorrecaoBilinear,
    /// Um raio por direção, até UMA das quatro sondas de cima, escolhida num padrão que alterna
    /// por sonda e por subdireção (a nossa reconstrução do *nearest-interlaced*, medida, não portada).
    VizinhaIntercalada,
}

impl Fusao {
    /// O valor da constante `override FUSAO` do `c2_rc.wgsl`.
    pub(crate) fn codigo(self) -> f64 {
        match self {
            Self::Classica => 0.0,
            Self::CorrecaoBilinear => 1.0,
            Self::VizinhaIntercalada => 2.0,
        }
    }
}

/// ⭐ Como a luz DIRETA (C3) chega à grade fina (W4b).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Refino {
    /// A visibilidade de cada lâmpada medida em TODO texel fino (uma marcha por texel e lâmpada).
    Tudo,
    /// A visibilidade das sondas da c0 interpolada; medida no texel fino só nas BORDAS.
    Bordas(Bordas),
}

/// Onde o refino pelas bordas mede a visibilidade no texel fino em vez de a interpolar — as
/// escolhas que a rodada da W4b fixou ([`REFINO_PADRAO`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bordas {
    /// Os quatro cantos da célula diferem mais que isto.
    pub limiar: f32,
    /// A célula mistura o fim da penumbra (0 ou 1) com o meio dela: o JOELHO que a interpolação
    /// corta.
    pub joelho: bool,
    /// ⭐ W5a (F2): a visibilidade das sondas CURVA mais que o limiar em volta da célula (a 2.ª
    /// diferença): o vale de um oclusor mais estreito que a célula, com os cantos iguais.
    pub curvatura: bool,
    /// O passe grosso marcha no passo FINO (senão no das cascatas).
    pub visibilidade_no_passo_fino: bool,
    /// Os raios fixos da abertura (nas sondas e no texel refinado).
    pub raios: u32,
    /// As bissecções da borda na abertura (nas sondas e no texel refinado): a borda fica a
    /// 1/(raios·2^bissecções) da abertura.
    pub bissecoes: u32,
}

/// Precisão de um caminho do solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precisao {
    F16,
    F32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub grade: Grade,
    /// Comprimento do intervalo da c0, em texels; o da cascata `i` é `intervalo_c0 · 4^i`.
    pub intervalo_c0: f32,
    /// Quantos texels finos por célula do campo de distância (C1).
    pub fator_jfa: u32,
    /// O passo da marcha perto do conteúdo, em texels.
    pub passo_perto: f32,
    pub fusao: Fusao,
    /// A aritmética da marcha e da fusão. `F16` exige `SHADER_F16` no dispositivo.
    pub aritmetica: Precisao,
    /// O formato das cascatas: `Rgba16Float` ou `Rgba32Float`.
    pub armazenamento: Precisao,
    /// ⭐ Texels FINOS por texel das cascatas (potência de 2): a fluência final — as cascatas
    /// interpoladas mais a luz direta das lâmpadas — vive na grade fina ([`Self::grade_da_fluencia`]).
    pub fina: u32,
    pub refino: Refino,
    /// ⭐ W5a (F1, F2): texels da captura onde a luz DIRETA marcha por texel das cascatas
    /// (potência de 2). Com mais de 1, uma 2.ª captura só dos OPACOS (cobertura e τ dentro): a das
    /// cascatas arredonda o canto do oclusor em ½–¾ do texel dela e perde o oclusor mais fino que
    /// ~1 texel. 1 = a das cascatas.
    pub captura: u32,
    /// ⭐ W5a (F1): o passo da marcha da luz DIRETA, em texels FINOS (× o `passo_perto`), e a fração
    /// dele onde a captura tem cobertura (a menos de um texel de uma borda — o raio que roça o canto
    /// não o salta).
    pub passo_direto: f32,
    pub passo_na_borda: f32,
    /// ⭐ W5a (F3): o INTERIOR de um opaco recebe a fluência do texel fino exterior mais próximo — um
    /// JFA do complemento em células de `n` texels finos (potência de 2); `None` = o interior fica 0
    /// e a bilinear do receptor mistura-o na face acesa.
    pub preenchimento: Option<u32>,
    /// O preenchimento corre na grade fina INTEIRA (senão só na região da janela: o interior de um
    /// opaco cortado pela borda dela acha o exterior mais próximo só do lado de dentro).
    pub preenche_a_grade_inteira: bool,
    /// ⭐ W5b (F4): a luz DIRETA vê os opacos FORA da grade no caminho até uma lâmpada que não cabe
    /// nela (`crate::distantes`); `false` = só a captura (o raio atravessa o que está fora).
    pub distantes: bool,
    /// ⭐ W5b (F4): o lado, em texels, da MÁSCARA do alfa de um sprite distante na moldura dele
    /// (potência de 2; desce sozinho quando as máscaras não cabem na atlas).
    pub mascara_px: u32,
    /// ⭐ W5c — o REBOTE (plano §2.4): a luz que chega a um corpo opaco que recebe luz volta ao
    /// plano × o albedo dele, um rebote por quadro (a fonte do quadro anterior, lida no MUNDO).
    pub rebote: bool,
    /// ⭐ W5d — quantos rebotes por quadro (≥ 1; com a holográfica): o 2.º repete no mesmo quadro a soma
    /// na emissão (com a fonte que o 1.º acabou de escrever), a cascata e a fonte — o rebote assenta
    /// no dobro da velocidade (K9), ao preço de uma cascata a mais.
    pub rebotes_por_quadro: u32,
    /// ⭐ W5c — a fusão das cascatas junto a uma parede (o ABERTO 1 da W5a).
    pub mascara_da_fusao: MascaraDaFusao,
    /// ⭐ W5c — a fronteira entre intervalos junto a uma face (o `FRONTEIRA_NA_FACE` do
    /// `c2_rc.wgsl`): uma sonda de cima dentro de um opaco (ou sobre a face) zerava a fusão inteira e o
    /// raio de baixo que acaba dentro da face perdia a entrada nela.
    pub fronteira_na_face: bool,
    /// ⭐ W5c — o texel fino que o EIXO MEDIAL de um opaco corta guarda a média das faces sobre a área
    /// dele, medidas pela perpendicular (o `MEDIA_NO_EIXO` do `c3_direta.wgsl`); `false` = o valor do
    /// ponto de fronteira mais perto do centro (a diagonal entre faces em escada, smoke do dono 10/10).
    pub media_no_eixo: bool,
    /// ⭐ W5d — o ponto de fronteira que um texel do INTERIOR lê toma a c0 pela média AO LONGO da face,
    /// nesta meia-largura em texels das cascatas (o `C0_AO_LONGO_DA_FACE`; 0 = no ponto): as sondas
    /// coladas a uma superfície que reflete variam ±15 % de uma para a outra.
    pub c0_ao_longo_da_face: f32,
    /// ⭐ W5d — a MEDIANA dos pontos ao longo da face (o `C0_AO_LONGO_MEDIANA`), não a média.
    pub c0_ao_longo_mediana: bool,
    /// ⭐ W5d — o INTERIOR de um opaco (a lei F3 e a luz lida na fronteira) decide «dentro» pela
    /// borda EXATA da cobertura (o `c3_borda.wgsl`, o AAEDT; o `BORDA_EXATA` do `c2_rc.wgsl`);
    /// `false` = o iso-½ da bilinear, que se desvia até ~0,08 texel (a passagem luz/sombra serrilhada
    /// das formas curvas, cena 5).
    pub borda_exata: bool,
    /// ⭐ W5g (F5) — o texel do INTERIOR lê a luz no PÉ dele na borda exata (o `PE_NA_BORDA` do
    /// `c3_pe.wgsl`; só com [`Self::borda_exata`]); `false` = o ponto de fronteira da busca, que salta
    /// ao longo da face com a fase da grade (os degraus nos pontos tangentes dos discos, cena 7).
    pub pe_na_borda: bool,
    /// ⭐ W5g (F5) — a lâmpada vista do ponto de fronteira de um texel do INTERIOR: a face tapa a
    /// abertura atrás da tangente (conta exata) e a marcha parte de meio texel da captura para fora (o
    /// `VIS_NA_FACE` do `c3_pe.wgsl`); `false` = a marcha a partir do ponto rente, cujos raios
    /// rasantes saem ora livres ora tapados com a fase da grade.
    pub vis_na_face: bool,
    /// ⭐ W5o — com [`Self::vis_na_face`], a abertura e os raios saem do próprio ponto da face e cada
    /// raio só se testa a partir de onde está a `AFASTA_NA_FACE` do plano dela (o `FACE_PELO_PONTO` do
    /// `c3_pe.wgsl`); `false` = a marcha a partir do ponto afastado (a penumbra deslocada).
    pub face_pelo_ponto: bool,
    /// ⭐ W5h — o MIOLO dos opacos pela DIFUSÃO (`c3_miolo.wgsl`, a lei do
    /// `ph2d_light2d_ref::OpcoesDoMiolo`); só com o preenchimento (que marca o interior). `None` = o
    /// exterior mais próximo em todo o miolo (a lei até a W5g).
    pub miolo: Option<Miolo>,
    /// ⭐ W5n — com a região menor que a grade, o miolo resolve-se no corpo INTEIRO: o anel de fora da
    /// região (`c3_anel.wgsl`) e a pirâmide na grade inteira nos níveis que a região lê; `false` = só
    /// na região, com Neumann na borda dela (a W5h: o interior mudava com o enquadramento).
    pub anel_fora_da_regiao: bool,
    /// ⭐ W5q — o anel só nos ladrilhos 8×8 com corpo da captura fina (ou a janela); `false` = na
    /// grade inteira (o antes: o mesmo bit, com um teste de corpo e uma escrita em cada texel).
    pub anel_por_ladrilhos: bool,
    /// ⭐ W5v — com o miolo e a suavidade 1, a fina SALTA o fundo dos opacos (os grupos 8×8 com todo texel e
    /// as 4 vizinhas dentro, com semente na célula) e o `miolo_final` dá a difusão EXATA no lugar do `mix`
    /// (que arredondava pelo valor do fundo: 1 ulp f16 num texel em 42 casos); `false` = o antes, ao bit.
    pub miolo_salta_o_fundo: bool,
    /// ⭐ W5o — ninguém lê a FLUÊNCIA deste solver (o entorno: só a c0 dele soma-se à da vista): a
    /// visibilidade das lâmpadas, a grade fina, o anel e o miolo não correm (nem alocam).
    pub so_a_c0: bool,
    /// ⭐ W5c — a CASCATA HOLOGRÁFICA (HRC, a do produto em todo nível) no lugar das cascatas de Sannikov: a mesma
    /// c0, outra conta (`c2_hrc.wgsl`). `None` = as cascatas.
    pub holografica: Option<Holografica>,
    /// ⭐ W5p — INSTRUMENTO: o texto do módulo das cascatas por cima do da casa
    /// ([`crate::fonte_do_rc`]) — a régua «antes × depois» de uma reescrita do WGSL no MESMO processo
    /// (o compilador do driver, o relógio da placa e a imagem). `None` no produto.
    pub modulo_rc: Option<&'static str>,
    /// ⭐ W5p — os buffers da cascata holográfica nascem, largam-se e nascem de novo (em memória já usada
    /// e largada): a `c2` do High num processo novo 24,7 → 19,0 ms (`medida_luz2d_w5o_lastro`, handoff W5p
    /// §7). `false` = uma vez (`PH2D_LIGHT2D_HRC_DUAS=0` no app compara).
    pub hrc_nasce_duas_vezes: bool,
    /// ⭐ W5x — com [`Self::hrc_nasce_duas_vezes`], o 1.º nascimento é um BERÇO de texturas cruas com os
    /// bytes de cada buffer: o driver da NVIDIA deixa COMPRIMÍVEIS as páginas onde uma textura foi ligada,
    /// e o buffer que nasce no buraco dela lê os seus zeros sem tocar na DRAM (handoff W5x). `false` = o
    /// 1.º nascimento em buffers, o da W5p (`PH2D_LIGHT2D_HRC_BERCO=0` no app compara).
    pub hrc_berco: bool,
    /// ⭐ W5x — INSTRUMENTO: a holográfica guarda os seus buffers grandes para
    /// [`crate::Solver::buffers_da_holografica`] (as réguas do regime). `false` no produto: guardá-los
    /// atrasava a devolução da memória quando a placa recusa (`a_luz_recua_quando_a_placa_recusa_a_memoria`).
    pub hrc_guarda_os_grandes: bool,
    /// ⭐ W5r — os T e os R da cascata holográfica por COLUNAS (`v` o índice mais rápido): as leituras
    /// em diagonal ficam contíguas — o mesmo bit, outros endereços (handoff W5r). `false` = por linhas.
    pub hrc_por_colunas: bool,
    /// ⭐ W5r — o arco de cada cone numa tabela que a placa escreve uma vez (não dois `atan` por fio na
    /// descida): o mesmo bit. `false` = recalculado por fio.
    pub hrc_arcos_em_tabela: bool,
    /// ⭐ W5s — o traço curto (T0–T2) que só cruza ladrilhos SEM cobertura devolve o vácuo sem marchar; a
    /// máscara sai da própria captura, na placa (handoff W5s): o mesmo bit. `false` = marcha sempre.
    pub hrc_salta_o_vazio: bool,
    /// ⭐ W5s — um fio por SONDA de T0–T2 (os intervalos dela num laço, o meio na ponta lido uma vez; o
    /// salto decidido pelo leque inteiro): o mesmo bit. `false` = um fio por intervalo.
    pub hrc_fio_por_sonda: bool,
    /// ⭐ W5t — a traça, a subida e a descida da holográfica despachadas em 3D (`(v, xi, k)` são o índice do
    /// fio, sem as divisões do 1D; só por colunas): o mesmo bit. `false` = 1D dobrado.
    pub hrc_grade_3d: bool,
    /// ⭐ W5u (só em 3D) — cones vizinhos por fio na descida (o T da aresta comum lido uma vez; `1` = um
    /// cone por fio) e pares (k par, k ímpar) por fio na subida (`0` = uma entrada por fio): o mesmo bit;
    /// o High vazio −17 %, o Medium −12 %, o Low −4 % (handoff W5u). Só nas grades de pelo menos
    /// `hrc::POR_FIO_DESDE` texels (a decisão vai no uniforme: redimensionar não recompila).
    pub hrc_cones_por_fio: u32,
    pub hrc_pares_na_subida: u32,
}

/// ⭐ W5h — o maior [`Config::captura`]: as subamostras da C0 grossa (o `validar` recusa acima).
pub const CAPTURA_MAX: u32 = crate::captura::SUB;

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

/// O lado da máscara de um sprite distante por omissão (W5b) — ver a rodada `medida_luz2d_w5b`.
pub const MASCARA_PX_PADRAO: u32 = 256;

/// O refino por omissão — MEDIDO (rodada `medida_luz2d_w4b`, RTX 5060 Ti, 1920×1080, grade fina de
/// 1 px; tabela no handoff W4b). Régua: a escada da cena 4 (degrau dos contornos 2 · 10 · 50 · 90 ·
/// 98 %, do canto até o muro) e o custo por lâmpada (Low · Medium):
/// - o limiar 0,1 (0,25 deixava o meio da penumbra interpolado: Low 0,51 px no de 10 %);
/// - o `joelho`: sem ele uma célula de cantos «0, 0, 0, 0,08» era interpolada e desenhava MANCHAS de
///   até 8 % de luz sobre o preto (vistas na FOTO; a régua de 10 · 50 · 90 % não as via): contorno de
///   2 % no Low 0,352 → 0,165 px;
/// - a visibilidade das sondas no passo FINO: no grosso o tremor da marcha entra onde a penumbra é
///   larga e se interpola (Medium 0,26 · 0,31 · 0,22 px → 0,18 · 0,12 · 0,08; a penumbra da lâmpada
///   grande também em Low), por +30 % de custo;
/// - 4 raios + 5 bissecções: 8+5 custava +50 % pela mesma régua (o exato); 2+6 mede igual e custa
///   −20 %, mas com 3 raios fixos um oclusor mais fino que MEIA abertura da lâmpada passa sem sombra
///   (com 5, um quarto) — fica a robustez;
/// - certificar a sonda (cone livre com a folga de uma célula): empate de custo E de resposta — o fio de
///   sombra de um oclusor de 0,8 texel entre as sondas dá 0,515 com, sem, e no exato (o `livre` já é
///   conservador em ~2 texels). Saiu: a mutação dele sobrevivia.
///
/// Resultado: Low 0,163 · 0,105 · 0,085 · 0,139 · 0,182 px · Medium 0,176 · 0,123 · 0,079 · 0,122 ·
/// 0,170 a 0,69 · 0,67 ms por lâmpada (o exato em todo texel: a mesma régua a 1,22 · 1,11 ms).
///
/// ⭐ W5a — a `curvatura` (F2): o vale de um oclusor de 5 cm mais estreito que a célula das sondas
/// saía com o fundo a 0,425 em vez de 0,168 (Low, a lâmpada pequena); com ela 0,172, a custo ~0.
pub const REFINO_PADRAO: Refino = Refino::Bordas(Bordas {
    limiar: 0.1,
    joelho: true,
    curvatura: true,
    visibilidade_no_passo_fino: true,
    raios: 4,
    bissecoes: 5,
});

/// O passo da luz direta por omissão (W5a) — MEDIDO (rodada `medida_luz2d_w5a`, 1920×1080, nível
/// Baixo; tabela no handoff W5a): o DEGRAU da sombra é do passo (com a captura fina e o passo de 1
/// texel fino em tudo, o contorno de 2 % ia a 0,237 px e o de 10 % a 0,194); o passo de ½ texel em
/// tudo cura-o (0,168 · 0,103) a +75 % por lâmpada; o de ½ só JUNTO À BORDA, a +7 %; ¼ junto à borda
/// não melhora (0,164 · 0,103) e custa +28 %.
pub const PASSO_DIRETO_PADRAO: f32 = 1.0;
/// Ver [`PASSO_DIRETO_PADRAO`].
pub const PASSO_NA_BORDA_PADRAO: f32 = 0.5;

/// O preenchimento por omissão (W5a, F3) — MEDIDO: a face acesa da parede da cena 1 sai 0,983 (Low) ·
/// 0,992 do gabarito com células de 1, 2, 4 ou 8 (sem ele 0,446: a linha escura); o custo do JFA na
/// grade inteira 0,29 (célula 2) · 0,17 (4) · 0,19 (8) ms, só na região 0,05. A célula serve só o
/// interior FUNDO: junto à borda o texel acha o ponto de fronteira mais perto numa janela de 5×5.
/// Corre na grade INTEIRA ([`Config::preenche_a_grade_inteira`]): só na região, o interior de um
/// opaco cortado pela borda da janela acharia o exterior só do lado de dentro e mudaria de cor no pan.
pub const PREENCHIMENTO_PADRAO: Option<u32> = Some(4);

/// Onde a cascata `i` começa, em múltiplos de `intervalo_c0`: `(4^i − 1)/3`.
fn inicio_relativo(i: u32) -> f64 {
    (4f64.powi(i32::try_from(i).unwrap_or(i32::MAX)) - 1.0) / 3.0
}

/// A menor contagem de cascatas cujo topo passa a diagonal da grade: todo raio sai da grade antes
/// de acabar, e o que escapa recebe o céu.
fn cascatas_para(l0: f64, largura: u32, altura: u32) -> u32 {
    let diagonal = f64::from(largura).hypot(f64::from(altura));
    let mut n = 1;
    while l0 * inicio_relativo(n) < diagonal {
        n += 1;
    }
    n
}

impl Config {
    /// A configuração padrão para `grade` — as escolhas da rodada de medição da W2 (a tabela está no
    /// handoff `docs/Luz2D/handoffs/HANDOFF_CONTINUACAO_W2_2026-10-08.md`, RTX 5060 Ti, 512²):
    /// - `CorrecaoBilinear`: custa 3,1× a clássica (K6 > 2×); a vizinha intercalada, medida como o
    ///   K6 manda, erra 5× mais (0,449 × 0,090) ⇒ aceita-se o custo;
    /// - `intervalo_c0` 1 — erro (penumbra · céu e vidro) e C2: ½ → 0,095 · 0,021 a 1,72 ms (+1
    ///   cascata); **1 → 0,090 · 0,013 a 1,45 ms**; 2 → 0,116 · 0,010 a 1,36 ms;
    /// - `passo_perto` 1: o passo ½ custa +40–75 % e só ajuda no Baixo (o governador da W5 escolhe
    ///   por degrau);
    /// - `fator_jfa` 2: o 4 empata na penumbra e perde no enxame denso; o 1 custa 2,3× na C1;
    /// - cascatas `F16`: as `F32` não mudam o erro (0,0900 × 0,0902) e gastam +53 % de memória;
    /// - aritmética `F32` (portátil): a `F16` mediu IGUAL neste desktop; ver [`Self::padrao_para`].
    #[must_use]
    pub fn padrao(grade: Grade) -> Self {
        Self {
            grade,
            intervalo_c0: 1.0,
            fator_jfa: 2,
            passo_perto: 1.0,
            fusao: Fusao::CorrecaoBilinear,
            aritmetica: Precisao::F32,
            armazenamento: Precisao::F16,
            fina: 1,
            refino: REFINO_PADRAO,
            captura: 1,
            passo_direto: PASSO_DIRETO_PADRAO,
            passo_na_borda: PASSO_NA_BORDA_PADRAO,
            preenchimento: PREENCHIMENTO_PADRAO,
            preenche_a_grade_inteira: true,
            distantes: true,
            mascara_px: MASCARA_PX_PADRAO,
            rebote: false,
            rebotes_por_quadro: 1,
            mascara_da_fusao: MascaraDaFusao::PeloFim,
            fronteira_na_face: true,
            media_no_eixo: true,
            c0_ao_longo_da_face: C0_AO_LONGO_DA_FACE_PADRAO,
            c0_ao_longo_mediana: true,
            borda_exata: true,
            pe_na_borda: true,
            vis_na_face: true,
            face_pelo_ponto: true,
            miolo: Some(MIOLO_PADRAO),
            anel_fora_da_regiao: true,
            anel_por_ladrilhos: true,
            miolo_salta_o_fundo: true,
            so_a_c0: false,
            holografica: None,
            modulo_rc: None,
            hrc_nasce_duas_vezes: true,
            hrc_berco: true,
            hrc_guarda_os_grandes: false,
            hrc_por_colunas: true,
            hrc_arcos_em_tabela: true,
            hrc_salta_o_vazio: true,
            hrc_fio_por_sonda: true,
            hrc_grade_3d: true,
            hrc_cones_por_fio: 4,
            hrc_pares_na_subida: 4,
        }
    }

    /// ⭐ Texels da captura onde a luz direta marcha por texel das cascatas ([`Self::captura`]).
    #[must_use]
    pub fn escala_da_captura(&self) -> u32 {
        self.captura
    }

    /// A luz direta marcha numa captura PRÓPRIA (mais fina que a das cascatas)?
    #[must_use]
    pub fn usa_captura_fina(&self) -> bool {
        self.escala_da_captura() > 1
    }

    /// A grade da captura fina (a efetiva, com o texel ÷ [`Self::escala_da_captura`] e a MESMA
    /// origem).
    #[must_use]
    pub fn grade_da_captura(&self) -> Grade {
        let g = self.grade_efetiva();
        let e = self.escala_da_captura();
        Grade {
            origem_m: g.origem_m,
            texel_m: g.texel_m / f64::from(e),
            largura: g.largura * e,
            altura: g.altura * e,
        }
    }

    /// As células do JFA do preenchimento (F3) sobre a grade fina, e os saltos dele (do maior ao 1,
    /// mais um de 1 a fechar).
    #[must_use]
    pub fn grade_do_preenchimento(&self) -> Option<([u32; 2], Vec<u32>)> {
        let f = self.preenchimento?;
        let g = self.grade_da_fluencia();
        let celulas = [g.largura.div_ceil(f), g.altura.div_ceil(f)];
        let mut k = celulas[0].max(celulas[1]).next_power_of_two() / 2;
        let mut saltos = Vec::new();
        while k >= 1 {
            saltos.push(k);
            k /= 2;
        }
        saltos.push(1);
        Some((celulas, saltos))
    }

    /// ⭐ W5h — os níveis da pirâmide do miolo acima da grade fina (o mais grosso com 4–7 texels no
    /// lado maior); `None` sem o miolo, sem o preenchimento (que marca o interior), com
    /// [`Self::so_a_c0`] ou numa grade fina de menos de 8 texels.
    #[must_use]
    pub fn niveis_do_miolo(&self) -> Option<u32> {
        self.miolo.filter(|_| !self.so_a_c0)?;
        self.preenchimento?;
        let g = self.grade_da_fluencia();
        let l = g.largura.max(g.altura).max(1).ilog2().checked_sub(2)?;
        (l >= 1).then_some(l)
    }

    /// O padrão com aritmética f16 onde o dispositivo tem `SHADER_F16` (o caminho f32 é o mesmo
    /// algoritmo; o solver RECUSA f16 sem a feature, nunca troca em silêncio).
    #[must_use]
    pub fn padrao_para(device: &wgpu::Device, grade: Grade) -> Self {
        let f16 = device.features().contains(wgpu::Features::SHADER_F16);
        Self {
            aritmetica: if f16 { Precisao::F16 } else { Precisao::F32 },
            ..Self::padrao(grade)
        }
    }

    /// A grade que o solver de facto usa: a pedida, estendida à direita e em cima até múltiplos de
    /// `2^(n−1)` (o arranjo direção-primeiro da cascata de topo precisa de blocos inteiros).
    #[must_use]
    pub fn grade_efetiva(&self) -> Grade {
        let l0 = f64::from(self.intervalo_c0);
        let mut g = self.grade;
        loop {
            let bloco = 1u32 << (cascatas_para(l0, g.largura, g.altura) - 1);
            let (w, h) = (
                g.largura.div_ceil(bloco) * bloco,
                g.altura.div_ceil(bloco) * bloco,
            );
            if (w, h) == (g.largura, g.altura) {
                return g;
            }
            (g.largura, g.altura) = (w, h);
        }
    }

    /// ⭐ A grade onde a FLUÊNCIA final vive (a que os receptores leem): a efetiva, com o texel
    /// ÷ [`Self::fina`] e a MESMA origem — a âncora do G11 vale para as duas.
    #[must_use]
    pub fn grade_da_fluencia(&self) -> Grade {
        let g = self.grade_efetiva();
        Grade {
            origem_m: g.origem_m,
            texel_m: g.texel_m / f64::from(self.fina),
            largura: g.largura * self.fina,
            altura: g.altura * self.fina,
        }
    }

    /// A configuração é aceitável? (o solver recusa-a antes de alocar nada)
    ///
    /// # Errors
    /// [`ErroSolver::ConfigInvalida`] com o campo que a recusa.
    pub(crate) fn validar(&self) -> Result<(), ErroSolver> {
        let cfg = self;
        let g = &cfg.grade;
        if !(g.texel_m.is_finite() && g.texel_m > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "texel_m tem de ser finito e > 0",
            ));
        }
        if !(g.origem_m[0].is_finite() && g.origem_m[1].is_finite()) {
            return Err(ErroSolver::ConfigInvalida("origem_m tem de ser finita"));
        }
        if g.largura == 0 || g.altura == 0 {
            return Err(ErroSolver::ConfigInvalida(
                "a grade tem de ter ao menos um texel",
            ));
        }
        if !(cfg.intervalo_c0.is_finite() && cfg.intervalo_c0 > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "intervalo_c0 tem de ser finito e > 0",
            ));
        }
        if cfg.fator_jfa == 0 {
            return Err(ErroSolver::ConfigInvalida("fator_jfa tem de ser ≥ 1"));
        }
        if !(cfg.passo_perto.is_finite() && cfg.passo_perto > 0.0) {
            return Err(ErroSolver::ConfigInvalida(
                "passo_perto tem de ser finito e > 0",
            ));
        }
        if !cfg.fina.is_power_of_two() {
            return Err(ErroSolver::ConfigInvalida(
                "fina tem de ser uma potência de 2",
            ));
        }
        if !(cfg.passo_direto.is_finite()
            && cfg.passo_direto > 0.0
            && cfg.passo_na_borda.is_finite()
            && cfg.passo_na_borda > 0.0)
        {
            return Err(ErroSolver::ConfigInvalida(
                "passo_direto e passo_na_borda têm de ser finitos e > 0",
            ));
        }
        // ⭐ Até as 8 subamostras da C0 grossa: um oclusor que POSSUI um texel da captura fina cobre
        // o centro dele, que é uma subamostra da grossa (onde ele entra como cobertura, mesmo menor
        // que o texel) — e é a grossa que semeia o campo de distância (W5a: ler a fina na semente
        // custava 0,71 ms; uma marca paralela, 0,17).
        if !cfg.captura.is_power_of_two() || cfg.captura > crate::captura::SUB {
            return Err(ErroSolver::ConfigInvalida(
                "captura tem de ser uma potência de 2 até as 8 subamostras da C0",
            ));
        }
        if cfg.preenchimento.is_some_and(|f| !f.is_power_of_two()) {
            return Err(ErroSolver::ConfigInvalida(
                "preenchimento: a célula tem de ser uma potência de 2",
            ));
        }
        if !cfg.mascara_px.is_power_of_two() || !(8..=4096).contains(&cfg.mascara_px) {
            return Err(ErroSolver::ConfigInvalida(
                "mascara_px tem de ser uma potência de 2 entre 8 e 4096",
            ));
        }
        if let Refino::Bordas(b) = cfg.refino
            && (!(0.0..=1.0).contains(&b.limiar) || b.raios == 0)
        {
            return Err(ErroSolver::ConfigInvalida(
                "refino: limiar fora de [0, 1] ou sem raios",
            ));
        }
        Ok(())
    }

    /// O passe grosso da visibilidade corre? Só com a grade fina MAIS fina que a das cascatas e o
    /// refino pelas bordas (com `fina = 1` o passe fino mede cada texel: é a sonda da c0).
    #[must_use]
    pub fn usa_visibilidade(&self) -> bool {
        self.fina > 1 && matches!(self.refino, Refino::Bordas(_))
    }

    #[must_use]
    pub fn n_cascatas(&self) -> u32 {
        let g = self.grade_efetiva();
        cascatas_para(f64::from(self.intervalo_c0), g.largura, g.altura)
    }

    /// `[início, fim)` do intervalo da cascata `nivel`, em texels.
    #[must_use]
    pub fn intervalo(&self, nivel: u32) -> (f32, f32) {
        let l0 = f64::from(self.intervalo_c0);
        (
            (l0 * inicio_relativo(nivel)) as f32,
            (l0 * inicio_relativo(nivel + 1)) as f32,
        )
    }

    /// As dimensões do campo de distância (C1).
    #[must_use]
    pub fn grade_jfa(&self) -> [u32; 2] {
        let g = self.grade_efetiva();
        [
            g.largura.div_ceil(self.fator_jfa),
            g.altura.div_ceil(self.fator_jfa),
        ]
    }

    /// Os saltos do JFA, do maior ao 1, mais um passo de 1 a fechar (JFA+1).
    #[must_use]
    pub fn saltos_jfa(&self) -> Vec<u32> {
        let [w, h] = self.grade_jfa();
        let mut k = w.max(h).next_power_of_two() / 2;
        let mut saltos = Vec::new();
        while k >= 1 {
            saltos.push(k);
            k /= 2;
        }
        saltos.push(1);
        saltos
    }
}
