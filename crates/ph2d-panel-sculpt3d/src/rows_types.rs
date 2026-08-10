//! **O QUE UMA ROW É** — o tipo, o lugar dela na seção e o grupo.
//!
//! Irmão do [`super::rows`], e o corte é de responsabilidade: aqui *o que uma row
//! É* (estável desde que nasceu), lá *quais rows existem* — a tabela, que cresce
//! uma entrada por wave e foi quem levou o arquivo ao teto de LOC do painel.
//!
//! Os três tipos são re-exportados pelo `rows`, então nenhum caminho de chamador
//! muda.

use ph2d_a11y::NodeId;

use crate::state::Sculpt3dUi;

/// Uma row de slider+chip: que número ela edita, sobre que faixa, e **quando ela
/// existe**.
pub struct Row {
    /// Chave i18n do rótulo.
    pub label: &'static str,
    /// Id da pista.
    pub slider: NodeId,
    /// Id do chip numérico ligado a ela.
    pub chip: NodeId,
    /// Mínimo do domínio (o valor em `track = 0`).
    pub min: f32,
    /// Máximo do domínio (o valor em `track = 1`).
    pub max: f32,
    /// Passo do arrasto do chip. ⚠️ Não é decoração: sem faixa+passo registrados
    /// o chip deriva o passo do texto do buffer e percorre ~50 unidades por
    /// PIXEL, o que o transforma num interruptor min↔max (o bug que o painel do
    /// Flip documentou — digitar sempre funcionou, só arrastar estava quebrado).
    pub step: f64,
    /// Quantas casas o readout mostra.
    pub decimals: usize,
    /// Lê o valor desta row do estado autorado.
    pub get: fn(&Sculpt3dUi) -> f32,
    /// Escreve o valor desta row no estado autorado.
    pub set: fn(&mut Sculpt3dUi, f32),
    /// **Esta row existe com este pincel em mãos?**
    ///
    /// ⚠️ O `Plane Offset` só é lido pelos quatro verbos de plano e o `Pinch` só
    /// pelo Crease — pintá-los sempre seriam dois knobs que não fazem nada em
    /// doze das dezesseis ferramentas, que é o controle morto que esta casa
    /// varre a cada wave. A pergunta é feita à porta do MOTOR
    /// (`Verb::uses_plane`), nunca a uma lista paralela de nomes.
    pub show: fn(&Sculpt3dUi) -> bool,
    /// **ONDE, na seção, esta row é desenhada.**
    ///
    /// ⚠️ Ela existe porque *posição na tela* é uma pergunta que a tabela não
    /// respondia, e a resposta errada custou um smoke: a pista de `Alpha Scale`
    /// nasceu no bloco de knobs, ou seja **acima** da fileira de chips que a
    /// governa e separada dela pelo Falloff — um controle órfão, que aparece do
    /// nada e não se liga a nada que o artista acabou de tocar.
    ///
    /// ⚠️ **A row continua na tabela**, e é isso que importa: `populate`, `event`
    /// e a varredura de costura seguem a percorrendo, então ela nasce registrada,
    /// viva e varrida como qualquer outra. O que este campo move é **onde ela é
    /// desenhada**, e só isso — a alternativa (tirá-la da tabela e pintá-la à
    /// mão) a tiraria das três listas de uma vez.
    pub place: Place,
}

/// Em que ponto da seção a row é pintada.
///
/// ⚠️ **Era um `bool`, e o terceiro valor o obrigou a virar isto.** Enquanto
/// havia só *no bloco* × *na cauda*, dois estados bastavam; os dois números do
/// extract são argumentos de um BOTÃO que mora no fim da seção, e pintá-los onde
/// as pistas do alpha moram os separaria do gesto que os lê. Um `bool` com um
/// `if` por id ao lado seria a enumeração que apodrece na quarta row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    /// O bloco de knobs contínuos, no topo da seção.
    Knobs,
    /// Logo abaixo do seletor de padrão, colada aos chips que a governa.
    AfterAlpha,
    /// No fim da seção, colada ao botão de extract que a lê.
    AfterExtract,
}

impl Row {
    /// Pista (`0..=1`) → o valor que ela significa.
    pub fn value_of(&self, track: f32) -> f32 {
        self.min + track * (self.max - self.min)
    }

    /// O valor → a pista dele. A inversa de [`Row::value_of`], e as duas têm de
    /// continuar inversas: o painel publica uma pista a partir do estado a cada
    /// frame e lê um valor de volta a cada arrasto, então um descasamento é um
    /// controle que **anda sozinho enquanto você o segura**
    /// ([[feedback_derived_coordinate_seed_must_match_sample]]).
    pub fn track_of(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// O `link_slider_number_mapped` exprime o mesmo mapa como
    /// `display = track * scale + offset`.
    pub fn scale(&self) -> f32 {
        self.max - self.min
    }

    /// Ver [`Row::scale`].
    pub fn offset(&self) -> f32 {
        self.min
    }
}

/// Um grupo de rows com título. A lista de seções **É** a ordem de pintura.
pub struct Section {
    /// Id do cabeçalho dobrável.
    pub id: NodeId,
    /// Chave i18n do título.
    pub title: &'static str,
    /// As rows dele.
    pub rows: &'static [Row],
}
