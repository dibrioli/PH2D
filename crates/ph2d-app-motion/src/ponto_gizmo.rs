//! ⭐⭐⭐ **O GIZMO DE UMA CORRENTE DE POSIÇÕES** — a segunda metade da ordem do dono
//! (2026-09-17, reaberta em 19/09 como report): *«nós como Grid, rope, etc, não passam de posições
//! do espaço, sem nenhuma capacidade de gerar pixels na tela»*, e ***«sem o duplicator só aparece
//! um gizmo de osso ou segmento de corda (ou outro tipo de segmento) que não renderiza em
//! runtime»***.
//!
//! A [`ph2d_eval_motion::tem_aparencia`] (a W1) cala o que não veio de uma forma. Este
//! ficheiro é o que aparece no lugar: um **manipulador de editor**, desenhado só com a ferramenta
//! Motion na mão e **nunca** no quadro que o produto entrega.
//!
//! ## A FEIÇÃO sai das COLUNAS, nunca de uma lista de nomes de nó
//!
//! | a corrente traz | é | porque |
//! |---|---|---|
//! | `parent` | um **OSSO** | cada elemento declara de quem PENDE — é a cadeia do `rig.*` |
//! | `rope_prev` | uma **CORDA** | o estado de Verlet; os pontos são consecutivos por construção |
//! | nada disso | **PONTOS** | uma nuvem sem ordem (grelha, dispersão, distribuições) |
//!
//! ⚠️ **Derivada, e é a diferença entre isto e uma tabela que envelhece:** um `rig.fabrik` novo, um
//! `motion.verlet_rope` com outro nome, um nó de terceiros — todos caem na feição certa sem
//! ninguém se lembrar de os inscrever. Uma lista de `type_name` ficaria muda no primeiro nó novo,
//! que é como o censo por prefixo desta casa já falhou (CLAUDE.md §5.0).
//!
//! ⚠️ **O `parent` ganha do `rope_prev`** quando os dois existem: pender de alguém é uma afirmação
//! mais forte do que ser consecutivo, e uma cadeia com ramos desenhada como corda ligaria pontos
//! que não se tocam.
//!
//! ## O que ele NÃO faz
//!
//! ⛔ Não tem alças e não edita nada. O gizmo do colisor e o do warp arrastam params; este só diz
//! *onde as posições estão*. Dar-lhe alças seria autorar a saída de um nó pela ponta, que é o que
//! o cartão faz.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

/// **Quantos elementos o gizmo desenha, no máximo, por corrente.**
///
/// ⚠️ **O RECURSO é o tempo de CODIFICAR os caminhos no quadro** (o mesmo do `MAX_CONTORNOS` do
/// gizmo do colisor), contra o orçamento de **`1,67 ms`** = 1/10 de um quadro de 60 Hz — a régua
/// que o indicador da pose já usa: *um gizmo é passageiro do quadro, não o assunto dele*.
///
/// **MEDIDO** (`mede_o_custo_do_gizmo_de_pontos`, `--release`, mediana de 9, `load 3,09`):
///
/// | elementos | Ponto | Corda | Osso | pior, em % do orçamento |
/// |---|---|---|---|---|
/// | 1 024 | 0,057 ms | 0,098 | 0,099 | **5,9 %** |
/// | **4 096** | 0,228 ms | 0,403 | 0,390 | **24,2 %** |
/// | 16 384 | 1,375 ms | 1,604 | 1,562 | **96,1 %** |
/// | 65 536 | 3,571 ms | 6,811 | 6,345 | 407,8 % |
///
/// ⇒ `4 096` é o maior degrau em que a **pior** feição fica abaixo de um quarto do orçamento; a
/// `16 384` ele está gasto. ⛔ **Não é um «razoável»**: a cena `=116` entrega **102 400** posições
/// num sink só, e desenhá-las custaria mais de quatro quadros inteiros de gizmo.
///
/// ⚠️ **O corte não apaga a contagem** — [`Grupo::total`] guarda quantas havia, para quem quiser
/// dizê-lo ao artista.
pub const MAX_PONTOS: usize = 4096;

/// Uma corrente de posições, pronta a desenhar.
#[derive(Clone, Debug, PartialEq)]
pub struct Grupo {
    /// O sink de que esta corrente saiu — o que o artista vê seleccionado no grafo.
    pub node: NodeId,
    /// As posições de MUNDO, já limitadas pelo [`MAX_PONTOS`].
    pub pontos: Vec<[f32; 2]>,
    /// ⭐⭐⭐ **A ROTAÇÃO de cada elemento, em graus** — `None` quando a corrente **não traz** a
    /// coluna (ordem do dono, 2026-09-19: *«no canvas simulam qualquer grafo normalmente»*).
    ///
    /// ⚠️ **`None` e `Some(vec![0; n])` NÃO são a mesma coisa, e a diferença é visível:** sem a
    /// coluna o gizmo não desenha a agulha da direcção — *uma agulha a apontar para a direita em
    /// toda a nuvem seria ruído sobre um grafo que nunca falou de direcção*. Com a coluna toda a
    /// zero, ela aponta para a direita **porque o grafo o disse**.
    pub rot: Option<Vec<f32>>,
    /// ⭐⭐⭐ **A ESCALA de cada elemento** — o MULTIPLICADOR do glifo, nunca uma medida de mundo.
    ///
    /// É isto que faz o `scale` de um `motion.oscillator` PULSAR no canvas sem uma forma ligada.
    /// `None` ⇒ [`ph2d_nodegraph::attr::SIZE_IDENTITY`], que é o glifo nu.
    ///
    /// ⚠️ **Um escalar e não um `[f32; 2]`:** o glifo é chrome e tem de continuar a ler-se como o
    /// mesmo símbolo — uma cruz esmagada num eixo lê-se como uma barra, e o artista deixaria de
    /// saber que aquilo é um ponto. A média dos dois eixos é o que a `SIZE_IDENTITY` torna `1`.
    pub escala: Option<Vec<f32>>,
    /// Quantas posições a corrente tinha ANTES do tecto — o que a legenda diria.
    pub total: usize,
}

impl Grupo {
    /// O multiplicador do glifo no elemento `i` — `1` quando a corrente não autorou escala.
    #[must_use]
    pub fn escala_em(&self, i: usize) -> f32 {
        self.escala
            .as_ref()
            .and_then(|v| v.get(i))
            .copied()
            .unwrap_or(1.0)
    }

    /// A rotação do elemento `i`, em **graus**, ou `None` se a corrente não traz direcção.
    #[must_use]
    pub fn rot_em(&self, i: usize) -> Option<f32> {
        self.rot.as_ref().and_then(|v| v.get(i)).copied()
    }
}

/// O retrato deste quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct PontoGizmoView {
    pub grupos: Vec<Grupo>,
}

/// ⭐⭐⭐ **A AMOSTRA QUE O TECTO DEIXA PASSAR — e ela não é um PREFIXO numa nuvem.**
///
/// > **Report do dono, 2026-09-19:** *«Gap y quebrou e movimenta tudo em vez de criar espaço»*,
/// > sobre a grelha da cena `=2` — `360 × 360 = 129 600` elementos.
///
/// ⛔⛔⛔ **A causa era o corte, e ele era `take(MAX_PONTOS)`.** Numa grelha *row-major* os
/// primeiros `4 096` de `129 600` são as primeiras **11,4 FILEIRAS de 360** — uma faixa na BORDA
/// de baixo, não uma amostra. Medido (`o_gap_y_na_grelha_do_dono`):
///
/// | `gap_y` | a nuvem inteira | o que o gizmo segurava |
/// |---|---|---|
/// | | extensão Y · centro Y | extensão Y · centro Y |
/// | `1,0` | `359,0` · **`0,000`** | `11,0` · `−174,0` |
/// | `2,0` | `718,0` · **`0,000`** | `22,0` · `−348,0` |
/// | `3,0` | `1077,0` · **`0,000`** | `33,0` · `−522,0` |
///
/// ⇒ o nó espaçava **certo** (o centro nunca se move) e o gizmo mostrava uma faixa que **voava**:
/// por cada `+5,5` de espaçamento ela deslocava-se `−87`, uma razão de **`15,8×`**. *Isso é, à
/// letra, «movimenta tudo em vez de criar espaço».*
///
/// ⚠️⚠️ **A sonda irmã [`super::motion_state::demo_router::census::o_gap_y_ainda_espaca`] mediu uma
/// grelha de `4 × 4` e saiu LIMPA — e não podia ser de outra maneira: `16` pontos cabem no tecto e
/// ele NUNCA ENGATA.** *Uma fixtura abaixo do tecto não testa o tecto.*
///
/// ⛔⛔ **E o passo NÃO é uniforme para toda feição, porque nem toda corrente é uma nuvem:**
///
/// | feição | corte | porquê |
/// |---|---|---|
/// | **Ponto** | passo uniforme | uma nuvem **não tem ordem**: uma amostra espalhada é representativa, e é ela que faz o `gap_y` ler-se como espaçamento |
/// | **Osso** · **Corda** | prefixo | ali a ORDEM **é** a topologia — saltar elementos ligaria juntas que não se tocam. O prefixo de uma cadeia é uma sub-cadeia **LIGADA**, que é a leitura honesta |
///
/// ⚠️⚠️ **TODA coluna lê pelos MESMOS índices**, e é isso que este tipo existe para garantir: a
/// `P`, a `size`, a `rot`, a forma e o tamanho do gizmo são **vectores paralelos**, e uma delas
/// amostrada com outro passo daria ao elemento `i` o tamanho do elemento `j` — em silêncio.
/// ⭐⭐ **UMA FÓRMULA SÓ, e a feição muda apenas o DENOMINADOR** (`indice(k) = k · alcance / n`):
/// numa nuvem o alcance é a corrente inteira (a amostra espalha-se), numa cadeia o alcance é a
/// própria amostra (`alcance == n` ⇒ `indice(k) == k`, o prefixo). ⛔ E com tudo a caber, `alcance
/// == n == total` nas TRÊS feições ⇒ a identidade, **ao bit**, que é o que mantém toda cena
/// pequena exactamente como estava.
///
/// ⛔ **Não é um passo INTEIRO** (`ceil(total/MAX)`), e a razão é orçamento: com `4 596` pontos um
/// passo de `2` entregaria `2 298` glifos — **metade do tecto desperdiçada** — enquanto esta
/// fórmula entrega os `4 096` que o tecto paga.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Amostra {
    /// O alcance que a amostra varre, na corrente. Ver o cabeçalho.
    alcance: usize,
    /// Quantos elementos a amostra tem.
    n: usize,
}

impl Amostra {
    /// **A FEIÇÃO e a amostra dela, LIDAS DA MESMA CORRENTE.**
    ///
    /// ⚠️⚠️ **Ela recebeu uma `Feicao` e recebe a CORRENTE, e a diferença nasceu de uma MUTAÇÃO
    /// SOBREVIVENTE:** com dois argumentos o `resolve` podia pedir a amostra errada e trinta
    /// gates ficavam verdes. ⭐ A cura foi a porta **deixar de poder ser chamada errada**.
    ///
    /// ⚠️ **As feições OSSO e CORDA saíram em 2026-09-19, por ordem do dono** (*«no caso dos
    /// ossos e segmentos de corda, criaremos no nó shape formas similares»*), e com elas saiu o
    /// braço do prefixo. O que fica é a nuvem, cujo alcance é a corrente INTEIRA — e é isso que
    /// faz o `gap_y` ler-se como espaçamento em vez de uma faixa a voar pela borda.
    pub(crate) fn de(s: &Stream) -> Self {
        let total = s.count();
        Self {
            alcance: total,
            n: total.min(MAX_PONTOS),
        }
    }

    /// A amostra que não corta nada — para quem mede a corrente INTEIRA.
    ///
    /// ⚠️ **`#[cfg(test)]` porque o PRODUTO nunca a quer:** o gizmo tem tecto sempre, e uma porta
    /// que o ignora só faz sentido a uma sonda ou a um gate. *Marcá-la assim é a resposta honesta;
    /// um `#[allow(dead_code)]` seria calar a pergunta.*
    #[cfg(test)]
    pub(crate) fn inteira(total: usize) -> Self {
        Self {
            alcance: total,
            n: total,
        }
    }

    /// O índice, na corrente, do `k`-ésimo elemento da amostra.
    fn indice(&self, k: usize) -> usize {
        if self.n == 0 {
            return 0;
        }
        k * self.alcance / self.n
    }

    /// **Lê uma coluna pelos índices DESTA amostra** — a porta por onde todas passam.
    fn colhe<T: Copy, U>(&self, v: &[T], f: impl Fn(T) -> U) -> Vec<U> {
        (0..self.n)
            .filter_map(|k| v.get(self.indice(k)).copied().map(&f))
            .collect()
    }
}

/// As posições de uma corrente, pelos índices da amostra.
fn posicoes(s: &Stream, am: &Amostra) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => am.colhe(v, |q| q),
        _ => Vec::new(),
    }
}

/// **O que o gizmo SEGURA de uma corrente** — a porta por onde uma sonda pergunta o mesmo que o
/// produto responde. ⚠️ Sem ela, uma sonda reimplementa o corte e mede a própria cópia.
///
/// ⚠️ `#[cfg(test)]`: no produto quem chama é o [`resolve`], que já tem a amostra em mãos.
#[cfg(test)]
#[must_use]
pub(crate) fn posicoes_amostradas(s: &Stream) -> Vec<[f32; 2]> {
    posicoes(s, &Amostra::de(s))
}

/// **A ESCALA de cada elemento, como MULTIPLICADOR do glifo** — `None` quando a corrente não a
/// autorou (ordem do dono, 2026-09-19).
///
/// ⚠️ **A identidade é [`SIZE_IDENTITY`], que é `1`**, e é por isso que a coluna se lê como um
/// multiplicador directo: um `motion.scale(amount = 0,4)` entrega `0,4`, e o glifo fica a 40 % —
/// *o gizmo mostra o que o grafo fez, e não o que o desenhador dele achou bonito*.
///
/// ⚠️ **Uma coluna `Vec2` colapsa na MÉDIA dos eixos** — ver [`Grupo::escala`] para a razão.
pub(crate) fn escalas(s: &Stream, am: &Amostra) -> Option<Vec<f32>> {
    match s.get("size") {
        Some(Column::Scalar(v)) => Some(am.colhe(v, |e| e)),
        Some(Column::Vec2(v)) => Some(am.colhe(v, |e| (e[0] + e[1]) * 0.5)),
        _ => None,
    }
}

/// **A ROTAÇÃO de cada elemento, em graus** — `None` quando a corrente não a traz.
pub(crate) fn rotacoes(s: &Stream, am: &Amostra) -> Option<Vec<f32>> {
    escalares(s, "rot", am)
}

/// Uma coluna escalar da corrente, pelos índices da amostra. ⚠️ **Uma porta e não três cópias:** a
/// `rot`, a forma e o tamanho do gizmo fazem a MESMA leitura, e três cópias divergiriam no dia em
/// que uma delas ganhasse um filtro — ou, pior, um PASSO diferente (ver [`Amostra`]).
fn escalares(s: &Stream, nome: &str, am: &Amostra) -> Option<Vec<f32>> {
    match s.get(nome) {
        Some(Column::Scalar(v)) => Some(am.colhe(v, |e| e)),
        _ => None,
    }
}

/// **AS TOMADAS que este gizmo precisa: TODOS os sinks.**
///
/// ⚠️ **Não dá para escolher só os que não têm aparência** — essa pergunta é sobre a CORRENTE, e a
/// corrente só existe depois do cozimento. Pedir todos é a resposta honesta.
///
/// ⛔ **E ELA É VAZIA COM A LEI DESLIGADA, que é a configuração de FÁBRICA:** *um gizmo que não é
/// desenhado não pode cobrar o cozimento de que ele precisaria.* Na rota do DISPOSITIVO a bomba
/// **não marcha**, logo uma tomada obriga o `cook_taps_only` a cozinhar aquele sink **na CPU**.
///
/// ⚠️⚠️ **E o PREÇO disso foi MEDIDO, depois de eu o ter invocado errado.** Eu justifiquei esta
/// cerca com os `195,9 ms` que a auditoria de performance do módulo mede para a CPU a `4,19 M`
/// objectos — *um número de outro regime*. Medido no sítio (`o_que_a_tomada_do_gizmo_custa`,
/// `--release`, mediana de 5, `load 6,03`):
///
/// | cena | linhas | o cozimento do sink | de um quadro |
/// |---|---|---|---|
/// | `=111` | 12 800 | `0,226 ms` | `1,4 %` |
/// | `=116` | **102 400** | **`0,216 ms`** | **`1,3 %`** |
/// | `=120` | 20 | `0,001 ms` | `0,0 %` |
///
/// ⇒ **a cerca fica pela lei e não pelo relógio** (zero custo na configuração de fábrica é a mesma
/// lei do interruptor único), e o gizmo é utilizável mesmo nas cenas grandes. *Invocar um número
/// medido noutro regime para justificar uma decisão é o que o §0.0 proíbe, e eu fi-lo aqui.*
///
/// ⛔⛔⛔ **E a tabela acima media OUTRO regime também (ciclo 12, [doc 120 §8.5]):** nas três
/// cenas o sink é barato de recozinhar; na escada dos tectos (`emissor → integrador → carimbo`)
/// recozinhá-lo é **refazer a SIMULAÇÃO inteira na CPU** — medido na RTX a `32 768` partículas,
/// `2,4 ms` por quadro, *com o dispositivo a cozinhar a mesma cadeia em `0,25 ms`*. Com o carimbo
/// na placa o custo do Motion no app **não descia**, e a causa era esta tomada.
///
/// ⭐⭐ **A saída é a pergunta que o próprio DISPOSITIVO já respondeu:** quando ele desenhou o
/// quadro anterior, o registo do cozimento dele diz que colunas cada sink levou, e o
/// [`ph2d_gpu_cook::instances::veredito_do_dispositivo`] — **a mesma porta** que decidiu se ele
/// desenhava — diz se aquelas colunas desenham. Se desenham, o gizmo não aparece (*ou se vêem as
/// peças, ou se vê o gizmo*) e a tomada seria cozimento para NADA ⇒ não se pede.
///
/// ⚠️ **Um quadro de atraso, NOMEADO:** o registo é do cozimento ANTERIOR (as tomadas armam-se
/// antes do cozimento deste quadro), logo o 1.º quadro na placa ainda paga a tomada e uma corrente
/// que PERDE a aparência mostra o gizmo um quadro depois. ⚠️ **E o veredito é o do DISPOSITIVO,
/// não o da CPU:** a placa lê a geometria pela EXISTÊNCIA da coluna e a CPU pelo VALOR (a
/// divergência nomeada no cabeçalho do `veredito_do_dispositivo`); segui-la aqui é o que mantém o
/// gizmo e os píxeis da placa a dizer a mesma coisa.
///
/// [doc 120 §8.5]: ../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md
#[must_use]
pub fn taps_for(motion: &MotionState, so_com_forma: bool) -> Vec<NodeId> {
    let ultimo_da_placa = motion.gpu_live.then(|| motion.gpu_cook.shape());
    taps_filtrados(&motion.sinks, so_com_forma, |sink| {
        ultimo_da_placa.is_some_and(|forma| a_placa_desenhou(forma.columns(sink)))
    })
}

/// **A lei de [`taps_for`], pura** — os sinks cujo desenho a placa ainda não resolveu.
#[must_use]
pub fn taps_filtrados(
    sinks: &[NodeId],
    so_com_forma: bool,
    desenhado_pela_placa: impl Fn(NodeId) -> bool,
) -> Vec<NodeId> {
    if !so_com_forma {
        return Vec::new();
    }
    sinks
        .iter()
        .copied()
        .filter(|&s| !desenhado_pela_placa(s))
        .collect()
}

/// **A placa desenhou este sink?** — `None` (o sink não foi encenado na placa) responde NÃO, e a
/// tomada é pedida como sempre.
#[must_use]
pub fn a_placa_desenhou(colunas: Option<&[String]>) -> bool {
    let Some(colunas) = colunas else {
        return false;
    };
    let tem = |nome: &str| colunas.iter().any(|c| c == nome);
    let lei = ph2d_render::SinkStyle {
        so_com_forma: true,
        ..ph2d_render::SinkStyle::PLAIN
    };
    ph2d_gpu_cook::instances::veredito_do_dispositivo(lei, tem("geometry_id"), tem("uv_rect")).0
}

/// A corrente que um nó entregou neste cozimento.
fn tap(motion: &MotionState, node: NodeId) -> Option<&Stream> {
    motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| s)
}

/// ⭐⭐⭐ **O RETRATO**: cada sink cuja corrente **não traz aparência** vira um grupo.
///
/// ⚠️ **A pergunta é a MESMA porta que o lowering usa** ([`ph2d_eval_motion::tem_aparencia`])
/// — um segundo predicado aqui divergiria no dia em que uma origem nova nascesse, e o artista
/// veria o gizmo E os pixels, ou nenhum dos dois.
///
/// `None` sem a ferramenta Motion na mão: um gizmo é de EDITOR, e o quadro que o produto entrega
/// não o tem.
///
/// ⛔⛔⛔ **E `None` com a LEI DESLIGADA** — report do dono, 2026-09-19: *«os retângulos voltaram e
/// os gizmos estão relativos ao zoom»*. Com a lei desligada as peças **desenham-se**, e o gizmo por
/// cima delas é ruído sobre arte correcta; pior, ele aparecia na configuração de FÁBRICA, que é a
/// lei desta casa violada (*tudo o que é novo shipa desligado*). ⭐ **Os dois lados são o mesmo
/// interruptor: ou se vêem as peças, ou se vê o gizmo.**
///
/// ⚠️ **A lei entra como ARGUMENTO e não é lida do ambiente aqui** — é a lição da auditoria do
/// §31: um gate constrói a resposta à mão e mede a LEI; quem lê o ambiente é a porta do produto,
/// num sítio só.
#[must_use]
pub fn resolve(
    motion: &MotionState,
    tool_is_motion: bool,
    so_com_forma: bool,
) -> Option<PontoGizmoView> {
    if !tool_is_motion || !so_com_forma {
        return None;
    }
    let mut grupos = Vec::new();
    for &node in &motion.sinks {
        let Some(s) = tap(motion, node) else {
            continue;
        };
        if ph2d_eval_motion::tem_aparencia(s) {
            continue;
        }
        let total = s.count();
        if total == 0 {
            continue;
        }
        let am = Amostra::de(s);
        let pontos = posicoes(s, &am);
        if pontos.is_empty() {
            continue;
        }
        grupos.push(Grupo {
            node,
            rot: rotacoes(s, &am),
            escala: escalas(s, &am),
            pontos,
            total,
        });
    }
    (!grupos.is_empty()).then_some(PontoGizmoView { grupos })
}

static VIEW: std::sync::Mutex<Option<PontoGizmoView>> = std::sync::Mutex::new(None);

/// Publica (ou limpa) o retrato deste quadro. ⚠️ Publicar de novo SUBSTITUI — largar a ferramenta
/// Motion limpa o gizmo em vez de o deixar a pairar.
pub fn publish(v: Option<PontoGizmoView>) {
    if let Ok(mut slot) = VIEW.lock() {
        *slot = v;
    }
}

/// O retrato deste quadro, se houver.
#[must_use]
pub fn view() -> Option<PontoGizmoView> {
    VIEW.lock().ok().and_then(|s| s.clone())
}

#[cfg(test)]
#[path = "ponto_gizmo_tests.rs"]
mod tests;
