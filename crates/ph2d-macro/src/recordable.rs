//! Gravabilidade é contrato de cada porta (Illustrator omite em silêncio o que não grava;
//! aqui cada porta de uma família DECLARA-SE). O censo de cada família compara a lista
//! declarada com as portas presentes no código — uma porta nova sem decisão reprova.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Recordability {
    Recorded,
    /// Recusada em voz alta, com o porquê.
    Refused {
        why: &'static str,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Door {
    pub name: &'static str,
    pub recordability: Recordability,
}

/// Portas presentes no código sem declaração.
#[must_use]
pub fn undecided<'a>(declared: &[Door], present: &[&'a str]) -> Vec<&'a str> {
    present
        .iter()
        .copied()
        .filter(|p| !declared.iter().any(|d| d.name == *p))
        .collect()
}

/// Declarações cuja porta já não existe no código.
#[must_use]
pub fn orphans(declared: &[Door], present: &[&str]) -> Vec<&'static str> {
    declared
        .iter()
        .map(|d| d.name)
        .filter(|n| !present.contains(n))
        .collect()
}
