//! Os gates de [`super`]: um desfazer global de brinquedo, com a MESMA ordem da shell
//! (`shells/desktop/src/undo_app.rs`: `post_frame_undo` e `apply_undo`) — o baseline, a pilha por estados
//! inteiros, o passo que só se regista com entrada (ou quando uma espera acaba) — e o palco a dizer quem
//! espera e quando a imagem aterra.

use super::{EdicaoQueEspera, EstadoComMacros};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
struct Estado {
    macros: BTreeMap<u64, Vec<u8>>,
    /// Um objecto que o artista move (uma acção global qualquer).
    posicao: i32,
    /// Os píxeis da imagem que a reescrita muda.
    imagem: u8,
}

impl EstadoComMacros for Estado {
    fn macro_de(&self, id: u64) -> Option<Vec<u8>> {
        self.macros.get(&id).cloned()
    }

    fn com_macro(&mut self, id: u64, m: Option<Vec<u8>>) {
        match m {
            Some(m) => self.macros.insert(id, m),
            None => self.macros.remove(&id),
        };
    }
}

const ID: u64 = 7;

struct Shell {
    mundo: Estado,
    base: Option<Estado>,
    undo: Vec<Estado>,
    redo: Vec<Estado>,
    espera: EdicaoQueEspera,
    /// O que o palco diz que espera, e se a reescrita escreveu neste quadro.
    esperam: Vec<u64>,
    aterrou: bool,
}

impl Shell {
    fn novo() -> Self {
        let mut s = Self {
            mundo: Estado {
                macros: BTreeMap::from([(ID, b"velho".to_vec())]),
                ..Estado::default()
            },
            base: None,
            undo: Vec::new(),
            redo: Vec::new(),
            espera: EdicaoQueEspera::default(),
            esperam: Vec::new(),
            aterrou: false,
        };
        s.quadro(false);
        s
    }

    fn regista(&mut self, novo_base: Estado) {
        if let Some(b) = self.base.replace(novo_base) {
            self.undo.push(b);
            self.redo.clear();
            self.espera.registou();
        }
    }

    /// O `post_frame_undo`.
    fn quadro(&mut self, entrada: bool) {
        let acabou = self.espera.quadro(&self.esperam, self.base.as_ref());
        let teve_entrada = entrada || std::mem::take(&mut self.aterrou) || acabou;
        let Some(base) = self.base.clone() else {
            self.base = Some(self.mundo.clone());
            return;
        };
        if !teve_entrada {
            return;
        }
        let atual = self.espera.a_comparar(self.mundo.clone());
        if atual != base {
            self.regista(atual);
        }
    }

    /// O `apply_undo`.
    fn ctrl(&mut self, redo: bool) {
        if self.espera.antes_de_desfazer(redo) {
            self.regista(self.mundo.clone());
        }
        let atual = self.mundo.clone();
        let saiu = if redo {
            self.redo.pop().inspect(|_| self.undo.push(atual.clone()))
        } else {
            self.undo.pop().inspect(|_| self.redo.push(atual.clone()))
        };
        if let Some(p) = saiu {
            let (aplica, base) = self.espera.restaurado(&p, &atual);
            self.mundo = aplica;
            self.base = Some(base);
        }
    }

    /// O artista edita uma etapa: o palco passa a esperar a imagem nova.
    fn edita(&mut self) {
        self.mundo.macros.insert(ID, b"novo".to_vec());
        self.esperam = vec![ID];
        self.quadro(true);
    }

    fn move_o_objecto(&mut self) {
        self.mundo.posicao += 1;
        self.quadro(true);
    }

    /// A imagem nova aterra (a reescrita escreve).
    fn aterra(&mut self) {
        if self.mundo.macros.get(&ID).map(Vec::as_slice) == Some(b"novo") {
            self.mundo.imagem = 1;
            self.aterrou = true;
        }
        self.esperam.clear();
        self.quadro(false);
    }

    fn macro_(&self) -> &[u8] {
        &self.mundo.macros[&ID]
    }
}

/// ⭐⭐ **O que o artista faz enquanto a edição espera é um passo DELE**: editar, mover um objecto, a imagem
/// aterra — o Ctrl+Z desfaz a edição com a imagem (o que se viu por último), e o seguinte o movimento.
/// ⛔ VERMELHO antes da W14: a espera suprimia o desfazer global e o movimento fundia-se com a edição (um
/// Ctrl+Z desfazia os dois).
#[test]
fn o_que_o_artista_faz_durante_a_espera_e_um_passo_dele() {
    let mut s = Shell::novo();
    s.edita();
    s.move_o_objecto();
    s.aterra();
    assert_eq!(s.undo.len(), 2, "o movimento e a edição: dois passos");
    s.ctrl(false);
    assert_eq!(
        (s.macro_(), s.mundo.imagem, s.mundo.posicao),
        (&b"velho"[..], 0, 1),
        "o 1.º Ctrl+Z desfaz a edição e a imagem, e o movimento fica"
    );
    s.ctrl(false);
    assert_eq!(s.mundo.posicao, 0, "o 2.º desfaz o movimento");
    s.ctrl(true);
    s.ctrl(true);
    assert_eq!(
        (s.macro_(), s.mundo.imagem, s.mundo.posicao),
        (&b"novo"[..], 1, 1),
        "o refazer volta tudo"
    );
}

/// ⭐ **Um Ctrl+Z durante a espera desfaz o que veio por cima, e a edição fica a esperar**; quando a imagem
/// aterra, a edição e ela são um passo.
#[test]
fn um_ctrl_z_durante_a_espera_desfaz_o_que_veio_por_cima() {
    let mut s = Shell::novo();
    s.edita();
    s.move_o_objecto();
    s.ctrl(false);
    assert_eq!(
        (s.macro_(), s.mundo.posicao),
        (&b"novo"[..], 0),
        "o movimento sai, a edição fica"
    );
    s.aterra();
    assert_eq!(s.mundo.imagem, 1, "a imagem aterrou");
    s.ctrl(false);
    assert_eq!(
        (s.macro_(), s.mundo.imagem),
        (&b"velho"[..], 0),
        "a edição e a imagem saem juntas"
    );
    assert!(s.undo.is_empty(), "nada mais na pilha");
}

/// ⭐ **Sem nada por cima, o Ctrl+Z durante a espera desfaz a própria edição** (e o Ctrl+Y volta-a).
#[test]
fn sem_nada_por_cima_o_ctrl_z_durante_a_espera_desfaz_a_edicao() {
    let mut s = Shell::novo();
    s.edita();
    s.ctrl(false);
    assert_eq!(s.macro_(), b"velho", "a edição saiu");
    s.ctrl(true);
    assert_eq!(s.macro_(), b"novo", "o refazer volta-a");
}

/// **A espera que acaba sem a imagem escrever** (a conferência decidiu, ou a reescrita não achou nada)
/// regista a edição já, sem esperar por uma entrada — que a fundiria com a acção seguinte.
#[test]
fn a_espera_que_acaba_sem_escrever_regista_a_edicao_ja() {
    let mut s = Shell::novo();
    s.mundo.macros.insert(ID, b"outro".to_vec());
    s.esperam = vec![ID];
    s.quadro(true);
    s.aterra();
    assert_eq!(s.undo.len(), 1, "a edição registou-se quando a espera acabou");
    s.move_o_objecto();
    s.ctrl(false);
    assert_eq!(
        (s.macro_(), s.mundo.posicao),
        (&b"outro"[..], 0),
        "o Ctrl+Z desfaz só o movimento"
    );
}
