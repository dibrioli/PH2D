//! **O INDICADOR DO PINCEL DE CONTORNO** — o troço de borda que vai dobrar, e
//! até onde a dobra entra na peça, vistos **antes** de premir.
//!
//! # ⚠️⚠️ Porque o anel do cursor descreve este verbo PIOR do que descreve os outros
//!
//! Nos outros pincéis a região sai do cursor e esmorece com a distância a ele —
//! o círculo **é** a região. Aqui ela sai da **BORDA**: o cursor só escolhe a
//! âncora, e o que se deforma é um troço da beirada mais uma faixa para dentro.
//! Um círculo ali promete uma coisa e a ferramenta faz outra. ⇒ o indicador é a
//! resposta às **duas** perguntas que o painel faz e o anel não sabe responder:
//!
//! | o que o artista mexe | o que o indicador mostra |
//! |---|---|
//! | `Falloff along the edge` (`Constant`/`Radius`/`Loop`/…) | **quanto da boca** entra — a cadeia desenhada, com o brilho de cada pedaço a ser o peso dele |
//! | `Origin offset` | **quão fundo** a dobra entra, e onde está o eixo — a linha da profundidade, com o anel no ponto-origem |
//!
//! # ⭐⭐ O custo, e a mesma disciplina que o osso da pose pagou
//!
//! As fases A–E são `O(faces)` + `O(V)`, e um indicador de sobrevoo que as
//! refizesse a cada movimento do rato seria o defeito que o [`super::pose_previa`]
//! documenta contra o alvo. As três peças são pagas na mesma moeda:
//!
//! | o que custa | como é pago |
//! |---|---|
//! | o **censo de bordas** (`O(faces)`) | construído por chave de terreno e reutilizado por todos os quadros de sobrevoo |
//! | as **fases B–E** (`O(V)`) | reconstruídas só quando a chave muda — e a chave traz exactamente o que elas leem |
//! | o **ritmo** | uma vez por quadro, e acima do orçamento nem isso — [`super::pose_previa::ORCAMENTO_POR_QUADRO`] |
//!
//! ⚠️ **O orçamento é PARTILHADO com o osso da pose, de propósito:** os dois são
//! chrome sobre um gesto que ainda não aconteceu, e dois números diferentes para
//! a mesma política seriam duas respostas à mesma pergunta.
//!
//! ⚠️⚠️ **MEDIDO no perfil em que o dono corre o smoke** (`--profile smoke`),
//! sobre a malha da própria cena `=42` — sonda `mede_o_indicador_desta_cena`,
//! `ph2d-app-sculpt3d`:
//!
//! | raio | pedaços | 1.ª construção | quadro repetido |
//! |---|---|---|---|
//! | `0,30` (omissão) | 48 | `0,096 ms` | `0,29 µs` |
//! | `0,60` | 48 | `0,083 ms` | `0,03 µs` |
//! | `1,00` | 48 | `0,084 ms` | `0,03 µs` |
//!
//! ⇒ nesta peça a construção cabe **dezassete vezes** no orçamento, logo a fita
//! segue o cursor **quadro a quadro**, e um quadro em que nada mudou custa
//! `0,0002 %` de um quadro de 60 fps. ⚠️ **O custo é da MALHA e não do pincel**
//! (o raio mal o move — ele só escolhe quantos anéis entram): numa peça densa
//! vale a tabela do [`super::pose_previa`], que mede a mesma ordem de grandeza
//! sobre `24 386` vértices, e é aí que o orçamento começa a falar.

use std::time::Instant;

use ph2d_mesh::{Face, Mesh};

use crate::{Brush, Symmetry};

/// Um pedaço da borda afectada, em espaço de **objecto**, com o peso que ele
/// leva (`0..1`).
///
/// ⚠️ **O peso é o da LEI, lido de [`ph2d_boundary::Contorno::pesos`]** — não uma
/// segunda conta. Na cadeia o anel é `0`, logo o peso ali é exactamente a *queda
/// ao longo do contorno*, que é o que o selector do painel muda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrechoDaBorda {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub peso: f32,
}

/// O que decide se o **censo de bordas** guardado ainda descreve esta malha.
#[derive(Clone, Copy, PartialEq, Debug)]
struct ChaveTerreno {
    vertices: usize,
    faces: usize,
}

/// O que decide se a **estrutura** guardada ainda é a que o pen-down construiria.
///
/// ⭐ Ela traz o que as fases B–E leem, e nada mais. ⛔ A **força** e o **modo**
/// ficam de fora com motivo: a força escala o deslocamento e o modo escolhe a
/// lei do passo — nenhum dos dois muda *quem* entra na região nem *com que
/// peso*, que é tudo o que esta figura desenha. *Um knob a mais aqui faz o
/// indicador reconstruir-se por nada; um a menos faz dele uma mentira.*
#[derive(Clone, Copy, PartialEq, Debug)]
struct ChaveContorno {
    terreno: ChaveTerreno,
    ancora: u32,
    /// ⚠️ **Onde a âncora ESTÁ** — é o que apanha a malha a mudar por baixo de
    /// um cursor parado sem varrer nada, como na irmã da pose.
    sob_a_ancora: [f32; 3],
    cursor: [f32; 3],
    raio_inicial: f32,
    raio_dinamico: f32,
    deslocamento_da_origem: f32,
    queda_no_contorno: ph2d_boundary::QuedaNoContorno,
    falloff: crate::Falloff,
    simetria: [bool; 3],
}

/// O censo de bordas, a figura e o relógio que decide quando a refazer.
///
/// ⚠️ **Vive no [`crate::SculptStroke`]**, ao lado da sessão do traço, para o
/// indicador e o gesto partilharem a mesma porta — e ser **inexprimível** que um
/// mostre um troço de borda e o outro deforme outro.
#[derive(Clone, Debug, Default)]
pub struct ContornoPrevia {
    terreno: Option<(ChaveTerreno, ph2d_boundary::Topologia)>,
    chave: Option<ChaveContorno>,
    borda: Vec<TrechoDaBorda>,
    profundidade: Option<[[f32; 3]; 2]>,
    /// Quantas chamadas (= quadros) desde a última construção.
    quadros: u32,
    /// O que a última construção custou — o único sítio onde o relógio entra.
    custo: std::time::Duration,
    /// **Instrumento:** quantas estruturas o indicador construiu. Lido por gate.
    pub(crate) construcoes: u32,
    /// **Instrumento:** quantos censos de bordas o indicador construiu.
    pub(crate) censos: u32,
}

impl ContornoPrevia {
    /// ⚠️ **Deita fora tudo**, censo incluído — chamado pelo
    /// [`crate::SculptStroke::begin`], o único momento em que a malha pode ter
    /// mudado de forma que a chave não veja.
    ///
    /// ⛔ **Declarado:** uma edição que troque as posições sem passar por um
    /// gesto **e** deixe a âncora exactamente onde estava mantém a figura
    /// desactualizada até o cursor se mover. É o indicador, nunca a deformação —
    /// o pen-down constrói sempre de raiz.
    pub fn esquecer(&mut self) {
        self.terreno = None;
        self.chave = None;
        self.borda.clear();
        self.profundidade = None;
        self.quadros = 0;
        self.custo = std::time::Duration::ZERO;
    }

    /// Os pedaços da borda que vão entrar, com o peso de cada um.
    #[must_use]
    pub fn borda(&self) -> &[TrechoDaBorda] {
        &self.borda
    }

    /// A linha da profundidade: da âncora ao **ponto-origem**, que é o eixo em
    /// torno do qual a lei roda. `None` quando não há contorno sob o cursor.
    #[must_use]
    pub fn profundidade(&self) -> Option<[[f32; 3]; 2]> {
        self.profundidade
    }

    fn quadros_de_silencio(&self) -> u32 {
        let razao =
            self.custo.as_secs_f64() / super::pose_previa::ORCAMENTO_POR_QUADRO.as_secs_f64();
        (razao.ceil() as u32).max(1)
    }
}

impl crate::SculptStroke {
    /// ⭐⭐ **O INDICADOR DO CONTORNO, e ele é UMA porta.**
    ///
    /// Durante um traço devolve a figura **viva** — a cadeia que o pen-down
    /// fotografou, desenhada onde os vértices dela estão **agora**, de graça.
    /// Fora dele, a que o pen-down construiria sob este cursor, com a cache e o
    /// orçamento deste módulo.
    ///
    /// ⚠️ **A linha da profundidade sai sempre do REPOUSO**, também durante o
    /// traço: o eixo é fotografado no pen-down (§A–E) e não se move enquanto a
    /// mão arrasta. Desenhá-lo a seguir os vértices faria a figura contar uma
    /// lei que o motor não tem.
    ///
    /// Devolve tudo vazio quando o verbo não é o de contorno, quando a malha não
    /// tem vértices, ou quando não há borda ao alcance — que é a recusa da §5.2
    /// e **é** informação: ali o pincel não faz nada.
    pub fn boundary_contorno(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        sym: Symmetry,
        centro: [f32; 3],
    ) -> &ContornoPrevia {
        if brush.verb != crate::Verb::Boundary {
            self.boundary_previa.borda.clear();
            self.boundary_previa.profundidade = None;
            return &self.boundary_previa;
        }
        let mut ctrl = brush.boundary.lei(brush);
        ctrl.simetria = [sym.x, sym.y, sym.z];

        // ⭐ O traço a decorrer: a estrutura já existe, fotografada no pen-down.
        if self.boundary_vivo(mesh) {
            return &self.boundary_previa;
        }

        let posicoes = mesh.positions();
        let terreno = ChaveTerreno {
            vertices: posicoes.len(),
            faces: mesh.faces().len(),
        };

        // ⭐⭐ **O quadro em que nada mudou custa `O(1)`** — a mesma lei da irmã
        // da pose, e pela mesma razão medida: achar a âncora é `O(V)`, e
        // reconstruir a chave com a âncora ANTERIOR responde à mesma pergunta
        // sem varrer a malha.
        if let Some(anterior) = self.boundary_previa.chave
            && let Some(&sob_a_ancora) = posicoes.get(anterior.ancora as usize)
        {
            let sonda = ChaveContorno {
                terreno,
                ancora: anterior.ancora,
                sob_a_ancora,
                cursor: centro,
                raio_inicial: ctrl.raio_inicial,
                raio_dinamico: ctrl.raio_dinamico,
                deslocamento_da_origem: ctrl.deslocamento_da_origem,
                queda_no_contorno: ctrl.queda_no_contorno,
                falloff: brush.falloff,
                simetria: ctrl.simetria,
            };
            if sonda == anterior {
                return &self.boundary_previa;
            }
        }

        // ⚠️ A ocultação por vértice **não está ligada**, e é a mesma ausência
        // declarada que o `stroke_boundary` escreve no pen-down: o indicador tem
        // de ler a malha exactamente como o gesto a vai ler.
        let escondido = vec![false; posicoes.len()];
        let Some(ancora) = ph2d_boundary::ancora::mais_proximo(posicoes, &escondido, centro) else {
            self.boundary_previa.borda.clear();
            self.boundary_previa.profundidade = None;
            return &self.boundary_previa;
        };
        let chave = ChaveContorno {
            terreno,
            ancora,
            sob_a_ancora: posicoes[ancora as usize],
            cursor: centro,
            raio_inicial: ctrl.raio_inicial,
            raio_dinamico: ctrl.raio_dinamico,
            deslocamento_da_origem: ctrl.deslocamento_da_origem,
            queda_no_contorno: ctrl.queda_no_contorno,
            falloff: brush.falloff,
            simetria: ctrl.simetria,
        };
        // O orçamento: uma construção cara compra silêncio proporcional a ela.
        // ⚠️ O contador anda AQUI e não no topo — um quadro em que a chave não
        // mudou não é um quadro de espera.
        self.boundary_previa.quadros += 1;
        if self.boundary_previa.quadros < self.boundary_previa.quadros_de_silencio() {
            return &self.boundary_previa;
        }

        let inicio = Instant::now();
        if self.boundary_previa.terreno.as_ref().map(|(k, _)| *k) != Some(terreno) {
            let topo = ph2d_boundary::Topologia::construir(
                posicoes.len(),
                mesh.faces().iter().map(Face::verts),
                &escondido,
            );
            self.boundary_previa.censos += 1;
            self.boundary_previa.terreno = Some((terreno, topo));
        }
        let (_, topo) = self
            .boundary_previa
            .terreno
            .as_ref()
            .expect("acabou de nascer");
        let curva = |p: f32| brush.falloff.weight(1.0 - p);
        let curva_ref: ph2d_boundary::Curva<'_> = &curva;
        let contorno = ph2d_boundary::Contorno::comecar(
            topo,
            posicoes,
            mesh.normals(),
            &escondido,
            ancora,
            centro,
            &ctrl,
            curva_ref,
            ph2d_boundary::Fatores {
                mascara: mesh.masks(),
                ..Default::default()
            },
        );
        let mut borda = std::mem::take(&mut self.boundary_previa.borda);
        borda.clear();
        let mut profundidade = None;
        if let Ok(k) = contorno {
            desenhar(&k, topo, posicoes, &mut borda);
            let (a, b) = k.linha_da_profundidade(posicoes);
            profundidade = Some([a, b]);
            self.boundary_previa.construcoes += 1;
        }
        self.boundary_previa.borda = borda;
        self.boundary_previa.profundidade = profundidade;
        self.boundary_previa.chave = Some(chave);
        self.boundary_previa.custo = inicio.elapsed();
        self.boundary_previa.quadros = 0;
        &self.boundary_previa
    }

    /// A figura **viva**, quando há traço a decorrer. `true` quando a preencheu.
    fn boundary_vivo(&mut self, mesh: &Mesh) -> bool {
        let Some((contorno, topo, p0)) = self.boundary_sessao_viva() else {
            return false;
        };
        let mut borda = Vec::new();
        // ⚠️ A CADEIA nas posições de AGORA (o artista vê a boca que está a
        // dobrar); a linha da profundidade no REPOUSO (o eixo é do pen-down).
        desenhar(contorno, topo, mesh.positions(), &mut borda);
        let (a, b) = contorno.linha_da_profundidade(p0);
        self.boundary_previa.borda = borda;
        self.boundary_previa.profundidade = Some([a, b]);
        true
    }
}

/// Os pedaços da cadeia, com o peso de cada um — a lei lida, nunca recalculada.
///
/// # ⛔⛔ A cadeia NÃO vem em ordem de passeio, e ligá-la pela ordem dela desenha CORDAS
///
/// A fase C anda a borda **nos dois sentidos ao mesmo tempo**, e a `cadeia`
/// sai ordenada por **distância à âncora**, alternando os lados: medido na boca
/// da tigela, `[94, 0, 92, 1, 90, 4, …]` com distâncias
/// `[0, 0,131, 0,131, 0,262, 0,262, …]` — **`45` de `47`** pares consecutivos
/// **não** são vizinhos de borda. Uma polilinha construída com `windows(2)`
/// sobre ela não desenha a boca: desenha um ziguezague de cordas **através** da
/// boca, que é uma figura que a ferramenta não faz.
///
/// ⇒ o traçado é reconstruído pela **adjacência de borda**: a partir da âncora,
/// um passeio para cada lado enquanto o vizinho ainda pertence à cadeia. É a
/// mesma informação, lida pela relação que a descreve.
fn desenhar(
    k: &ph2d_boundary::Contorno,
    topo: &ph2d_boundary::Topologia,
    posicoes: &[[f32; 3]],
    saida: &mut Vec<TrechoDaBorda>,
) {
    let e = k.estrutura();
    let w = k.pesos();
    let peso = |v: u32| w.get(v as usize).copied().unwrap_or(0.0).clamp(0.0, 1.0);
    let ponto = |v: u32| posicoes.get(v as usize).copied().unwrap_or([0.0; 3]);

    let mut na_cadeia = vec![false; posicoes.len()];
    for &v in &e.cadeia {
        if let Some(c) = na_cadeia.get_mut(v as usize) {
            *c = true;
        }
    }
    let mut visitado = vec![false; posicoes.len()];
    let Some(&ancora) = e.cadeia.first() else {
        return;
    };
    if let Some(c) = visitado.get_mut(ancora as usize) {
        *c = true;
    }

    // Um passeio por sentido. ⚠️ Num bordo variedade cada vértice tem
    // **dois** vizinhos de borda, logo «o que não é de onde vim» é sempre o
    // seguinte — e é isso que torna o passeio determinista.
    let andar = |arranque: u32, visitado: &mut Vec<bool>| {
        let mut lado = Vec::new();
        let (mut anterior, mut actual) = (ancora, arranque);
        while na_cadeia.get(actual as usize).copied().unwrap_or(false) && !visitado[actual as usize]
        {
            visitado[actual as usize] = true;
            lado.push(actual);
            let Some(&prox) = topo
                .vizinhos_de_borda(actual)
                .iter()
                .find(|&&u| u != anterior)
            else {
                break;
            };
            anterior = actual;
            actual = prox;
        }
        lado
    };
    let lados: Vec<u32> = topo.vizinhos_de_borda(ancora).to_vec();
    let mut frente = Vec::new();
    let mut tras = Vec::new();
    for (i, arranque) in lados.into_iter().enumerate() {
        let lado = andar(arranque, &mut visitado);
        if i == 0 {
            frente = lado;
        } else {
            tras = lado;
        }
    }

    // A ordem do desenho: um lado invertido, a âncora, o outro lado.
    let mut ordem: Vec<u32> = Vec::with_capacity(frente.len() + tras.len() + 1);
    ordem.extend(tras.iter().rev().copied());
    ordem.push(ancora);
    ordem.extend(frente);

    for par in ordem.windows(2) {
        saida.push(TrechoDaBorda {
            a: ponto(par[0]),
            b: ponto(par[1]),
            peso: 0.5 * (peso(par[0]) + peso(par[1])),
        });
    }
    // ⚠️ **A cadeia pode FECHAR** (a boca de uma tigela é um laço) e o passeio
    // pára por ter dado a volta: sem este pedaço falta sempre um vão, e é logo
    // o que está do lado oposto ao cursor — o sítio onde o artista repara.
    if let (Some(&primeiro), Some(&ultimo)) = (ordem.first(), ordem.last())
        && primeiro != ultimo
        && topo.vizinhos_de_borda(ultimo).contains(&primeiro)
    {
        saida.push(TrechoDaBorda {
            a: ponto(ultimo),
            b: ponto(primeiro),
            peso: 0.5 * (peso(ultimo) + peso(primeiro)),
        });
    }
}

#[cfg(test)]
#[path = "boundary_previa_tests.rs"]
mod tests;
