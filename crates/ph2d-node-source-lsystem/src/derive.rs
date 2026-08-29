//! **A reescrita** — o axioma aplicado às regras `n` vezes (Lindenmayer 1968).
//!
//! # As três dimensões, e todas as três estão aqui
//!
//! | | o que é | ABOP |
//! |---|---|---|
//! | **paramétrica** | um módulo carrega números, e o sucessor calcula-os | §1.10, cap. 5 |
//! | **estocástica** | várias produções para o mesmo predecessor, com pesos | §1.7 |
//! | **sensível a contexto** | a produção olha para o vizinho, ATRAVESSANDO ramos | §1.8 |
//!
//! ⭐ A vizinhança **respeita os colchetes**, que é a metade que separa um L-System a
//! sério de um `replace()` sobre texto: numa cadeia com ramos, o vizinho da ESQUERDA de um
//! módulo dentro de `[ ]` é o módulo ANTES do colchete (o pai), não o `[`; e o vizinho da
//! DIREITA pode estar dentro de qualquer ramo que comece ali. É o que faz um sinal
//! propagar-se por uma planta em vez de por uma fita.
//!
//! # Cada módulo lembra em que GERAÇÃO nasceu
//!
//! É o que torna as gerações **fraccionárias** possíveis (o *Generations* do L-System SOP
//! do Houdini): com `4.3`, derivam-se 5 gerações e a mais nova cresce a 30 % do
//! comprimento. Um módulo que nenhuma regra reescreveu **mantém a sua geração** — senão
//! toda a planta rejuvenesceria a cada passo, e ela cresceria por pulsos em vez de crescer.
//!
//! # O TETO é a cadeia, nunca o número de iterações
//!
//! A taxa de expansão é uma propriedade da REGRA: `F -> FF` duplica, `F -> F[+F]F[-F]F`
//! quintuplica, e `F -> F` não cresce. Um teto sobre as iterações seria generoso para uma e
//! fatal para a outra. ⇒ A derivação pára quando a geração SEGUINTE não cabe no orçamento,
//! e devolve **quantas completou**.
//!
//! ⚠️ **Gerações INTEIRAS, nunca meia.** Cortar a meio de uma passagem deixa uma cadeia
//! quimera — parte reescrita, parte não — que desenha como uma planta partida, e o artista
//! leria isso como um defeito do nó em vez de um limite. *Uma geração ou nenhuma.*

use crate::grammar::{MAX_ARGS, Module, Rule};
use crate::hash::hash3;
use ph2d_expr::{Bindings, Expr, eval};

/// Símbolos que a procura de contexto ATRAVESSA — a pontuação da tartaruga, que não tem
/// papel na reescrita (`#ignore` do cpfg).
///
/// ⚠️ Sem isto, `A < B` nunca casaria em `A+B`: o vizinho da esquerda de `B` seria o `+`.
fn ignored(sym: u8) -> bool {
    matches!(sym, b'+' | b'-' | b'|' | b'!' | b'"' | b'\'' | b'%')
}

/// O índice logo a seguir ao `]` que fecha o `[` em `open`.
fn skip_branch(chain: &[Module], open: usize) -> usize {
    let mut depth = 0i32;
    let mut j = open;
    while j < chain.len() {
        match chain[j].sym {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
    chain.len()
}

/// **O vizinho da ESQUERDA de `i`**, saltando ramos completos e subindo para o pai quando
/// `i` está dentro de um.
pub(crate) fn left_neighbour(chain: &[Module], i: usize) -> Option<usize> {
    let mut j = i;
    while j > 0 {
        j -= 1;
        match chain[j].sym {
            // Um ramo COMPLETO à esquerda é um irmão, não um antepassado: salta-se inteiro.
            b']' => {
                let mut depth = 0i32;
                loop {
                    match chain[j].sym {
                        b']' => depth += 1,
                        b'[' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    if j == 0 {
                        return None;
                    }
                    j -= 1;
                }
            }
            // Um `[` aberto significa que `i` vive dentro dele ⇒ sobe-se ao pai.
            b'[' => {}
            s if ignored(s) => {}
            _ => return Some(j),
        }
    }
    None
}

/// **Um vizinho da DIREITA de `i` cujo símbolo é `want`** — o vizinho imediato, ou o
/// primeiro módulo de QUALQUER ramo que comece ali (ABOP fig. 1.31).
pub(crate) fn right_match(chain: &[Module], i: usize, want: u8) -> Option<usize> {
    let mut j = i + 1;
    while j < chain.len() {
        let s = chain[j].sym;
        if ignored(s) {
            j += 1;
            continue;
        }
        match s {
            b'[' => {
                if let Some(k) = right_match(chain, j, want) {
                    return Some(k);
                }
                j = skip_branch(chain, j);
            }
            // O fim do ramo em que `i` vive: não há vizinho à direita.
            b']' => return None,
            _ => return (s == want).then_some(j),
        }
    }
    None
}

/// As ligações visíveis a uma expressão de sucessor ou de condição.
struct Env<'a> {
    /// `(nome, valor)` — os formais do predecessor e dos contextos, por ordem de ligação.
    ///
    /// ⚠️ **Uma lista de `&str`, e a fatia empresta das REGRAS.** Uma `String` por formal
    /// por módulo poria uma alocação por letra numa cadeia de 131 072 — a mesma razão pela
    /// qual [`Module`] não tem `Vec`. E a varredura linear é mais barata que um mapa: são
    /// unidades de nomes.
    names: &'a [(&'a str, f32)],
    rand: f32,
    born: f32,
    /// Os params do NÓ, por nome — para o artista poder escrever `F(step*0.5)`.
    params: &'a dyn Fn(&str) -> f32,
}

impl Bindings for Env<'_> {
    fn attr(&self, name: &str) -> f32 {
        // Um formal GANHA de um param do nó: quem escreve `A(step)` está a nomear o
        // argumento, e a regra é sobre ele.
        if let Some((_, v)) = self.names.iter().find(|(n, _)| *n == name) {
            return *v;
        }
        match name {
            "rand" => self.rand,
            "n" => self.born,
            _ => (self.params)(name),
        }
    }
    fn param(&self, name: &str) -> f32 {
        (self.params)(name)
    }
}

/// Liga os formais de `pred` aos argumentos do módulo `m`.
fn bind<'a>(out: &mut Vec<(&'a str, f32)>, formals: &'a [String], m: &Module) {
    for (k, f) in formals.iter().enumerate() {
        out.push((f.as_str(), m.arg(k).unwrap_or(0.0)));
    }
}

/// Liga o predecessor e os contextos de `r` para o módulo em `i`, ou `false` se o contexto
/// não casa. `scratch` é reutilizado entre tentativas (limpo aqui).
fn bind_all<'a>(
    scratch: &mut Vec<(&'a str, f32)>,
    r: &'a Rule,
    chain: &[Module],
    i: usize,
) -> bool {
    scratch.clear();
    bind(scratch, &r.pred.formals, &chain[i]);
    if let Some(l) = &r.left {
        let Some(j) = left_neighbour(chain, i).filter(|j| chain[*j].sym == l.sym) else {
            return false;
        };
        bind(scratch, &l.formals, &chain[j]);
    }
    if let Some(rc) = &r.right {
        let Some(j) = right_match(chain, i, rc.sym) else {
            return false;
        };
        bind(scratch, &rc.formals, &chain[j]);
    }
    true
}

/// **Uma passagem de reescrita.** Devolve `None` se a geração não coubesse no orçamento.
///
/// `step` é a geração que está a ser produzida (a primeira é `1`), e é o que cada módulo
/// novo grava.
fn rewrite(
    chain: &[Module],
    rules: &[Rule],
    step: u16,
    seed: u32,
    budget: usize,
    params: &dyn Fn(&str) -> f32,
) -> Option<Vec<Module>> {
    let mut out: Vec<Module> = Vec::with_capacity(chain.len().saturating_mul(2).min(budget));
    // Reutilizados por módulo — ver a nota do `Env::names`.
    let mut scratch: Vec<(&str, f32)> = Vec::new();
    let mut hits: Vec<usize> = Vec::new();
    for (i, m) in chain.iter().enumerate() {
        hits.clear();
        for (k, r) in rules.iter().enumerate() {
            if r.pred.sym != m.sym || !bind_all(&mut scratch, r, chain, i) {
                continue;
            }
            if let Some(c) = &r.cond {
                let env = Env {
                    names: &scratch,
                    rand: hash3(seed, u32::from(step), i as u32),
                    born: f32::from(step),
                    params,
                };
                if eval(c, &env) == 0.0 {
                    continue;
                }
            }
            hits.push(k);
        }

        // ⚠️ **Nenhuma regra ⇒ o módulo passa INTACTO, com a geração que já tinha.** É a
        // produção identidade do ABOP, e é ela que faz um `F` desenhado na geração 1
        // continuar a ser da geração 1 na geração 7.
        let Some(chosen) = pick(rules, &hits, seed, step, i) else {
            if out.len() >= budget {
                return None;
            }
            out.push(*m);
            continue;
        };
        let rule = &rules[chosen];
        // Re-ligar só a regra ESCOLHIDA: o `scratch` da varredura ficou com a última
        // candidata, que raramente é esta.
        bind_all(&mut scratch, rule, chain, i);
        let env = Env {
            names: &scratch,
            rand: hash3(seed, u32::from(step), i as u32),
            born: f32::from(step),
            params,
        };
        if out.len() + rule.succ.len() > budget {
            return None;
        }
        for sm in &rule.succ {
            let mut nm = Module::bare(sm.sym, step);
            let n = sm.args.len().min(MAX_ARGS);
            for (k, a) in sm.args.iter().take(n).enumerate() {
                nm.args[k] = eval(a, &env);
            }
            nm.nargs = n as u8;
            out.push(nm);
        }
    }
    Some(out)
}

/// **A escolha estocástica**, por peso, com um sorteio determinístico de
/// `(semente, geração, posição)`.
///
/// ⚠️ Uma candidata só ⇒ nenhum sorteio é gasto. Não é optimização: é o que faz uma
/// gramática determinística ser **byte-idêntica qualquer que seja a semente**.
fn pick(rules: &[Rule], hits: &[usize], seed: u32, step: u16, i: usize) -> Option<usize> {
    match hits {
        [] => None,
        [only] => Some(*only),
        _ => {
            let total: f32 = hits.iter().map(|k| rules[*k].weight).sum();
            // Uma lane própria para o sorteio da REGRA, para que ele não seja o mesmo número
            // que o `rand` visível às expressões — senão a escolha do ramo e o tamanho dele
            // ficariam correlacionados, e a planta sairia com um padrão em vez de variedade.
            let mut t = hash3(seed, u32::from(step), i as u32 ^ 0x5bf0_3635) * total;
            for k in hits {
                t -= rules[*k].weight;
                if t <= 0.0 {
                    return Some(*k);
                }
            }
            hits.last().copied()
        }
    }
}

/// O resultado de uma derivação.
pub(crate) struct Derived {
    pub chain: Vec<Module>,
    /// ⭐⭐⭐ **A CADEIA DA GERAÇÃO ANTERIOR** — o oráculo do tamanho de onde a fracção parte.
    ///
    /// A âncora do crescimento suave de uma gramática de refinamento é *«qual é o factor que
    /// põe a geração nova, com as viragens fechadas, exactamente por cima da anterior?»*. Esse
    /// factor é **geométrico** e não se conta a partir da gramática — mede-se, percorrendo as
    /// duas. É o preço que o Enio aprovou em 2026-08-29 (*"desenhar a planta duas vezes"*), e
    /// ele só se paga numa geração FRACCIONÁRIA: numa inteira nada disto corre.
    ///
    /// Vazia quando não houve reescrita nenhuma.
    pub previous: Vec<Module>,
    /// Quantas gerações COMPLETAS correram. Menor do que o pedido ⇒ o orçamento saturou.
    pub generations: u16,
}

/// **Deriva `gens` gerações** a partir de `axiom`, parando ao fim do orçamento.
pub(crate) fn derive(
    axiom: &[Module],
    rules: &[Rule],
    gens: u16,
    seed: u32,
    budget: usize,
    params: &dyn Fn(&str) -> f32,
) -> Derived {
    let mut chain: Vec<Module> = axiom.to_vec();
    chain.truncate(budget);
    let mut done = 0u16;
    let mut previous: Vec<Module> = Vec::new();
    if !rules.is_empty() {
        for step in 1..=gens {
            match rewrite(&chain, rules, step, seed, budget, params) {
                Some(next) => {
                    previous = std::mem::replace(&mut chain, next);
                    done = step;
                }
                None => break,
            }
        }
    }
    Derived {
        chain,
        generations: done,
        previous,
    }
}

/// O axioma, com os argumentos constantes já avaliados (geração `0`).
pub(crate) fn axiom_modules(src: &str, params: &dyn Fn(&str) -> f32) -> Vec<Module> {
    let names: [(&str, f32); 0] = [];
    let env = Env {
        names: &names,
        rand: 0.0,
        born: 0.0,
        params,
    };
    crate::grammar::parse_succ(src)
        .into_iter()
        .map(|sm| {
            let mut m = Module::bare(sm.sym, 0);
            let n = sm.args.len().min(MAX_ARGS);
            for (k, a) in sm.args.iter().take(n).enumerate() {
                m.args[k] = eval(a, &env);
            }
            m.nargs = n as u8;
            m
        })
        .collect()
}

/// Uma expressão constante, para quem só precisa de um número.
#[allow(dead_code)]
pub(crate) fn constant(e: &Expr) -> Option<f32> {
    match e {
        Expr::Const(v) => Some(*v),
        _ => None,
    }
}

#[cfg(test)]
#[path = "derive_tests.rs"]
mod tests;
