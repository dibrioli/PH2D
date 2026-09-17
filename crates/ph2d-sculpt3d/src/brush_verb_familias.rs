//! ⭐⭐ **AS TRÊS FAMÍLIAS DE LEITURA que a UI pergunta** — irmão (`#[path]`) do
//! [`super::brush_verb_predicados`], e o corte é o SUJEITO: lá mora *o que um
//! verbo É*, e aqui *a que família de LEITURA ele pertence* — que superfície ele
//! consulta antes de decidir para onde puxar.
//!
//! São exactamente as três que o gate
//! `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera: a
//! **máscara**, o **plano** e o **anel**.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro-mãe chegou a `725` de
//! `700` ao registar porque o pincel de plano entrou na família do plano) e é
//! melhor por isso: as três respostas que um mesmo gate compara passam a viver
//! num sítio onde se leem juntas, em vez de espalhadas por uma lista de vinte
//! predicados. ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2
//! proíbe por escrito.*

use super::verb::Verb;

impl Verb {
    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    ///
    /// ⛔⛔⛔ **O [`Self::Plane`] entrou aqui em 2026-09-17, e a ausência dele era
    /// um CONTROLO INALCANÇÁVEL.** Ele lê o `plane_offset` desde que existe — o
    /// [`crate::plano_da_pegada`] faz `lift = dab.radius × plane_offset`, a espec
    /// §2.4 mede-o (`+0,2` num raio `0,4` move o plano `+0,08000`, razão `0,2000`
    /// ao dígito impresso) e **duas fixturas do corpus o exercitam** — e o painel
    /// **não lhe oferecia a fileira**, porque o único consumidor deste predicado
    /// é o `show` dela.
    ///
    /// ⚠️ **Quem o apanhou foi o G-13**, ao IMPRIMIR a população que mede: a
    /// lista saiu `Flatten · Fill · Scrape · Clay` no deslocamento e `Plane` só
    /// nos dois tectos. *É a coluna «o painel esconde × o knob CHEGA» da tabela
    /// do próprio censo — o **inalcançável**, cuja cura é OPOSTA à do morto: um
    /// liga-se, o outro apaga-se.* ⛔ Um censo que só procura knobs mortos nunca
    /// o encontraria: ele não é pintado, logo não entra na varredura.
    ///
    /// ⚠️⚠️ **E o plano que o [`Self::Plane`] ajusta NÃO é o dos outros quatro** —
    /// eles usam o [`crate::stroke::plane`] (portado, pesado pela máscara sobre a
    /// pegada inteira) e ele usa o [`crate::plano_da_pegada`] (curva suave, dois
    /// raios próprios), que diferem `17,1 %` do raio no centro. ⇒ *este predicado
    /// responde «este verbo LÊ o `plane_offset`», e não «qual lei de plano ele
    /// corre»* — quem o usar para ROTEAR entrega o plano errado, que é o defeito
    /// que a porta de bancada [`crate::SculptStroke::plano_do_ultimo_dab_para_teste`]
    /// já pagou.
    #[must_use]
    pub fn uses_plane(self) -> bool {
        matches!(
            self,
            Self::Flatten | Self::Fill | Self::Scrape | Self::Clay | Self::Plane
        )
    }

    /// Este verbo lê o anel de vizinhos? (Quem responde `true` custa a
    /// travessia do CSR por vértice, e é o que decide se o `vert_verts` pode um
    /// dia virar preguiçoso.)
    ///
    /// ⚠️ **O [`Self::SurfaceSmooth`] o percorre DUAS vezes** — uma para a média
    /// das posições, outra para a média dos `b` —, e ele nasceu FORA desta
    /// lista: a pergunta que ela responde é *quem precisa da adjacência*, e uma
    /// resposta falsa aqui é como um `vert_verts` preguiçoso deixaria de
    /// construí-la exatamente para o verbo que mais a usa. O gate irmão
    /// `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera
    /// os nomes, e ficou VERDE sobre a omissão até alguém a procurar.
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(
            self,
            Self::Smooth | Self::Sharpen | Self::SurfaceSmooth | Self::SmearMultires
        )
    }
}
