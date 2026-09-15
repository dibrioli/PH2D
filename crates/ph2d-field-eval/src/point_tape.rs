//! ⭐⭐⭐ **A FITA `f64` PONTO A PONTO** — o mesmo grafo, a mesma aritmética, sem o preço por chamada.
//!
//! # ⚠️ O mecanismo, e a versão dele que a MEDIÇÃO derrubou
//!
//! [`Field::at`](crate::Field::at) chamava `Context::eval_xyz`, cujo próprio doc-comment da `fidget`
//! diz *«This is extremely inefficient»*, e que abre assim:
//!
//! ```text
//! pub fn eval(&self, root, vars) -> Result<f64, EvalError> {
//!     let mut cache = vec![None; self.ops.len()].into();   // O(contexto inteiro), por PONTO
//!     self.eval_inner(root, vars, &mut cache)
//! }
//! ```
//!
//! ⛔ **A 1.ª redacção desta nota dizia que ERA isso — a alocação por chamada — e a tabela do preço
//! desmentiu-a:** o caminho novo não aloca nada por amostra e **continua a crescer com o documento**,
//! a `~2,4 ns por nó` contra `~5,2` do velho. ⇒ o que se paga por amostra é **percorrer a fita**; a
//! alocação (mais a `HashMap` de três variáveis e a recursão memoizada com `Option<f64>`) era uma
//! parcela, não a causa. *Uma citação do doc-comment de uma dependência é uma pista, não uma
//! medição* — a tabela vive em `docs/3DModeling/11_a_avaliacao_ponto_a_ponto.md` §11.2.
//!
//! ⇒ é daí que vem a [`PointTape::eval_many`]: se o preço é o percurso, o que há a amortizar é a
//! **descodificação** dele, que é a mesma para todos os pontos.
//!
//! ⭐⭐⭐ **A propriedade que torna esta cura barata de aceitar: ela é BIT-A-BIT a mesma resposta, por
//! CONSTRUÇÃO — não por medição, e muito menos por promessa.** Três factos encaixam:
//!
//! 1. A fita percorre **o mesmo grafo** (`Context::get_op`), e não uma re-derivação da árvore — logo
//!    a desduplicação e o dobramento de constantes que o `Context::import` fez já estão lá dentro.
//! 2. Cada valor sai de `BinaryOpcode::eval` / `UnaryOpcode::eval` — **as funções da própria
//!    `fidget`**, as mesmas que o `eval_inner` chama.
//! 3. A memoização do `eval_inner` faz cada nó ser avaliado **exactamente uma vez**; a ordem
//!    topológica também. Muda a **ordem de visita**, e o valor de um nó só depende dos filhos.
//!
//! ⇒ *nenhuma régua deste módulo se move.* Esta fita não troca a aritmética; troca o **andaime**.
//!
//! ⛔⛔ **E a alternativa óbvia — o gradiente ANALÍTICO da `fidget`, que o [`hybrid`](crate::hybrid)
//! já usa — está RECUSADA com número.** Não é uma questão de `f32` contra `f64`: num **vinco** a
//! derivada não existe, e as duas grandezas divergem por construção — a diferença central com
//! `eps = 1e-4` põe um pé de cada lado e devolve a **média** dos dois gradientes laterais, o
//! analítico escolhe **um ramo**. Medido sobre pontos POSTOS em cima de uma aresta: **`1,876e-1`**,
//! contra uma folga de módulo de `2,0e-2` — **`9,4×`** a folga inteira. Tabela e a armadilha de
//! amostragem que quase inverteu o veredito: `docs/3DModeling/11_a_avaliacao_ponto_a_ponto.md` §11.4
//! e §11.5.
//!
//! ⚠️ **O que ela NÃO é:** o caminho rápido do traçado. Esse é o JIT em lote do
//! [`hybrid`](crate::hybrid), que corre `f32` sobre milhares de pontos de uma vez e continua a ser o
//! que o produto usa para desenhar. Esta fita serve quem pergunta **um ponto de cada vez** — as
//! sondas, os gates e a escolha do que está debaixo do rato.

use fidget::context::{BinaryOpcode, Context, Node, Op, UnaryOpcode};
use fidget::var::Var;
use std::collections::BTreeMap;

/// Um passo da fita. Os índices são **densos e para trás** — o slot de um filho é sempre menor que o
/// do pai, que é o que permite o passe único para a frente.
#[derive(Clone, Copy)]
pub(crate) enum Instr {
    X,
    Y,
    Z,
    /// ⭐⭐⭐ **UMA FOLHA QUE NÃO É UMA EXPRESSÃO** — a escultura, pelo índice dela.
    ///
    /// ⚠️ **Ela não tem avaliador de CPU, e a ausência é a cerca.** A `fidget` não sabe consultar
    /// dados (a álgebra dela é fechada), e é por isso que o [`crate::hybrid`] existe. Aqui a folha
    /// entra como uma **variável genérica**, que o gerador de WGSL traduz para uma chamada — e a
    /// fita que a contém é recusada pelo [`PointTape::eval`] em vez de responder um número.
    Var(u32),
    Const(f64),
    Unary(UnaryOpcode, u32),
    Binary(BinaryOpcode, u32, u32),
}

impl Instr {
    /// ⭐⭐⭐ **Este passo ocupa um REGISTO, ou é uma referência que se reescreve onde é usada?**
    ///
    /// `p.x` é uma componente de um parâmetro e `k[7]` é uma leitura de buffer: as duas custam a
    /// mesma instrução no sítio do uso que custariam num `let`, logo **não há valor para guardar**.
    /// O [`crate::wgsl`] escreve-as **inline** exactamente por isso, e o [`TapeShape::vivos`] e o
    /// [`crate::tape_schedule`] contam o mesmo — *uma régua que conta o que o código emitido não
    /// guarda mede outro programa*.
    ///
    /// ⚠️ **A escultura FICA de fora da isenção**, e não por simetria: `escultura_k(p)` é uma
    /// consulta a uma grade com oito amostras, e reescrevê-la em cada uso **duplica o trabalho**.
    pub(crate) const fn ocupa_registo(self) -> bool {
        matches!(self, Self::Var(_) | Self::Unary(..) | Self::Binary(..))
    }
}

/// O grafo achatado em ordem topológica, com o scratch fora do caminho quente.
///
/// ⚠️ **Um documento cuja raiz é inalcançável ou que pede uma variável que não seja `X`/`Y`/`Z` não
/// tem fita** — e é `None`, não um `Vec` vazio. O `eval_xyz` devolvia `Err` nesses dois casos e o
/// [`Field::at`](crate::Field::at) traduzia-o para `NaN`; a fita tem de dizer a **mesma** coisa, e um
/// `Vec` vazio diria `0.0`.
pub(crate) struct PointTape {
    code: Option<Vec<Instr>>,
    /// O slot da raiz. Ver a nota no fim de [`PointTape::build`].
    raiz: u32,
}

/// ⭐⭐ **Quantas fitas de PONTO foram compiladas** — o gémeo do
/// [`crate::hybrid::FLOAT_TAPES`], para o caminho que o [`crate::Field`] usa.
///
/// # ⚠️ Porque ele nasceu, e o que a ausência dele deixou passar
///
/// O `FLOAT_TAPES` conta a fita do **traçado** (a da [`crate::hybrid::Hybrid`]); esta é a do
/// **ponto** — a que o `Field::new` compila, e que a `Owners` compila **uma por folha**. Um gate
/// escrito contra o contador errado leu **zero de zero** e o piso dele apanhou-o: *uma régua que
/// mede zero nos dois lados é verde e não afirma nada.*
///
/// ⚠️ **É a mesma frase que o doc do `FLOAT_TAPES` já escrevia**, sobre a mesma família de defeito:
/// *um custo que nenhuma sonda conta é um custo que nenhuma mutação mata*. A montagem desta fita é
/// um **JIT**, e ela corre por folha a cada mudança de geometria.
///
/// ⚠️ **Sob `cargo test` ele vê-se entre threads** — ver `docs/Render3d/05` §9. Quem o lê corre por
/// `nextest`, que dá um processo por teste.
#[doc(hidden)]
pub static POINT_TAPES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl PointTape {
    /// Achata a subárvore alcançável a partir de `root`.
    ///
    /// ⚠️ **A travessia é ITERATIVA de propósito.** O `eval_inner` da `fidget` é recursivo, e uma
    /// pilha de perfis encadeados é funda; trocar uma recursão por outra herdaria o estouro em vez
    /// de o deixar para trás.
    pub(crate) fn build(ctx: &Context, root: Node) -> Self {
        Self::build_with_vars(ctx, root, &BTreeMap::new())
    }

    /// ⭐⭐⭐ **A mesma fita, com as VARIÁVEIS GENÉRICAS traduzidas** — ver [`Instr::Var`].
    ///
    /// `vars` diz que índice cada `Var::V` ocupa. ⚠️ Uma variável que não esteja no mapa **mata a
    /// fita** (o mesmo `None` de sempre): ela seria uma folha sem lei, e responder `0` desenharia
    /// um plano onde não há nada.
    pub(crate) fn build_with_vars(
        ctx: &Context,
        root: Node,
        vars: &BTreeMap<fidget::var::Var, u32>,
    ) -> Self {
        Self::build_com(ctx, root, vars, true)
    }

    /// ⭐⭐⭐ **A MESMA fita, com o escalonamento como PARÂMETRO** — ver [`crate::tape_schedule`].
    ///
    /// ⚠️ **`escalonar` é um parâmetro da porta e não uma bandeira global**, pelo mesmo motivo que o
    /// `tecto` do pintor: a sonda que mede o que ele compra tem de o poder atravessar nos dois
    /// sentidos, na mesma corrida. *Uma bandeira global é uma corrida escrita à mão — e um número
    /// que ninguém consegue voltar a medir é um palpite com data.*
    pub(crate) fn build_com(
        ctx: &Context,
        root: Node,
        vars: &BTreeMap<fidget::var::Var, u32>,
        escalonar: bool,
    ) -> Self {
        POINT_TAPES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut slot: BTreeMap<Node, u32> = BTreeMap::new();
        let mut code: Vec<Instr> = Vec::new();
        // `false` = ainda por expandir · `true` = filhos prontos, emite-se agora.
        let mut stack: Vec<(Node, bool)> = vec![(root, false)];

        while let Some((n, pronto)) = stack.pop() {
            if slot.contains_key(&n) {
                continue;
            }
            let Some(op) = ctx.get_op(n) else {
                // `EvalError::BadNode` — o `at` respondia `NaN`.
                return Self {
                    code: None,
                    raiz: 0,
                };
            };
            if !pronto {
                stack.push((n, true));
                for c in op.iter_children() {
                    if !slot.contains_key(&c) {
                        stack.push((c, false));
                    }
                }
                continue;
            }
            // ⚠️ Quando `(n, true)` sai da pilha, todo filho já tem slot: eles foram empilhados
            // ACIMA dele e saem primeiro.
            let instr = match op {
                Op::Input(Var::X) => Instr::X,
                Op::Input(Var::Y) => Instr::Y,
                Op::Input(Var::Z) => Instr::Z,
                // ⭐ Uma variável genérica é uma FOLHA QUE NÃO É EXPRESSÃO — ver [`Instr::Var`].
                // ⚠️ Fora do mapa ela continua a matar a fita, que é o que o `eval_xyz` fazia:
                // `EvalError::MissingVar` condena a avaliação INTEIRA, não só este nó.
                Op::Input(v @ Var::V(_)) => match vars.get(v) {
                    Some(i) => Instr::Var(*i),
                    None => {
                        return Self {
                            code: None,
                            raiz: 0,
                        };
                    }
                },
                Op::Const(c) => Instr::Const(c.0),
                Op::Unary(opcode, a) => Instr::Unary(*opcode, slot[a]),
                Op::Binary(opcode, a, b) => Instr::Binary(*opcode, slot[a], slot[b]),
            };
            slot.insert(n, u32::try_from(code.len()).expect("a fita cabe num u32"));
            code.push(instr);
        }

        // ⚠️ **A raiz LÊ-SE do mapa, e não do fim da fita.** Ela é de facto a última a ser emitida
        // (é o fundo da pilha, e tudo acima dela é descendente dela) — mas *«acontece de ser a
        // última»* é uma invariante que a próxima pessoa a mexer na travessia parte sem aviso.
        let raiz = slot[&root];
        // ⭐⭐⭐ **A ordem da DFS acima é a da `fidget`, e ela põe os N segmentos de uma cadeia de
        // `min` VIVOS ao mesmo tempo** — ver [`crate::tape_schedule`], que a troca por uma ordem
        // topológica com a pressão mínima. A permutação não toca em valor nenhum.
        let (code, raiz) = if escalonar {
            crate::tape_schedule::schedule(&code, raiz)
        } else {
            (code, raiz)
        };
        Self {
            code: Some(code),
            raiz,
        }
    }

    /// `f(x, y, z)` — um passe para a frente, sem alocar.
    ///
    /// ⚠️ **O scratch é `thread_local`, e não um campo.** O `at` recebe `&self`, então um `RefCell`
    /// dentro da [`PointTape`] tirar-lhe-ia o `Sync` — e há gates deste módulo que constroem um campo
    /// por item e os correm nos núcleos todos. Um `Vec` por chamada devolveria metade do preço que
    /// esta fita veio tirar.
    pub(crate) fn eval(&self, x: f64, y: f64, z: f64) -> f64 {
        let Some(code) = self.code.as_deref() else {
            return f64::NAN;
        };
        SCRATCH.with(|s| {
            let mut v = s.borrow_mut();
            // ⚠️ **Pré-dimensionar e escrever por ÍNDICE**, e não `push`: o `push` volta a
            // verificar a capacidade a cada nó, dentro do laço mais quente que esta crate tem.
            cresce(&mut v, code.len());
            for (i, instr) in code.iter().enumerate() {
                v[i] = match *instr {
                    Instr::X => x,
                    Instr::Y => y,
                    Instr::Z => z,
                    Instr::Var(_) => f64::NAN,
                    Instr::Const(c) => c,
                    Instr::Unary(op, a) => op.eval(v[a as usize]),
                    Instr::Binary(op, a, b) => op.eval(v[a as usize], v[b as usize]),
                };
            }
            v[self.raiz as usize]
        })
    }

    /// ⭐⭐⭐ **`N` pontos numa passagem só** — a porta que o gradiente usa.
    ///
    /// # ⚠️ Por que ela existe (e é MEDIDA, não suposta)
    ///
    /// A tabela do preço mostra que o custo é `~2,4 ns por NÓ` e cresce com o documento: o que se
    /// paga por amostra é **percorrer a fita inteira**, e uma boa fatia disso é *descodificar* cada
    /// instrução — carregar o `Instr`, saltar no `match`, resolver os índices. Esse trabalho é o
    /// **mesmo para todos os pontos**.
    ///
    /// ⇒ inverter os dois laços (instrução por fora, ponto por dentro) amortiza a descodificação
    /// por `N`, e deixa a aritmética num laço curto de passo fixo — que é a forma que o
    /// vectorizador reconhece.
    ///
    /// ⭐ **Continua bit-a-bit a mesma resposta:** a faixa `k` calcula exactamente as operações que
    /// a chamada escalar do ponto `k` calcularia, sobre os mesmos operandos. Trocar a ORDEM entre
    /// pontos independentes não toca em nenhum valor.
    pub(crate) fn eval_many<const N: usize>(&self, pts: &[[f64; 3]; N]) -> [f64; N] {
        let Some(code) = self.code.as_deref() else {
            return [f64::NAN; N];
        };
        SCRATCH.with(|s| {
            let mut v = s.borrow_mut();
            cresce(&mut v, code.len() * N);
            for (i, instr) in code.iter().enumerate() {
                let base = i * N;
                match *instr {
                    // ⛔⛔ **Uma fita com [`Instr::Var`] não tem avaliador de CPU** — `NaN` é a mesma resposta
                    // que uma fita AUSENTE dá. Ela só nasce pelo caminho do dispositivo
                    // ([`crate::device`]), que não expõe avaliação nenhuma: a cerca é a ausência
                    // do método, e esta linha é o degenerado seguro se alguém a contornar.
                    Instr::Var(_) => v[base..base + N].fill(f64::NAN),
                    Instr::X => v[base..base + N]
                        .iter_mut()
                        .zip(pts)
                        .for_each(|(o, p)| *o = p[0]),
                    Instr::Y => v[base..base + N]
                        .iter_mut()
                        .zip(pts)
                        .for_each(|(o, p)| *o = p[1]),
                    Instr::Z => v[base..base + N]
                        .iter_mut()
                        .zip(pts)
                        .for_each(|(o, p)| *o = p[2]),
                    Instr::Const(c) => v[base..base + N].fill(c),
                    Instr::Unary(op, a) => {
                        let a = a as usize * N;
                        for k in 0..N {
                            v[base + k] = op.eval(v[a + k]);
                        }
                    }
                    Instr::Binary(op, a, b) => {
                        let (a, b) = (a as usize * N, b as usize * N);
                        for k in 0..N {
                            v[base + k] = op.eval(v[a + k], v[b + k]);
                        }
                    }
                }
            }
            let r = self.raiz as usize * N;
            let mut saida = [0.0; N];
            saida.copy_from_slice(&v[r..r + N]);
            saida
        })
    }
}

/// Quantas faixas a [`PointTape::eval_slice`] leva de uma vez.
///
/// ⚠️ **Medido, não escolhido** — ver `docs/3DModeling/11_a_avaliacao_ponto_a_ponto.md` §11.7. O
/// recurso é a **cache**: o scratch é `código × L` em `f64`, e a partir de certo `L` a fita deixa de
/// caber no L1 e o ganho da descodificação amortizada é comido pelos acessos à memória.
const L: usize = 8;

impl PointTape {
    /// ⭐⭐⭐ **Uma VARREDURA inteira, em faixas de [`L`]** — a porta de quem pergunta muitos pontos.
    ///
    /// # Por que ela existe
    ///
    /// Medido: numa chamada de `worst_gradient` do censo, **`474 552` de ~`600 000`** avaliações são
    /// o teste de banda da casca — `f.at(p).abs() > 0,03` — e cada uma percorria a fita **sozinha**.
    /// São `79 %` do trabalho, feitas no formato mais caro que existe: uma descodificação por ponto.
    ///
    /// ⚠️ **A cauda é PREENCHIDA, não tratada à parte:** um bloco final com menos de `L` pontos
    /// repete o último ponto nas faixas que sobram e **descarta-as** na saída. Uma faixa com lixo
    /// produziria `NaN`/denormais que não se lêem mas se pagam; e cair para o caminho escalar aqui
    /// dentro esbarraria no `RefCell` que este bloco já tem emprestado.
    pub(crate) fn eval_slice(&self, pts: &[[f64; 3]], out: &mut Vec<f64>) {
        out.clear();
        let Some(code) = self.code.as_deref() else {
            out.resize(pts.len(), f64::NAN);
            return;
        };
        out.reserve(pts.len());
        SCRATCH.with(|s| {
            let mut v = s.borrow_mut();
            cresce(&mut v, code.len() * L);
            let mut faixa = [[0.0f64; 3]; L];
            for bloco in pts.chunks(L) {
                let n = bloco.len();
                faixa[..n].copy_from_slice(bloco);
                // A cauda repete o último ponto — dados válidos, resultado descartado.
                for slot in faixa.iter_mut().take(L).skip(n) {
                    *slot = bloco[n - 1];
                }
                for (i, instr) in code.iter().enumerate() {
                    let base = i * L;
                    match *instr {
                        // ⛔ Ver a nota do `eval_many`: sem avaliador de CPU, `NaN`.
                        Instr::Var(_) => v[base..base + L].fill(f64::NAN),
                        Instr::X => {
                            for k in 0..L {
                                v[base + k] = faixa[k][0];
                            }
                        }
                        Instr::Y => {
                            for k in 0..L {
                                v[base + k] = faixa[k][1];
                            }
                        }
                        Instr::Z => {
                            for k in 0..L {
                                v[base + k] = faixa[k][2];
                            }
                        }
                        Instr::Const(c) => v[base..base + L].fill(c),
                        Instr::Unary(op, a) => {
                            let a = a as usize * L;
                            for k in 0..L {
                                v[base + k] = op.eval(v[a + k]);
                            }
                        }
                        Instr::Binary(op, a, b) => {
                            let (a, b) = (a as usize * L, b as usize * L);
                            for k in 0..L {
                                v[base + k] = op.eval(v[a + k], v[b + k]);
                            }
                        }
                    }
                }
                let r = self.raiz as usize * L;
                out.extend_from_slice(&v[r..r + n]);
            }
        });
    }
}

/// ⭐⭐ **Dá espaço ao scratch SEM o limpar** — e o «sem o limpar» é a metade que interessa.
///
/// ⛔ A 1.ª redacção fazia `clear()` + `resize(n, 0.0)`, que **escreve zeros sobre o buffer inteiro a
/// cada chamada** — exactamente o `O(documento) por ponto` que esta wave veio tirar da `fidget`,
/// reintroduzido pela porta das traseiras (numa peça de 2 048 nós, um `gradient_norm` memsetava
/// `98 KB`).
///
/// ⚠️ **Não zerar é seguro por uma invariante da fita, não por sorte:** o slot de cada instrução é
/// escrito antes de qualquer pai o ler, porque a fita está em ordem topológica e os índices apontam
/// só para trás. O lixo da chamada anterior nunca é lido.
fn cresce(v: &mut Vec<f64>, n: usize) {
    if v.len() < n {
        v.resize(n, 0.0);
    }
}

thread_local! {
    /// O andaime reutilizado entre chamadas — cresce até ao maior documento que esta thread viu e
    /// fica lá.
    static SCRATCH: std::cell::RefCell<Vec<f64>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// ⭐⭐ **A fita atravessa threads, e isto é um GATE — não um comentário.**
///
/// A razão de o scratch ser `thread_local` é exactamente esta: um campo com estado mutável por
/// dentro deixaria de ser partilhável, e o `em_paralelo` dos gates deste módulo deixaria de compilar
/// num sítio muito pior que aqui.
const _: () = {
    const fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<PointTape>();
};

/// ⭐⭐⭐ **O RETRATO DE UMA FITA** — o que decide se ela cabe num interpretador de GPU.
///
/// Duas grandezas, e a segunda é a que manda:
///
/// - **`ops`**: quantas instruções a fita tem. É o custo de a percorrer, por amostra.
/// - **`vivos`**: quantos valores estão vivos ao mesmo tempo, no pior instante. ⭐ **É este o
///   scratch por THREAD** de um interpretador: a fita é SSA com índices para trás, logo um slot
///   morre assim que o último leitor dele passa, e os slots reaproveitam-se. *Guardar a fita
///   inteira por invocação seria `8 KB` por thread e nenhuma GPU o aceita; guardar os VIVOS pode
///   ser meia dúzia de registos.*
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TapeShape {
    pub ops: usize,
    pub vivos: usize,
    /// ⭐⭐⭐ **Quantos valores o shader de facto GUARDA** — ver `Instr::ocupa_registo`.
    ///
    /// ⚠️ **Não é o `ops`**: uma constante está na fita e **não** é executada no dispositivo (ela é
    /// escrita onde é usada), logo `ops` conta trabalho que ninguém faz. É este o número de linhas
    /// do WGSL, e é ele que o custo por amostra segue.
    pub guardados: usize,
}

impl PointTape {
    /// A fita, para quem a sabe traduzir — ver [`crate::wgsl`].
    pub(crate) fn code(&self) -> Option<&[Instr]> {
        self.code.as_deref()
    }

    /// O slot da raiz.
    pub(crate) fn root(&self) -> u32 {
        self.raiz
    }

    /// Ver [`TapeShape`]. `None` quando o documento não tem fita.
    pub fn shape(&self) -> Option<TapeShape> {
        let code = self.code.as_ref()?;
        // O último leitor de cada slot — depois dele o valor pode ser reciclado.
        let mut ultimo = vec![0usize; code.len()];
        for (i, instr) in code.iter().enumerate() {
            match instr {
                Instr::Unary(_, a) => ultimo[*a as usize] = i,
                Instr::Binary(_, a, b) => {
                    ultimo[*a as usize] = i;
                    ultimo[*b as usize] = i;
                }
                _ => {}
            }
        }
        ultimo[self.raiz as usize] = code.len();
        // ⚠️ **Quantos morrem em cada passo, contados uma vez** — a redacção anterior varria
        // `ultimo[0..=i]` DENTRO do laço, o que é `O(fita²)`: numa fita de `8 124` instruções (um
        // contorno de 256 arestas) são 33 milhões de comparações para responder a uma contagem.
        // *A sonda que compara duas ORDENS da mesma fita chama isto uma vez por ordem.*
        let mut morrem: Vec<u32> = vec![0; code.len()];
        for (j, u) in ultimo.iter().enumerate() {
            // ⚠️ **Só quem ocupa um registo entra na conta** — ver [`Instr::ocupa_registo`].
            if j != self.raiz as usize && code[j].ocupa_registo() {
                morrem[*u] += 1;
            }
        }
        // Uma varredura para a frente: nasce um, morrem os que já ninguém lê.
        let (mut vivos, mut pico) = (0usize, 0usize);
        for (i, m) in morrem.iter().enumerate() {
            if code[i].ocupa_registo() {
                vivos += 1;
                pico = pico.max(vivos);
            }
            vivos -= *m as usize;
        }
        Some(TapeShape {
            ops: code.len(),
            vivos: pico,
            guardados: code.iter().filter(|i| i.ocupa_registo()).count(),
        })
    }

    /// ⚠️ Só para o gate: quantos passos da fita ocupam um registo — ver [`Instr::ocupa_registo`].
    #[cfg(test)]
    pub(crate) fn ocupam_registo(&self) -> usize {
        self.code
            .as_ref()
            .map_or(0, |c| c.iter().filter(|i| i.ocupa_registo()).count())
    }

    /// ⚠️ **Só para o diagnóstico**: que ESPÉCIE de instrução está viva no instante de pico.
    /// Uma contagem alta não diz de onde vem a pressão, e a cura de cada espécie é diferente.
    #[cfg(test)]
    pub(crate) fn peak_kinds(&self) -> Vec<(String, usize)> {
        let Some(code) = self.code.as_ref() else {
            return Vec::new();
        };
        let mut ultimo = vec![0usize; code.len()];
        for (i, instr) in code.iter().enumerate() {
            match instr {
                Instr::Unary(_, a) => ultimo[*a as usize] = i,
                Instr::Binary(_, a, b) => {
                    ultimo[*a as usize] = i;
                    ultimo[*b as usize] = i;
                }
                _ => {}
            }
        }
        ultimo[self.raiz as usize] = code.len();
        let (mut vivos, mut pico, mut quando) = (0usize, 0usize, 0usize);
        for i in 0..code.len() {
            vivos += usize::from(code[i].ocupa_registo());
            if vivos > pico {
                pico = vivos;
                quando = i;
            }
            vivos -= ultimo
                .iter()
                .enumerate()
                .filter(|(j, u)| **u == i && *j != self.raiz as usize && code[*j].ocupa_registo())
                .count();
        }
        let mut conta: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for (j, u) in ultimo.iter().enumerate() {
            // ⚠️ Só o que ocupa registo — ver [`Instr::ocupa_registo`]. A 1.ª redacção listava
            // as constantes e dizia «1 208 vivos» sobre uma fita cujo pico REAL era 48.
            if j <= quando && *u > quando && code[j].ocupa_registo() {
                let nome = match code[j] {
                    Instr::X => "X".to_string(),
                    Instr::Y => "Y".to_string(),
                    Instr::Z => "Z".to_string(),
                    Instr::Var(_) => "Var".to_string(),
                    Instr::Const(_) => "Const".to_string(),
                    Instr::Unary(op, _) => format!("{op:?}"),
                    Instr::Binary(op, _, _) => format!("{op:?}"),
                };
                *conta.entry(nome).or_default() += 1;
            }
        }
        let mut v: Vec<(String, usize)> = conta.into_iter().collect();
        v.sort_by_key(|e| std::cmp::Reverse(e.1));
        v
    }
}
