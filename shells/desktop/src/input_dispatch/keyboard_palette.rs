//! ⭐⭐ **AS TECLAS DO PALETTE DE NÓS** — irmão dos outros `keyboard_*`, e pela mesma lei:
//! uma FAMÍLIA de teclas vive numa porta própria e o `key_input` chama-a.
//!
//! ⚠️ **O corte foi obrigado pela INTEGRAÇÃO de 2026-09-04** — duas linhas acrescentaram
//! ramos ao `key_input` no mesmo dia (as teclas da Hierarquia e as do redesenho) e o
//! ficheiro passou a `602 / 600`. ⛔ Nenhuma delas o via sozinha; *um tecto de LOC é a
//! única coisa deste repo que só a árvore COMBINADA acusa*.
//!
//! ⭐ Escolheu-se ESTE bloco por ser o mais auto-contido do corpo: ele é **modal** (engole
//! a tecla inteira, press e release) e não partilha estado com nenhum ramo vizinho.

//! # ⛔⛔⛔ ELA VEM ANTES DO `ramo_teclas_3d`, E ISSO FOI UM REPORT (2026-09-20)
//!
//! > Enio: *«O modal não captura o que escrevo. O painel lateral captura os atalhos.»*
//!
//! Esta porta estava **duas linhas abaixo** do `ramo_teclas_3d` no [`super::keyboard::key_input`],
//! e aquele ramo, com a escultura na mão, devolve `true` em `1`–`0`, `G`, `H`, `T`, `S`, `A` e
//! `M`. ⇒ escrever `clay` na busca **trocava o pincel por baixo do modal** e não punha uma letra
//! na caixa. *Uma promessa escrita neste ficheiro («vem PRIMEIRO», logo abaixo) e violada por duas
//! linhas noutro.*
//!
//! ⚠️⚠️ **É a MESMA CLASSE que aquele ramo já pagou uma vez, com outra pergunta.** O doc dele
//! conta: uma porta que perguntava *«a cena existe?»* passou a comer *«os dez dígitos e ~26 letras
//! de todo painel do app, para sempre»* no dia em que o pill fez a cena sobreviver a sair do modo.
//! A cura de então foi perguntar pelo **PONTEIRO** (`sculpt3d_keys_live`) — e um **MODAL é outra
//! pergunta**: enquanto ele está no ecrã, nada por baixo dele tem teclado, esteja o ponteiro onde
//! estiver. *A cura anterior respondeu a uma das duas perguntas, e a nota não disse que havia duas.*
//!
//! ⛔ **Só o capturador de atalhos do Input Map fica acima dela**, e o doc dele diz porquê: ele
//! está a ESCUTAR uma tecla para a gravar, e os dois nunca estão abertos ao mesmo tempo (o
//! press-to-bind vive na janela do *Input Map*).
//!
//! ⚠️ **E ela vem antes do `handler.on_key` também, de propósito:** com um modal aberto, o dedo do
//! jogador e a fita de input não devem ver a tecla — pela mesma lei.
//!
//! ⛔ Gate: `shells/desktop/tests/it/um_modal_aberto_tem_o_teclado_antes_da_cena_3d.rs`, com as
//! três metades e a prova de mutação.

use crate::App;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

impl App {
    /// `true` ⇒ o palette consumiu a tecla e o `key_input` devolve JÁ.
    pub(crate) fn command_palette_keys(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        text: Option<&str>,
    ) -> bool {
        // O palette de "Add Node" (tela cheia, Motion) é MODAL — enquanto aberto ele COME toda tecla:
        // caracteres imprimíveis vão pro campo de busca, Enter escolhe o topo do filtro, Backspace apaga,
        // Escape fecha. Vem PRIMEIRO (antes dos atalhos de painel/ferramenta) para uma letra digitada nunca
        // vazar num atalho de grafo embaixo. O `A` que ABRIU o palette foi capturado no quadro ANTERIOR
        // pelo painel (o palette só abre no quadro seguinte, na ponte), então a tecla de abertura flui normal.
        if !self.command_palette_open() {
            return false;
        }
        if state == ElementState::Pressed {
            if let PhysicalKey::Code(code) = physical_key {
                match code {
                    KeyCode::Escape => {
                        self.command_palette_close();
                        return true;
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        self.command_palette_confirm();
                        return true;
                    }
                    KeyCode::Backspace => {
                        self.command_palette_backspace();
                        return true;
                    }
                    _ => {}
                }
            }
            if let Some(s) = text {
                for ch in s.chars() {
                    self.command_palette_type(ch);
                }
            }
        }
        // Modal: engole TODA a tecla (press e release), aberto ou não haja o que digitar.
        true
    }
}
