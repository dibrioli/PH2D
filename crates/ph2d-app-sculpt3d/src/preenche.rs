//! ⭐⭐ **O `Fill`** — pinta a peça inteira com a cor do pincel, respeitando a
//! máscara, e desfaz-se com UM `Ctrl+Z`.
//!
//! Ordem do dono (2026-09-24, *«1 e 2 no mesmo ciclo»*). A lei vive na crate
//! pura ([`ph2d_sculpt3d::preenche`]) e é a MESMA conta de máscara por amostra
//! que o carimbo do pincel faz; aqui mora só o que pede a cena — onde o plano
//! está, a entrada de desfazer e o que a placa tem de voltar a ler.
//!
//! ⚠️ **Os dois canais são preenchidos SEMPRE que existem**, e não um OU outro:
//! a cor por vértice é o que a vista grossa, a exportação e o semear de um
//! plano novo lêem, e o prefixo por-vértice do plano é **ao bit** o mesmo
//! valor (gate na crate pura) — logo preencher os dois nunca os põe a
//! discordar.

use super::{PlanoInteiro, Sculpt3dScene, StrokeUndo};

/// O que o `Fill` fez — para a linha do terminal e para os gates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Preenchido {
    /// Pintou; `fina` diz se o plano de tinta fina foi pintado também.
    Feito { fina: bool },
    /// Tudo o que havia a pintar já estava mascarado: nada mudou, e não há
    /// passo de desfazer.
    NadaMudou,
    /// Não há peça.
    SemPeca,
    /// ⚠️ A meio de um traço o plano está EMPRESTADO ao gesto, e preencher a
    /// peça por baixo dele deixaria o traço a devolver um plano antigo por
    /// cima do preenchido. O botão não é alcançável com o dedo em baixo — a
    /// recusa é a rede.
    TracoAberto,
}

impl Sculpt3dScene {
    /// ⭐⭐ **Preenche a peça activa com a cor do pincel.**
    pub(crate) fn fill_color(&mut self) -> Preenchido {
        if self.stroke.tinta_fina.is_some() {
            return Preenchido::TracoAberto;
        }
        let cor = self.brush.color;
        let level = self.level();
        let Some(obj) = self.obj_mut() else {
            return Preenchido::SemPeca;
        };
        let colors_antes = obj.stack.mesh().colors().map(<[[f32; 3]]>::to_vec);
        let finas_antes = obj.tinta.as_ref().map(PlanoInteiro::de);
        let mudou_vertices = ph2d_sculpt3d::preenche::preenche_vertices(obj.stack.mesh_mut(), cor);
        let (mudou_plano, fina) = match obj.tinta.as_mut() {
            Some(t) => match ph2d_sculpt3d::preenche::preenche_plano(t, obj.stack.mesh(), cor) {
                Ok(m) => (m, true),
                // ⚠️ Um plano que não descreve a malha: o quadro seguinte
                // re-semeia-o da cor por vértice (a `tinta_da_peca::garante`),
                // que acabou de ser preenchida — a peça fica certa, e o
                // terminal di-lo.
                Err(ph2d_sculpt3d::preenche::Recusa::NaoDescreve) => {
                    eprintln!(
                        "[sculpt3d] fill: o plano de tinta fina nao descreve a malha -- \
                         pintei a cor por vertice e o plano volta a nascer dela"
                    );
                    (false, false)
                }
            },
            None => (false, false),
        };
        if !mudou_vertices && !mudou_plano {
            // ⚠️ O `colors_mut` pode ter MATERIALIZADO um plano de cor que não
            // existia; se nada mudou, devolve-se a malha ao que era.
            if colors_antes.is_none() {
                obj.stack.mesh_mut().take_colors();
            }
            return Preenchido::NadaMudou;
        }
        obj.uploaded = false;
        if mudou_plano {
            obj.tinta_suja = true;
        }
        self.record(StrokeUndo::Fill {
            level,
            colors: colors_antes,
            finas: if mudou_plano { finas_antes } else { None },
        });
        self.edits += 1;
        Preenchido::Feito { fina }
    }
}
