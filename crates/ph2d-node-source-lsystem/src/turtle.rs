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
    /// A geração mais nova e quanto dela já cresceu, `(geração, fracção)`. `fracção = 1`
    /// significa geração inteira.
    pub youngest: (u16, f32),
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
fn draws_or_marks(sym: u8) -> bool {
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
    // ⭐⭐⭐ **A FRACÇÃO SÓ SE APLICA SE HOUVER ALGO VELHO PARA CONTRASTAR.**
    //
    // ⚠️ **Report do Enio, 2026-08-28: *"a cada ramo que vai nascer tudo se apaga e aparece de
    // vez"*** — e a medição dá-lhe razão: com a gramática clássica `F -> F[+F]F[-F]F` a altura
    // da planta CAI a 25 % em cada cruzamento de geração e volta a subir (13,5 → 10,1 → 40,5
    // → 30,4). Não é a fracção estar partida: é ela não ter sujeito.
    //
    // O mecanismo: aquela regra reescreve **o próprio símbolo que desenha**, então ao fim de
    // cada passagem **todo** módulo de desenho é da geração mais nova — «o rebento» é a planta
    // inteira, e escalar «o rebento» escala tudo. A lei estava certa e o conjunto a que ela se
    // aplica estava vazio de contraste.
    //
    // ⇒ Se nenhum módulo de desenho for VELHO, a geração é INTEIRA. A planta passa a saltar
    // entre inteiros (que é honesto: aquela gramática de facto refina a planta toda, e triplica
    // de altura a cada passagem) em vez de encolher para nada e voltar — *um passo é uma
    // mudança, um pisca-pisca é uma mentira sobre o que crescer parece*.
    //
    // ⛔ **A alternativa NÃO foi construída, e está nomeada:** escalar a planta INTEIRA por um
    // factor que a faça coincidir com a geração anterior em `frac = 0` faria mesmo uma
    // gramática de refinamento animar (ela dá um *zoom* auto-semelhante). Custa uma segunda
    // derivação para medir a razão, e muda o que `Generations` quer dizer — é decisão de
    // produto, não uma correcção.
    let has_old_drawing = chain
        .iter()
        .any(|m| m.born != set.youngest.0 && draws_or_marks(m.sym));
    let youngest = if has_old_drawing {
        set.youngest
    } else {
        (set.youngest.0, 1.0)
    };
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
                    let rot = st.heading - out.wrot[par];
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
                out.push(
                    par as f32,
                    0.0,
                    0.0,
                    out.wrot[par],
                    out.p[par],
                    st.width,
                    st.depth,
                    m.born,
                    m.sym,
                );
            }
            b'+' => st.heading += m.arg(0).unwrap_or(set.angle),
            b'-' => st.heading -= m.arg(0).unwrap_or(set.angle),
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
