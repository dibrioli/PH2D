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
enum Instr {
    X,
    Y,
    Z,
    Const(f64),
    Unary(UnaryOpcode, u32),
    Binary(BinaryOpcode, u32, u32),
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

impl PointTape {
    /// Achata a subárvore alcançável a partir de `root`.
    ///
    /// ⚠️ **A travessia é ITERATIVA de propósito.** O `eval_inner` da `fidget` é recursivo, e uma
    /// pilha de perfis encadeados é funda; trocar uma recursão por outra herdaria o estouro em vez
    /// de o deixar para trás.
    pub(crate) fn build(ctx: &Context, root: Node) -> Self {
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
                // `EvalError::MissingVar` — o `eval_xyz` só liga `X`, `Y` e `Z`, e o erro dele
                // condena a avaliação INTEIRA, não só este nó.
                Op::Input(Var::V(_)) => {
                    return Self {
                        code: None,
                        raiz: 0,
                    };
                }
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
