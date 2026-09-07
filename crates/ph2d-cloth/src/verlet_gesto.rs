//! ⭐⭐⭐ **O GESTO da referência** — a área simulada, a banda graduada, o factor por
//! vértice e os oito modos, portados clean-room da
//! [espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md) §2 e §4.
//!
//! ⚠️ **Este módulo não sabe o que é uma malha nem um pincel.** Recebe posições,
//! normais, o anel-1 de cada vértice e o cursor; devolve, por passo, o que o
//! solver precisa (forças, âncoras, desvios de repouso) e corre-o. É o adaptador
//! do `ph2d-sculpt3d` que traduz `Brush`/`Dab`/`Mesh` para isto.

use crate::V3;
use crate::verlet::{Verlet, dist, norm, unit};

/// O VOCABULÁRIO DOS CONTROLOS — ver o doc do módulo.
#[path = "verlet_gesto_pincel.rs"]
mod pincel_mod;
pub use pincel_mod::{Area, Curva, FalloffForca, Modo, Pincel, banda};

/// A NORMAL E O CENTRO DA ÁREA — ver o doc do módulo.
#[path = "verlet_gesto_area.rs"]
mod area_mod;
pub use area_mod::{RAIO_DA_NORMAL, normal_da_area, normal_e_centro_da_area};

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
    /// ⭐⭐⭐ **A NORMAL DA ÁREA do último passo** (espec §4.2-bis) — um vector
    /// por passo, e a mesma grandeza que o Push usa como direcção, o referencial
    /// local usa como `ẑ` e o falloff de plano usa como normal (§4.4).
    ///
    /// ⚠️⚠️ **O vector NULO aqui não é um caso degenerado: é o regime normal do
    /// Push à força de omissão** (§4.2-bis (5) e (8)). O disco de amostragem tem
    /// raio `R · 0,5` e mede-se contra as posições **de agora**; numa folha que o
    /// próprio Push já afundou chega um passo em que nenhum vértice está a menos
    /// disso do cursor — e nesse passo **o gesto não escreve aceleração nenhuma**.
    /// ⭐ E volta a disparar quando o cursor avança para terreno pouco afundado.
    ///
    /// *Ela é pública porque é o único observável que separa «empurrei com uma
    /// direcção inclinada» de «não empurrei» — as duas coisas que faziam a frente
    /// de ataque ficar `16×` mais curta que a do alvo.*
    pub normal_da_area: V3,
    /// Ainda não houve passo nenhum? (o 1.º constrói e não simula)
    pub primeiro: bool,
}

impl PincelTecido {
    /// O pen-down: a simulação nasce nas posições ACTUAIS, que passam a ser o
    /// repouso do traço.
    ///
    /// ⚠️⚠️ **A `ordem` é um ARGUMENTO e não é derivada aqui, de propósito.** Ela
    /// é a ordem de visita da malha ([`Self::ordem`]), e quem a sabe é quem tem a
    /// topologia: o produto deriva-a com
    /// [`crate::particao::ordem_de_visita`], e a bancada de paridade **lê-a da
    /// fixture do alvo** — porque na esfera o desempate do eixo da bissecção não
    /// é resolúvel com a precisão que as fixtures têm. *Se ela fosse derivada
    /// aqui, a bancada mediria uma partição espelhada e chamar-lhe-ia paridade.*
    ///
    /// ⛔ **E é obrigatória porque esquecê-la é silencioso:** ela muda o
    /// resultado e não muda a compilação. Passar `Vec::new()` diz «esta malha não
    /// tem partição conhecida» e cai na ordem crescente — que é o que uma malha
    /// abaixo do tecto da folha dá de qualquer maneira.
    #[must_use]
    pub fn pen_down(pincel: Pincel, posicoes: &[V3], cursor: V3, ordem: Vec<u32>) -> Self {
        Self {
            raio0: pincel.raio,
            pincel,
            sim: Verlet::nascer(posicoes.to_vec()),
            inicio: cursor,
            anterior: cursor,
            dentro: Vec::new(),
            ordem,
            mascara: Vec::new(),
            normal_da_area: [0.0; 3],
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
        // ⭐⭐⭐ **O AGARRAR NÃO SEGUE O CURSOR — nem sequer na área *Dynamic*.**
        //
        // A lei já estava escrita neste ficheiro, uma porta adiante: o disco da
        // normal da área é centrado em [`Self::inicio`] no Agarrar, *«é isso que
        // faz o Grab pegar num conjunto FIXO de vértices»* (espec §4.3). Ela vale
        // uma porta ANTES, na área simulada: se a esfera seguisse o cursor,
        // material NOVO entraria na simulação a meio do traço — que é
        // exactamente o que *pegar num conjunto fixo* exclui.
        //
        // ⚠️⚠️ **É o PAR que cura, e nenhuma metade sozinha o faz** (medido em
        // 07/09 sobre `esfera_agarrar_radial_dinamica`, o único Agarrar em área
        // *Dynamic* do corpus — os outros dez são *Local* ou *Global*, onde esta
        // porta já devolvia o pen-down e nada muda):
        //
        // | o que fica no pen-down | vértices movidos (alvo `1863`) | erro |
        // |---|---|---|
        // | nada (a esfera segue o cursor) | `2123` | `0,182` |
        // | só a BANDA | `1728` | `0,129` |
        // | só a PERTENÇA | `1666` | `0,182` |
        // | ⭐ **as duas** | **`1864`** | **`0,033`** |
        //
        // *Uma metade melhora pouco, a outra nada, e a célula `(1,1)` fecha o
        // traço* — a família que a memória desta casa regista como «duas metades
        // de uma cura, cada uma recusada sozinha, não a refutam».
        if self.pincel.modo == Modo::Agarrar {
            return (self.inicio, self.raio0);
        }
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
                // ⚠️ Local: o teste da construção é sobre o REPOUSO (espec §3.1)
                // — ou sobre a BASE PERSISTENTE, se houver (§6.4 leitura 2): é a
                // base que decide **quem entra na simulação**, não só quão
                // esticado ele está.
                Area::Local => dist(self.sim.base_de(vi), c) < alcance,
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
                        // Espec §6.4 leitura 3: o teste E a força da âncora radial
                        // saem da BASE ⇒ com base, o conjunto agarrado é o dela.
                        let d0 = dist(self.sim.base_de(vi), self.inicio);
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
                    // Espec §6.4 leitura 4: a CONDIÇÃO de criação do pino é
                    // avaliada na BASE. ⛔ O ALVO dele continua a ser o repouso do
                    // traço (`Alvo::Repouso`), e a banda `w` do factor por vértice
                    // também — a base muda a rede, nunca o alvo nem o peso.
                    let wv = banda(
                        self.sim.base_de(vi),
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
            let m = 1.0 - self.mascara_de(v);
            // §5.2 — o factor das varreduras traz a banda.
            self.sim.phi[v] = m * self.w_com(p0, cursor, self.pincel.escala_phi);
            // §5.4 — o da integração NÃO a traz; ela entra uma vez, e só na
            // velocidade, por `w_repouso`.
            self.sim.phi_integracao[v] = m;
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
        self.normal_da_area = n_area;
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
