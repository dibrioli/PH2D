//! ⭐⭐⭐ **O GESTO da referência** — a área simulada, a banda graduada, o factor por
//! vértice e os oito modos, portados clean-room da
//! [espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md) §2 e §4.
//!
//! ⚠️ **Este módulo não sabe o que é uma malha nem um pincel.** Recebe posições,
//! normais, o anel-1 de cada vértice e o cursor; devolve, por passo, o que o
//! solver precisa (forças, âncoras, desvios de repouso) e corre-o. É o adaptador
//! do `ph2d-sculpt3d` que traduz `Brush`/`Dab`/`Mesh` para isto.

use crate::V3;
use crate::verlet::{Solver, Verlet, dist, norm, unit};

/// Os oito tipos de deformação (espec §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    /// Força na direcção UNITÁRIA do movimento do cursor — a mesma para todos.
    Arrastar,
    /// Força para DENTRO, ao longo da normal da área, com magnitude `2R`.
    Empurrar,
    /// Força unitária do vértice PARA o cursor.
    ApertarPonto,
    /// Força para a LINHA do traço (só as componentes perpendiculares).
    ApertarLinha,
    /// Força ao longo da normal do vértice, para fora.
    Inflar,
    /// Âncora `p⁰ + δ_total · f`, pegada congelada na malha de partida.
    Agarrar,
    /// Âncora `x + δ_incremental · f`, re-pegada a cada passo.
    Gancho,
    /// Desvio do comprimento de repouso, `τ += 0,01 · f` por passo.
    Expandir,
}

/// A área simulada (espec §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    /// Esfera de raio `R₀(1+L)` centrada na localização inicial do traço.
    Local,
    /// Tudo; `w ≡ 1`.
    Global,
    /// Esfera de raio `R(1+L)` centrada no cursor actual.
    Dinamica,
}

/// A forma espacial do peso da força (espec §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FalloffForca {
    Radial,
    Plano,
}

/// A curva de falloff do pincel (espec §4.1). Só as que o oráculo usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curva {
    /// `3u² − 2u³`, `u = 1 − d/R`.
    Suave,
    /// `u²`.
    Aguda,
    /// `1`.
    Constante,
}

impl Curva {
    /// O peso em `[0, 1]` de uma distância `d` num raio `r`.
    #[must_use]
    pub fn peso(self, d: f64, r: f64) -> f64 {
        if r <= 0.0 || d >= r {
            return 0.0;
        }
        let u = 1.0 - d / r;
        match self {
            Self::Suave => 3.0 * u * u - 2.0 * u * u * u,
            Self::Aguda => u * u,
            Self::Constante => 1.0,
        }
    }
}

/// Os controlos do pincel de tecido (espec §8.1 — as omissões do CÓDIGO).
#[derive(Clone, Copy, Debug)]
pub struct Pincel {
    pub modo: Modo,
    pub area: Area,
    pub falloff_forca: FalloffForca,
    pub curva: Curva,
    /// O raio, em unidades de objecto.
    pub raio: f64,
    /// *Strength* em `[0,1]` — ⚠️ o alvo eleva ao QUADRADO nos modos de força.
    pub forca: f64,
    /// A dureza `h ∈ [0,1]`: distância `< h·R` ⇒ peso `1`.
    pub dureza: f64,
    /// *Simulation Limit* `L` (omissão `2,5`).
    pub limite: f64,
    /// *Simulation Falloff* `F` (omissão `0,75`).
    pub banda: f64,
    /// *Pin Simulation Boundary* (só *Local*; omissão `false`).
    pub pino: bool,
    /// `flip = ±1` (Add/Subtract).
    pub flip: f64,
    pub solver: Solver,
    /// ⚠️ **Experimento de paridade** — a escala do `limite` que o leitor de
    /// banda das RESTRIÇÕES (`φ`) usa: `1` = a espec §2.2 (o mesmo `R(1+L)` da
    /// força). Medido em 2026-09-06 porque nenhum conjunto de restrições
    /// reproduz Local e Global ao mesmo tempo.
    pub escala_phi: f64,
    /// Idem para o leitor da RETENÇÃO de velocidade.
    pub escala_retencao: f64,
    /// **Quantas PASSAGENS o traço faz por passo** (as cópias de simetria; `1`
    /// = sem simetria). Só a área *Local* a lê, e é ela que decide quantas
    /// vezes a lista de restrições é construída — ver [`Self::construcoes`].
    pub passagens: u32,
}

impl Pincel {
    /// **Quantas vezes a lista de restrições é construída** (espec, emenda Q8):
    /// na área *Local* são `passagens + 1`; nas outras é uma só.
    ///
    /// ⚠️ **É daqui que sai a rigidez que separava a *Local* da *Global***, e
    /// não de uma contagem de varreduras: os dois ramos relaxam o mesmo número
    /// de vezes, mas na *Local* cada restrição está na lista `passagens + 1`
    /// vezes, porque o registo de duplicados vive UMA construção
    /// ([`ph2d_cloth::verlet::Verlet::reabrir`]).
    ///
    /// ⚠️ **Sem isto a *Local* rende como a *Global*** — medido em 06/09 sobre
    /// as 50 fixtures: `plano_arrastar_radial_local` erra `125 %` com uma cópia
    /// e `7 %` com duas, e a contagem de vértices movidos passa a bater EXACTA
    /// em oito traços. ⛔ Não «optimize» a lista deduplicando-a.
    #[must_use]
    pub fn construcoes(&self) -> u32 {
        if self.area == Area::Local {
            self.passagens.max(1) + 1
        } else {
            1
        }
    }
}

impl Default for Pincel {
    fn default() -> Self {
        Self {
            modo: Modo::Arrastar,
            area: Area::Local,
            falloff_forca: FalloffForca::Radial,
            curva: Curva::Suave,
            raio: 0.35,
            forca: 1.0,
            dureza: 0.0,
            limite: 2.5,
            banda: 0.75,
            pino: false,
            flip: 1.0,
            solver: Solver::default(),
            escala_phi: 1.0,
            escala_retencao: 1.0,
            passagens: 1,
        }
    }
}

/// **O peso de BANDA** `w(p)` (espec §2.2): `1` dentro do início, `0` fora do
/// limite, *smoothstep* entre os dois.
#[must_use]
pub fn banda(p: V3, c: V3, r: f64, limite: f64, falloff: f64) -> f64 {
    let fim = r * (1.0 + limite);
    let inicio = r * (1.0 + limite * falloff);
    let d = dist(p, c);
    if d < inicio {
        1.0
    } else if d > fim || fim <= inicio {
        0.0
    } else {
        let t = 1.0 - (d - inicio) / (fim - inicio);
        3.0 * t * t - 2.0 * t * t * t
    }
}

/// O que um passo do pincel recebe de fora (espec §4).
pub struct Passo<'a> {
    /// A localização do cursor neste passo (re-apanhada na superfície nos modos
    /// de força; no pen-down para o Grab; `c + δ` para o Gancho).
    pub cursor: V3,
    /// O delta de agarrar `δ` (espec §4.3): incremental para todos os modos,
    /// TOTAL para o Grab.
    ///
    /// ⭐⭐⭐ **Ele NÃO é a diferença dos dois pontos 3D do cursor — é a PROJECÇÃO
    /// dela no plano do ECRÃ** (espec §4.3, emenda Q12). As duas des-projecções
    /// são feitas à mesma profundidade, logo numa vista ortográfica a componente
    /// ao longo do eixo da vista é descartada **por construção**. ⚠️ Numa folha
    /// plana vista de frente as duas coisas são a MESMA ao bit, e é por isso que
    /// isto só aparece em superfície curva: na esfera das fixtures são `15,83°`
    /// de direcção e até `1,039×` de módulo.
    /// ⛔ **O plano do ecrã não é o plano tangente do pen-down** — essa rota foi
    /// medida a piorar o Agarrar de `0,265` para `0,605`.
    pub delta: V3,
    /// **A diferença dos dois pontos 3D do cursor**, sem projecção.
    ///
    /// ⚠️ **Só o ARRASTO a lê**, e é dela que sai a direcção dele (espec §4.2) —
    /// é por isso que o arrasto é o único modo que se comporta igual nas duas
    /// superfícies. Os outros sete tiram tudo de [`Self::delta`].
    pub delta_3d: V3,
    /// O cursor não se mexeu no ecrã? ⇒ sem forças neste passo.
    pub parado: bool,
    /// **A direcção da SUPERFÍCIE PARA O OLHO**, unitária — o que reparte os
    /// vértices nos dois baldes da normal da área (espec §4.2-bis (4)).
    ///
    /// ⚠️ **É o oposto da direcção do raio que apanhou o cursor**: o `Dab` da
    /// casa guarda o olho a apontar para DENTRO da peça, e aqui a convenção é a
    /// da espec — `n̂ · v̂ > 0` é o balde da frente.
    pub vista: V3,
    /// As normais ACTUAIS por vértice (o Inflate lê-as).
    pub normais: &'a [V3],
    /// Pressão em `[0,1]`.
    pub pressao: f64,
}

/// **O «Normal Radius» dos presets de tecido** (espec §4.2-bis (3), proveniência
/// `A` — lido dos treze pincéis da biblioteca do alvo).
///
/// ⚠️⚠️ **A normal da área é amostrada num disco de METADE do raio do pincel.**
/// Um port que a tire do disco inteiro tem uma direcção diferente assim que a
/// superfície deixa de ser plana ou de estar em repouso — foi essa a diferença
/// que deixou o `plano_empurrar_plano_local` a errar `0,944`.
pub const RAIO_DA_NORMAL: f64 = 0.5;

/// **A NORMAL DA ÁREA** (espec §4.2-bis) — um vector por passo, e a mesma
/// grandeza que o Push usa como direcção, o referencial local usa como `ẑ` e o
/// falloff de plano usa como normal (§4.4).
///
/// ⛔ **A regra de desempate NÃO é uma média:** os vértices repartem-se em dois
/// baldes pelo sinal de `n̂ · v̂`, e a resposta é a soma normalizada do PRIMEIRO
/// balde que esteja **não-vazio E com soma de comprimento não-nulo**, nesta
/// ordem fixa — nunca a mistura, nunca «o balde com mais vértices». ⇒ basta UM
/// vértice virado para a vista para que os virados ao contrário não contem.
///
/// ⚠️ **E o teste é não-vazio E soma não-nula, balde a balde:** se as normais do
/// balde da frente se cancelarem, a resposta é a do balde de trás. ⛔ Não é
/// «escolher o balde e só depois olhar para a soma».
///
/// Sem resposta nenhuma ⇒ **vector NULO**, e o Push desse passo é força zero,
/// sem `NaN` e sem direcção de reserva (espec §4.2-bis (5)).
#[must_use]
pub fn normal_da_area(
    posicoes: &[V3],
    normais: &[V3],
    dentro: &[u32],
    cursor: V3,
    raio: f64,
    vista: V3,
) -> V3 {
    normal_e_centro_da_area(posicoes, normais, dentro, cursor, raio, vista).0
}

/// **A NORMAL e o CENTRO DA ÁREA, da MESMA varredura** (espec §4.2-bis e §4.4).
///
/// ⭐⭐⭐ **O centro da área NÃO é o centroide do disco.** Cada vértice entra na
/// média já **puxado para o cursor**:
///
/// ```text
/// contribuição(v) = c + (p_v − c) · (1 − a_v)      a_v = 3p² − 2p³
/// ```
///
/// ⇒ o peso `1 − a` vale **zero no cursor** e cresce para a borda do disco: quanto
/// mais perto do cursor um vértice está, mais completamente ele é **substituído**
/// por ele. Numa folha em repouso o centro da área é praticamente o cursor; numa
/// folha já cavada ele fica muito mais perto do cursor do que o centroide.
///
/// ⚠️⚠️ **É por isso que a medição de 06/09 — «o plano pelo cursor reproduz o alvo
/// e o plano pelo centro da área afasta-o» — não refuta esta lei: o que foi medido
/// foi um CENTROIDE, e o alvo não usa um centroide** (`empurrar 0,944 → 1,250`,
/// `arrastar 0,233 → 0,716`). *Uma recusa medida responde a UMA pergunta, e aquela
/// respondeu «o centroide não serve», não «o centro da área não serve».* O plano
/// pelo cursor é a aproximação de **primeira ordem** desta lei, e é por isso que
/// ele passava quase.
///
/// ⛔ **O balde é escolhido UMA vez, pela regra da normal** (§4.2-bis (4)) — as
/// duas grandezas saem do mesmo balde, e não de dois desempates independentes.
/// Sem balde nenhum, o centro é o **cursor** e a normal é o vector nulo.
///
/// ⚠️ **Só o plano de queda lê o centro da área.** A origem do referencial local
/// é o cursor (§4.4), e a localização da área *Local* (§2.1) é outra coisa ainda
/// — essa fica no pen-down durante todo o traço, e mora em
/// [`PincelTecido::localizacao_da_area`].
#[must_use]
pub fn normal_e_centro_da_area(
    posicoes: &[V3],
    normais: &[V3],
    dentro: &[u32],
    cursor: V3,
    raio: f64,
    vista: V3,
) -> (V3, V3) {
    let alcance = raio * RAIO_DA_NORMAL;
    let mut baldes = [[0.0f64; 3]; 2];
    let mut centros = [[0.0f64; 3]; 2];
    let mut contas = [0usize; 2];
    for &v in dentro {
        let vi = v as usize;
        let p = posicoes[vi];
        let d = dist(p, cursor);
        if d > alcance {
            continue;
        }
        let n = normais[vi];
        // O peso é a mesma smoothstep da banda, sobre `1 − d/alcance`.
        let t = (1.0 - d / alcance.max(1e-30)).clamp(0.0, 1.0);
        let w = t * t * (3.0 - 2.0 * t);
        let frente = usize::from(n[0] * vista[0] + n[1] * vista[1] + n[2] * vista[2] <= 0.0);
        contas[frente] += 1;
        for c in 0..3 {
            baldes[frente][c] += n[c] * w;
            centros[frente][c] += cursor[c] + (p[c] - cursor[c]) * (1.0 - w);
        }
    }
    // ⚠️⚠️ **A metade «soma não-nula» da regra é carregada pelo [`unit`], e não
    // por um `if` aqui — e isso é MEDIDO, não suposto.** Um `if` que a testasse
    // sobrevive a toda mutação, porque:
    //
    // - o balde da FRENTE, se não estiver vazio, **nunca** tem soma nula: ela é
    //   `Σ wᵢ n̂ᵢ` com todos os `wᵢ > 0` e todos os `n̂ᵢ · v̂ > 0`, logo o produto
    //   dela com `v̂` é positivo e o vector não pode ser zero;
    // - o de TRÁS pode cancelar-se (ali `n̂ · v̂ ≤ 0` admite o zero), e o [`unit`]
    //   de um vector nulo é o vector nulo — que é exactamente o que a espec
    //   §4.2-bis (5) manda devolver.
    //
    // *Uma linha que nenhuma mutação mata é redundante ou falta-lhe um gate; esta
    // era redundante, e a prova está no gate `sem_balde_valido_a_normal_da_area_e_nula`.*
    for b in 0..2 {
        if contas[b] > 0 {
            let k = contas[b] as f64;
            let c = [centros[b][0] / k, centros[b][1] / k, centros[b][2] / k];
            return (unit(baldes[b]), c);
        }
    }
    ([0.0; 3], cursor)
}

/// **O pincel de tecido de UM traço**: a sessão, o centro da área e a simulação.
#[derive(Clone, Debug)]
pub struct PincelTecido {
    pub pincel: Pincel,
    pub sim: Verlet,
    /// A localização INICIAL do traço (o centro da área *Local*; a origem do
    /// delta total do Grab).
    pub inicio: V3,
    /// `R₀` — o raio no 1.º passo (a área *Local* ignora a pressão).
    pub raio0: f64,
    /// O cursor do passo anterior (para o delta incremental).
    pub anterior: V3,
    /// Quem está na área simulada neste passo, **na ordem de visita**.
    pub dentro: Vec<u32>,
    /// ⭐⭐⭐ **A ORDEM DE VISITA da malha** (espec §3.1-bis): a concatenação, por
    /// célula da árvore espacial em índice crescente, dos **vértices próprios**
    /// de cada uma em índice crescente.
    ///
    /// ⛔⛔ **Ela NÃO é a ordem crescente de índice**, e a diferença é
    /// observável: cada vértice é próprio de UMA só célula, logo a sequência é
    /// uma **permutação agrupada por célula**. Na grelha das fixtures ela é a
    /// identidade **rodada** (a célula `1` fica com `[2080..4224]` e a `2` com
    /// `[0..2079]`), com o único descenso exactamente no pen-down das fixtures
    /// `_origem`.
    ///
    /// ⚠️ **É propriedade da MALHA, não do traço** — calcula-se uma vez, sobre as
    /// posições de repouso, e não muda enquanto o traço corre. Vazia = a ordem
    /// crescente, que é o que uma malha sem partição conhecida dá.
    pub ordem: Vec<u32>,
    /// A máscara por vértice em `[0,1]` (`1` = imóvel); vazia = sem máscara.
    pub mascara: Vec<f64>,
    /// Ainda não houve passo nenhum? (o 1.º constrói e não simula)
    pub primeiro: bool,
}

impl PincelTecido {
    /// O pen-down: a simulação nasce nas posições ACTUAIS, que passam a ser o
    /// repouso do traço.
    #[must_use]
    pub fn pen_down(pincel: Pincel, posicoes: &[V3], cursor: V3) -> Self {
        Self {
            raio0: pincel.raio,
            pincel,
            sim: Verlet::nascer(posicoes.to_vec()),
            inicio: cursor,
            anterior: cursor,
            dentro: Vec::new(),
            ordem: Vec::new(),
            mascara: Vec::new(),
            primeiro: true,
        }
    }

    /// A LOCALIZAÇÃO e o raio da área simulada neste passo (espec §2.1).
    ///
    /// ⛔ **Isto não é o «centro da área» do §4.4**, que é outra grandeza com
    /// outro consumidor: esta fica no pen-down durante todo o traço na área
    /// *Local*, e aquela é reavaliada a cada passo à volta do cursor e só o plano
    /// de queda a lê ([`normal_e_centro_da_area`]). *Chamar às duas «o centro da
    /// área» foi o que fez a primeira medição desta linha responder à pergunta
    /// errada.*
    #[must_use]
    pub fn localizacao_da_area(&self, cursor: V3) -> (V3, f64) {
        match self.pincel.area {
            Area::Local | Area::Global => (self.inicio, self.raio0),
            Area::Dinamica => (cursor, self.pincel.raio),
        }
    }

    /// O peso de banda de um ponto (espec §2.2), `1` em *Global*.
    #[must_use]
    pub fn w(&self, p: V3, cursor: V3) -> f64 {
        self.w_com(p, cursor, 1.0)
    }

    /// O peso de banda com o `limite` ESCALADO — os três leitores da espec §2.2
    /// (força · `φ` · retenção) partilham a forma e, por experimento, o alcance.
    #[must_use]
    pub fn w_com(&self, p: V3, cursor: V3, escala: f64) -> f64 {
        if self.pincel.area == Area::Global {
            return 1.0;
        }
        let (c, r) = self.localizacao_da_area(cursor);
        banda(p, c, r, self.pincel.limite * escala, self.pincel.banda)
    }

    fn mascara_de(&self, v: usize) -> f64 {
        self.mascara.get(v).copied().unwrap_or(0.0)
    }

    /// **UM PASSO DO PINCEL** (espec §1, as cinco fases). `posicoes` são as
    /// posições ACTUAIS da malha; `anel(v)` o anel-1. Devolve `true` se simulou
    /// (o 1.º passo nunca simula) — e nesse caso `sim.x` traz as posições novas
    /// dos vértices ACTIVOS, que o chamador escreve na malha.
    pub fn passo(
        &mut self,
        posicoes: &[V3],
        anel: &dyn Fn(u32) -> Vec<u32>,
        passo: &Passo<'_>,
    ) -> bool {
        let n = posicoes.len();
        debug_assert_eq!(n, self.sim.len());
        let cursor = passo.cursor;
        // fase 1 — a área deste passo, e as restrições de quem entra nela
        let (c, r) = self.localizacao_da_area(cursor);
        let alcance = r * (1.0 + self.pincel.limite);
        self.dentro.clear();
        // ⚠️ **A varredura segue a ORDEM DE VISITA** (espec §3.1-bis) — célula a
        // célula, vértice próprio a vértice próprio —, e é ela que fixa a ordem
        // da lista de restrições. Sem partição conhecida, a ordem crescente.
        let n_v = posicoes.len();
        let visita: &[u32] = &self.ordem;
        let crescente: Vec<u32>;
        let visita = if visita.len() == n_v {
            visita
        } else {
            crescente = (0..u32::try_from(n_v).unwrap_or(u32::MAX)).collect();
            &crescente
        };
        for &v in visita {
            let vi = v as usize;
            let dentro = match self.pincel.area {
                Area::Global => true,
                // ⚠️ Local: o teste da construção é sobre o REPOUSO (espec §3.1).
                Area::Local => dist(self.sim.repouso[vi], c) < alcance,
                Area::Dinamica => dist(posicoes[vi], c) < alcance,
            };
            if dentro {
                self.dentro.push(v);
            }
        }
        let novos: Vec<u32> = self
            .dentro
            .iter()
            .copied()
            .filter(|v| !self.sim.construido[*v as usize])
            .collect();
        // ⚠️⚠️ **A CONSTRUÇÃO CORRE `construcoes()` VEZES** (espec, emenda Q8), e
        // entre passagens o registo de duplicados é ESQUECIDO. Na área *Local*
        // isso deixa cada restrição repetida na lista, e como a lista é resolvida
        // na ORDEM, `[c₁..c_N, c₁..c_N]` percorrida `k` vezes é bit a bit a lista
        // simples percorrida `2k` vezes. *A repetição não é desperdício: é a
        // rigidez que separava a Local da Global, e os dois ramos relaxam o
        // mesmo número de vezes.*
        //
        // ⚠️ **A passagem repete o VÉRTICE INTEIRO, não só os pares** — a âncora,
        // o pino e o amolecimento entram na mesma lista e não têm registo de
        // duplicados. Medido em 06/09: repetir só os pares deixa as âncoras no
        // meio da lista e piora o Grab de `0,050` para `0,415` e o Snake Hook de
        // `0,127` para `0,612`. *Onde uma restrição está na lista é tão
        // load-bearing quanto quantas vezes ela lá está.*
        for passagem in 0..self.pincel.construcoes() {
            if passagem > 0 {
                self.sim.reabrir(&novos);
            }
            for &v in &novos {
                let vizinhos = anel(v);
                let vi = v as usize;
                // ⚠️⚠️ **A ORDEM DE NASCIMENTO por vértice é LEI, e é esta**
                // (espec §5.2 nº 1, emenda Q14): **corpo mole → estruturais →
                // âncora → pino**. As quatro espécies vivem numa lista SÓ e num
                // laço SÓ, resolvido de fio a pavio — não há «primeiro as
                // distâncias, depois as âncoras» —, e Gauss-Seidel não comuta,
                // logo onde cada uma cai decide a resposta. ⛔ Antes de 06/09
                // esta casa nascia por outra ordem (estruturais → pino → corpo
                // mole → âncora), o que só é observável nos dois traços que
                // ligam o pino ou a plasticidade.
                if self.pincel.solver.plasticidade > 0.0 {
                    self.sim.amolecer(v);
                }
                self.sim.construir(v, &vizinhos);
                // As âncoras de deformação nascem com o vértice (espec §4.3).
                match self.pincel.modo {
                    Modo::Agarrar => {
                        let d0 = dist(self.sim.repouso[vi], self.inicio);
                        match self.pincel.falloff_forca {
                            FalloffForca::Radial => {
                                if d0 < self.raio0 {
                                    let s = 0.1 * self.pincel.curva.peso(d0, self.raio0);
                                    self.sim.ancorar(v, s);
                                    self.sim.sigma[vi] = 1.0;
                                }
                            }
                            FalloffForca::Plano => {
                                self.sim.ancorar(v, 0.1);
                            }
                        }
                    }
                    Modo::Gancho => self.sim.ancorar(v, 0.35),
                    _ => {}
                }
                // Espec §2.3: o pino da fronteira, só em Local, força `1 − w`.
                // ⚠️ Ele é o ÚLTIMO do bloco do vértice (§5.2 nº 1).
                if self.pincel.pino && self.pincel.area == Area::Local {
                    let wv = banda(
                        self.sim.repouso[vi],
                        cursor,
                        self.raio0,
                        self.pincel.limite,
                        self.pincel.banda,
                    );
                    if wv < 1.0 {
                        self.sim.pregar(v, 1.0 - wv);
                    }
                }
            }
        }
        // O 1.º passo de uma passagem nunca simula (espec §1 fase 0): o alvo
        // precisa de um deslocamento do cursor válido para orientar a ponta, e
        // no 1.º passo ele é zero.
        if self.primeiro {
            self.primeiro = false;
            self.anterior = cursor;
            return false;
        }
        // fase 2 — guardar o estado: x ← malha
        self.sim.x.copy_from_slice(posicoes);
        // fase 3 — activar
        self.sim.activo.fill(false);
        for &v in &self.dentro {
            self.sim.activo[v as usize] = true;
        }
        // φ e a retenção de banda são lidos no REPOUSO do traço (espec §2.2).
        for v in 0..n {
            let p0 = self.sim.repouso[v];
            self.sim.phi[v] =
                (1.0 - self.mascara_de(v)) * self.w_com(p0, cursor, self.pincel.escala_phi);
            self.sim.w_repouso[v] = self.w_com(p0, cursor, self.pincel.escala_retencao);
        }
        // fase 4 — o gesto
        self.gesto(posicoes, passo);
        // fase 5 — o passo de simulação
        self.sim.passo(&self.pincel.solver);
        self.anterior = cursor;
        true
    }

    /// **O factor por vértice `f`** (espec §4.1), sem o `B`: máscara · banda ·
    /// corte no raio · curva com dureza.
    fn factor(&self, p: V3, cursor: V3, r: f64, d: f64) -> f64 {
        if d >= r {
            return 0.0;
        }
        let h = self.pincel.dureza.clamp(0.0, 1.0);
        let d_remap = if h > 0.0 {
            if d < h * r {
                0.0
            } else {
                (d - h * r) / (1.0 - h)
            }
        } else {
            d
        };
        self.w(p, cursor) * self.pincel.curva.peso(d_remap, r)
    }

    /// A distância que a curva lê (espec §4.1): esférica ao **centro da queda**
    /// do modo, ou ao **PLANO de falloff** — que passa pelo **centro da área**
    /// (§4.4) com normal `δ̂`.
    ///
    /// ⚠️ **Os dois pontos são grandezas diferentes e chegam separados de
    /// propósito:** o centro da queda muda com o modo (o cursor · a localização
    /// inicial no Agarrar · o cursor do passo ANTERIOR no Gancho), e o ponto do
    /// plano é sempre o centro da área do passo. *Enquanto os dois eram o mesmo
    /// argumento, o plano passava pelo cursor — a aproximação de primeira ordem.*
    fn distancia(&self, p: V3, centro: V3, centro_area: V3, delta_u: V3) -> f64 {
        match self.pincel.falloff_forca {
            FalloffForca::Radial => dist(p, centro),
            FalloffForca::Plano => {
                let q = [
                    p[0] - centro_area[0],
                    p[1] - centro_area[1],
                    p[2] - centro_area[2],
                ];
                (q[0] * delta_u[0] + q[1] * delta_u[1] + q[2] * delta_u[2]).abs()
            }
        }
    }

    /// Fase 4 (espec §4): forças → `a`; âncoras; desvios de repouso.
    fn gesto(&mut self, posicoes: &[V3], passo: &Passo<'_>) {
        let cursor = passo.cursor;
        let r = self.pincel.raio;
        let delta_u = unit(passo.delta);
        let alpha = self.pincel.forca * self.pincel.forca;
        let flip = self.pincel.flip;
        let pressao = passo.pressao.clamp(0.0, 1.0);
        // ⚠️ Uma varredura, duas grandezas (espec §4.2-bis e §4.4): o mesmo disco
        // de meio raio, os mesmos dois baldes, o mesmo desempate.
        //
        // ⚠️⚠️ **O disco é centrado na LOCALIZAÇÃO DO CURSOR do modo** (espec
        // §4.2-bis (3), «distância ao cursor»), e no Agarrar essa localização
        // **fica no ponto do pen-down durante todo o traço** (§4.3) — é isso que
        // faz o Grab pegar num conjunto FIXO de vértices. Medido em 06/09: com o
        // disco a seguir o cursor que anda, o `plano_agarrar_plano_local` sobe de
        // `0,180` para `1,094`, porque o plano de queda passa a derivar por baixo
        // de uma pegada que é medida na malha de PARTIDA.
        let c_cursor = if self.pincel.modo == Modo::Agarrar {
            self.inicio
        } else {
            cursor
        };
        let (n_area, c_area) = normal_e_centro_da_area(
            posicoes,
            passo.normais,
            &self.dentro,
            c_cursor,
            r,
            passo.vista,
        );
        // O referencial local do traço (espec §4.4).
        let x_hat = unit(cruz(n_area, delta_u));
        // ⛔ **O guarda de «passo sem movimento» vale para os OITO modos** (espec
        // §4.3): ele corre ANTES de o modo ser escolhido, e o que ele testa é o
        // `δ` projectado, não a posição 3D. Antes de 06/09 ele vivia dentro do
        // braço dos modos de força, e os dois de âncora escapavam-lhe.
        if passo.parado {
            return;
        }
        let dentro = self.dentro.clone();
        match self.pincel.modo {
            Modo::Agarrar => {
                // Espec §4.3: âncora = p⁰ + δ_total · f, com f medido na malha
                // de PARTIDA e o raio inicial; σ = 1 (radial) ou clamp(f) (plano).
                //
                // ⚠️ **O passo começa com `σ ≡ 0` em TODA a malha** (espec §4.3,
                // emenda Q9): os DOIS modos de âncora zeram a força por passo
                // antes de a reescrever, e o que os distingue não é zerar ou
                // não — é o valor com que reescrevem e o facto de o conjunto do
                // Grab ser fixo. ⛔ A redacção anterior desta espec dizia que o
                // Grab não zerava, e estava errada.
                self.sim.sigma.fill(0.0);
                for &v in &dentro {
                    let vi = v as usize;
                    let p0 = self.sim.repouso[vi];
                    let d = self.distancia(p0, self.inicio, c_area, delta_u);
                    let f = self.factor(p0, self.inicio, self.raio0, d) * self.pincel.forca;
                    let m = 1.0 - self.mascara_de(vi);
                    let f = f * m;
                    self.sim.ancora[vi] = [
                        p0[0] + passo.delta[0] * f,
                        p0[1] + passo.delta[1] * f,
                        p0[2] + passo.delta[2] * f,
                    ];
                    // σ reescrito a cada passo: `1` no radial (e só para quem
                    // o raio INICIAL alcança, que é o conjunto fixo em que a
                    // âncora nasceu) · `clamp(f)` no plano.
                    if self.pincel.falloff_forca == FalloffForca::Plano {
                        self.sim.sigma[vi] = f.clamp(0.0, 1.0);
                    } else if d < self.raio0 {
                        self.sim.sigma[vi] = 1.0;
                    }
                }
            }
            Modo::Gancho => {
                // Espec §4.3: âncora = x + δ · f, e σ = f reescrito a cada passo —
                // ZERO fora do pincel.
                //
                // ⚠️⚠️ **O centro da queda é ONDE O PINCEL ESTAVA, não onde ele
                // chegou** (espec, emenda Q9 de 06/09): a localização do pincel
                // deixa de ser lida do evento e passa a ser avançada pelo delta
                // do passo ANTERIOR, logo fica um passo atrasada — e no 1.º
                // passo simulado é exactamente o pen-down. É por isso que
                // [`Self::anterior`] serve tal e qual: ela é essa localização.
                //
                // ⚠️ **É defeito de LUGAR, e as réguas de amplitude são cegas a
                // ele.** Medido em 06/09 com a sonda por passo: com o centro no
                // cursor o nosso pico ficava a `0,05R` do cursor e o do alvo a
                // `0,86R` — nós apanhávamos material novo a cada passo, o alvo
                // arrasta o que já pegou. A amplitude estava certa.
                //
                // ⚠️ **As posições são as ACTUAIS** — só o Grab mede no repouso.
                let centro = self.anterior;
                // Idem §4.3: `σ ≡ 0` em toda a malha antes de reescrever.
                self.sim.sigma.fill(0.0);
                let b = self.pincel.forca * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, centro, c_area, delta_u);
                    let f = self.factor(p, centro, r, d) * b * (1.0 - self.mascara_de(vi));
                    self.sim.ancora[vi] = [
                        p[0] + passo.delta[0] * f,
                        p[1] + passo.delta[1] * f,
                        p[2] + passo.delta[2] * f,
                    ];
                    self.sim.sigma[vi] = f;
                }
            }
            Modo::Expandir => {
                let b = 0.1 * alpha * flip * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, c_area, delta_u);
                    let f = self.factor(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    self.sim.tau[vi] += 0.01 * f;
                }
            }
            Modo::Arrastar
            | Modo::Empurrar
            | Modo::ApertarPonto
            | Modo::ApertarLinha
            | Modo::Inflar => {
                let b = 10.0 * alpha * flip * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, c_area, delta_u);
                    let f = self.factor(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    if f == 0.0 {
                        continue;
                    }
                    let u: V3 = match self.pincel.modo {
                        // ⛔ **O arrasto é o ÚNICO modo que NÃO tira a direcção de
                        // `δ`** (espec §4.2/§4.3): ela é a diferença dos dois
                        // pontos 3D do cursor, normalizada.
                        Modo::Arrastar => unit(passo.delta_3d),
                        Modo::Empurrar => [
                            -n_area[0] * 2.0 * r,
                            -n_area[1] * 2.0 * r,
                            -n_area[2] * 2.0 * r,
                        ],
                        Modo::ApertarPonto => match self.pincel.falloff_forca {
                            FalloffForca::Radial => {
                                unit([cursor[0] - p[0], cursor[1] - p[1], cursor[2] - p[2]])
                            }
                            // Espec §4.2: com falloff de plano o alvo é o PLANO.
                            FalloffForca::Plano => {
                                let q = [p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]];
                                let s = q[0] * delta_u[0] + q[1] * delta_u[1] + q[2] * delta_u[2];
                                let sg = if s > 0.0 { -1.0 } else { 1.0 };
                                [delta_u[0] * sg, delta_u[1] * sg, delta_u[2] * sg]
                            }
                        },
                        Modo::ApertarLinha => {
                            let para = unit([cursor[0] - p[0], cursor[1] - p[1], cursor[2] - p[2]]);
                            let cx = para[0] * x_hat[0] + para[1] * x_hat[1] + para[2] * x_hat[2];
                            let cz =
                                para[0] * n_area[0] + para[1] * n_area[1] + para[2] * n_area[2];
                            [
                                x_hat[0] * cx + n_area[0] * cz,
                                x_hat[1] * cx + n_area[1] * cz,
                                x_hat[2] * cx + n_area[2] * cz,
                            ]
                        }
                        Modo::Inflar => unit(passo.normais[vi]),
                        _ => [0.0; 3],
                    };
                    let inv_m = 1.0 / self.pincel.solver.massa.max(1e-9);
                    for (c, uc) in u.iter().enumerate() {
                        self.sim.a[vi][c] += f * uc * inv_m;
                    }
                }
            }
        }
    }
}

/// `a × b`.
#[must_use]
pub fn cruz(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Sem uso fora dos testes de paridade, mas é a régua: `|v|`.
#[must_use]
pub fn comprimento(v: V3) -> f64 {
    norm(v)
}
