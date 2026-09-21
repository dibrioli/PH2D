//! ⭐⭐⭐ **O PEN-DOWN DE UM GESTO QUE MUDA A TOPOLOGIA** — a fotografia que o
//! `Ctrl+Z` vai devolver, e a triangulação que os dois motores exigem.
//!
//! Irmão (`#[path]`) do [`super`], cortado dele pelo tecto de LOC (`712` contra
//! `700`) e pelo ASSUNTO: lá mora a FILA do desfazer — o que se grava, o que se
//! troca, o que se conta —, e aqui o que o gesto precisa de ter feito **antes
//! do primeiro dab** para que haver o que gravar.

use super::Sculpt3dScene;

impl Sculpt3dScene {
    /// **A malha de ANTES do traço**, quando a topologia dinâmica está armada.
    ///
    /// ⚠️ Chamada no pen-down e só ali: um traço que refina não tem janela
    /// por-índice para desfazer, e a foto tem de ser tirada antes do primeiro
    /// dab. Ver o `dyn_before`.
    pub(crate) fn open_dyntopo_stroke(&mut self) {
        // ⭐⭐ **A foto é tirada para quem MEXE na topologia, e desde 14/09 isso
        // inclui quem corre SEM o interruptor** (ordem do dono sobre o pincel de
        // densidade). *Perguntar só pelo interruptor deixaria um traço que muda
        // a contagem sem nada para o `Ctrl+Z` devolver.*
        //
        // ⭐⭐⭐⭐ **E O GESTO TEM DE IR MESMO MEXER NA TOPOLOGIA** — desde
        // 2026-09-20 a terceira metade da porta é a que também vê o plano de
        // tinta fina ([`crate::tinta_da_peca::o_gesto_muda_a_topologia`]).
        // *Sem ela um traço de COR com o plano armado clonava a malha inteira
        // por pen-down para desfazer uma mudança de topologia que já não
        // acontece.*
        //
        // ⭐⭐⭐⭐ **E AS TRÊS METADES PASSARAM A SER UMA PORTA em 2026-09-21**
        // ([`crate::tinta_da_peca::o_passe_corre_no_pen_down`]): a VOZ do
        // pen-down perguntava por UMA delas (o interruptor) e ficava calada
        // exactamente no `Density`, que corre sem ele. *Duas respostas à mesma
        // pergunta divergem no dia em que uma ganha uma cerca — e esta tinha
        // ganho duas.*
        if !self.o_passe_de_topologia_corre_no_pen_down() {
            self.dyn_before = None;
            return;
        }
        self.dyn_before = Some(Box::new(self.mesh().clone()));
        // ⛔⛔ **E QUEM MEXE NA TOPOLOGIA HERDA O TRABALHO QUE O INTERRUPTOR
        // FAZIA.** Os dois motores recusam quads por GEOMETRIA
        // (`Refine::NotTriangles` — partir a aresta de um quad devolve um
        // triângulo e um pentágono), e quem os triangulava era o
        // `toggle_dyntopo`. Sem esta linha o gesto seria um **no-op silencioso**
        // em toda peça que ainda é de quads: uma primitiva acabada de nascer, ou
        // a saída do botão de retopologia.
        //
        // ⚠️⚠️ **A condição era `livre && !armed` e passou a ser a PORTA**, por
        // duas razões que apontam ao mesmo sítio: com a tinta fina armada o
        // `toggle_dyntopo` **já não triangula** (ver o doc dele), logo alguém
        // tem de o fazer; e com o interruptor ligado numa peça já triangulada a
        // chamada devolve `0` e custa uma varredura das faces, ao lado de uma
        // cópia da malha inteira que esta função já paga.
        //
        // ⚠️ **DEPOIS da foto, de propósito:** triangular muda a malha, e o
        // gesto inteiro — triangular *mais* adensar — tem de desfazer num passo
        // só. Antes da foto, o `Ctrl+Z` devolveria a malha já triangulada.
        {
            let added = self
                .obj_mut()
                .map_or(0, |o| o.stack.mesh_mut().triangulate());
            if added > 0 {
                self.mesh_rebuilt();
                eprintln!(
                    "[sculpt3d] {} triangulou {added} faces -- os dois motores de \
                     topologia recusam quads, e o Ctrl+Z devolve a malha de antes",
                    self.brush.verb.label()
                );
            }
        }
    }
}
