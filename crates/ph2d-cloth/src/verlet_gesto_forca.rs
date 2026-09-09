//! ⭐⭐⭐ **A LEI DA FORÇA POR VÉRTICE** — a fase 4 da espec §4: quanto do gesto
//! cada vértice recebe, e em que direcção.
//!
//! Irmão (`#[path]`) do [`super`], cortado por RESPONSABILIDADE: o
//! `verlet_gesto.rs` responde *«quem está na área, e o que corre em que ordem»*
//! (as fases 1, 2, 3 e 5 do passo); aqui vive *«o que a mão QUER dizer»* — o
//! factor `f` (máscara · banda · curva com dureza), o accionamento que separa o
//! traço do filtro, a distância que cada modo mede, e a direcção de cada um dos
//! oito modos de deformação.
//!
//! ⚠️ **A ordem das fases continua toda no vizinho**, e é ela que é lei: um
//! leitor que queira saber *quando* isto corre lê lá, e não aqui.
//!
//! ⚠️ **O gatilho do corte foi o `architecture_workspace_file_loc_cap`** (`782`
//! de um tecto de `700`), e ele estava **latente** desde a wave que trouxe o
//! filtro: mora em `ph2d-editor-core/tests/`, então nenhum fechamento por
//! `cargo test -p ph2d-cloth` o alcança.

use super::{Accionamento, FalloffForca, Modo, Passo, PincelTecido, cruz, normal_e_centro_da_area};
use crate::V3;
use crate::verlet::{dist, norm, unit};

/// ⭐⭐⭐ **O QUANTUM DE ARRASTO do filtro — e ele NÃO é um número escolhido: está
/// escrito no cabeçalho de cada fixture do alvo** (`avanco_por_passo_px 90`,
/// `escala_da_ui 1.0`, e a lei da força é `S = 0,001 · px · escala`) ⇒ `0,09`.
///
/// # ⛔⛔ O defeito que ele cura: o *Expand* contava EVENTOS, não arrasto
///
/// O [`Modo::Expandir`] soma `τ += 0,01 · f` **por passo**, e no filtro um passo
/// é **um movimento do rato**. Como `S` é a distância acumulada ao ponto de
/// pressão, dois artistas que arrastem exactamente o mesmo tanto recebem `τ`
/// proporcionais ao **polling do rato deles**. Medido sobre uma esfera, o MESMO
/// arrasto (`s` de `0` a `1`), volume normalizado ao repouso:
///
/// | amostras | 8 | 15 | 30 | 60 | 120 | 240 |
/// |---|---:|---:|---:|---:|---:|---:|
/// | **Gravity** | `1,000` | `1,000` | `1,000` | `1,000` | `1,000` | `1,000` |
/// | **Inflate** | `1,188` | `1,165` | `1,168` | `1,167` | `1,168` | `1,169` |
/// | **Scale** | `1,285` | `1,215` | `1,224` | `1,219` | `1,217` | `1,216` |
/// | **Expand** | `1,967` | `2,933` | `3,498` | `5,494` | `19,509` | **`55,841`** |
///
/// ⭐⭐ **Três dos quatro tipos já eram invariantes** — eles escrevem uma força
/// ou uma âncora, e o equilíbrio contra a rede de restrições é fixado pela
/// MAGNITUDE de `S`, que chega a `1` seja qual for o número de passos. O Expand é
/// o único que **acumula**, e por isso o único que a amostragem multiplica: `28×`
/// entre as duas pontas da tabela. *Não é «falta um tecto»: é a lei do Painter,
/// que este repo pagou seis vezes — o traço é facto do CAMINHO, nunca de quão
/// fino o motor amostrou o caminho.*
///
/// ⇒ o incremento passa a ser pesado por `|Δs| / QUANTUM_DE_ARRASTO`.
///
/// ⭐ **As duas fixtures do Expand ficam BYTE-IDÊNTICAS por construção**: nelas o
/// avanço é uniforme (`0,09` por passo, e `−0,09` na negativa), logo o peso vale
/// exactamente `1` em todos os oito passos. ⚠️ **O valor absoluto é
/// load-bearing** — sem ele a fixture negativa inverteria o sinal de `τ` e o
/// Expand para trás faria a peça CRESCER.
///
/// ⚠️ **O traço não passa por aqui** (peso `1`): ali quem parametriza o caminho é
/// o espaçamento dos dabs, que já é uma lei de arco.
pub(super) const QUANTUM_DE_ARRASTO: f64 = 0.09;

impl PincelTecido {
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
    /// **O factor por vértice DO ACCIONAMENTO EM VIGOR** (espec §4.1 · §7).
    ///
    /// No traço é o [`Self::factor`] inteiro — máscara · banda · corte no raio ·
    /// curva com dureza. No filtro **não há pincel**: a espec §7 dá
    /// `(1 − máscara) · S`, e a parte da máscara já entra em cada arm, logo o que
    /// sobra aqui é `1`.
    ///
    /// ⛔ **Isto não se finge com um raio enorme e uma curva constante.** Foi
    /// medido: o [`Modo::Empurrar`] traz o raio DENTRO da magnitude, e um raio de
    /// `1e3` faz o pico dele saltar `2000×`
    /// (`tests/mede_a_composicao_do_filtro.rs`). *Um raio que finge ser infinito
    /// é lido como comprimento por quem tem comprimento na lei.*
    fn factor_accionado(&self, p: V3, centro: V3, r: f64, d: f64) -> f64 {
        match self.pincel.accionamento {
            Accionamento::Traco => self.factor(p, centro, r, d),
            Accionamento::Filtro { .. } => 1.0,
        }
    }

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
    pub(super) fn gesto(&mut self, posicoes: &[V3], passo: &Passo<'_>) {
        let cursor = passo.cursor;
        let r = self.pincel.raio;
        let delta_u = unit(passo.delta);
        let alpha = self.pincel.forca * self.pincel.forca;
        let flip = self.pincel.flip;
        let pressao = passo.pressao.clamp(0.0, 1.0);
        // ⭐⭐⭐ **O ACCIONAMENTO** (espec §4.1 contra §7) — ver [`Accionamento`].
        //
        // ⚠️ **No traço o `B` muda por arm e no filtro NÃO**, e o factor entre os
        // dois arms do traço é `100`. A espec §7 dá um `f` só aos cinco tipos do
        // filtro, e as constantes que sobram (`0,01` do Expand, `0,01` da âncora
        // da Escala) vivem dentro do tipo. ⛔ Escrever `b_expand = s / 100` aqui
        // seria emprestar ao filtro uma escada que é do traço.
        //
        // ⚠️ **O terceiro valor é o PESO DO ARRASTO**, e só o Expand o lê — ver
        // [`QUANTUM_DE_ARRASTO`] para o defeito que ele cura e a tabela medida.
        let (b_forca, b_expand, peso_do_arrasto) = match self.pincel.accionamento {
            Accionamento::Traco => (
                10.0 * alpha * flip * pressao,
                0.1 * alpha * flip * pressao,
                1.0,
            ),
            Accionamento::Filtro { s } => {
                let avanco = (s - self.arrasto_anterior).abs() / QUANTUM_DE_ARRASTO;
                self.arrasto_anterior = s;
                (s, s, avanco)
            }
        };
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
                let b = b_expand;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, c_area, delta_u);
                    let f =
                        self.factor_accionado(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    self.sim.tau[vi] += 0.01 * f * peso_do_arrasto;
                }
            }
            // ⭐ **SÓ O FILTRO** (espec §7): a âncora `p⁰ + p⁰ · f`, com as
            // componentes dos eixos desligados anuladas no referencial, e a força
            // de âncora `0,01` — escrita na construção, como as outras duas.
            //
            // ⚠️ **A âncora é sobre `p⁰` CRU**, logo a homotetia é em torno da
            // origem do objecto (e não do centroide) — a mesma escolha, e a mesma
            // consequência nomeada, do `FilterKind::Scale` do filtro de malha.
            //
            // ⚠️ **`σ` é reescrito a cada passo como nos outros dois modos de
            // âncora** (espec §4.3, emenda Q9): o passo começa com `σ ≡ 0` em
            // toda a malha antes de o valor novo entrar.
            Modo::Escala => {
                self.sim.sigma.fill(0.0);
                let eixos = self.pincel.referencial.eixos;
                let activo = self.pincel.referencial.activo;
                for &v in &dentro {
                    let vi = v as usize;
                    let p0 = self.sim.repouso[vi];
                    let f = b_forca * (1.0 - self.mascara_de(vi));
                    // O deslocamento `p⁰ · f`, com os eixos desligados anulados
                    // **no referencial** — projecta, filtra, e volta.
                    let mut desloc = [0.0; 3];
                    for k in 0..3 {
                        if !activo[k] {
                            continue;
                        }
                        let e = eixos[k];
                        let c = p0[0] * e[0] + p0[1] * e[1] + p0[2] * e[2];
                        for (dc, ec) in desloc.iter_mut().zip(e) {
                            *dc += c * ec;
                        }
                    }
                    self.sim.ancora[vi] = [
                        p0[0] + desloc[0] * f,
                        p0[1] + desloc[1] * f,
                        p0[2] + desloc[2] * f,
                    ];
                    // ⭐⭐⭐ **`σ = 1`, e o `0,01` da espec é a RIGIDEZ da restrição,
                    // não isto.** A frase da §7 — *«força de âncora `0,01`»* —
                    // nomeia **um** número, e esta casa tem **dois**: o `ancorar(v, s)`
                    // da construção (a rigidez) e o `σ` do passo (a activação). ⛔ Ler
                    // a frase como os dois aplica-o **duas vezes**, e o erro contra o
                    // oráculo foi de `0,988876` — a peça mal se mexia. Com o `0,01` só
                    // na rigidez: **`0,007443`**. *O vizinho de cima já dizia a
                    // convenção — o Grab põe `0,1` na rigidez e `1` no σ.*
                    self.sim.sigma[vi] = 1.0;
                }
            }
            Modo::Arrastar
            | Modo::Empurrar
            | Modo::ApertarPonto
            | Modo::ApertarLinha
            | Modo::Gravidade
            | Modo::Inflar => {
                let b = b_forca;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, c_area, delta_u);
                    let f =
                        self.factor_accionado(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
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
                        // ⭐⭐⭐ **A ÚNICA direcção deste ficheiro que não sai da
                        // malha nem do cursor** (espec §7): o chamador dita-a, já
                        // resolvida em coordenadas de mundo. Medido antes de
                        // existir: os oito modos do traço rodam com a peça
                        // (equivariância `≤ 5,0e-11`), logo nenhum deles a
                        // exprimia — `tests/mede_a_composicao_do_filtro.rs`.
                        Modo::Gravidade => unit(self.pincel.eixo_da_gravidade),
                        // ⚠️ **A fotografia das normais é a que o CHAMADOR dá**
                        // (espec §4.2-ter contra §7): o traço lê as do início e o
                        // filtro refresca-as por passo. A lei é a mesma e o
                        // parâmetro é [`Passo::normais`] — medido a divergir
                        // **30,30%** em seis passos, e `0,000` num só (o
                        // controlo). *A mesma palavra nomeia duas leis, e é o
                        // chamador que escolhe qual.*
                        Modo::Inflar => unit(passo.normais[vi]),
                        _ => [0.0; 3],
                    };
                    let inv_m = 1.0 / self.pincel.solver.massa.max(1e-9);
                    // ⭐⭐⭐ **A LEI DO ALVO, CONVERGIDA** — instrumento do §5.2-ter,
                    // ⛔ DESLIGADO por omissão (o caminho de omissão é byte-idêntico).
                    //
                    // Os dois apertos são os ÚNICOS modos cuja direcção é função da
                    // posição ACTUAL do vértice (`u = unit(cursor − p)`); os outros três
                    // que escrevem aceleração usam uma direcção constante no passo. Com
                    // o impulso máximo a valer **`2,1×` a aresta** (espec §5.2-ter), um
                    // vértice atravessa o cursor num passo só — e do outro lado a força
                    // dele **inverte 180°**. É isso, e não a magnitude, que põe o
                    // resultado à mercê da ORDEM de resolução.
                    //
                    // ⚠️ **Sub-dividir o passo re-avaliando `u` converge para uma coisa
                    // que se escreve em fechado:** o vértice caminha em linha recta até
                    // ao alvo e **pára lá** — porque a espec já define que separação nula
                    // dá força nula. ⇒ o limite é `avanço ← min(avanço, o que falta)`.
                    // *A trava do §5.2-ter (b) não é uma mudança de produto: é a lei do
                    // próprio alvo integrada fino.* O nó é artefacto de passo grosso.
                    //
                    // ⛔ Só os apertos: nos outros três a direcção não depende de `p`,
                    // logo não há nada a convergir e o `min` seria uma lei nova.
                    let f = if self.pincel.converge_aperto
                        && matches!(self.pincel.modo, Modo::ApertarPonto | Modo::ApertarLinha)
                    {
                        // O avanço que ESTE termo produz na integração (§5.4:
                        // `x += a · φ_int · DT`), e o que falta até ao alvo do modo.
                        let avanco = f * inv_m * self.sim.phi_integracao[vi] * crate::verlet::DT;
                        let nu = norm(u);
                        let falta = match self.pincel.modo {
                            // O alvo é o CURSOR; `u` é unitário.
                            Modo::ApertarPonto
                                if matches!(self.pincel.falloff_forca, FalloffForca::Radial) =>
                            {
                                dist(p, cursor)
                            }
                            // O alvo é o PLANO do cursor: falta a distância a ele.
                            Modo::ApertarPonto => {
                                let q = [p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]];
                                (q[0] * delta_u[0] + q[1] * delta_u[1] + q[2] * delta_u[2]).abs()
                            }
                            // ⚠️ `u` do aperto de LINHA **não é unitário** (a projecção
                            // deixa-o `≤ 1`): o que falta mede-se ao longo de `û`.
                            _ => {
                                if nu <= 1e-12 {
                                    0.0
                                } else {
                                    let uh = [u[0] / nu, u[1] / nu, u[2] / nu];
                                    let q = [cursor[0] - p[0], cursor[1] - p[1], cursor[2] - p[2]];
                                    (q[0] * uh[0] + q[1] * uh[1] + q[2] * uh[2]).max(0.0)
                                }
                            }
                        };
                        let percorrido = avanco * nu;
                        if percorrido > falta && percorrido > 0.0 {
                            f * (falta / percorrido)
                        } else {
                            f
                        }
                    } else {
                        f
                    };
                    for (c, uc) in u.iter().enumerate() {
                        self.sim.a[vi][c] += f * uc * inv_m;
                    }
                }
            }
        }
    }
}
