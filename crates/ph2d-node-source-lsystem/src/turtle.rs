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
    /// **O comprimento da geração mais nova estica com a fracção** (*Continuous length* do
    /// Houdini). É a lei que anima uma gramática que cresce pela PONTA.
    pub(crate) continuous_length: bool,
    /// **E a viragem dela ABRE com a fracção** (*Continuous angles*). É a lei que anima uma
    /// gramática de REFINAMENTO, que não tem ponta nenhuma para esticar.
    pub(crate) continuous_angle: bool,
    /// ⭐⭐⭐ **DE ONDE O COMPRIMENTO DA GERAÇÃO NOVA PARTE**, numa gramática de refinamento —
    /// o factor que a põe, com as viragens fechadas, exactamente por cima da anterior.
    ///
    /// **MEDIDO** por quem chama (o [`crate::build`] percorre as duas cadeias), nunca contado
    /// a partir da gramática: a razão é geométrica. `1.0` = sem âncora (o neutro).
    pub(crate) anchor: f32,
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

/// **A maior dimensão da caixa que esta cadeia desenha** — o oráculo da âncora.
///
/// ⚠️ **Chama o `walk` e deita o stream fora, de propósito.** Uma segunda travessia «leve»
/// seria a MESMA lei escrita duas vezes, e é a família de defeito que este módulo já pagou
/// três vezes; o preço de uma alocação vale mais que a divergência. E ela **só corre numa
/// geração fraccionária** — numa inteira a âncora não é precisa e ninguém a mede.
pub(crate) fn span(chain: &[Module], set: &Setup) -> f32 {
    let s = walk(chain, set);
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            (x1 - x0).max(y1 - y0)
        }
        _ => 0.0,
    }
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
    // ⭐⭐⭐ **E A ALTERNATIVA NOMEADA ACIMA ERA A CURA ERRADA** — pesquisa de 2026-08-29, a
    // pedido do Enio (*"acho que o ideal é o crescimento suave. como fazem os grandes apps?"*).
    //
    // O L-System SOP do Houdini tem **DOIS** interruptores, não um: *Continuous angles* e
    // *Continuous length* — e a documentação dele diz o que escalam: *"the angles rotated by
    // the last generation's turtle operations are scaled by the amount into the generation,
    // and the lengths taken by the last generation's turtle operations are scaled by the
    // amount into the generation"*.
    //
    // ⇒ **São duas leis para as duas famílias, e eu tinha construído uma só.** Medido pela
    // bancada (`examples/preset_report.rs`, razão de expansão por geração):
    //
    // | família | razão | o que cresce | a lei que a anima |
    // |---|---|---|---|
    // | Tree · Fern · Wild · Sprig | `1,63 → 1,06` (converge para 1) | a PONTA | o **comprimento** estica |
    // | Bush · Koch | **`3,00` em toda geração** | a figura INTEIRA | o **ângulo** abre |
    // | Weed · Dragon | `2,03` · `~1,41` | idem | idem |
    //
    // Uma gramática de refinamento não tem ponta que estique — mas as dobras NOVAS dela podem
    // **abrir** de `0` até ao ângulo cheio, e aí a figura desdobra-se em vez de aparecer.
    // *A fracção não tinha sujeito porque eu estava a procurá-lo na grandeza errada.*
    let has_old_drawing = chain
        .iter()
        .any(|m| m.born != set.youngest.0 && draws_or_marks(m.sym));
    // ⭐⭐⭐ **DUAS LEIS, E O `has_old_drawing` ESCOLHE ENTRE ELAS** — a cura do report do Enio
    // de 2026-08-29 (*"acho que o ideal é o crescimento suave"*).
    //
    // | família | de onde o comprimento novo PARTE | porquê |
    // |---|---|---|
    // | cresce pela PONTA (há desenho velho) | **`0`** | um rebento sai do nada e estica |
    // | REFINA (não há) | **`1/spread`** | os `spread` sub-segmentos deitados em fila cobrem EXACTAMENTE o segmento que substituíram |
    //
    // ⚠️⚠️ **É a âncora `1/spread` que faltava, e é por isso que o `Step Scale` sozinho não
    // curava**: ele aplica uma CONSTANTE, e o que a travessia precisa é de um LERP ancorado na
    // taxa de expansão medida. Com as viragens novas fechadas e o passo a `1/spread`, a
    // geração `n+1` desenha-se **por cima** da `n`; a partir daí a fracção abre as viragens e
    // estica o passo, e a figura desdobra-se em vez de saltar.
    //
    // ⚠️ Numa gramática de refinamento o `+`/`-` da geração anterior **SOBREVIVE** (nenhuma
    // regra o reescreve — só o `F` é reescrito), então fechar as viragens NOVAS deixa a forma
    // antiga de pé. Foi essa a peça que a leitura do código deu e a intuição não.
    let frac = set.youngest.1;
    let (len_frac, ang_frac) = if has_old_drawing {
        // A lei de sempre (a cura do pisca-pisca de 28/08): o rebento estica de zero. O ângulo
        // é inerte aqui por construção — a viragem nova é seguida de um não-terminal que ainda
        // não desenha nada.
        (frac, frac)
    } else {
        // A lei do REFINAMENTO: o passo parte da ÂNCORA MEDIDA, e as viragens abrem.
        //
        // ⛔ **A âncora NÃO se conta a partir da gramática, e tentei.** A `F -> F[+F]F[-F]F`
        // põe **5** módulos de desenho por cada um e a figura cresce **3,00×** — dois deles
        // estão dentro de parênteses e não estendem o caminho. A Koch põe 5 sem parênteses
        // nenhum e cresce **3,00×** na mesma, porque as viragens a dobram. *A razão é
        // geométrica; contar símbolos dá 5 onde a resposta é 3.*
        // ⛔⛔ **VEREDITO DO DONO DO PRODUTO, 2026-08-29: *"os que vc tentou corrigir não
        // ficarão bons"*** — e a medição concorda com ele. Mesmo com a âncora, os quatro ficam
        // em `9 %`/`9 %`/`17 %`/`31 %` de pior passo contra os `5–8 %` dos que crescem pela
        // ponta; e o que se vê não é crescer, é a figura **desdobrar-se**, que é outro gesto.
        // ⇒ a lei fica atrás do `Grow Angle`, que shipa **DESLIGADO**: com ele desligado o
        // caminho é byte-idêntico ao que sempre houve (o degrau inteiro).
        if set.continuous_angle {
            (set.anchor + (1.0 - set.anchor) * frac, frac)
        } else {
            (1.0, 1.0)
        }
    };
    let len_frac = if set.continuous_length { len_frac } else { 1.0 };
    let youngest = (set.youngest.0, len_frac);
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
