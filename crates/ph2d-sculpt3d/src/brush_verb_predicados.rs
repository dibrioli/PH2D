//! ⭐ **O QUE UM VERBO É** — os predicados que o resto do módulo consulta em vez
//! de manter listas paralelas.
//!
//! ⚠️ **Cada um destes é uma PORTA, e é isso que os justifica:** a alternativa é
//! uma lista de verbos escrita à mão em cada sítio que precisa da resposta, e
//! essa lista envelhece calada no dia em que um verbo novo nasce. *Uma lei
//! escrita em dois sítios ainda não é uma lei — só uma porta é.*
//!
//! Irmão do [`super::brush_verb`], e o corte é *o que o verbo É* (aqui) contra
//! *quais verbos existem e como se chamam* (lá).

use super::verb::Verb;
use crate::grip::Grip;

impl Verb {
    /// **Este verbo pode ACUMULAR?** — a porta única do `accumulate`.
    ///
    /// Só a família do CARIMBO. Os outros três grips carregam o gesto TOTAL
    /// desde o pen-down (o puxão, o ângulo varrido, a fração de escala) e
    /// carimbam `accum = 1` ou congelam a pegada: somar um total N vezes seria
    /// multiplicar o gesto pelo número de eventos de ponteiro, que é exatamente
    /// a dependência de taxa de amostragem que a lei do traço existe para não
    /// ter.
    ///
    /// ⚠️ Porta e não um `matches!` no sítio de uso: o painel pergunta para
    /// OFERECER o interruptor e o aplicador pergunta para HONRAR o clique, e
    /// duas cópias divergiriam num controle que aparece e não faz nada.
    ///
    /// ⚠️ **A DEMÃO fica de fora, e é a referência que a tira:** o `layer.cc`
    /// mede as distâncias contra `orig_data.positions` **incondicionalmente** —
    /// ele não consulta o `BRUSH_ACCUMULATE`, ao contrário dos irmãos de
    /// carimbo. E há razão para isso: o que o Accumulate compra num Draw é
    /// *deixar o pincel não se esgotar*, e a demão já tem o próprio motor de
    /// saturação no [`crate::GripLaw::coat`]. Oferecer o interruptor aqui seria
    /// um segundo controle sobre a mesma pergunta.
    #[must_use]
    pub fn accumulates(self) -> bool {
        matches!(self.grip(), Grip::Stamp) && self != Self::Layer
    }

    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// O sinal (o `Ctrl` de todo app de escultura) muda o RESULTADO deste verbo?
    ///
    /// ⚠️ **Era uma blacklist, e ela MENTIA.** Ao excluir só `Smooth`/`Sharpen` e
    /// `Pinch`/`Magnify`, ela afirmava sinal para `Flatten`, `Fill` e `Scrape` —
    /// e o `invert` **nunca chega neles**: o alvo dos três é `project(base,
    /// plane)` (`stroke.rs:410-424`), que não lê o `reach`, o único canal por
    /// onde o sinal viaja até um verbo de posição. Três controles mortos, com uma
    /// função afirmando que estavam vivos.
    ///
    /// A lista verdadeira é a de quem CONSOME o sinal: `Draw`, `Inflate` e
    /// `Clay` somam `reach` (`stroke.rs:397,398,427`), `Crease` soma `-reach`
    /// (`:435`), e `Mask` troca o alvo do canal dele de 1 para 0
    /// (`apply_mask`, `:481`).
    ///
    /// ⚠️ **Whitelist e não blacklist, e a direção é o conserto.** Numa
    /// blacklist um verbo NOVO nasce reivindicando um sinal que talvez não tenha,
    /// em silêncio — que é exatamente como este defeito nasceu. Numa whitelist
    /// ele nasce sem sinal, e quem o tem escreve o nome aqui.
    ///
    /// ⚠️ **Isto NÃO é um `uses_reach()`.** O `Mask` não lê `reach` — o alvo de
    /// posição dele é o próprio lugar — e mesmo assim tem oposto. A pergunta é
    /// sobre *o resultado que o artista vê*, e `reach` e `apply_mask::goal` são
    /// duas implementações dela.
    ///
    /// **As três alternativas, e por que cada uma morre:** *"faça o invert
    /// funcionar no Flatten"* — não há o que negar, o Flatten projeta nos dois
    /// sentidos e o oposto dele é ele mesmo; *"Ctrl troca Fill↔Scrape"* — é o
    /// `_negative` do `Flatten.js`, mas ele tem UM tool com um toggle e nós temos
    /// DOIS verbos com dois chips, então o rail destacaria "Fill" enquanto a
    /// ferramenta raspa; *"Ctrl nega o `plane_offset`"* — o slider já tem sinal
    /// nos dois sentidos, com gate provando
    /// (`the_plane_offset_lifts_the_plane_the_verbs_project_onto`).
    ///
    /// ⚠️ **Nenhuma UI pergunta isto hoje** (o shell arma `invert = ctrl`
    /// incondicionalmente, `sculpt3d.rs`): o consumidor é o [`Brush::reach`], e o
    /// chip que decide oferecer ou não o controle é da wave que trouxer painel.
    ///
    /// ⚠️ **Os dois [`Grip::Turn`] ficam de fora, e a razão é que o gesto já tem
    /// sinal:** varrer para o outro lado torce ao contrário, arrastar para a
    /// esquerda encolhe. Um `Ctrl` ali seria a segunda maneira de dizer a mesma
    /// coisa — e uma que **compõe** com a primeira, então varrer ao contrário
    /// com `Ctrl` apertado voltaria a torcer no sentido original.
    #[must_use]
    pub fn honours_invert(self) -> bool {
        matches!(
            self,
            Self::Draw
                | Self::Inflate
                | Self::Clay
                | Self::Crease
                | Self::Blob
                | Self::Mask
                // ⚠️ **O Ctrl VIRA O V**, e o oposto de cavar um vinco é
                // enchê-lo: com o ângulo negativo as duas normais tombam ao
                // contrário, o telhado vira vale e o culling de lado se desliga
                // (`if (angle >= 0.0f)`). É o `if (flip) angle *= -1` do
                // `multiplane_scrape.cc:657`, e não uma força negativa.
                | Self::MultiplaneScrape
                // ⚠️ **A DEMÃO cava, e é o `brush.direction` da referência** —
                // no `layer.cc` o sinal viaja no `cache.bstrength`, que o
                // Blender já entrega negativo. Aqui ele viaja no alvo (o `sign`
                // do `compute_target`), porque o nosso `accum` é a MAGNITUDE da
                // demão e uma magnitude não tem lado.
                | Self::Layer
                // ⚠️ **O TECIDO honra o Ctrl desde 2026-09-06**, quando a lei da
                // referência passou a ser a de omissão: ela carrega um sinal
                // `±1` que multiplica a força do gesto (o *Add/Subtract* do
                // alvo), e o adaptador lê-o do `Brush::invert`. ⛔ A lei VBD
                // anterior não o lia, e é por isso que esta linha não existia.
                | Self::Cloth
        )
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    #[must_use]
    pub fn uses_plane(self) -> bool {
        matches!(self, Self::Flatten | Self::Fill | Self::Scrape | Self::Clay)
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
        matches!(self, Self::Smooth | Self::Sharpen | Self::SurfaceSmooth)
    }

    /// **ESTE VERBO PASSA PELO APLICADOR POR-VÉRTICE?**
    ///
    /// ⚠️ **A porta existe porque uma feature nova pode ESVAZIAR o censo de
    /// outra pessoa**, e o tecido é a primeira que o faz: os censos do
    /// `stroke_apply` varrem `Verb::ALL` e afirmam coisas sobre o `accum` e o
    /// `target` — dois planos que a simulação **nunca escreve**, porque ela
    /// desvia antes do laço. Excluí-lo por NOME faria cada verbo novo editar o
    /// teste de outra pessoa; excluí-lo por LEI é o que mantém a população dos
    /// censos derivada.
    ///
    /// ⛔ *Nada aqui diz que o tecido é uma excepção tolerada:* ele tem os
    /// próprios gates, no `stroke_cloth_tests`, e eles medem o que ele de facto
    /// promete.
    ///
    /// ⚠️ **E ela nasceu SEPARANDO um `#[must_use]` do item dele** — a primeira
    /// redação foi inserida entre o atributo do `uses_neighbours` e a assinatura
    /// dele, e o atributo mudou de dono em silêncio. É a armadilha que este repo
    /// já tinha registada; quem a apanhou foi o clippy, não uma leitura.
    #[must_use]
    pub fn writes_through_applicator(self) -> bool {
        !matches!(self.grip(), Grip::Simulate)
    }
}
