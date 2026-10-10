//! **As entradas que o jogo publica para o grafo do Motion** (ciclo 19, doc 128 §3.6).
//!
//! Três escritores — o componente `Motion Input` (pela ponte da `ph2d-app-components`), o Luau
//! (`motion.set` · `motion.color` · `motion.fire`) e os sinais do runtime — e um leitor: a
//! `ph2d-app-motion`, que publica cada nome como o external `$input:<nome>` antes do cozimento,
//! no mesmo quadro (a simulação e os scripts correm antes do Motion).
//!
//! Um RECURSO e não um componente: é estado de jogo, por quadro — fora do save e fora do undo,
//! pela mesma razão do `HealthBarsState`.
//!
//! **Uma linha por escritor**, em ordem estável ([`Escritor`]): o `value.input` lê o campo
//! inteiro, e um param dirigido lê a primeira linha.
//!
//! **Um gatilho é uma CONTAGEM** — quem dispara soma um; o pulso de um tique é a bomba do Motion
//! que o deriva (o quadro não sabe quantos tiques vai cozer).

use bevy_ecs::prelude::Resource;
use std::collections::BTreeMap;

/// Quem escreveu uma linha. A ordem das variantes é a ordem das linhas.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Escritor {
    /// Um script Luau (`motion.set`): um por nome.
    Script,
    /// Um sinal do runtime com o nome da entrada.
    Sinal,
    /// A linha de comando (`PH2D_MOTION_INPUTS`): o jogo lançado já com valores.
    Linha,
    /// Um objecto com o componente `Motion Input` — pelo NOME do objecto, e os bits da entidade
    /// só desempatam (a ordem não muda de uma sessão para outra).
    Objeto { nome: String, bits: u64 },
}

/// O valor de uma linha.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Valor {
    Numero(f32),
    /// RGBA linear.
    Cor([f32; 4]),
}

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct MotionInputs {
    valores: BTreeMap<String, BTreeMap<Escritor, Valor>>,
    disparos: BTreeMap<String, BTreeMap<Escritor, u64>>,
}

fn nome_limpo(nome: &str) -> Option<String> {
    let n = nome.trim();
    (!n.is_empty()).then(|| n.to_string())
}

impl MotionInputs {
    /// Escreve o valor de `escritor` em `nome` (fica até ser reescrito ou esquecido).
    pub fn set(&mut self, nome: &str, escritor: Escritor, valor: Valor) {
        if let Some(n) = nome_limpo(nome) {
            self.valores.entry(n).or_default().insert(escritor, valor);
        }
    }

    /// Um disparo de `escritor` no gatilho `nome`.
    pub fn fire(&mut self, nome: &str, escritor: Escritor) {
        if let Some(n) = nome_limpo(nome) {
            *self
                .disparos
                .entry(n)
                .or_default()
                .entry(escritor)
                .or_insert(0) += 1;
        }
    }

    /// Esquece os VALORES dos objectos que `vivo` não reconhece (a ponte reescreve os dela a
    /// cada quadro). As contagens dos que ficam não se tocam — reiniciá-las seria um disparo.
    pub fn retem_objetos(&mut self, vivo: impl Fn(&str, u64, &str) -> bool) {
        for (nome, linhas) in &mut self.valores {
            linhas.retain(|e, _| match e {
                Escritor::Objeto { nome: o, bits } => vivo(o, *bits, nome),
                _ => true,
            });
        }
        self.valores.retain(|_, l| !l.is_empty());
        for (nome, linhas) in &mut self.disparos {
            linhas.retain(|e, _| match e {
                Escritor::Objeto { nome: o, bits } => vivo(o, *bits, nome),
                _ => true,
            });
        }
        self.disparos.retain(|_, l| !l.is_empty());
    }

    /// Os nomes com valores, cada um com as linhas em ordem.
    pub fn valores(&self) -> impl Iterator<Item = (&str, Vec<Valor>)> {
        self.valores
            .iter()
            .map(|(n, l)| (n.as_str(), l.values().copied().collect()))
    }

    /// Os gatilhos, cada um com as contagens em ordem.
    pub fn disparos(&self) -> impl Iterator<Item = (&str, Vec<u64>)> {
        self.disparos
            .iter()
            .map(|(n, l)| (n.as_str(), l.values().copied().collect()))
    }

    /// O valor que `escritor` publicou em `nome`, se publicou.
    #[must_use]
    pub fn valor_de(&self, nome: &str, escritor: &Escritor) -> Option<Valor> {
        self.valores.get(nome.trim())?.get(escritor).copied()
    }

    /// Esquece tudo (um documento novo, o fim do jogo).
    pub fn clear(&mut self) {
        self.valores.clear();
        self.disparos.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(n: &str, b: u64) -> Escritor {
        Escritor::Objeto {
            nome: n.into(),
            bits: b,
        }
    }

    #[test]
    fn rows_come_in_writer_order_and_a_blank_name_writes_nothing() {
        let mut m = MotionInputs::default();
        m.set("vida", obj("b", 1), Valor::Numero(0.2));
        m.set(" vida ", obj("a", 9), Valor::Numero(0.9));
        m.set("vida", Escritor::Script, Valor::Numero(0.5));
        m.set("  ", Escritor::Script, Valor::Numero(1.0));
        let v: Vec<_> = m.valores().collect();
        assert_eq!(v.len(), 1);
        assert_eq!(
            v[0].1,
            vec![Valor::Numero(0.5), Valor::Numero(0.9), Valor::Numero(0.2)],
            "script, depois os objectos pelo nome"
        );
    }

    #[test]
    fn a_fire_counts_and_forgetting_a_dead_object_keeps_the_others_counts() {
        let mut m = MotionInputs::default();
        m.fire("dano", obj("a", 1));
        m.fire("dano", obj("a", 1));
        m.fire("dano", obj("b", 2));
        m.retem_objetos(|n, _, _| n == "a");
        assert_eq!(m.disparos().collect::<Vec<_>>(), vec![("dano", vec![2])]);
    }
}
