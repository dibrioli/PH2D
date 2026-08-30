//! **A tartaruga** — a cadeia derivada lida como um desenho (ABOP §1.6).
//!
//! # O que sai daqui é uma ÁRVORE, não uma nuvem
//!
//! ⭐ **É a decisão de desenho desta wave.** Um interpretador ingénuo cospe pontos, e a
//! linha entre eles não existe em lado nenhum — o desenho só aparece porque alguém carimba
//! uma forma em cada vértice. Aqui cada elemento sai com `parent` · `len` · `rot` · `wrot`,
//! que é **exactamente o contrato de colunas do `rig.*`**: o mesmo que o `rig.skeleton`
//! publica e que o `rig.fk` resolve. ⇒ um L-System entra em toda a maquinaria de esqueleto
//! da casa **sem uma linha nova**, e a informação *quem é tronco e quem é folha* fica
//! guardada em vez de se perder.
//!
//! ⚠️ **E é irrecuperável se não for emitida agora**: uma nuvem de pontos já não sabe
//! distinguir tronco de folha, e nenhum consumidor a jusante o pode reconstruir.
//!
//! # A posição é calculada como o `rig.fk` a calcula, e isso é um contrato
//!
//! `P[i] = P[pai] + len·(cos, sin)/‖(cos, sin)‖`, com o MESMO seno parabólico (HR-5) e a
//! MESMA normalização. Não é coincidência de fórmula: é o que faz `source.lsystem → rig.fk`
//! ser a identidade **ao bit**, e há gate a medi-lo (`ph2d-node-registry-init`).
//! Sem a normalização, o par parabólico está ~0,1 % fora do círculo unitário e cada osso
//! sairia 0,1 % comprido ou curto **conforme o ângulo** — a planta cresceria torta de uma
//! maneira que nenhum knob explica.
//!
//! # O alfabeto
//!
//! | símbolo | o quê | argumento |
//! |---|---|---|
//! | `F` `G` | anda e **desenha** (nasce um elemento) | comprimento |
//! | `f` `g` | anda **sem** desenhar — e CORTA a cadeia (ver abaixo) | comprimento |
//! | `J` `K` `M` | pousa um elemento **sem** segmento (folha, flor, instância) | — |
//! | `+` `-` | vira à esquerda / direita | ângulo |
//! | `\|` | meia-volta | — |
//! | `[` `]` | empilha / desempilha o estado | — |
//! | `!` | multiplica a **espessura** por `Width Scale` | espessura absoluta |
//! | `"` | multiplica o **passo** por `Length Scale` | passo absoluto |
//! | `%` | **corta** o resto deste ramo | — |
//!
//! Toda outra letra é um módulo **mudo**: existe para a reescrita e não desenha nada. É a
//! metade prática do *homomorfismo* do ABOP (§1.7.2) — o `X` de `F[+X]F[-X]+X` estrutura a
//! planta sem lhe acrescentar um traço.
//!
//! ⚠️ **`f` corta a cadeia de propósito.** Depois de um salto, o elemento seguinte nasce
//! **raiz**: se ele se pendurasse no anterior, `‖P − P[pai]‖` deixaria de ser `len` e a
//! invariante que sustenta o contrato do rig (*"um osso nunca estica"*) passaria a ser falsa
//! — silenciosamente, e só num documento que usasse `f`.

use crate::grammar::Module;
use crate::trig;
use ph2d_nodegraph::attr::{Column, Stream};

const DEGREES_PER_TURN: f32 = 360.0;

/// Uma raiz não tem pai. `f32` porque a coluna `parent` é escalar (o contrato do rig).
const NO_PARENT: f32 = -1.0;

/// Os números que a tartaruga leva do painel.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Setup {
    pub angle: f32,
    pub step: f32,
    pub width: f32,
    pub width_scale: f32,
    pub length_scale: f32,
    pub root_angle: f32,
    /// Quanto o passo se dobra por unidade de `sin` entre a direcção actual e a do tropismo
    /// (graus). `0` = sem tropismo.
    pub tropism: f32,
    /// Para onde o tropismo puxa (graus).
    pub tropism_angle: f32,
    /// **Quanto da viragem da geração mais nova já ABRIU** — `1` = aberta, `0` = fechada.
    ///
    /// Irmã do `youngest.1` (que é a mesma pergunta para o COMPRIMENTO). As duas chegam
    /// resolvidas: quem as calcula é o [`crate::build`], porque escolhê-las exige MEDIR.
    pub(crate) angle_frac: f32,
    /// A geração mais nova e quanto dela já cresceu, `(geração, fracção)`. `fracção = 1`
    /// significa geração inteira.
    pub youngest: (u16, f32),
    /// **O que a coluna `rot` quer dizer** — e a pergunta tem dois donos.
    ///
    /// ⚠️⚠️ **Report do Enio, 2026-08-28: *"as shapes não rotacionam, mas parece que deveriam
    /// rotacionar para se alinhar com a direção do crescimento"*** — e ele tem razão. O
    /// lowering desenha cada instância com o ângulo da coluna **`rot`**
    /// (`ph2d-eval-motion/src/lower.rs`), e o contrato do `rig.*` diz que `rot` é o ângulo
    /// **LOCAL** da junta em relação ao pai. Num galho a direito o local é ≈ `0`, então a forma
    /// carimbada saía sempre em pé.
    ///
    /// ⇒ **Dois consumidores lêem o mesmo nome com sentidos diferentes**, e nenhum está
    /// errado: o desenho pergunta *«para onde esta peça aponta no mundo»*, e o `rig.fk`
    /// pergunta *«quanto esta junta virou em relação à anterior»*. A escolha é do artista, e
    /// é por isso que ela é um param em vez de uma decisão escondida.
    ///
    /// `true` = mundo (o default, o que o desenho quer) · `false` = local (o contrato do rig).
    pub orient_world: bool,
}

#[derive(Clone, Copy)]
struct State {
    x: f32,
    y: f32,
    heading: f32,
    step: f32,
    width: f32,
    /// O elemento que está NESTA posição, ou `-1` se a tartaruga saltou para aqui.
    cur: i32,
    depth: u16,
}

/// As colunas em construção.
struct Out {
    parent: Vec<f32>,
    len: Vec<f32>,
    rot: Vec<f32>,
    wrot: Vec<f32>,
    p: Vec<[f32; 2]>,
    size: Vec<[f32; 2]>,
    depth: Vec<f32>,
    born: Vec<f32>,
    sym: Vec<f32>,
}

impl Out {
    fn with_capacity(n: usize) -> Self {
        Self {
            parent: Vec::with_capacity(n),
            len: Vec::with_capacity(n),
            rot: Vec::with_capacity(n),
            wrot: Vec::with_capacity(n),
            p: Vec::with_capacity(n),
            size: Vec::with_capacity(n),
            depth: Vec::with_capacity(n),
            born: Vec::with_capacity(n),
            sym: Vec::with_capacity(n),
        }
    }

    /// Escreve um elemento e devolve o índice dele.
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        parent: f32,
        len: f32,
        rot: f32,
        wrot: f32,
        p: [f32; 2],
        w: f32,
        depth: u16,
        born: u16,
        sym: u8,
    ) -> i32 {
        self.parent.push(parent);
        self.len.push(len);
        self.rot.push(rot);
        self.wrot.push(wrot);
        self.p.push(p);
        self.size.push([w, w]);
        self.depth.push(f32::from(depth));
        self.born.push(f32::from(born));
        self.sym.push(f32::from(sym));
        (self.parent.len() - 1) as i32
    }
}

/// Os símbolos que fazem nascer um elemento — os únicos cuja idade decide se há crescimento
/// para mostrar. Um `[`, um `+` ou um `!` da geração nova não são um rebento.
pub(crate) fn draws_or_marks(sym: u8) -> bool {
    matches!(sym, b'F' | b'G' | b'f' | b'g' | b'J' | b'K' | b'M')
}

/// `(cos, sin, 1/‖(cos, sin)‖)` na direcção `heading` — os **três** separados, e a separação
/// é o gate.
///
/// ⚠️⚠️ **A ASSOCIAÇÃO IMPORTA.** A 1.ª redacção devolvia o par já normalizado (`c*inv`,
/// `s*inv`) e o passo era `d * (c*inv)`; o `rig.fk` escreve `len * cos * inv`, que associa
/// à ESQUERDA — `(d*c)*inv`. A multiplicação em `f32` **não é associativa**, e o gate
/// `the_fk_pass_does_not_move_a_single_lsystem_element` apanhou a diferença: **1 ULP**, num
/// elemento, só na gramática paramétrica (onde `d` deixa de ser o `step` redondo).
///
/// ⇒ Quem devolve os factores é esta função; quem os MULTIPLICA é o sítio do passo, na mesma
/// ordem que o `rig.fk`. *Uma fórmula igual escrita com outra associação é outro número.*
fn dir(heading: f32) -> (f32, f32, f32) {
    let (c, s) = trig::cos_sin_cycles(heading / DEGREES_PER_TURN);
    let inv = 1.0 / (c * c + s * s).sqrt();
    (c, s, inv)
}

/// O ângulo que o tropismo acrescenta a `heading` depois de um passo (ABOP §2.3.2:
/// `α = e·|H × T|`, com o sinal do produto a dar o lado).
fn tropism_turn(heading: f32, set: &Setup) -> f32 {
    if set.tropism == 0.0 {
        return 0.0;
    }
    let (hc, hs, hi) = dir(heading);
    let (tc, ts, ti) = dir(set.tropism_angle);
    let (hx, hy) = (hc * hi, hs * hi);
    let (tx, ty) = (tc * ti, ts * ti);
    set.tropism * (hx * ty - hy * tx)
}

/// Garante que há um elemento na posição da tartaruga, criando uma RAIZ se ela saltou.
fn anchor(st: &mut State, out: &mut Out, born: u16, sym: u8) -> usize {
    if st.cur < 0 {
        st.cur = out.push(
            NO_PARENT,
            0.0,
            st.heading,
            st.heading,
            [st.x, st.y],
            st.width,
            st.depth,
            born,
            sym,
        );
    }
    st.cur as usize
}

/// Salta do `%` até imediatamente antes do `]` que fecha o ramo actual.
fn cut(chain: &[Module], from: usize) -> usize {
    let mut depth = 0i32;
    let mut j = from + 1;
    while j < chain.len() {
        match chain[j].sym {
            b'[' => depth += 1,
            b']' => {
                if depth == 0 {
                    return j - 1;
                }
                depth -= 1;
            }
            _ => {}
        }
        j += 1;
    }
    chain.len()
}

/// ⭐⭐⭐ **QUANTAS DIREÇÕES A RÉGUA AMOSTRA — medido, não escolhido** (§0.0).
///
/// Rodando a MESMA figura de `0` a `90°` pelo `Root Angle`, uma régua perfeita não muda nada.
/// Medido pela bancada [`examples/probe_ruler.rs`](../../examples/probe_ruler.rs) sobre os oito
/// moldes, e o custo por A/B contra a régua antiga
/// ([`examples/probe_ceiling.rs`](../../examples/probe_ceiling.rs)):
///
/// | régua | ondulação ao rodar | custo no TECTO |
/// |---|---|---|
/// | `max(w, h)` — a de até 2026-08-30 | **`10,9 %`–`32,5 %`** (varia com a forma) | grátis |
/// | `K = 4` | `7,8 %` | ~0,4 ms |
/// | `K = 8` | `1,9 %` | ~0,7 ms |
/// | **`K = 16`** | **`0,48 %`** | **`1,46 ms`** (8,7 % de um quadro) |
/// | `K = 32` | `0,12 %` | ~2,9 ms |
/// | `K = 64` | `0,06 %` | ~5,8 ms |
///
/// ⚠️ **Só o `K = 16` foi cronometrado; as outras linhas são LINEARES nele** (uma passagem
/// sobre a nuvem com `K` acumuladores), e estão marcadas com `~` por isso.
///
/// ⚠️⚠️ **A ondulação da média é uma constante de `K`, não uma propriedade da figura** — medido,
/// um rectângulo `1×3`, uma agulha, um quadrado e uma agulha de aspecto `100` dão os MESMOS
/// `0,48 %` a `K = 16` (só um círculo desce, a `0,115 %` — e ali o número é o MESMO em `K = 8`,
/// `16` e `32`, porque um círculo tem largura constante e não há nada que a discretização
/// deixe por resolver). É a régua
/// de EIXO que varia com a forma, de `10,9 %` num rectângulo `3:1` a `32,5 %` no pior molde.
///
/// ⛔⛔ **E o TECTO desta tabela foi corrigido pela auditoria de 2026-08-30 — a 1.ª redacção
/// nomeava `65 537` elementos e o alcançável é `262 145`, o QUÁDRUPLO.** Eu tinha derivado o
/// pior caso da cadeia do **Dragon**, e o pior caso é da gramática que mais DESENHA: o Dragon é
/// metade viragens, e um `F -> FFFF` é tudo `F`. ⚠️ E ele está **dentro do arrasto do slider**
/// (`8,5` gerações, e o slider vai a 12), não a três vezes acima dele.
///
/// Medido por A/B no tecto (`F -> FFFF` a `g = 8,5`, 262 145 desenhados): a cozedura custa
/// `11,23 ms` com a régua antiga e `12,69` com esta ⇒ **`+1,46 ms`, `+13 %`**. ⚠️ Os `76 %` de
/// um quadro que a cozedura ali custa são **quase todos pré-existentes** — as três travessias
/// de medição, que a lei do crescimento já pagava antes desta mudança.
///
/// ⇒ o `16` é o joelho: **`23×`–`68×`** melhor que a caixa de eixo por `8,7 %` de um quadro no
/// pior caso, e por `0,05 ms` (`0,3 %`) num molde do catálogo no máximo do slider.
///
/// ⚠️ **O recurso é o QUADRO, e o número diz de que ele é.**
const WIDTH_DIRECTIONS: usize = 16;

/// ⭐⭐⭐ **O TAMANHO desta figura — a LARGURA MÉDIA de Cauchy**, e ela é o oráculo de toda a
/// lei do crescimento.
///
/// `largura(u) = max⟨P,u⟩ − min⟨P,u⟩`, e o que se devolve é a MÉDIA dela sobre
/// [`WIDTH_DIRECTIONS`] direções uniformes no semicírculo. Para um convexo isto é o
/// `perímetro/π`.
///
/// # ⚠️⚠️ Por que não é a caixa alinhada aos eixos (report do Enio, 2026-08-30)
///
/// *"em dragon enquanto cresce (aumentando Generations) parece piscar"*. Ele estava a ver
/// um defeito da RÉGUA: até 2026-08-30 isto devolvia `max(w, h)`, que **não é invariante à
/// rotação** — e a curva do dragão **roda `45°` por geração** por construção. A lei põe o
/// que esta função devolve numa rampa recta; quando a caixa troca de lado longo, a lei passa
/// a fixar a OUTRA dimensão e o tamanho verdadeiro **estagna e depois arranca**. Medido: o
/// menor passo do arrasto era `4,5 %` do passo médio (uma paragem), e a régua de eixo lia
/// `66,6 %` — *cega ao defeito que ela própria causava*.
///
/// # ⛔ Duas réguas invariantes foram MEDIDAS e REJEITADAS, pela mesma causa
///
/// | tentativa | por que caiu |
/// |---|---|
/// | raio de giração (RMS) | é medida de **distribuição**: ao atravessar uma geração a contagem de elementos DUPLICA e os novos nascem coincidentes com os pais ⇒ salto puro de amostragem (Tree: passo `−7 991 %` do médio) |
/// | maior distância ao **centroide** | o centroide salta pela mesma razão (Tree `151×`, Wild `395×` de ondulação) |
///
/// ⇒ a régua tem de ser um EXTENSO **sem centroide**: `max − min` é invariante à translação
/// por construção, e pontos coincidentes não o movem.
///
/// ⚠️ **As direções saem do [`dir`], não de `f32::cos`** — a mesma tabela sem transcendentais
/// que a tartaruga usa, por HR-5. O `0,09 %` de erro de direção é comum às três medições que
/// a lei compara, então cancela na razão.
///
/// ⛔ **E uma terceira hipótese caiu por medição: a figura NÃO salta de sítio.** Um
/// deslocamento lê-se da cadeira como um salto de tamanho, e a lei não olha para onde a figura
/// está — mas medido (`examples/probe_drift.rs`), o pior salto de posição do Dragon num passo do
/// slider é **`0,51 %` do tamanho dele, o MENOR dos oito moldes**, contra `10,75 %` do Tree, de
/// que ninguém se queixou. *O molde acusado é o que menos se mexe.*
///
/// ⚠️ A bancada que mede o defeito e a cura é
/// [`examples/probe_flicker.rs`](../../examples/probe_flicker.rs) (o arrasto, com o observador
/// invariante ao lado da régua de eixo); a que escolhe o `K` é
/// [`examples/probe_ruler.rs`](../../examples/probe_ruler.rs).
///
/// ⚠️ **Chama o `walk` e deita o stream fora, de propósito.** Uma segunda travessia «leve»
/// seria a MESMA lei escrita duas vezes, e é a família de defeito que este módulo já pagou
/// três vezes; o preço de uma alocação vale mais que a divergência.
///
/// ⚠️ **Ela só corre numa geração fraccionária** — numa inteira a âncora não é precisa e
/// ninguém a mede (medido: a inteira é `2,5×`–`2,7×` mais barata). ⛔ **Com uma excepção que a
/// auditoria de 2026-08-30 nomeou: `Growth < 1` torna TODA posição do slider fraccionária**
/// (`g = 12,0` custa `0,114 ms` com `Growth = 1,0` e `0,305 ms` com `0,999`).
pub(crate) fn mean_width(chain: &[Module], set: &Setup) -> f32 {
    let s = walk(chain, set);
    let Some(Column::Vec2(v)) = s.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    // As direções, UMA vez por travessia — nunca por elemento (a mesma cerca do `powf`).
    let mut u = [(0.0f32, 0.0f32); WIDTH_DIRECTIONS];
    for (k, slot) in u.iter_mut().enumerate() {
        let deg = DEGREES_PER_TURN / 2.0 * k as f32 / WIDTH_DIRECTIONS as f32;
        // ⚠️ `sn`, e não `s`: o `s` desta função é o stream, e sombreá-lo aqui já enganou uma
        // leitura.
        let (c, sn, inv) = dir(deg);
        *slot = (c * inv, sn * inv);
    }
    // ⚠️ **Os pontos por FORA e as direções por DENTRO** — uma passagem só sobre a nuvem, com
    // os 32 acumuladores (16 mínimos + 16 máximos) em registos. Ao contrário (`K` passagens
    // completas) paga-se `K` vezes o tráfego de cache: medido, a forma invertida é **1,24×**
    // mais lenta. ⚠️ A bancada `probe_ruler.rs` usa a forma INVERTIDA, então os relógios dela
    // são um tecto — é por isso que a coluna de custo da tabela do [`WIDTH_DIRECTIONS`] vem de
    // um A/B do PRODUTO (`probe_ceiling.rs`) e não dela.
    let mut lo = [f32::MAX; WIDTH_DIRECTIONS];
    let mut hi = [f32::MIN; WIDTH_DIRECTIONS];
    for q in v {
        for k in 0..WIDTH_DIRECTIONS {
            let t = q[0] * u[k].0 + q[1] * u[k].1;
            lo[k] = lo[k].min(t);
            hi[k] = hi[k].max(t);
        }
    }
    // ⛔ **RECUSA MEDIDA (auditoria 2026-08-30): NÃO troque o `f32::min/max` por um `if`.**
    // A asm não vectoriza (o corpo é escalar, 16 `minss` + 16 `maxss` totalmente desenrolados),
    // o que convida à hipótese de que um `if t < lo[k]` seria mais rápido. Medido: ele é
    // **2,4× MAIS LENTO** com bits idênticos — o `minss` é sem ramo, e o `if` gera saltos
    // imprevisíveis sobre dados geométricos.
    let total: f32 = (0..WIDTH_DIRECTIONS).map(|k| hi[k] - lo[k]).sum();
    total / WIDTH_DIRECTIONS as f32
}

/// **Interpreta a cadeia** e devolve o stream de elementos.
pub(crate) fn walk(chain: &[Module], set: &Setup) -> Stream {
    let mut out = Out::with_capacity(chain.len() / 2 + 1);
    let mut st = State {
        x: 0.0,
        y: 0.0,
        heading: set.root_angle,
        step: set.step,
        width: set.width,
        cur: -1,
        depth: 0,
    };
    // A raiz da planta nasce ANTES do primeiro símbolo: sem ela um axioma que comece por
    // `[` não teria a que se pendurar, e o primeiro ramo sairia solto.
    st.cur = out.push(
        NO_PARENT,
        0.0,
        st.heading,
        st.heading,
        [0.0, 0.0],
        st.width,
        0,
        0,
        b'^',
    );
    // ⚠️ **A tartaruga não DECIDE mais nada sobre o crescimento** — ela recebe as duas
    // fracções já resolvidas e desenha. A lei mudou-se para o [`crate::build`] em 2026-08-29
    // porque ela passou a precisar de MEDIR (percorrer a geração anterior e a nova), e uma lei
    // que mede não cabe dentro do laço que ela mede. *Quem decide precisa do que o desenho
    // custa; quem desenha só precisa de dois números.*
    let youngest = set.youngest;
    let ang_frac = set.angle_frac;
    let mut stack: Vec<State> = Vec::new();
    let mut i = 0usize;
    while i < chain.len() {
        let m = &chain[i];
        // Quanto desta geração já cresceu: `1` para tudo o que não é a mais nova.
        let grow = if m.born == youngest.0 {
            youngest.1
        } else {
            1.0
        };
        let grow_a = if m.born == youngest.0 { ang_frac } else { 1.0 };
        match m.sym {
            b'F' | b'G' | b'f' | b'g' => {
                let d = m.arg(0).unwrap_or(st.step) * grow;
                let draws = matches!(m.sym, b'F' | b'G');
                if draws {
                    let par = anchor(&mut st, &mut out, m.born, m.sym);
                    let (c, sn, inv) = dir(st.heading);
                    // ⚠️ `d * c * inv`, associado à ESQUERDA — a MESMA ordem do `rig.fk`.
                    // Ver [`dir`]: com `d * (c * inv)` a identidade sai a 1 ULP.
                    let p = [out.p[par][0] + d * c * inv, out.p[par][1] + d * sn * inv];
                    // Ver [`Setup::orient_world`]: o mesmo nome, dois sentidos.
                    let rot = if set.orient_world {
                        st.heading
                    } else {
                        st.heading - out.wrot[par]
                    };
                    st.cur = out.push(
                        par as f32, d, rot, st.heading, p, st.width, st.depth, m.born, m.sym,
                    );
                    (st.x, st.y) = (p[0], p[1]);
                } else {
                    let (c, sn, inv) = dir(st.heading);
                    st.x += d * c * inv;
                    st.y += d * sn * inv;
                    // ⚠️ O salto CORTA a cadeia — ver o cabeçalho.
                    st.cur = -1;
                }
                st.heading += tropism_turn(st.heading, set);
            }
            b'J' | b'K' | b'M' => {
                let par = anchor(&mut st, &mut out, m.born, m.sym);
                // Uma marca não tem osso, mas TEM direcção: ela aponta como o ramo em que
                // pousou. Em local isso é `0` (ela não virou nada em relação ao pai).
                let mark_rot = if set.orient_world { out.wrot[par] } else { 0.0 };
                out.push(
                    par as f32,
                    0.0,
                    mark_rot,
                    out.wrot[par],
                    out.p[par],
                    st.width,
                    st.depth,
                    m.born,
                    m.sym,
                );
            }
            // ⚠️ **A viragem da geração mais nova ABRE com a fracção** — é esta linha que faz
            // uma gramática de refinamento animar. `grow_a` é `1` para tudo o que não é novo.
            b'+' => st.heading += m.arg(0).unwrap_or(set.angle) * grow_a,
            b'-' => st.heading -= m.arg(0).unwrap_or(set.angle) * grow_a,
            b'|' => st.heading += DEGREES_PER_TURN / 2.0,
            b'!' => st.width = m.arg(0).unwrap_or(st.width * set.width_scale),
            b'"' => st.step = m.arg(0).unwrap_or(st.step * set.length_scale),
            b'[' => {
                stack.push(st);
                st.depth = st.depth.saturating_add(1);
            }
            b']' => {
                if let Some(prev) = stack.pop() {
                    st = prev;
                }
            }
            b'%' => i = cut(chain, i),
            _ => {}
        }
        i += 1;
    }

    let n = out.parent.len();
    Stream::new(n)
        .with("P", Column::Vec2(out.p))
        .with("parent", Column::Scalar(out.parent))
        .with("len", Column::Scalar(out.len))
        .with("rot", Column::Scalar(out.rot))
        .with("wrot", Column::Scalar(out.wrot))
        // A ESPESSURA da tartaruga viaja na coluna reservada de escala, e é por isso que ela
        // já é grossa no tronco e fina no galho sem um nó a jusante. Sem nenhum `!` na
        // gramática e com `Width = 1`, isto é exactamente `SIZE_IDENTITY`.
        .with("size", Column::Vec2(out.size))
        // As três colunas que só um L-System sabe, e que qualquer `value.attribute` alcança:
        // *quão fundo no ramo*, *em que geração nasci*, e *que letra me desenhou* — esta
        // última é o que separa uma folha (`J`) de um tronco (`F`) para o `motion.cull` e
        // para o `field.index_range`, sem uma paleta dentro da gramática.
        .with("depth", Column::Scalar(out.depth))
        .with("gen", Column::Scalar(out.born))
        .with("sym", Column::Scalar(out.sym))
        .with(
            "Index",
            Column::Scalar((0..n).map(|i| i as f32).collect::<Vec<_>>()),
        )
        .with("Count", Column::Scalar(vec![n as f32; n]))
}

#[cfg(test)]
#[path = "turtle_tests.rs"]
mod tests;
