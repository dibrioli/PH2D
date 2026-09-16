//! **O QUE UMA ROW É** — o tipo, o lugar dela na seção e o grupo.
//!
//! Irmão do [`super::rows`], e o corte é de responsabilidade: aqui *o que uma row
//! É* (estável desde que nasceu), lá *quais rows existem* — a tabela, que cresce
//! uma entrada por wave e foi quem levou o arquivo ao teto de LOC do painel.
//!
//! Os três tipos são re-exportados pelo `rows`, então nenhum caminho de chamador
//! muda.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{fracao_para_pista, pista_para_fracao};

use crate::state::{Sculpt3dUi, UiLevel};

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
    /// **A PARTIR DE QUE PROFUNDIDADE esta row é oferecida** (§2 do plano).
    ///
    /// ⚠️ **Um campo e não um `&&` dentro do [`Row::show`]**, e a diferença é o
    /// que torna a regra auditável: enfiado na closure, *qual é o nível desta
    /// row* deixaria de ser uma pergunta que alguém pode fazer à tabela — e é
    /// exatamente a pergunta que o gate de costura faz para varrer o conjunto de
    /// Pro, e que o `the_basic_level_never_hides_the_two_knobs_every_brush_has`
    /// faz para provar que esconder uma não deixa o artista sem a ferramenta.
    /// ⚠️ (Esta linha nomeava um gate que nunca existiu — a propriedade já tinha
    /// gate, com outro nome, no `tests/it/seam.rs`; medido 2026-09-13.)
    pub level: UiLevel,
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
    /// **Logo abaixo do interruptor da topologia dinâmica**, que é o que a
    /// arma — e não no bloco de knobs no fim da secção, onde vivem os
    /// argumentos do botão de retopologia.
    ///
    /// ⚠️ **A mesma lei do [`Self::AfterAlpha`] e do [`Self::AfterExtract`]:**
    /// uma pista mora ao lado do controlo que a lê. Aqui ela é o ALVO de
    /// densidade do passe, logo o sítio dela é debaixo da caixa que liga o
    /// passe — no fim da secção ela seria lida como mais um argumento do
    /// *Quad Retopology*, que é outro assunto.
    AfterDyntopo,
}

impl Row {
    /// **ESTA ROW EXISTE AGORA?** — a porta única das duas perguntas.
    ///
    /// ⚠️ *Este pincel a lê?* e *este nível a oferece?* são independentes, e
    /// juntá-las aqui é o que impede um sítio de pintura de perguntar uma e
    /// esquecer a outra. O `paint`, a cauda do pincel e a varredura de costura
    /// chamam ESTA função — três cópias do `&&` divergiriam no dia em que
    /// nascesse a terceira pergunta.
    pub fn visible(&self, ui: &Sculpt3dUi) -> bool {
        ui.ui_level.shows(self.level) && (self.show)(ui)
    }

    /// ⭐ **A CURVA da pista — DERIVADA da faixa, nunca escolhida por linha.**
    ///
    /// Uma faixa com piso positivo que atravessa [`ORDENS_PARA_CURVAR`] ordens de grandeza é
    /// [`PISTA_CURVA`] (cúbica); toda outra é linear, ao bit de sempre. Hoje só o RAIO a atravessa
    /// (`1..5000` px — a maior razão a seguir é `200`, a massa do tecido), e a razão é o report
    /// de 2026-09-16: com o tecto na diagonal da vista, uma pista linear punha o pincel de 50 px a
    /// 1 % da trilha. ⚠️ Derivada e não um campo: um campo por linha seria 58 literais a dizer
    /// «linear», e uma lista de ids ao lado seria a enumeração que apodrece.
    pub fn curva(&self) -> f32 {
        if self.min > 0.0 && self.max >= ORDENS_PARA_CURVAR * self.min {
            PISTA_CURVA
        } else {
            1.0
        }
    }

    /// Pista (`0..=1`) → o valor que ela significa.
    pub fn value_of(&self, track: f32) -> f32 {
        self.min + pista_para_fracao(track, self.curva()) * (self.max - self.min)
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
        let fracao = ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        fracao_para_pista(fracao, self.curva())
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

/// Quantas vezes o topo de uma faixa tem de valer o piso para a pista deixar de ser linear — três
/// ordens de grandeza, o ponto em que um pixel de arrasto já não pode valer a mesma quantidade no
/// fundo e no topo. Ver [`Row::curva`].
pub const ORDENS_PARA_CURVAR: f32 = 1000.0; // LITERAL-PX-OK: razao adimensional de uma faixa, nao metrica de design

/// O expoente da pista curva: com a faixa do raio (`1..5000` px) o pincel de 50 px fica a ~21 %
/// da trilha, o tecto antigo (125 px) a ~29 % e uma peça inteira na vista de omissão a ~37 %.
pub const PISTA_CURVA: f32 = 3.0; // LITERAL-PX-OK: expoente de uma curva de pista, nao metrica de design

/// Um grupo de rows com título. A lista de seções **É** a ordem de pintura.
pub struct Section {
    /// Id do cabeçalho dobrável.
    pub id: NodeId,
    /// Chave i18n do título.
    pub title: &'static str,
    /// As rows dele.
    pub rows: &'static [Row],
}

#[cfg(test)]
mod tests {
    use crate::rows::rows;

    /// ⭐ **GATE — só o RAIO tem pista curva, e TODA pista vai e volta.**
    ///
    /// ⚠️ A curva é derivada da faixa (`Row::curva`), e esta metade é o censo dela: uma linha nova
    /// cuja faixa cruze três ordens de grandeza passa a curva sem ninguém a escolher, e tem de
    /// chegar aqui com o nome. A outra metade: uma ida-e-volta que não fecha é um controlo que anda
    /// sozinho enquanto se segura.
    #[test]
    fn so_o_raio_tem_pista_curva_e_toda_pista_vai_e_volta() {
        let curvas: Vec<&str> = rows()
            .filter(|r| r.curva() > 1.0)
            .map(|r| r.label)
            .collect();
        assert_eq!(
            curvas,
            ["panel.sculpt3d.radius"],
            "as pistas curvas mudaram"
        );
        for r in rows() {
            for k in 0..=20u8 {
                let t = f32::from(k) / 20.0;
                let volta = r.track_of(r.value_of(t));
                assert!((volta - t).abs() < 1e-4, "{}: {t} -> {volta}", r.label);
            }
        }
        let raio = crate::rows::row_for(crate::ids::SCULPT3D_RADIUS).expect("a pista do raio");
        let omissao = raio.track_of(crate::state::Sculpt3dUi::default().radius_px);
        assert!(
            (0.15..0.3).contains(&omissao),
            "o pincel de omissao caiu a {omissao} da pista — a curva deixou de o servir"
        );
    }

    /// ⭐ **GATE — a firmeza do CENTRO só existe com a da NORMAL ligada** (espec §6.3), e só no
    /// pincel de plano; a da normal existe sempre nele.
    #[test]
    fn a_firmeza_do_centro_e_gateada_pela_da_normal() {
        let linha = |id| crate::rows::row_for(id).expect("a linha existe");
        let centro = linha(crate::ids::SCULPT3D_PLANO_FIRMEZA_CENTRO);
        let normal = linha(crate::ids::SCULPT3D_PLANO_FIRMEZA_NORMAL);
        let mut ui = crate::state::Sculpt3dUi {
            ui_level: crate::state::UiLevel::Pro,
            ..Default::default()
        };
        ui.brush.verb = ph2d_sculpt3d::Verb::Plane;
        ui.brush.plano_firmeza_normal = 1.0;
        assert!(centro.visible(&ui) && normal.visible(&ui));
        ui.brush.plano_firmeza_normal = 0.0;
        assert!(!centro.visible(&ui), "o centro apareceu sem a normal");
        assert!(normal.visible(&ui), "a normal sumiu");
        ui.brush.plano_firmeza_normal = 1.0;
        ui.brush.verb = ph2d_sculpt3d::Verb::Draw;
        assert!(
            !centro.visible(&ui) && !normal.visible(&ui),
            "as firmezas chegaram a outro pincel"
        );
    }

    /// ⭐ **GATE — o REGISTO leva a curva de cada pista.** Sem ela o número mostraria o valor
    /// linear enquanto a pista arrasta, e o painel reescrevê-lo-ia no quadro seguinte.
    #[test]
    fn o_registo_leva_a_curva_de_cada_pista() {
        let mut store = ph2d_editor_core::interaction::WidgetStore::with_capacity(512);
        crate::populate::populate(&mut store);
        for r in rows() {
            assert!(
                (store.linked_slider_curve(r.chip) - r.curva()).abs() < f32::EPSILON,
                "{}: o registo perdeu a curva",
                r.label
            );
        }
    }
}
