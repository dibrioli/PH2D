//! Os AVISOS da luz do quadro (W4) — o que o artista lê no Inspector e no aviso que sobe na tela.
//! Ficheiro irmão do `quadro.rs` pelo tecto de 700 LOC (W5e).

use bevy_ecs::entity::Entity;
use ph2d_editor_core::luz_edits::AvisoDaLuz;
use ph2d_light2d_gpu::{ErroSolver, Motivo, Quem};

impl super::LuzDoQuadro {
    /// ⭐ **O que o artista precisa de saber sobre a lâmpada `e`** no último quadro (W4), do mais
    /// grave para o mais brando: a recusa do solver (a cena inteira apagada), o que a captura fez
    /// com ESTA lâmpada, e os sprites que entraram pelo quad. Recusa que só o terminal vê é botão
    /// mudo — cada caso aqui era um `eprintln!` ou nada.
    #[must_use]
    pub fn avisos_de(&self, e: Entity) -> Vec<AvisoDaLuz> {
        let mut out = Vec::new();
        if let Some(erro) = &self.erro {
            out.push(match erro {
                ErroSolver::GradeGrandeDemais { lado, limite } => AvisoDaLuz::GradeGrandeDemais {
                    lado: *lado,
                    limite: *limite,
                },
                ErroSolver::SemShaderF16 | ErroSolver::ConfigInvalida(_) => {
                    AvisoDaLuz::SolverIndisponivel
                }
                ErroSolver::SemMemoria { bytes } => AvisoDaLuz::SemMemoria {
                    mb: bytes / 1_000_000,
                },
            });
        }
        if let (k @ 1.., true) = self.fina_engrossada()
            && self.acesa()
        {
            out.push(AvisoDaLuz::SombraEngrossada { vezes: 1 << k });
        }
        let indice = self
            .cena
            .luzes_emissoras
            .iter()
            .find(|(quem, _)| *quem == e)
            .map(|&(_, i)| i);
        if let (Some(k), Some(r)) = (indice, &self.relatorio) {
            for f in r.fora.iter().filter(|f| f.quem == Quem::Emissor(k)) {
                out.push(match f.motivo {
                    Motivo::PontoSemArea => AvisoDaLuz::PontoSemTamanho,
                    Motivo::MenorQueOTexel => AvisoDaLuz::PequenaDemaisParaOZoom,
                    Motivo::ForaDaGrade => AvisoDaLuz::ForaDaVista,
                });
            }
        }
        let (por_texel, total) = self.sprites_por_texel;
        if self.acesa() && por_texel < total {
            out.push(AvisoDaLuz::SpritesPeloQuad {
                com_quad: total - por_texel,
                total,
            });
        }
        out
    }

    /// ⭐ **A recusa NOVA deste quadro**, já em palavras (o aviso que sobe na tela): `Some` só no
    /// quadro em que o solver passa a recusar; enquanto a recusa dura, `None`.
    pub fn recusa_nova(&mut self) -> Option<String> {
        let Some(erro) = &self.erro else {
            self.recusa_dita = false;
            return None;
        };
        if self.recusa_dita {
            return None;
        }
        self.recusa_dita = true;
        Some(match erro {
            ErroSolver::GradeGrandeDemais { lado, limite } => ph2d_i18n::tr_with(
                "light2d.toast.grid_too_big",
                &[("lado", lado), ("limite", limite)],
            ),
            ErroSolver::SemShaderF16 | ErroSolver::ConfigInvalida(_) => {
                ph2d_i18n::tr("light2d.toast.solver_unavailable").to_string()
            }
            ErroSolver::SemMemoria { bytes } => ph2d_i18n::tr_with(
                "light2d.toast.out_of_memory",
                &[("mb", &(bytes / 1_000_000))],
            ),
        })
    }
}
