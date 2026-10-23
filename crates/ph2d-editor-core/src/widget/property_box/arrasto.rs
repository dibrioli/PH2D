//! ⭐ **Que linha de propriedade está sob o rato** — o 8b do CodeEditor (doc 07 §3.10): arrastar uma
//! linha do Inspector para o código escreve o nome dela.
//!
//! Um painel que oferece as linhas abre uma [`Gravacao`] ao pintar; dentro dela, cada rótulo
//! ([`super::paint_property_label`]) grava a sua área e o texto, com o título do cartão que o
//! cabeçalho pintou por último ([`crate::widget::paint_section_header`]). A shell pergunta [`sob`] ao
//! carregar. O texto é o que o artista lê — quem o traduz para o nome de código é a família.
//!
//! ⚠️ Fora de uma [`Gravacao`] nada se grava: os outros painéis pintam as mesmas linhas.

use crate::zones::Rect;
use std::cell::{Cell, RefCell};

thread_local! {
    static CLIP: Cell<Option<Rect>> = const { Cell::new(None) };
    static CARTAO: RefCell<String> = const { RefCell::new(String::new()) };
    static SCRIPT: RefCell<Option<String>> = const { RefCell::new(None) };
    static LINHAS: RefCell<Vec<Linha>> = const { RefCell::new(Vec::new()) };
}

/// Uma linha pintada: onde está o nome, o que diz e de que cartão é.
#[derive(Clone, Debug, PartialEq)]
pub struct Linha {
    pub area: Rect,
    pub texto: String,
    pub cartao: String,
    /// `Some(script)`: a linha é uma propriedade declarada pelo script com esse nome.
    pub script: Option<String>,
}

/// O painel está a pintar as suas linhas (só as visíveis dentro de `clip` entram).
pub struct Gravacao {
    anterior: Option<Rect>,
}

impl Gravacao {
    /// Começa um quadro novo: esquece as linhas do anterior.
    #[must_use]
    pub fn nova(clip: Rect) -> Self {
        LINHAS.with_borrow_mut(Vec::clear);
        CARTAO.with_borrow_mut(String::clear);
        SCRIPT.set(None);
        Self {
            anterior: CLIP.replace(Some(clip)),
        }
    }
}

impl Drop for Gravacao {
    fn drop(&mut self) {
        CLIP.set(self.anterior);
    }
}

/// Um cabeçalho de cartão: as linhas seguintes são dele.
pub(crate) fn cartao(titulo: &str) {
    if CLIP.get().is_some() {
        CARTAO.with_borrow_mut(|c| titulo.clone_into(c));
        SCRIPT.set(None);
    }
}

/// O cartão corrente mostra as propriedades do script `nome`.
pub fn script_do_cartao(nome: &str) {
    if CLIP.get().is_some() {
        SCRIPT.set(Some(nome.to_owned()));
    }
}

pub(crate) fn linha(area: Rect, texto: &str) {
    let Some(clip) = CLIP.get() else {
        return;
    };
    if texto.is_empty() || !clip.contains(area.x + area.w * 0.5, area.y + area.h * 0.5) {
        return;
    }
    let cartao = CARTAO.with_borrow(Clone::clone);
    LINHAS.with_borrow_mut(|v| {
        v.push(Linha {
            area,
            texto: texto.to_owned(),
            cartao,
            script: None,
        });
    });
}

/// A última linha gravada é uma propriedade do script do cartão ([`script_do_cartao`]).
pub fn e_propriedade_do_script() {
    let script = SCRIPT.with_borrow(Clone::clone);
    LINHAS.with_borrow_mut(|v| {
        if let Some(l) = v.last_mut() {
            l.script = script;
        }
    });
}

/// A linha cujo nome está em `(x, y)` (coordenadas lógicas), no último quadro pintado.
#[must_use]
pub fn sob(x: f32, y: f32) -> Option<Linha> {
    LINHAS.with_borrow(|v| v.iter().rev().find(|l| l.area.contains(x, y)).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIP_DO_PAINEL: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 300.0,
        h: 400.0,
    };

    fn r(y: f32) -> Rect {
        Rect::new(10.0, y, 100.0, 14.0)
    }

    #[test]
    fn cada_linha_leva_o_cartao_em_que_foi_pintada() {
        let _g = Gravacao::nova(CLIP_DO_PAINEL);
        cartao("Health");
        linha(r(20.0), "Max");
        cartao("Script");
        script_do_cartao("heroi.luau");
        linha(r(40.0), "Source");
        linha(r(60.0), "speed");
        e_propriedade_do_script();
        let max = sob(50.0, 25.0).expect("Max");
        assert_eq!((max.texto.as_str(), max.cartao.as_str()), ("Max", "Health"));
        assert_eq!(max.script, None);
        assert_eq!(sob(50.0, 45.0).expect("Source").script, None);
        assert_eq!(
            sob(50.0, 65.0).expect("speed").script.as_deref(),
            Some("heroi.luau")
        );
        assert_eq!(sob(200.0, 25.0), None, "fora do nome");
    }

    #[test]
    fn o_clip_julga_o_centro_da_area_e_nao_o_canto() {
        let _g = Gravacao::nova(Rect::new(0.0, 0.0, 100.0, 100.0));
        // Centro (10, 17) dentro, canto (-20, 10) fora.
        linha(Rect::new(-20.0, 10.0, 60.0, 14.0), "CentroDentro");
        // Canto (90, 40) dentro, centro (140, 47) fora.
        linha(Rect::new(90.0, 40.0, 100.0, 14.0), "CantoDentro");
        assert_eq!(sob(10.0, 17.0).expect("centro dentro").texto, "CentroDentro");
        assert_eq!(sob(95.0, 45.0), None, "só o canto está no clip: não grava");
    }

    #[test]
    fn um_cartao_novo_esquece_o_script_do_anterior() {
        let _g = Gravacao::nova(CLIP_DO_PAINEL);
        cartao("Script");
        script_do_cartao("heroi.luau");
        cartao("Health");
        linha(r(20.0), "Max");
        e_propriedade_do_script();
        assert_eq!(sob(50.0, 25.0).expect("Max").script, None);
    }

    #[test]
    fn sob_areas_sobrepostas_vence_a_ultima_pintada() {
        let _g = Gravacao::nova(CLIP_DO_PAINEL);
        linha(r(20.0), "Por baixo");
        linha(r(20.0), "Por cima");
        assert_eq!(sob(50.0, 25.0).expect("sobreposta").texto, "Por cima");
    }

    #[test]
    fn fora_da_gravacao_ou_do_clip_nada_entra_e_um_quadro_novo_esquece() {
        linha(r(20.0), "Max");
        {
            let _g = Gravacao::nova(CLIP_DO_PAINEL);
            linha(r(20.0), "Max");
            linha(r(500.0), "Escondida");
        }
        linha(r(80.0), "Depois");
        assert!(sob(50.0, 25.0).is_some());
        assert_eq!(sob(50.0, 505.0), None, "fora do clip (rolada para fora)");
        assert_eq!(sob(50.0, 85.0), None, "pintada fora da gravação");
        let _g = Gravacao::nova(CLIP_DO_PAINEL);
        assert_eq!(sob(50.0, 25.0), None, "o quadro novo esquece");
    }
}
