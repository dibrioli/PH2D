//! ⭐ **COMO O GESTO É CONDUZIDO** — o [`Grip`] de cada verbo, e a leitura dele
//! que o pen-down faz.
//!
//! Irmão (`#[path]`) do [`super::brush_verb`], e o corte é o mesmo que separa o
//! [`super::brush_verb_predicados`] dele, com outro sujeito: lá mora *quais
//! verbos existem e como se chamam*, aqui *como a mão os conduz*.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro-mãe chegou a `719`
//! com o verbo da densidade) e é melhor por isso: a lei do gesto passa a ter um
//! ficheiro onde se lê inteira, em vez de ser o rabo de uma tabela de nomes.
//! ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2 proíbe por
//! escrito* — um tecto por-ficheiro é a única grandeza deste repo que soma entre
//! linhas sem ninguém a contar.

use super::verb::Verb;
use crate::grip::{Amount, Grip};

impl Verb {
    /// **Como este verbo consome o gesto** — ver [`Grip`]. A porta única de que
    /// [`Self::anchors`] é uma leitura, e sobre a qual o kernel e o shell fazem
    /// perguntas diferentes.
    #[must_use]
    pub fn grip(self) -> Grip {
        match self {
            Self::Cloth => Grip::Simulate,
            Self::Move => Grip::Hold,
            Self::SnakeHook => Grip::Hook,
            // ⭐⭐ **OS DOIS GESTOS TANGENCIAIS NÃO TRAZEM GRIP NOVO**, e é o
            // achado que os torna baratos: o que os separa um do outro é
            // exactamente o que já separa o agarrar do gancho — de que pose a
            // pegada é medida e se o puxão é o TOTAL ou o INCREMENTO. A conta
            // do deslocamento é a mesma nos dois.
            Self::Thumb => Grip::Hold,
            Self::Nudge => Grip::Hook,
            // ⭐ **A POSE também não traz grip novo.** Ela precisa exactamente
            // do que o `Hold` já promete — a pegada presa no pen-down e o
            // `pull` como deslocamento **TOTAL** desde então —, porque a lei
            // dela é função do arrasto acumulado e não de um incremento
            // (`T = C + G·s`). ⚠️ E o `Hold` traz de graça a outra metade de
            // que ela precisa: *não percorre o caminho*, e um traço de pose
            // resolve-se **uma vez por evento**, nunca por passo de espaçamento.
            Self::Pose => Grip::Hold,
            // ⭐ **O contorno também não traz grip novo.** Ele precisa do que o
            // `Hold` já promete — a pegada presa no pen-down e o `pull` como
            // deslocamento **TOTAL** —, porque a lei dele é função do arrasto
            // acumulado (§14.2: o resultado não depende do caminho nem do número
            // de eventos). ⚠️ E o `Hold` traz de graça a outra metade: *não
            // percorre o caminho*, e as fases A–E do contorno são **fotografadas
            // no pen-down**.
            Self::Boundary => Grip::Hold,
            // ⭐ **A densidade é CARIMBO, e é o grip certo apesar de ela não
            // carimbar nada:** o que ela precisa é do caminho do carimbo, que é
            // o único que corre o passe de topologia por dab. O que ela NÃO
            // precisa é da lei por-vértice, e é o [`Self::sem_lei_por_vertice`]
            // que a tira de lá — não um grip novo. *Um grip novo para um verbo
            // sem lei seria um grip sem lei.*
            Self::Density => Grip::Stamp,
            Self::Twist => Grip::Turn(Amount::Angle),
            Self::LocalScale => Grip::Turn(Amount::Fraction),
            // O CARIMBO: a faixa compõe sobre a lista de dabs como o Draw.
            Self::ClayStrips => Grip::Stamp,
            Self::Mask => Grip::Paint,
            _ => Grip::Stamp,
        }
    }

    /// **Este verbo PEGA uma âncora no pen-down** em vez de carimbar? — uma
    /// leitura de [`Self::grip`] em vez de um segundo predicado.
    ///
    /// Os três grips que não são [`Grip::Stamp`] têm em comum que o primeiro
    /// toque **escolhe um ponto e não move nada**: o barro só anda quando o dedo
    /// anda, porque no instante do pen-down o gesto ainda vale zero (o puxão, o
    /// incremento, o ângulo varrido, a fração de escala).
    ///
    /// ⚠️ **O nome era `pulls()`, e ele passou a MENTIR quando o
    /// [`Grip::Turn`] chegou** — um redemoinho não puxa nada. O que a pergunta
    /// sempre quis dizer é *este verbo tem âncora?*, e é essa a palavra que
    /// sobrevive a um quinto grip.
    ///
    /// ⚠️ **Ela era `!matches!(grip, Stamp)`, e o quinto grip a tornou FALSA:**
    /// o [`Grip::Paint`] também não carimba geometria, e um verbo de máscara
    /// não tem âncora nenhuma. A pergunta passou a ser feita pelo lado
    /// POSITIVO — quem de fato pega um ponto no pen-down —, que é a forma que
    /// sobrevive ao sexto grip em vez de o adotar em silêncio.
    /// ⛔⛔ **E O SEXTO GRIP CHEGOU SEM RESPONDER A ESTA PERGUNTA — report do
    /// dono, 2026-09-05: *«não funciona, nada aconteceu ao pintar»*.** O
    /// [`Grip::Simulate`] entrou sem entrar nesta lista, então o pen-down do
    /// tecido caía no `else` (`sculpt_at`) e a âncora do dedo nunca era tomada;
    /// no arrasto, o `hook_step` sai no primeiro `if` porque `self.grab` é
    /// `None`, e **nada acontece**. O parágrafo acima previu exatamente esta
    /// classe e ainda assim ela passou: *uma lista pelo lado positivo obriga o
    /// grip novo a declarar-se, e nada obriga QUEM O ESCREVE a ler a lista.*
    ///
    /// ⇒ o que fecha a classe é o gate
    /// `the_sculpt_pen_down_arms_what_the_drag_needs` (shell), que confere as
    /// DUAS metades: todo grip que o arrasto conduz a partir da âncora tem de
    /// aparecer aqui.
    #[must_use]
    pub fn anchors(self) -> bool {
        matches!(
            self.grip(),
            Grip::Hold | Grip::Hook | Grip::Turn(_) | Grip::Simulate
        )
    }
}
