//! §6 — da cadeia para os mapas afins.
//!
//! Cada vértice é governado, **por segmento**, por um mapa afim. Há um mapa por
//! segmento e por **octante de espelho** (as 8 combinações de sinal dos três
//! eixos), e o octante de um vértice lê-se dos **sinais das coordenadas do
//! próprio vértice**.
//!
//! A lei, em álgebra:
//!
//! ```text
//! X_{i,a}(p)  =  T(Õ) · F · [ R · S · T(Õ − Õ₀) ] · F⁻¹ · T(−Õ) · p
//! ```
//!
//! — leva-se `p` ao referencial do pivô, aplica-se lá dentro *rotação, escala e
//! a translação que a origem sofreu*, e devolve-se.
//!
//! ⚠️ **A espec não prescreve como este mapa é armazenado nem factorizado** — só
//! o que ele faz. Pré-multiplicar por octante (o que fazemos, 8 × n_segmentos
//! por evento) é escolha de quem implementa.

use crate::cadeia::Cadeia;
use crate::vetor::{Rot, V3, add, cruz, direccao_entre_posicoes, escalar, normalizar, ponto, sub};
use crate::{Controlos, Deformacao};

/// O mapa afim de um segmento num octante.
#[derive(Clone, Debug)]
pub struct Mapa {
    origem: V3,
    origem_inicial: V3,
    rot: Rot,
    escala: V3,
    /// A base local do §6. `None` é a identidade — que é o caso de **todos** os
    /// modos menos espremer/esticar.
    quadro: Option<[V3; 3]>,
}

impl Mapa {
    pub fn aplicar(&self, p: V3) -> V3 {
        let mut q = sub(p, self.origem);
        if let Some(f) = self.quadro {
            q = [ponto(q, f[0]), ponto(q, f[1]), ponto(q, f[2])];
        }
        q = add(q, sub(self.origem, self.origem_inicial));
        q = [
            q[0] * self.escala[0],
            q[1] * self.escala[1],
            q[2] * self.escala[2],
        ];
        q = self.rot.aplicar(q);
        if let Some(f) = self.quadro {
            q = add(
                add(escalar(f[0], q[0]), escalar(f[1], q[1])),
                escalar(f[2], q[2]),
            );
        }
        add(q, self.origem)
    }
}

/// Quais eixos este octante reflecte, dado o ponto âncora do traço.
///
/// ⚠️ **Duas inversões cancelam-se:** um ponto inverte a coordenada de um eixo
/// uma vez se o bit do eixo está no octante, e **outra vez** se a coordenada
/// correspondente da âncora for negativa. *Escrever só a primeira metade dá uma
/// deformação espelhada no lado errado, e só em malhas cuja âncora cai em
/// coordenada negativa — que é como este defeito passa despercebido.*
fn reflexoes(octante: usize, ancora: V3, simetria: [bool; 3]) -> [bool; 3] {
    let mut f = [false; 3];
    for (eixo, marca) in f.iter_mut().enumerate() {
        if !simetria[eixo] {
            continue;
        }
        let bit = (octante >> eixo) & 1 == 1;
        *marca = bit != (ancora[eixo] < 0.0);
    }
    f
}

fn espelhar_ponto(mut p: V3, f: [bool; 3]) -> V3 {
    for (eixo, &marca) in f.iter().enumerate() {
        if marca {
            p[eixo] = -p[eixo];
        }
    }
    p
}

fn espelhar_rot(mut r: Rot, f: [bool; 3]) -> Rot {
    for (eixo, &marca) in f.iter().enumerate() {
        if marca {
            r = r.espelhada(eixo);
        }
    }
    r
}

/// O octante de um vértice: **os sinais das coordenadas dele**, e só nos eixos
/// de simetria activos.
pub fn octante(p: V3, simetria: [bool; 3]) -> usize {
    let mut a = 0usize;
    for (eixo, &activo) in simetria.iter().enumerate() {
        if activo && p[eixo] < 0.0 {
            a |= 1 << eixo;
        }
    }
    a
}

/// Constrói os `8 × n_segmentos` mapas deste evento.
pub fn construir(cadeia: &Cadeia, ctrl: &Controlos) -> Vec<Mapa> {
    let espremer = ctrl.deformacao() == Deformacao::Espremer;
    let n = cadeia.segmentos.len();
    let mut saida = Vec::with_capacity(8 * n);
    for octante in 0..8usize {
        let f = reflexoes(octante, cadeia.ancora, ctrl.simetria);
        for seg in &cadeia.segmentos {
            let origem = espelhar_ponto(seg.origem, f);
            let origem_inicial = espelhar_ponto(seg.origem_inicial, f);
            // ⚠️ **É `F` que faz o espremer/esticar agir ao longo do SEGMENTO**
            // e não ao longo do eixo `z` do mundo — e é por isso que naquele
            // modo a orientação vive em `F` e a rotação `R` não é usada.
            let quadro = espremer
                .then(|| {
                    let cabeca = espelhar_ponto(seg.cabeca_inicial, f);
                    // ⭐ A **quarta** consumidora da direcção inicial, e pela
                    // mesma porta: aqui a diferença é entre a cabeça espelhada e
                    // a origem corrente espelhada (§6).
                    direccao_entre_posicoes(origem, cabeca).map(base_com_z)
                })
                .flatten();
            saida.push(Mapa {
                origem,
                origem_inicial,
                rot: if espremer {
                    Rot::IDENTIDADE
                } else {
                    espelhar_rot(seg.rot, f)
                },
                escala: seg.escala,
                quadro,
            });
        }
    }
    saida
}

/// Uma base ortonormal com o eixo `z` dado.
///
/// ⭐ **A escolha de `x` e `y` é LIVRE, e isso é uma propriedade da lei, não
/// desleixo:** o único modo que usa esta base escala `x` e `y` **pelo mesmo
/// factor**, e `diag(a, a, b)` comuta com qualquer rotação em torno de `z` ⇒ o
/// mapa é **invariante** à completação escolhida. *Quem procurar aqui uma
/// convenção do alvo procura uma coisa que não é observável.*
fn base_com_z(z: V3) -> [V3; 3] {
    let auxiliar = if z[0].abs() <= z[1].abs() && z[0].abs() <= z[2].abs() {
        [1.0, 0.0, 0.0]
    } else if z[1].abs() <= z[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let x = normalizar(cruz(auxiliar, z)).unwrap_or([1.0, 0.0, 0.0]);
    let y = cruz(z, x);
    [x, y, z]
}
