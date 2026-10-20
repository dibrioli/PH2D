//! ⭐ **De quem é o teclado e o rato com a cena a correr** (CodeEditor item 7). A lei, o oráculo do Godot 4.7.2 e as
//! decisões do dono: `docs/CodeEditor/06_posse_do_teclado.md`. A decisão inteira mora aqui, sem janela; a shell só
//! pergunta (o observador do teclado, o clique, o início do quadro, a moldura).

use crate::posse_do_jogo_registo as registo;
use ph2d_input::{Event, InputState, Key};

/// O que a shell vê da cena num instante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cena {
    /// O relógio anda (`Playhead::is_playing`).
    pub a_correr: bool,
    /// A cena está à vista e nada escreve por cima dela: nenhuma aba de documento, nenhum campo de texto.
    pub a_vista: bool,
}

/// ⭐ **O foco do jogo**: o Play dá-o (qualquer porta — detectado pela SUBIDA do relógio, nunca porta a porta), um
/// clique na cena dá-o, um clique fora tira-o (Godot, `06` §2).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FocoDoJogo {
    focado: bool,
    corria: bool,
    tinha: bool,
    visto: Option<(bool, bool, bool)>,
}

impl FocoDoJogo {
    /// O jogo tem o teclado: a cena corre, está à vista e o foco é dele.
    #[must_use]
    pub fn tem_o_teclado(&self, cena: Cena) -> bool {
        cena.a_correr && cena.a_vista && self.focado
    }

    /// Uma vez por quadro, ANTES dos leitores do teclado (o jogador e os scripts). Devolve se o jogo tem o teclado.
    pub fn quadro(&mut self, input: &mut InputState, cena: Cena) -> bool {
        if cena.a_correr && !self.corria {
            self.focado = cena.a_vista;
        }
        self.corria = cena.a_correr;
        self.assenta(input, cena);
        self.tinha
    }

    /// ⭐ **A tecla, de ponta a ponta**: decide ([`destino`]) e entrega ao jogo. `chave` = a tecla no idioma do jogo
    /// (`None` = sem nome: nunca chega ao jogo); `nome` só vai para a régua. O transporte fica para quem chama.
    pub fn tecla(
        &self,
        input: &mut InputState,
        cena: Cena,
        t: Tecla,
        chave: Option<Key>,
        previa: bool,
        nome: &dyn std::fmt::Debug,
    ) -> Destino {
        let d = destino(t, self.tem_o_teclado(cena), previa && cena.a_vista);
        if d.jogo
            && let Some(k) = chave
        {
            input.apply_event(if t.desceu {
                Event::KeyDown(k)
            } else {
                Event::KeyUp(k)
            });
        }
        if t.desceu {
            registo::tecla(nome, d);
        }
        d
    }

    /// ⭐ **Um botão do rato, de ponta a ponta**: o premido NA CENA entra no jogo (o largar sempre) e o premido
    /// muda o foco. `true` = o clique é SÓ do jogo.
    pub fn clique(
        &mut self,
        input: &mut InputState,
        cena: Cena,
        botao: ph2d_host::PointerButton,
        desceu: bool,
        na_cena: bool,
    ) -> bool {
        let na_cena = desceu && na_cena && cena.a_vista;
        crate::script_input::game_mouse(input, botao, desceu, na_cena);
        if !desceu {
            return false;
        }
        let so = self.premido(input, cena, na_cena);
        registo::clique(&botao, na_cena, so);
        so
    }

    /// Um botão do rato premido: na cena dá o foco ao jogo, fora tira-o. `true` = o clique é SÓ do jogo (nenhum ramo
    /// do editor age, salvo os que são do jogo: o HUD e o laboratório de física).
    pub fn premido(&mut self, input: &mut InputState, cena: Cena, na_cena: bool) -> bool {
        self.focado = na_cena;
        self.assenta(input, cena);
        cena.a_correr && cena.a_vista && na_cena
    }

    /// ⛔ Perder a posse LARGA tudo: uma seta presa ao pausar ou ao clicar num painel deixava o personagem a andar.
    fn assenta(&mut self, input: &mut InputState, cena: Cena) {
        let tem = self.tem_o_teclado(cena);
        if self.tinha && !tem {
            input.keyboard.release_all();
        }
        let agora = (cena.a_correr, cena.a_vista, tem);
        if self.visto != Some(agora) {
            self.visto = Some(agora);
            registo::posse(cena.a_correr, cena.a_vista, tem);
        }
        self.tinha = tem;
    }
}

/// Uma tecla física, como a shell a vê.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tecla {
    /// Desceu (`false` = subiu).
    pub desceu: bool,
    /// Ctrl, Alt ou Super carregado.
    pub acorde: bool,
    /// Shift carregado.
    pub shift: bool,
    /// É o F5 — a tecla do transporte (decisão do dono, 20/10: a do Roblox Studio).
    pub f5: bool,
}

/// O que o transporte faz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transporte {
    /// F5: pausa ou continua — a mesma porta do Espaço.
    Alterna,
    /// Shift+F5: pára e rebobina — a mesma porta do botão Rebobinar.
    Para,
}

/// O destino de uma tecla.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Destino {
    /// Entra no teclado do jogo (`InputState`).
    pub jogo: bool,
    /// Nenhum atalho do editor a vê.
    pub consumida: bool,
    /// É o transporte.
    pub transporte: Option<Transporte>,
}

/// ⭐ **A lei da tecla.** `jogo_tem` = [`FocoDoJogo::tem_o_teclado`]; `previa` = a pré-visualização do Morph (o modal
/// que É um jogo pequeno e lê o teclado com a cena parada).
///
/// - F5/Shift+F5 são do transporte, sempre, e nunca do jogo.
/// - O soltar entra sempre no jogo (uma subida não descida é ignorada pelo `KeyboardState`).
/// - Um acorde é sempre do editor (*«os players pulam sozinhos»* com Ctrl+Z).
/// - Com o jogo a ter o teclado, a tecla nua é SÓ dele; sem, não desce nele (Godot: editor em foco ⇒ o jogo nada).
#[must_use]
pub fn destino(t: Tecla, jogo_tem: bool, previa: bool) -> Destino {
    let nada = Destino {
        jogo: false,
        consumida: false,
        transporte: None,
    };
    if t.f5 {
        return Destino {
            consumida: t.desceu,
            transporte: t.desceu.then_some(if t.shift {
                Transporte::Para
            } else {
                Transporte::Alterna
            }),
            ..nada
        };
    }
    if !t.desceu {
        return Destino { jogo: true, ..nada };
    }
    if t.acorde {
        return nada;
    }
    Destino {
        jogo: jogo_tem || previa,
        consumida: jogo_tem,
        ..nada
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_input::{Event, Key};

    const CORRE: Cena = Cena {
        a_correr: true,
        a_vista: true,
    };
    const PARADA: Cena = Cena {
        a_correr: false,
        a_vista: true,
    };
    const SETA: Key = Key(0x25);

    fn nua(desceu: bool) -> Tecla {
        Tecla {
            desceu,
            acorde: false,
            shift: false,
            f5: false,
        }
    }

    fn presa(input: &mut InputState) {
        input.apply_event(Event::KeyDown(SETA));
        input.begin_frame();
    }

    #[test]
    fn o_play_da_o_teclado_ao_jogo() {
        let mut f = FocoDoJogo::default();
        let mut input = InputState::new();
        f.quadro(&mut input, PARADA);
        assert!(!f.tem_o_teclado(PARADA));
        f.quadro(&mut input, CORRE);
        assert!(
            f.tem_o_teclado(CORRE),
            "a subida do relógio dá o foco, sem clique"
        );
    }

    #[test]
    fn com_uma_aba_por_cima_o_play_nao_da_o_teclado() {
        let mut f = FocoDoJogo::default();
        let mut input = InputState::new();
        let tapada = Cena {
            a_correr: true,
            a_vista: false,
        };
        f.quadro(&mut input, tapada);
        assert!(!f.tem_o_teclado(tapada));
        f.quadro(&mut input, CORRE);
        assert!(
            !f.tem_o_teclado(CORRE),
            "fechar a aba não devolve o foco: o Play deu-o a uma cena tapada"
        );
        assert!(
            f.premido(&mut input, CORRE, true),
            "um clique na cena devolve-o"
        );
        assert!(f.tem_o_teclado(CORRE));
    }

    #[test]
    fn um_clique_fora_tira_o_teclado_e_larga_as_teclas() {
        let mut f = FocoDoJogo::default();
        let mut input = InputState::new();
        f.quadro(&mut input, CORRE);
        presa(&mut input);
        assert!(
            !f.premido(&mut input, CORRE, false),
            "o clique num painel é do editor"
        );
        assert!(!f.tem_o_teclado(CORRE));
        assert!(!input.keyboard.held(SETA), "a seta presa larga-se");
        assert!(input.keyboard.released(SETA), "e o jogo vê-a subir");
        assert!(
            f.premido(&mut input, CORRE, true),
            "o clique na cena é só do jogo"
        );
        assert!(f.tem_o_teclado(CORRE));
    }

    #[test]
    fn pausar_larga_as_teclas_presas() {
        let mut f = FocoDoJogo::default();
        let mut input = InputState::new();
        f.quadro(&mut input, CORRE);
        presa(&mut input);
        f.quadro(&mut input, PARADA);
        assert!(!input.keyboard.held(SETA));
        f.quadro(&mut input, CORRE);
        assert!(
            f.tem_o_teclado(CORRE),
            "continuar devolve o teclado ao jogo"
        );
    }

    #[test]
    fn com_a_cena_parada_um_clique_nao_e_do_jogo() {
        let mut f = FocoDoJogo::default();
        let mut input = InputState::new();
        assert!(!f.premido(&mut input, PARADA, true));
    }

    #[test]
    fn com_o_jogo_a_tecla_nua_e_so_do_jogo() {
        let d = destino(nua(true), true, false);
        assert!(d.jogo && d.consumida && d.transporte.is_none());
    }

    #[test]
    fn sem_o_jogo_nenhuma_tecla_desce_nele() {
        let d = destino(nua(true), false, false);
        assert!(!d.jogo && !d.consumida);
    }

    #[test]
    fn a_previa_do_morph_continua_a_ler_o_teclado() {
        let d = destino(nua(true), false, true);
        assert!(d.jogo && !d.consumida, "o modal decide o resto, como antes");
    }

    #[test]
    fn um_acorde_e_sempre_do_editor() {
        let t = Tecla {
            acorde: true,
            ..nua(true)
        };
        for (jogo_tem, previa) in [(true, false), (false, true), (true, true)] {
            let d = destino(t, jogo_tem, previa);
            assert!(!d.jogo && !d.consumida, "{jogo_tem} {previa}");
        }
    }

    #[test]
    fn o_soltar_passa_sempre() {
        for t in [
            nua(false),
            Tecla {
                acorde: true,
                ..nua(false)
            },
        ] {
            for jogo_tem in [true, false] {
                let d = destino(t, jogo_tem, false);
                assert!(d.jogo && !d.consumida, "{t:?} {jogo_tem}");
            }
        }
    }

    #[test]
    fn f5_e_shift_f5_sao_do_transporte_e_nunca_do_jogo() {
        let f5 = Tecla {
            f5: true,
            ..nua(true)
        };
        for jogo_tem in [true, false] {
            let d = destino(f5, jogo_tem, false);
            assert_eq!(d.transporte, Some(Transporte::Alterna));
            assert!(d.consumida && !d.jogo);
            let d = destino(Tecla { shift: true, ..f5 }, jogo_tem, true);
            assert_eq!(d.transporte, Some(Transporte::Para));
            assert!(d.consumida && !d.jogo);
            let d = destino(
                Tecla {
                    desceu: false,
                    ..f5
                },
                jogo_tem,
                false,
            );
            assert!(d.transporte.is_none() && !d.jogo);
        }
    }
}
