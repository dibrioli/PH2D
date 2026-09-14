//! A ponte entre o traço de escultura e a lei do pincel de CONTORNO.
//!
//! ⚠️⚠️ **Este verbo desvia antes do `dab_core` e antes do espelho, e as duas
//! coisas têm razões DIFERENTES:**
//!
//! - **antes do `dab_core`**, porque não há núcleo por-vértice nenhum: a lei não
//!   tem atenuação radial, e a região dela não sai do cursor — sai da **BORDA**;
//! - **antes do espelho**, porque a espec manda cada passagem de simetria
//!   **refazer as fases A–E do zero** para a região dela (§12.1), incluindo
//!   escolher uma âncora nova a partir do ponto reflectido. A expansão genérica
//!   de espelho aplica a MESMA região duas vezes, que é outra coisa.
//!
//! ⚠️⚠️ **E a segunda razão é observável, não teórica:** com a queda no contorno
//! diferente de `CONSTANT` as duas passagens escrevem valores **diferentes** na
//! zona de sobreposição, e sem o filtro de região a segunda **apaga** a
//! primeira. Os autores do alvo acharam isto no dia seguinte ao lançamento do
//! pincel.

use ph2d_mesh::{Face, Mesh};

use crate::{Brush, Dab, Symmetry};

/// O que sobrevive de um evento para o outro dentro de um traço de contorno.
#[derive(Clone, Debug)]
pub(super) struct BoundarySessao {
    /// O censo de bordas — ⚠️ **`O(malha)` e construído UMA vez por traço**
    /// (espec §4.5 exige uma vez por malha; guardá-lo entre traços pede um
    /// carimbo de topologia que a `Mesh` não tem, e está nomeado no handoff).
    topo: ph2d_boundary::Topologia,
    /// As posições da malha **no início do traço**.
    ///
    /// ⚠️ **A malha INTEIRA, e não a pegada** — pela mesma razão da pose: aqui
    /// não há raio que limite quem pode mover-se, e a lei mede tudo a partir
    /// daqui.
    p0: Vec<[f32; 3]>,
    /// As normais de repouso — lidas pelo `INFLATE` e pelo eixo do `BEND`.
    nrm0: Vec<[f32; 3]>,
    /// ⭐ **Uma passagem por combinação de eixos de espelho**, cada uma com a
    /// sua estrutura A–E e o seu arrasto reflectido. Vazia = o traço foi
    /// **recusado** em todas as passagens.
    passagens: Vec<Passagem>,
}

#[derive(Clone, Debug)]
struct Passagem {
    contorno: ph2d_boundary::Contorno,
    /// O arrasto reflectido para a região desta passagem.
    espelho: [f32; 3],
    /// Os eixos que ESTA passagem reflecte — lidos pelo filtro de região.
    simetria: [bool; 3],
}

impl crate::SculptStroke {
    /// A estrutura **viva** do traço a decorrer, o censo dela e as posições de
    /// repouso — o que o indicador precisa para desenhar de graça.
    ///
    /// ⚠️ **A primeira passagem, e não uma por eixo de espelho:** a figura
    /// descreve o gesto que a mão está a fazer; as passagens reflectidas são a
    /// mesma lei noutro sítio, e desenhá-las todas poria linhas sob o cursor que
    /// não respondem a ele. Vazio quando o traço foi recusado em todas.
    pub(crate) fn boundary_sessao_viva(
        &self,
    ) -> Option<(
        &ph2d_boundary::Contorno,
        &ph2d_boundary::Topologia,
        &[[f32; 3]],
    )> {
        let s = self.boundary.as_ref()?;
        let p = s.passagens.first()?;
        Some((&p.contorno, &s.topo, &s.p0))
    }

    /// Um evento de contorno. Devolve quantos vértices se moveram.
    pub(super) fn boundary_dab(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
    ) -> usize {
        let ctrl = brush.boundary.lei(brush);
        let sessao = match self.boundary.take() {
            Some(s) => s,
            None => match Self::boundary_comecar(mesh, brush, dab, sym, &ctrl) {
                Some(s) => {
                    self.boundary_construcoes += 1;
                    s
                }
                None => return 0,
            },
        };
        if sessao.passagens.is_empty() {
            self.boundary = Some(sessao);
            return 0;
        }

        // ⚠️ **As posições vivas são o alvo da escrita** e o `p0` a referência —
        // as duas são precisas, porque cinco dos seis modos partem do repouso e
        // o alisar parte da posição actual (e por isso acumula).
        let mut saida = std::mem::take(&mut self.boundary_saida);
        saida.clear();
        saida.extend_from_slice(mesh.positions());
        for p in &sessao.passagens {
            let mut c = ctrl;
            c.simetria = p.simetria;
            let ev = ph2d_boundary::Evento {
                arrasto: [
                    dab.pull[0] * p.espelho[0],
                    dab.pull[1] * p.espelho[1],
                    dab.pull[2] * p.espelho[2],
                ],
            };
            p.contorno
                .passo(&sessao.topo, &c, &ev, &sessao.p0, &sessao.nrm0, &mut saida);
        }

        // A mesma escrita em três passos do irmão da pose, e pela mesma razão: o
        // `capture` congela o `pre` do undo e tem de correr ANTES de a malha ser
        // escrita.
        let mut movidos = std::mem::take(&mut self.moved);
        movidos.clear();
        for (i, (destino, actual)) in saida.iter().zip(mesh.positions()).enumerate() {
            if actual != destino {
                movidos.push(u32::try_from(i).unwrap_or(u32::MAX));
            }
        }
        for &v in &movidos {
            self.capture(mesh, v);
        }
        let posicoes = mesh.positions_mut();
        for &v in &movidos {
            posicoes[v as usize] = saida[v as usize];
        }
        self.moved = movidos;
        self.boundary_saida = saida;
        self.boundary = Some(sessao);

        // ⚠️ **ESCRITA, e não herdada** — a mesma linha que o tecido e a pose
        // pagam: sem ela, um traço de contorno logo a seguir a um de MÁSCARA
        // subiria a janela errada.
        self.last_paints_mask = false;
        if self.moved.is_empty() {
            return 0;
        }
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// O pen-down: o censo de bordas e as fases A–E, **uma vez por passagem**.
    fn boundary_comecar(
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
        ctrl: &ph2d_boundary::Controlos,
    ) -> Option<BoundarySessao> {
        let p0 = mesh.positions().to_vec();
        let nrm0 = mesh.normals().to_vec();
        // ⛔ **A ocultação de vértices não está ligada, e é uma ausência
        // DECLARADA:** a lei aceita-a (e a espec §4.2 diz que geometria
        // escondida **produz contorno**), mas essa é a única regra da espec
        // **sem fixtura** — o harness do oráculo nunca esconde nada. *Escrito
        // aqui para que quem a ligar saiba onde ela entra, e que tem de
        // reconferir com uma corrida nova.*
        let escondido = vec![false; p0.len()];
        let topo = ph2d_boundary::Topologia::construir(
            p0.len(),
            mesh.faces().iter().map(Face::verts),
            &escondido,
        );
        // ⚠️⚠️ **`1 − p`, e a inversão é a LEI — não um sinal trocado.** As duas
        // casas escrevem a mesma curva com argumentos OPOSTOS, e as duas estão
        // certas em casa: para a [`ph2d_boundary::Curva`] o argumento é *quanto
        // FALTA* (`1 − anel/K`, portanto `1` **na borda**), e para o
        // [`crate::Falloff::weight`] é *quanto já se ANDOU* (`d/R`, portanto `0`
        // no centro do carimbo). Ligadas sem esta linha, o peso na borda lê
        // `curva(1) = 0`: a boca fica **parada** e o miolo dobra — o report do
        // dono de 2026-09-14, medido na coluna do cursor como
        // `0 · 0,126 · 0,332 · 0,452 · 0,440 · 0,330 · 0,172 · 0`, uma CORCOVA
        // onde tem de haver uma rampa.
        //
        // ⛔ **O corpus não o apanhava e não podia:** as 61 fixturas correm a
        // crate **directamente**, com a convenção dela — `51 de 61` fecharam
        // sobre esta ponte partida. *Uma paridade medida a montante de uma
        // conversão não afirma nada sobre a conversão.*
        let curva = |p: f32| brush.falloff.weight(1.0 - p);
        let curva: ph2d_boundary::Curva<'_> = &curva;

        let mut passagens = Vec::new();
        for espelho in espelhos(sym) {
            let contacto = [
                dab.center[0] * espelho[0],
                dab.center[1] * espelho[1],
                dab.center[2] * espelho[2],
            ];
            // ⚠️ **Cada passagem escolhe uma âncora NOVA** a partir do ponto
            // reflectido — e ela **pode não existir**, caso em que aquela
            // passagem simplesmente não deforma nada (espec §12.1).
            let Some(sob) = ph2d_boundary::ancora::mais_proximo(&p0, &escondido, contacto) else {
                continue;
            };
            let mut c = *ctrl;
            c.simetria = [sym.x, sym.y, sym.z];
            let Ok(contorno) = ph2d_boundary::Contorno::comecar(
                &topo,
                &p0,
                &nrm0,
                &escondido,
                sob,
                contacto,
                &c,
                curva,
                ph2d_boundary::Fatores {
                    mascara: mesh.masks(),
                    ..Default::default()
                },
            ) else {
                continue;
            };
            passagens.push(Passagem {
                contorno,
                espelho,
                simetria: [sym.x, sym.y, sym.z],
            });
        }
        Some(BoundarySessao {
            topo,
            p0,
            nrm0,
            passagens,
        })
    }
}

/// Os sinais de reflexão de cada passagem — ⚠️ **um eixo activo DUPLICA as
/// passagens** (espec §12.1), logo com dois eixos são quatro.
fn espelhos(sym: Symmetry) -> Vec<[f32; 3]> {
    let mut saida = vec![[1.0f32, 1.0, 1.0]];
    for (eixo, activo) in [sym.x, sym.y, sym.z].into_iter().enumerate() {
        if !activo {
            continue;
        }
        let atuais = saida.clone();
        for mut e in atuais {
            e[eixo] = -e[eixo];
            saida.push(e);
        }
    }
    saida
}

#[cfg(test)]
#[path = "stroke_boundary_tests.rs"]
mod tests;
