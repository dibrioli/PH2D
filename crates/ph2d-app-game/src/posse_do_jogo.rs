//! ⭐ **De quem é o teclado e o rato com a cena a correr** (CodeEditor item 7). A lei, o oráculo do Godot 4.7.2 e as
//! decisões do dono: `docs/CodeEditor/06_posse_do_teclado.md`. A decisão inteira mora aqui, sem janela; a shell só
//! pergunta (o observador do teclado, o clique, o início do quadro, a moldura).

use crate::posse_do_jogo_registo as registo;
use ph2d_input::{Event, InputState, Key, MouseButton};

/// ⭐ **Um botão do rato para o JOGO**: só um clique que COMEÇA na cena conta (um clique no Play
/// não é um tiro); o largar passa sempre, senão um arrasto que sai da cena ficava preso.
fn game_mouse(
    input: &mut InputState,
    button: ph2d_host::PointerButton,
    down: bool,
    on_scene: bool,
) {
    let b = match button {
        ph2d_host::PointerButton::Primary => MouseButton::Left,
        ph2d_host::PointerButton::Secondary => MouseButton::Right,
        ph2d_host::PointerButton::Middle => MouseButton::Middle,
    };
    if !down {
        input.apply_event(Event::MouseButtonUp(b));
    } else if on_scene {
        input.apply_event(Event::MouseButtonDown(b));
    }
}

/// O que a shell vê da cena num instante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cena {
    /// O relógio anda (`Playhead::is_playing`).
    pub a_correr: bool,
    /// A cena está à vista e nada escreve por cima dela: nenhuma aba de documento, nenhum campo de texto.
    pub a_vista: bool,
    /// O jogo está no palco (janela própria ou ecrã cheio) — [`Tapa::palco`].
    pub palco: bool,
}

/// O que pode tapar a cena num instante (a shell preenche; a lei é [`Tapa::a_vista`]).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tapa {
    /// O jogo está no palco (janela própria ou ecrã cheio): o jogo é o que se vê.
    pub palco: bool,
    /// Uma aba de documento (Painter, editor de animações, de scripts…) está à frente no editor.
    pub aba: bool,
    /// Um campo de texto tem o teclado.
    pub texto: bool,
    /// Um modo do editor dono do teclado: a paleta, um menu ou diálogo, a prévia do Morph.
    pub modo: bool,
    /// A Timeline anima a cena (sessão «Animating», doc 17 item 11): a agulha a tocar é a prévia do
    /// editor, e o rato na cena é das curvas e das chaves, não do jogo.
    pub animar: bool,
}

impl Tapa {
    /// ⭐ No palco as abas ficam ATRÁS do jogo e não o tapam (report do dono 25/10: com o editor de
    /// animações aberto a janela do jogo e o ecrã cheio não recebiam teclas); o texto e os modos tapam sempre.
    #[must_use]
    pub fn a_vista(self) -> bool {
        (self.palco || !self.aba) && !self.texto && !self.modo && !self.animar
    }
}

/// ⭐ **O foco do jogo**: o Play dá-o (qualquer porta — detectado pela SUBIDA do relógio, nunca porta a porta), a
/// abertura do palco dá-o (é o mesmo gesto, a chegar uns quadros depois: a janela abre depois do quadro do Play), um
/// clique na cena dá-o, um clique fora tira-o (Godot, `06` §2).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FocoDoJogo {
    focado: bool,
    corria: bool,
    palcava: bool,
    tinha: bool,
    visto: Option<(bool, bool, bool)>,
    /// Os botões cujo premido foi SÓ do jogo: o largar deles também o é (as ferramentas do editor não veem um largar
    /// sem premido).
    do_jogo: u8,
}

impl Cena {
    /// O jogo está parado a meio, no palco: nada do editor, que está atrás, pode reagir.
    #[must_use]
    pub fn pausada_no_palco(self) -> bool {
        self.palco && !self.a_correr
    }
}

impl FocoDoJogo {
    /// O jogo tem o teclado: a cena corre, está à vista e o foco é dele.
    #[must_use]
    pub fn tem_o_teclado(&self, cena: Cena) -> bool {
        cena.a_correr && cena.a_vista && self.focado
    }

    /// Uma vez por quadro, ANTES dos leitores do teclado (o jogador e os scripts). Devolve se o jogo tem o teclado.
    pub fn quadro(&mut self, input: &mut InputState, cena: Cena) -> bool {
        let subiu = cena.a_correr && !self.corria;
        let abriu = cena.a_correr && cena.palco && !self.palcava;
        if subiu || abriu {
            self.focado = cena.a_vista;
        }
        self.corria = cena.a_correr;
        self.palcava = cena.a_correr && cena.palco;
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
        let mut d = destino(t, self.tem_o_teclado(cena), previa);
        // ⭐ Jogo pausado no palco: a tecla nua não chega ao editor, que está atrás e não se vê (Delete, G, Tab…).
        d.consumida |= cena.pausada_no_palco() && t.desceu && !t.acorde;
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
        game_mouse(input, botao, desceu, na_cena && cena.a_correr);
        let bit = 1 << (botao as u8);
        if !desceu {
            let so = self.do_jogo & bit != 0;
            self.do_jogo &= !bit;
            return so;
        }
        // O botão do meio é a navegação da vista (o pan): o jogo vê-o, mas ele nunca é SÓ do jogo
        // (report do dono 27/10 — com o Play o pan morria).
        let so = self.premido(input, cena, na_cena)
            && (botao != ph2d_host::PointerButton::Middle || cena.palco);
        if so {
            self.do_jogo |= bit;
        } else {
            self.do_jogo &= !bit;
        }
        registo::clique(&botao, na_cena, so);
        so
    }

    /// Um botão do rato premido: na cena dá o foco ao jogo, fora tira-o. `true` = o clique é SÓ do jogo (nenhum ramo
    /// do editor age, salvo os que são do jogo: o HUD e o laboratório de física).
    pub fn premido(&mut self, input: &mut InputState, cena: Cena, na_cena: bool) -> bool {
        // ⭐ No palco não há editor a quem dar o foco: um clique nos botões do jogo não tira o teclado ao jogo.
        self.focado = na_cena || cena.palco;
        self.assenta(input, cena);
        (cena.a_correr || cena.palco) && cena.a_vista && na_cena
    }

    /// ⛔ Perder a posse LARGA tudo: uma seta presa ao pausar ou ao clicar num painel deixava o personagem a andar.
    fn assenta(&mut self, input: &mut InputState, cena: Cena) {
        // Tapar a cena (uma aba, a paleta, um menu, um campo de texto) tira o foco ao jogo: destapá-la não lho devolve
        // sem um clique ou um Play (Godot, Unity).
        if !cena.a_vista {
            self.focado = false;
        }
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
    /// A repetição automática de uma tecla segura.
    pub repete: bool,
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
            transporte: (t.desceu && !t.repete).then_some(if t.shift {
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
mod tests;
