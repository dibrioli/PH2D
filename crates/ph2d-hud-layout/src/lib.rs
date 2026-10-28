//! **O HUD que se arruma** — relações entre IRMÃOS de um canvas resolvidas pelo Cassowary incremental (`kasuari`).
//! Porquê e o que se recusou: [ADR-0194](../../../docs/architecture/decisions/0194-o-hud-arruma-irmaos-com-cassowary.md),
//! desenho em `docs/BlenderConstraints/06_W5_resto.md` §3.3. A âncora ao PAI e a raiz do HUD continuam procedurais
//! (`VecAnchors`, `ph2d-hud`); isto corre depois delas, só sobre quem tem relação.
//!
//! # O sistema (o oráculo `docs/BlenderConstraints/ferramentas/oraculo_kiwisolver_hud.py` monta o MESMO)
//!
//! Toda entrada do solver passa pela grelha [`GRELHA`]. Só entra quem está numa relação válida. Por elemento, nesta ordem, com variáveis `x y w h`:
//! - fica onde foi desenhado: `x == caixa.x`, `y == caixa.y`, `w == caixa.w` com os pesos [`PESOS`] (abaixo de
//!   `WEAK`: qualquer relação, até `Fraca`, manda mais que o desenho) e `h == caixa.h` `STRONG`;
//! - `w >= 0`, `h >= 0` `REQUIRED`; dentro do canvas, os quatro lados, `STRONG`.
//!
//! Depois as relações, na ordem dada. O canvas (`x y w h`) e o conteúdo medido de cada `LarguraDoConteudo` são
//! *edit variables* de força [`FORCA_EDICAO`] (acima de `STRONG`; o `REQUIRED` não é permitido numa edição).
//!
//! **Desempate (medido):** com os quatro stays a `WEAK`, como na espec, 10 de 20 casos do oráculo têm ótimo NÃO único
//! (a resposta muda com a ordem de inserção) — a moeda e a pontuação empatam em quem anda. Por isso quem é o `a` de
//! uma relação (o que SEGUE) e não é âncora de ninguém cede primeiro, e mexer a posição custa menos que mexer a
//! largura. (Medido 27/10, W5.5: a pontuação que segue o minimapa E é a âncora da moeda andava com a moeda.)
//!
//! # Regras de robustez
//! Índice fora da lista ou `a == b` ⇒ a relação é ignorada. `REQUIRED` impossível que o solver recusa ⇒ essa
//! restrição fica de fora (a primeira inserida ganha). Nunca pânico.

use std::collections::BTreeMap;

use kasuari::{Constraint, Expression, RelationalOperator as Op, Solver, Strength, Term, Variable};
use serde::{Deserialize, Serialize};

/// A caixa de um elemento no referencial do canvas (y para cima): canto mínimo + tamanho.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caixa {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// `REQUIRED` / `STRONG` / `MEDIUM` / `WEAK` do Cassowary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Forca {
    Requerida,
    Forte,
    Media,
    Fraca,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lado {
    Esquerda,
    Direita,
    Acima,
    Abaixo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eixo {
    X,
    Y,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Relacao<I = usize> {
    /// `a` encostado ao `lado` de `b` com `folga` (Direita: a.x == b.x + b.w + folga; Esquerda: a.x + a.w + folga == b.x;
    /// Acima: a.y == b.y + b.h + folga; Abaixo: a.y + a.h + folga == b.y). No eixo cruzado nada.
    AoLado { a: I, b: I, lado: Lado, folga: f64 },
    /// Os centros de `a` e `b` coincidem no eixo.
    Centrado { a: I, b: I, eixo: Eixo },
    /// `a` não passa a borda de `b` do lado de onde vem: Direita = a fica à esquerda de b (a.x + a.w + folga <= b.x);
    /// Esquerda: a.x >= b.x + b.w + folga; Acima: a.y + a.h + folga <= b.y; Abaixo: a.y >= b.y + b.h + folga.
    NaoInvade { a: I, b: I, lado: Lado, folga: f64 },
    /// A largura de `a` quer ser a do conteúdo (o texto medido) — `a.w == conteudo` com a força do par;
    /// e NUNCA maior que ele (REQUIRED `a.w <= conteudo`). Sem conteúdo medido, inerte.
    LarguraDoConteudo { a: I },
}

impl<I: Copy> Relacao<I> {
    /// `(a, b)`; a largura do conteúdo só tem `a`.
    pub fn ids(&self) -> (I, Option<I>) {
        match *self {
            Self::AoLado { a, b, .. }
            | Self::Centrado { a, b, .. }
            | Self::NaoInvade { a, b, .. } => (a, Some(b)),
            Self::LarguraDoConteudo { a } => (a, None),
        }
    }

    /// O eixo em que a relação manda (a largura do conteúdo é do X).
    pub fn eixo(&self) -> Eixo {
        match *self {
            Self::AoLado { lado, .. } | Self::NaoInvade { lado, .. } => match lado {
                Lado::Esquerda | Lado::Direita => Eixo::X,
                Lado::Acima | Lado::Abaixo => Eixo::Y,
            },
            Self::Centrado { eixo, .. } => eixo,
            Self::LarguraDoConteudo { .. } => Eixo::X,
        }
    }

    /// A mesma relação sobre outros ids (`StableId` → índice da lista); `None` se algum não se traduz.
    pub fn mapeia<J>(&self, mut f: impl FnMut(I) -> Option<J>) -> Option<Relacao<J>> {
        Some(match *self {
            Self::AoLado { a, b, lado, folga } => Relacao::AoLado {
                a: f(a)?,
                b: f(b)?,
                lado,
                folga,
            },
            Self::Centrado { a, b, eixo } => Relacao::Centrado {
                a: f(a)?,
                b: f(b)?,
                eixo,
            },
            Self::NaoInvade { a, b, lado, folga } => Relacao::NaoInvade {
                a: f(a)?,
                b: f(b)?,
                lado,
                folga,
            },
            Self::LarguraDoConteudo { a } => Relacao::LarguraDoConteudo { a: f(a)? },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Elemento {
    pub caixa: Caixa,
    pub conteudo: Option<f64>,
}

/// Força das *edit variables* (canvas e conteúdos): `1000 × STRONG`, abaixo do `REQUIRED` (`1_001_001_000`).
pub const FORCA_EDICAO: f64 = 1.0e9;

/// O solver trabalha numa grelha diádica de `1/256` px: com coeficientes `±1`/`½`, somas e deltas das edições são
/// EXATOS, logo qualquer caminho do simplex dá o mesmo bit — e o caminho muda: o `kasuari` guarda o quadro em
/// `hashbrown` com semente aleatória por processo (foldhash). Medido sem a grelha: ida e volta de `61,3` → `377,9`
/// → `61,3` devolvia `w` a 2 ulp.
pub const GRELHA: f64 = 256.0;

/// Abaixo disto (por lado da caixa) o arranjo NÃO mexeu: a soma de três números na grelha difere da soma
/// arredondada em até `1,5` células (medido 27/10, W5.5: a moeda «andava» `0,0023` com a folga `0,4`). Publicar
/// esse resto pagaria a cópia da geometria todo quadro por uma fracção de pixel.
pub const PARADA: f64 = 2.0 / GRELHA;

/// Um número na grelha do solver (o que se grava, para juntar uma relação não mexer em nada).
#[must_use]
pub fn na_grelha(x: f64) -> f64 {
    g(x)
}

fn g(x: f64) -> f64 {
    (x * GRELHA).round() / GRELHA
}

/// Pesos do «fica onde foi desenhado» (unidades de `WEAK`): posição e largura de quem SEGUE, posição e largura de
/// quem é âncora. Diádicos e sem somas pequenas iguais entre si — o desempate do ótimo. A posição de quem SÓ é âncora
/// (não segue ninguém) é `STRONG`: é onde o artista e as âncoras o puseram (medido 27/10, W5.5: a `0,5` o minimapa
/// era empurrado até à borda antes de a pontuação encolher). Quem segue E é âncora (o meio de uma pilha) fica nos
/// pesos de âncora: a `STRONG` empatava com a âncora de cima (ótimo não único no oráculo).
pub const PESOS: [f64; 4] = [0.125, 0.3125, 0.5, 0.6875];

fn forca(f: Forca) -> Strength {
    match f {
        Forca::Requerida => Strength::REQUIRED,
        Forca::Forte => Strength::STRONG,
        Forca::Media => Strength::MEDIUM,
        Forca::Fraca => Strength::WEAK,
    }
}

/// `Σ coef·var + k  op  0`.
fn restricao(termos: &[(Variable, f64)], k: f64, op: Op, s: Strength) -> Constraint {
    let termos = termos.iter().map(|&(v, c)| Term::new(v, c)).collect();
    Constraint::new(Expression::new(termos, k), op, s)
}

#[derive(Clone, Copy)]
struct Vars {
    x: Variable,
    y: Variable,
    w: Variable,
    h: Variable,
}

/// Uma `LarguraDoConteudo`: as duas restrições, postas só enquanto há conteúdo medido.
struct Largura {
    a: usize,
    conteudo: Variable,
    cs: [Constraint; 2],
    postas: [bool; 2],
}

/// Valida a relação contra `n` elementos: `(a, Some(b))`, `(a, None)` para a largura, ou `None` se é má.
fn indices(r: &Relacao, n: usize) -> Option<(usize, Option<usize>)> {
    let (a, b) = r.ids();
    let ok = a < n && b.is_none_or(|b| b < n && b != a);
    ok.then_some((a, b))
}

fn relacao(r: &Relacao, a: Vars, b: Vars, s: Strength) -> Constraint {
    match *r {
        Relacao::AoLado { lado, folga, .. } => match lado {
            Lado::Direita => restricao(
                &[(a.x, 1.0), (b.x, -1.0), (b.w, -1.0)],
                -g(folga),
                Op::Equal,
                s,
            ),
            Lado::Esquerda => restricao(
                &[(a.x, 1.0), (a.w, 1.0), (b.x, -1.0)],
                g(folga),
                Op::Equal,
                s,
            ),
            Lado::Acima => restricao(
                &[(a.y, 1.0), (b.y, -1.0), (b.h, -1.0)],
                -g(folga),
                Op::Equal,
                s,
            ),
            Lado::Abaixo => restricao(
                &[(a.y, 1.0), (a.h, 1.0), (b.y, -1.0)],
                g(folga),
                Op::Equal,
                s,
            ),
        },
        Relacao::Centrado { eixo, .. } => {
            let (ap, am, bp, bm) = match eixo {
                Eixo::X => (a.x, a.w, b.x, b.w),
                Eixo::Y => (a.y, a.h, b.y, b.h),
            };
            restricao(
                &[(ap, 1.0), (am, 0.5), (bp, -1.0), (bm, -0.5)],
                0.0,
                Op::Equal,
                s,
            )
        }
        Relacao::NaoInvade { lado, folga, .. } => match lado {
            Lado::Direita => restricao(
                &[(a.x, 1.0), (a.w, 1.0), (b.x, -1.0)],
                g(folga),
                Op::LessOrEqual,
                s,
            ),
            Lado::Esquerda => restricao(
                &[(a.x, 1.0), (b.x, -1.0), (b.w, -1.0)],
                -g(folga),
                Op::GreaterOrEqual,
                s,
            ),
            Lado::Acima => restricao(
                &[(a.y, 1.0), (a.h, 1.0), (b.y, -1.0)],
                g(folga),
                Op::LessOrEqual,
                s,
            ),
            Lado::Abaixo => restricao(
                &[(a.y, 1.0), (b.y, -1.0), (b.h, -1.0)],
                -g(folga),
                Op::GreaterOrEqual,
                s,
            ),
        },
        Relacao::LarguraDoConteudo { .. } => unreachable!("a largura monta-se em `Arranjo::new`"),
    }
}

/// Um componente conexo do grafo das relações: o seu próprio solver. Grupos não partilham variáveis, logo o ótimo
/// de cada um é o do sistema inteiro — e o custo deixa de ser quadrático no HUD todo.
struct Grupo {
    solver: Solver,
    canvas: [Variable; 4],
    /// O último valor sugerido a cada edição: sugerir o mesmo é trabalho (o `kasuari` varre as linhas) sem efeito.
    sugerido: BTreeMap<Variable, f64>,
    larguras: Vec<Largura>,
}

impl Grupo {
    fn sugere(&mut self, v: Variable, k: f64) {
        let k = g(k);
        if self.sugerido.get(&v) != Some(&k) {
            self.sugerido.insert(v, k);
            let _ = self.solver.suggest_value(v, k);
        }
    }

    fn edicao(&mut self, v: Variable, k: f64) {
        self.edicao_com(v, k, Strength::new(FORCA_EDICAO));
    }

    fn edicao_com(&mut self, v: Variable, k: f64, s: Strength) {
        let _ = self.solver.add_edit_variable(v, s);
        self.sugere(v, k);
    }
}

fn raiz(pai: &mut [usize], mut i: usize) -> usize {
    while pai[i] != i {
        pai[i] = pai[pai[i]];
        i = pai[i];
    }
    i
}

/// ⭐ O arranjo incremental: monta o sistema uma vez; `atualiza` muda só as edit variables (a caixa do canvas
/// e os conteúdos medidos) e devolve as caixas. Elementos SEM relação devolvem a caixa deles, ao bit, sem entrar no solver.
pub struct Arranjo {
    caixas: Vec<Caixa>,
    /// `(grupo, variáveis)` de quem entra no solver.
    vars: Vec<Option<(usize, Vars)>>,
    grupos: Vec<Grupo>,
    valor: BTreeMap<Variable, f64>,
}

impl Arranjo {
    pub fn new(canvas: Caixa, elementos: &[Elemento], relacoes: &[(Relacao, Forca)]) -> Self {
        let n = elementos.len();
        let validas: Vec<_> = relacoes
            .iter()
            .filter_map(|(r, f)| indices(r, n).map(|ab| (r, *f, ab)))
            .collect();
        let (mut entra, mut segue, mut ancora) = (vec![false; n], vec![false; n], vec![false; n]);
        let mut pai: Vec<usize> = (0..n).collect();
        for &(_, _, (a, b)) in &validas {
            entra[a] = true;
            if let Some(b) = b {
                entra[b] = true;
                segue[a] = true;
                ancora[b] = true;
                let (ra, rb) = (raiz(&mut pai, a), raiz(&mut pai, b));
                pai[ra.max(rb)] = ra.min(rb);
            }
        }
        let mut arr = Arranjo {
            caixas: elementos.iter().map(|e| e.caixa).collect(),
            vars: vec![None; n],
            grupos: Vec::new(),
            valor: BTreeMap::new(),
        };
        // Grupos pela ordem do seu primeiro elemento; dentro de cada um, elementos por índice e relações pela ordem.
        let mut grupo_da_raiz: BTreeMap<usize, usize> = BTreeMap::new();
        let (req, forte) = (Strength::REQUIRED, Strength::STRONG);
        for i in (0..n).filter(|&i| entra[i]) {
            let r = raiz(&mut pai, i);
            let gi = *grupo_da_raiz.entry(r).or_insert_with(|| {
                let cv = [
                    Variable::new(),
                    Variable::new(),
                    Variable::new(),
                    Variable::new(),
                ];
                let mut gr = Grupo {
                    solver: Solver::new(),
                    canvas: cv,
                    sugerido: BTreeMap::new(),
                    larguras: Vec::new(),
                };
                for (v, k) in cv.into_iter().zip([canvas.x, canvas.y, canvas.w, canvas.h]) {
                    gr.edicao(v, k);
                }
                arr.grupos.push(gr);
                arr.grupos.len() - 1
            });
            let gr = &mut arr.grupos[gi];
            let [cx, cy, cw, ch] = gr.canvas;
            let c = elementos[i].caixa;
            let v = Vars {
                x: Variable::new(),
                y: Variable::new(),
                w: Variable::new(),
                h: Variable::new(),
            };
            let (pp, pw) = match (segue[i], ancora[i]) {
                (true, false) => (Strength::new(PESOS[0]), PESOS[1]),
                (true, true) => (Strength::new(PESOS[2]), PESOS[3]),
                (false, _) => (Strength::STRONG, PESOS[3]),
            };
            // O «fica onde foi desenhado» é EDIÇÃO (com a força do stay): a caixa muda sem remontar o sistema.
            for (var, k, s) in [
                (v.x, c.x, pp),
                (v.y, c.y, pp),
                (v.w, c.w, Strength::new(pw)),
                (v.h, c.h, forte),
            ] {
                gr.edicao_com(var, k, s);
            }
            for cn in [
                restricao(&[(v.w, 1.0)], 0.0, Op::GreaterOrEqual, req),
                restricao(&[(v.h, 1.0)], 0.0, Op::GreaterOrEqual, req),
                restricao(&[(v.x, 1.0), (cx, -1.0)], 0.0, Op::GreaterOrEqual, forte),
                restricao(&[(v.y, 1.0), (cy, -1.0)], 0.0, Op::GreaterOrEqual, forte),
                restricao(
                    &[(v.x, 1.0), (v.w, 1.0), (cx, -1.0), (cw, -1.0)],
                    0.0,
                    Op::LessOrEqual,
                    forte,
                ),
                restricao(
                    &[(v.y, 1.0), (v.h, 1.0), (cy, -1.0), (ch, -1.0)],
                    0.0,
                    Op::LessOrEqual,
                    forte,
                ),
            ] {
                let _ = gr.solver.add_constraint(cn);
            }
            arr.vars[i] = Some((gi, v));
        }
        let mut conteudo: BTreeMap<usize, Variable> = BTreeMap::new();
        for (r, f, (a, b)) in validas {
            let (gi, va) = arr.vars[a].expect("quem tem relação tem variáveis");
            let gr = &mut arr.grupos[gi];
            let Some(b) = b else {
                let cvar = *conteudo.entry(a).or_insert_with(|| {
                    let v = Variable::new();
                    gr.edicao(v, elementos[a].conteudo.unwrap_or(0.0));
                    v
                });
                let cs = [
                    restricao(&[(va.w, 1.0), (cvar, -1.0)], 0.0, Op::Equal, forca(f)),
                    restricao(&[(va.w, 1.0), (cvar, -1.0)], 0.0, Op::LessOrEqual, req),
                ];
                let postas = match elementos[a].conteudo {
                    Some(_) => cs.clone().map(|c| gr.solver.add_constraint(c).is_ok()),
                    None => [false; 2],
                };
                gr.larguras.push(Largura {
                    a,
                    conteudo: cvar,
                    cs,
                    postas,
                });
                continue;
            };
            let (_, vb) = arr.vars[b].expect("quem tem relação tem variáveis");
            let _ = gr.solver.add_constraint(relacao(r, va, vb, forca(f)));
        }
        arr
    }

    /// O quadro vivo: as caixas (âncoras, rótulos re-cozidos) E os conteúdos mudam. Elementos a menos ficam com a
    /// última caixa; a mais, ignorados.
    pub fn atualiza_elementos(&mut self, canvas: Caixa, elementos: &[Elemento]) -> Vec<Caixa> {
        for (i, e) in elementos.iter().enumerate().take(self.caixas.len()) {
            self.caixas[i] = e.caixa;
            if let Some((gi, v)) = self.vars[i] {
                let gr = &mut self.grupos[gi];
                for (var, k) in [
                    (v.x, e.caixa.x),
                    (v.y, e.caixa.y),
                    (v.w, e.caixa.w),
                    (v.h, e.caixa.h),
                ] {
                    gr.sugere(var, k);
                }
            }
        }
        let conteudos: Vec<Option<f64>> = (0..self.caixas.len())
            .map(|i| elementos.get(i).map_or(self.conteudo_de(i), |e| e.conteudo))
            .collect();
        self.atualiza(canvas, &conteudos)
    }

    /// O conteúdo que a largura `i` sugeriu por último (quem não tem largura: `None`).
    fn conteudo_de(&self, i: usize) -> Option<f64> {
        let (gi, _) = self.vars[i]?;
        let gr = &self.grupos[gi];
        let l = gr.larguras.iter().find(|l| l.a == i)?;
        l.postas[0]
            .then(|| gr.sugerido.get(&l.conteudo).copied())
            .flatten()
    }

    pub fn atualiza(&mut self, canvas: Caixa, conteudos: &[Option<f64>]) -> Vec<Caixa> {
        for gr in &mut self.grupos {
            for (v, k) in gr
                .canvas
                .into_iter()
                .zip([canvas.x, canvas.y, canvas.w, canvas.h])
            {
                gr.sugere(v, k);
            }
            for li in 0..gr.larguras.len() {
                let (a, cvar) = (gr.larguras[li].a, gr.larguras[li].conteudo);
                match conteudos.get(a).copied().flatten() {
                    Some(c) => {
                        gr.sugere(cvar, c);
                        let l = &mut gr.larguras[li];
                        if l.postas == [false; 2] {
                            l.postas = l.cs.clone().map(|c| gr.solver.add_constraint(c).is_ok());
                        }
                    }
                    None => {
                        let l = &mut gr.larguras[li];
                        for (c, posta) in l.cs.iter().zip(&mut l.postas) {
                            if std::mem::take(posta) {
                                let _ = gr.solver.remove_constraint(c);
                            }
                        }
                    }
                }
            }
            for &(v, k) in gr.solver.fetch_changes() {
                self.valor.insert(v, k);
            }
        }
        let val = |v: Variable| self.valor.get(&v).copied().unwrap_or(0.0);
        self.vars
            .iter()
            .zip(&self.caixas)
            .map(|(v, c)| match v {
                Some((_, v)) => Caixa {
                    x: val(v.x),
                    y: val(v.y),
                    w: val(v.w),
                    h: val(v.h),
                },
                None => *c,
            })
            .collect()
    }
}

/// A ordem que uma relação impõe num eixo: `(eixo, antes, depois)` — antes = menor coordenada (y para cima).
fn ordem<I: Copy>(r: &Relacao<I>) -> Option<(Eixo, I, I)> {
    use Lado::{Abaixo, Acima, Direita, Esquerda};
    // `a` fica DEPOIS de `b`? (AoLado diz onde `a` está; NaoInvade diz de que lado `b` está.)
    let (a, b, a_depois) = match *r {
        Relacao::AoLado { a, b, lado, .. } => (a, b, matches!(lado, Direita | Acima)),
        Relacao::NaoInvade { a, b, lado, .. } => (a, b, matches!(lado, Esquerda | Abaixo)),
        Relacao::Centrado { .. } | Relacao::LarguraDoConteudo { .. } => return None,
    };
    let eixo = r.eixo();
    Some(if a_depois { (eixo, b, a) } else { (eixo, a, b) })
}

/// ⭐ A lei da ferramenta: `nova` CONTRADIZ as `existentes` se fecha um círculo de lados num eixo (a depois de b,
/// …, b depois de a). O Cassowary aceita os dois `REQUIRED` e viola um em silêncio — por isso se recusa ANTES.
#[must_use]
pub fn contradiz<I: Copy + PartialEq>(existentes: &[Relacao<I>], nova: &Relacao<I>) -> bool {
    let Some((eixo, antes, depois)) = ordem(nova) else {
        return false;
    };
    // Há caminho `depois → … → antes` nas existentes do mesmo eixo?
    let arestas: Vec<(I, I)> = existentes
        .iter()
        .filter_map(ordem)
        .filter(|&(e, _, _)| e == eixo)
        .map(|(_, x, y)| (x, y))
        .collect();
    let mut visto = vec![depois];
    let mut fila = vec![depois];
    while let Some(n) = fila.pop() {
        if n == antes {
            return true;
        }
        for &(x, y) in &arestas {
            if x == n && !visto.contains(&y) {
                visto.push(y);
                fila.push(y);
            }
        }
    }
    false
}

/// Uma corrida de um só quadro: `Arranjo::new(..).atualiza(canvas, conteúdos dos elementos)`.
pub fn resolve(canvas: Caixa, elementos: &[Elemento], relacoes: &[(Relacao, Forca)]) -> Vec<Caixa> {
    let conteudos: Vec<Option<f64>> = elementos.iter().map(|e| e.conteudo).collect();
    Arranjo::new(canvas, elementos, relacoes).atualiza(canvas, &conteudos)
}
