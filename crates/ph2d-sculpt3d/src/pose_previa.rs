//! **O INDICADOR DO PINCEL DE POSE** — o osso que se vê **antes** de premir.
//!
//! O pincel de pose não tem atenuação radial: o anel do cursor não diz nada
//! sobre o que ele vai mover. O que diz é a **cadeia** — onde está a dobradiça
//! que ele achou e até onde vai o membro que ele vai rodar. Desenhá-la é a
//! diferença entre um gesto que se aprende à segunda tentativa e um que se
//! aprende ao vê-lo.
//!
//! # ⭐⭐ Onde este ficheiro ganha ao alvo, e o número de cada linha
//!
//! O alvo desenha o mesmo indicador e **reconstrói a cadeia inteira a cada
//! movimento do rato**, sem traço nenhum — é a causa registada em quatro
//! relatos públicos de o editor engasgar em malha densa (espec §10, §13). Aqui
//! são **três** coisas diferentes, e nenhuma é uma promessa solta:
//!
//! | o que custa | como é pago |
//! |---|---|
//! | a **adjacência** (`O(faces)`), e com peças desligadas o emparelhamento `O(V²)` | construída **uma vez por gesto**, guardada em [`PosePrevia`] e reutilizada por todos os quadros de sobrevoo |
//! | a **cadeia** (varreduras `O(V)` + suavização `O(V·N)` por segmento) | reconstruída só quando a **chave** muda — e a chave é exactamente o que a §2–§3 lê |
//! | o **ritmo** | o alvo paga **por EVENTO de ponteiro** (~16 por quadro a 1 kHz); aqui o indicador corre **uma vez por quadro**, e acima do orçamento nem isso |
//!
//! ⚠️⚠️ **MEDIDO, e o PERFIL DE BUILD decide o número** — a tabela que governa o
//! produto é a do perfil em que o dono corre o smoke (`smoke` herda `release`),
//! sobre a malha da própria cena `=41` (sonda `mede_o_indicador_nesta_cena`,
//! `ph2d-app-sculpt3d`):
//!
//! | segmentos | 1.ª construção | quadro repetido | quadros de silêncio |
//! |---|---|---|---|
//! | **1** (omissão) | `2,88 ms` | `0,37 µs` | `2` |
//! | `3` | `5,96 ms` | `0,23 µs` | `4` |
//! | `20` (o tecto do painel) | `22,69 ms` | `0,74 µs` | `14` |
//!
//! ⇒ **na configuração de omissão o osso segue o cursor a ~30 Hz** e um quadro
//! em que nada mudou custa `0,002 %` de um quadro; no tecto do painel uma
//! construção sozinha passa o quadro de `60 fps`, e o orçamento converte *«o
//! editor arrasta»* — a queixa pública contra o alvo — em *«o indicador
//! demora»*, que é a troca certa para uma figura que descreve um gesto que
//! ainda não aconteceu.
//!
//! ⚠️ A mesma sonda no perfil de **teste** lê `22,12 / 97,69 / 344,63 ms` — `8×`
//! a `15×` mais lento. *Uma tabela sem o perfil ao lado mede outro programa*, e
//! um tecto em milissegundos calibrado ali seria `15×` conservador de mais no
//! sítio onde o artista está.
//!
//! ⚠️ E a lei escala com a malha, não com o pincel: a sonda irmã da `ph2d-pose`
//! (perfil de teste, corpus do oráculo) lê `0,16 ms` a `1 298` vértices e
//! `0,66 ms` a `4 930`, contra `265,87 ms` a `20` segmentos × `100`
//! suavizações — a faixa é de **`1 600×`**, e é essa faixa que torna um tecto
//! fixo um palpite.

use std::time::{Duration, Instant};

use ph2d_mesh::{Face, Mesh};

use crate::{Brush, Symmetry};

/// ⭐ **O ORÇAMENTO: o indicador nunca gasta mais do que um décimo de um quadro
/// de 60 fps** — `16,7 ms / 10`.
///
/// Ele é chrome sobre um gesto que **ainda não aconteceu**: se ele for a razão
/// de um quadro ser perdido, é melhor não existir. Da última construção tira-se
/// quantos quadros ela vale (`ceil(custo / orçamento)`) e são esses os quadros
/// em que a próxima não pode acontecer. Na configuração de omissão
/// (`t ≈ 0,2 ms`) isso é **um** quadro ⇒ o indicador segue o cursor
/// exactamente; a `20` segmentos (`t ≈ 50 ms`) são `30` quadros ⇒ ele deixa de
/// perseguir o cursor em vez de **arrastar o editor**, que é exactamente a
/// queixa pública registada contra o alvo.
///
/// ⚠️⚠️ **A contagem é de CHAMADAS, e não de relógio de parede, e a diferença é
/// o que torna isto gateável:** o indicador é chamado uma vez por quadro, logo
/// contar chamadas **é** contar quadros — e um gate consegue afirmar «dentro de
/// N quadros ele reconstrói», que com um relógio de parede seria mais um membro
/// da família de flakes sob fan-out (`CLAUDE.md` §5.0). *O relógio entra só
/// onde é insubstituível: a medir o que a construção custou, que é o número que
/// se auto-calibra ao perfil em que o produto de facto corre.*
const ORCAMENTO_POR_QUADRO: Duration = Duration::from_micros(1_670);

/// Um segmento da cadeia, como o indicador o desenha: em espaço de **objecto**.
///
/// ⚠️ **Não é uma segunda versão da lei** — os dois pontos saem de
/// [`ph2d_pose::Cadeia::ossos`], que é a lei do §6 aplicada à cabeça do próprio
/// osso. Ver o doc-comment de lá, que é onde a armadilha está escrita.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Osso {
    /// O lado da **dobradiça** — o pivô daquele segmento.
    pub origem: [f32; 3],
    /// O lado da **mão** — a extremidade que segue o arrasto.
    pub cabeca: [f32; 3],
}

/// O que decide se a **adjacência** guardada ainda descreve esta malha.
///
/// ⚠️ **As ligações artificiais entre peças dependem das POSIÇÕES** (§2.4) e a
/// chave não as pode ler sem varrer a malha inteira. Quem as invalida é o
/// [`PosePrevia::esquecer`] que todo gesto novo dispara — e o que sobra está
/// declarado no doc daquele método.
#[derive(Clone, Copy, PartialEq, Debug)]
struct ChaveTerreno {
    vertices: usize,
    faces: usize,
    so_conectado: bool,
    distancia_max: f32,
}

/// O que decide se a **cadeia** guardada ainda é a que o pen-down construiria.
///
/// ⭐⭐ **Ela traz exactamente o que a §2–§3 lê, e nada mais** — e isso é uma
/// afirmação medível, não arrumação: a `força`, a `inversão`, a `âncora`, a
/// `trava de rotação` e as **suavizações** não entram, porque a construção
/// geométrica não as consulta (as suavizações mexem nos **pesos**, e um osso
/// não tem peso). *Pôr um knob a mais aqui faz o indicador reconstruir-se por
/// nada; pôr um a menos faz dele uma mentira.*
#[derive(Clone, Copy, PartialEq, Debug)]
struct ChaveOsso {
    terreno: ChaveTerreno,
    eleito: u32,
    /// ⚠️ **Onde o vértice eleito ESTÁ.** É o que apanha a malha a mudar por
    /// baixo de um cursor parado — um desfazer, um filtro, outra peça — sem
    /// varrer nada: a cadeia nasce ali, logo se aquele ponto não se moveu a
    /// vizinhança dele quase de certeza também não.
    sob_o_cursor: [f32; 3],
    cursor: [f32; 3],
    modo: ph2d_pose::Modo,
    segmentos: u32,
    desvio_da_origem: f32,
    raio: f32,
    simetria: [bool; 3],
}

/// A adjacência, a cadeia e o relógio que decide quando as refazer.
///
/// ⚠️ **Vive no [`crate::SculptStroke`]**, ao lado da sessão do traço, e não
/// numa estrutura própria da cena: assim o indicador e o gesto partilham a
/// mesma porta ([`crate::SculptStroke::pose_ossos`]) e **é inexprimível** que
/// um mostre uma cadeia e o outro construa outra.
#[derive(Clone, Debug, Default)]
pub struct PosePrevia {
    terreno: Option<(ChaveTerreno, ph2d_pose::Vizinhanca)>,
    chave: Option<ChaveOsso>,
    ossos: Vec<Osso>,
    inerte: bool,
    /// Quantas chamadas (= quadros) desde a última construção.
    quadros: u32,
    /// O que a última construção custou — o único sítio onde o relógio entra.
    custo: Duration,
    /// **Instrumento:** quantas cadeias o indicador construiu. Lido por gate —
    /// *uma vantagem escrita num cabeçalho é promessa; uma com contador é
    /// propriedade.*
    pub(crate) construcoes: u32,
    /// **Instrumento:** quantas adjacências o indicador construiu.
    pub(crate) adjacencias: u32,
}

impl PosePrevia {
    /// ⚠️ **Deita fora tudo**, adjacência incluída — chamado pelo
    /// [`crate::SculptStroke::begin`], que é o único momento em que a malha
    /// pode ter mudado de forma que a chave não veja.
    ///
    /// ⛔ **O que fica declarado:** uma operação que troque as posições sem
    /// passar por um gesto (um desfazer, um remalhamento que preserve as
    /// contagens) **e** deixe o vértice sob o cursor exactamente onde estava
    /// mantém o indicador desactualizado até o cursor se mover. É o indicador,
    /// nunca a deformação — o pen-down constrói sempre de raiz.
    pub fn esquecer(&mut self) {
        self.terreno = None;
        self.chave = None;
        self.ossos.clear();
        self.inerte = false;
        self.quadros = 0;
        self.custo = Duration::ZERO;
    }

    /// Quantos quadros a última construção comprou de silêncio — ver
    /// [`ORCAMENTO_POR_QUADRO`]. `1` quando ela coube no orçamento, que é o
    /// caso da configuração de omissão.
    fn quadros_de_silencio(&self) -> u32 {
        let razao = self.custo.as_secs_f64() / ORCAMENTO_POR_QUADRO.as_secs_f64();
        (razao.ceil() as u32).max(1)
    }

    /// O caso §11.1 — **o pivô caiu em cima do cursor e o pincel não faz nada**.
    ///
    /// ⭐ É isto que torna o indicador mais do que um enfeite: o alvo cala-se
    /// ali, e o artista arrasta e não acontece coisa nenhuma.
    pub fn inerte(&self) -> bool {
        self.inerte
    }
}

impl crate::SculptStroke {
    /// ⭐⭐ **O INDICADOR, e ele é UMA porta.**
    ///
    /// Durante um traço devolve os ossos **vivos** — a cadeia que está a ser
    /// resolvida, de graça, já dobrada pelo arrasto. Fora dele devolve a cadeia
    /// que o pen-down construiria **sob este cursor**, com a cache e o
    /// orçamento deste módulo.
    ///
    /// ⚠️ **Duas funções — uma «em repouso» e outra «a mexer» — seriam duas
    /// respostas à mesma pergunta**, e a que o artista vê é a que envelhece.
    ///
    /// Devolve vazio quando o verbo não é o de pose, quando a malha não tem
    /// vértices, ou quando o orçamento ainda não deixou reconstruir e não há
    /// nada guardado.
    pub fn pose_ossos(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        sym: Symmetry,
        centro: [f32; 3],
    ) -> &[Osso] {
        if brush.verb != crate::Verb::Pose {
            return &[];
        }
        let mut ctrl = brush.pose.lei(brush);
        ctrl.simetria = [sym.x, sym.y, sym.z];

        // ⭐ O traço a decorrer: a cadeia já existe e já está resolvida.
        if let Some(vivos) = self.pose_ossos_vivos(&ctrl) {
            self.pose_previa.ossos = vivos;
            self.pose_previa.inerte = false;
            return &self.pose_previa.ossos;
        }

        let posicoes = mesh.positions();
        let terreno = ChaveTerreno {
            vertices: posicoes.len(),
            faces: mesh.faces().len(),
            so_conectado: ctrl.so_conectado,
            distancia_max: ctrl.distancia_max_entre_pecas,
        };

        // ⭐⭐ **O quadro em que nada mudou custa `O(1)`, e essa palavra é o
        // ponto.** A chave traz o vértice eleito, que é `O(V)` a achar — mas
        // reconstruir a chave com o eleito ANTERIOR responde à mesma pergunta
        // sem varrer a malha: se ele continua onde estava e o resto da chave
        // bate, o eleito de hoje é o mesmo.
        //
        // ⚠️⚠️ **MEDIDO, e era um defeito meu:** enquanto o `mais_proximo_global`
        // corria antes desta comparação, um quadro de sobrevoo **parado** custava
        // `456 µs` na malha da cena `=41` (`24 386` vértices) — `2,7 %` de um
        // quadro para não fazer nada, mais uma alocação da malha inteira por
        // quadro. *Uma cache que faz o trabalho caro antes de perguntar se
        // precisa dele não é uma cache.*
        //
        // ⚠️ A comparação é de **struct inteira** de propósito: um campo novo na
        // chave entra nela sem ninguém se lembrar, e uma lista de `&&` escrita à
        // mão é exactamente onde ele seria esquecido.
        if let Some(anterior) = self.pose_previa.chave
            && let Some(&sob_o_cursor) = posicoes.get(anterior.eleito as usize)
        {
            let sonda = ChaveOsso {
                terreno,
                eleito: anterior.eleito,
                sob_o_cursor,
                cursor: centro,
                modo: ctrl.modo,
                segmentos: ctrl.segmentos,
                desvio_da_origem: ctrl.desvio_da_origem,
                raio: ctrl.raio,
                simetria: ctrl.simetria,
            };
            if sonda == anterior {
                return &self.pose_previa.ossos;
            }
        }

        // ⚠️ A ocultação por vértice **não está ligada** neste módulo — é a
        // mesma ausência declarada que o `stroke_pose` escreve no pen-down, e
        // ela é partilhada de propósito: o indicador tem de ler a malha
        // exactamente como o gesto a vai ler.
        let escondido = vec![false; posicoes.len()];
        let Some(eleito) = ph2d_pose::cadeia::mais_proximo_global(posicoes, &escondido, centro)
        else {
            self.pose_previa.ossos.clear();
            return &[];
        };
        let chave = ChaveOsso {
            terreno,
            eleito,
            sob_o_cursor: posicoes[eleito as usize],
            cursor: centro,
            modo: ctrl.modo,
            segmentos: ctrl.segmentos,
            desvio_da_origem: ctrl.desvio_da_origem,
            raio: ctrl.raio,
            simetria: ctrl.simetria,
        };
        // O orçamento: uma construção cara compra silêncio proporcional a ela.
        // ⚠️ O contador anda **aqui**, e não no topo: um quadro em que a chave
        // não mudou não é um quadro de espera — não houve nada a adiar.
        self.pose_previa.quadros += 1;
        if self.pose_previa.quadros < self.pose_previa.quadros_de_silencio() {
            return &self.pose_previa.ossos;
        }

        let inicio = Instant::now();
        if self.pose_previa.terreno.as_ref().map(|(k, _)| *k) != Some(terreno) {
            let mut viz = ph2d_pose::Vizinhanca::construir(
                posicoes.len(),
                mesh.faces().iter().map(Face::verts),
                &escondido,
            );
            if !ctrl.so_conectado {
                viz.ligar_pecas(posicoes, ctrl.distancia_max_entre_pecas);
            }
            self.pose_previa.adjacencias += 1;
            self.pose_previa.terreno = Some((terreno, viz));
        }
        let (_, viz) = self.pose_previa.terreno.as_ref().expect("acabou de nascer");
        let pose = ph2d_pose::Pose::comecar(viz, posicoes, &escondido, eleito, centro, &ctrl);
        let mut pares = Vec::new();
        pose.ossos(&ctrl, &mut pares);
        self.pose_previa.ossos = pares
            .into_iter()
            .map(|[origem, cabeca]| Osso { origem, cabeca })
            .collect();
        self.pose_previa.inerte = pose.inerte();
        self.pose_previa.chave = Some(chave);
        self.pose_previa.construcoes += 1;
        self.pose_previa.custo = inicio.elapsed();
        self.pose_previa.quadros = 0;
        &self.pose_previa.ossos
    }

    /// `true` quando o pivô caiu em cima do cursor — ver [`PosePrevia::inerte`].
    pub fn pose_previa_inerte(&self) -> bool {
        self.pose_previa.inerte()
    }

    /// Os ossos da cadeia **viva**, quando há traço a decorrer.
    fn pose_ossos_vivos(&self, ctrl: &ph2d_pose::Controlos) -> Option<Vec<Osso>> {
        let mut pares = Vec::new();
        self.pose_sessao()?.ossos(ctrl, &mut pares);
        Some(
            pares
                .into_iter()
                .map(|[origem, cabeca]| Osso { origem, cabeca })
                .collect(),
        )
    }
}
