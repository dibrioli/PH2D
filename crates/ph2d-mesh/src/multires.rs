//! **A MULTIRESOLUÇÃO** — os níveis, e o ir-e-voltar entre eles.
//!
//! Port de `reference/sculptgl/src/mesh/multiresolution/` (`Multimesh.js`,
//! `MeshResolution.js`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! Subdividir já dá resolução; o que a multiresolução dá é **descer**. O artista
//! esculpe o detalhe fino no nível 3, volta ao nível 0 para mover a forma
//! GRANDE, e sobe de novo com o detalhe intacto — que é a única maneira de
//! corrigir uma proporção depois de já ter feito a pele.
//!
//! # O detalhe é um DESLOCAMENTO, e ele vive num frame LOCAL
//!
//! Guardar as posições do nível de cima seria inútil: mover a base embaixo não
//! as moveria. O que se guarda é a **diferença** entre onde o vértice está e
//! onde a subdivisão o PORIA — e essa diferença é expressa nos eixos
//! `(normal, tangente, binormal)` daquele vértice. Assim, quando a base é
//! entortada, o detalhe **gira junto**: uma verruga continua saindo perpendicular
//! à pele em vez de apontar para onde a pele apontava antes.
//!
//! ```text
//! descer:  esperado = síntese(base, detalhe)  (o topo que ninguém tocou)
//!          base    += (topo − esperado)[..V]  (SÓ o que o artista moveu)
//!          detalhe  = (topo − subdivide(base)) no frame de cada vértice
//!
//! subir:   topo ← subdivide(base)             (a previsão)
//!          topo += detalhe, no MESMO frame
//! ```
//!
//! ⚠️ **O que desce é a DIFERENÇA, e a linha de cima já foi `base ← topo[..V]`**
//! — o que o original faz (`copyDataFromHigherRes`). Ela custava o modelo do
//! artista: `topo[i]` para um vértice par é a *regra par* aplicada à base, ou
//! seja um alisamento, então subdividir e descer **sem esculpir nada** encolhia
//! a base de raio médio 1,000 para **0,972**, e compunha a cada ciclo (2,81% ·
//! 3,32% · 3,45%). Ver [`Multires::lower`].
//!
//! ⚠️ **A ida e a volta são EXATAS quando nada muda embaixo**, e isso não é
//! aspiração: `previsão + (topo − previsão) = topo` ao bit, desde que o frame
//! seja o mesmo dos dois lados. É o gate que decide este módulo.
//!
//! ⚠️ **E é por isso que o frame tem UMA porta** ([`local_frame`]). Encode e
//! decode são as duas metades de uma inversa; escritos duas vezes, eles
//! divergem — e a divergência não aparece como erro, aparece como a escultura
//! escorregando um pouco a cada viagem.
//!
//! ⚠️ **A NORMAL do frame é a do topo, e a síntese lê a que CODIFICOU.** É o que
//! torna a viagem exata: enquanto o artista trabalha embaixo ninguém toca a
//! malha de cima, e ao subir o frame lido é literalmente o que codificou.
//! Recompor as normais ANTES do decode — o que um `rebuild` distraído faria —
//! quebraria a inversa em silêncio, e o sintoma seria o detalhe **derivando** a
//! cada subida.
//!
//! ⚠️ **E DEPOIS do decode o detalhe é re-expresso no frame novo**, porque o
//! `rebuild` do topo acabou de trocá-lo. É o mesmo deslocamento noutras
//! coordenadas — não um segundo modelo —, e sem isso a descida seguinte
//! compararia o topo com uma síntese torta e **inventaria um carimbo** que o
//! artista não fez.
//!
//! ⚠️ **A tangente sai do PRIMEIRO vizinho do anel.** A escolha é arbitrária e
//! o que importa é ser a MESMA nos dois lados — ela é, porque a adjacência do
//! topo não muda entre as duas chamadas.

use crate::mesh::Mesh;
use crate::subdivide::{Predicted, predict, subdivide};

/// O detalhe de um nível em relação ao de baixo.
///
/// ⚠️ O de POSIÇÃO vive no frame local; os de canal são deltas simples. Um
/// canal não tem orientação — girar a base não gira uma cor.
#[derive(Clone, Debug, Default)]
struct Details {
    /// `(normal, tangente, binormal)` por vértice do nível de cima.
    xyz: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
}

/// **Um nível fora da pilha** — a malha e o detalhe dela, como uma coisa só.
///
/// Opaco de propósito: quem o segura (a fila de refazer do editor) não tem nada
/// a perguntar a ele — só a devolvê-lo inteiro. Ver [`Multires::drop_top`].
///
/// ⚠️ **O `details` viaja junto e HOJE nenhum gate o vê** — a mutação que o troca
/// por um vazio passa nos doze. O mecanismo: `details[i]` só é lido por
/// [`Multires::higher`] ao ENTRAR no nível `i`, o que exige estar em `i − 1`, o
/// que exige um [`Multires::lower`] antes — e o `lower` **reescreve**
/// `details[i]`. O detalhe que um nível destacado carrega é, portanto,
/// sobrescrito antes de qualquer leitura. Ele fica porque *um nível É malha e
/// detalhe* e porque carregá-lo é um move, não uma cópia; a alternativa seria um
/// tipo que se diz um nível e devolve metade dele.
#[derive(Clone, Debug)]
pub struct DetachedLevel {
    mesh: Mesh,
    details: Details,
}

/// **A pilha de níveis.** O nível 0 é a base; cada nível acima é uma subdivisão
/// do anterior mais o detalhe que o artista pôs nele.
#[derive(Clone, Debug)]
pub struct Multires {
    levels: Vec<Mesh>,
    /// `details[i]` é o detalhe do nível `i` contra o `i − 1`; o do nível 0 é
    /// vazio e nunca lido.
    details: Vec<Details>,
    sel: usize,
}

/// **O piso do que conta como *o artista moveu isto*, relativo ao tamanho do
/// modelo.**
///
/// ⚠️ **Ele existe porque a régua é RECOMPUTADA, e o número dele é medido.** O
/// `encode` projeta o deslocamento num frame ortonormal e a síntese o
/// reconstrói; essa ida-e-volta erra **1,49e-8** num modelo de raio 1 — um ulp.
/// Sem piso, *todo* passeio entre níveis mediria um carimbo de 1e-8, e a entrada
/// de desfazer de quem só foi OLHAR custaria uma cópia inteira da base.
///
/// ⚠️ **Ele é RELATIVO à maior aresta da caixa do modelo** porque a malha não tem escala
/// canônica: um OBJ importado pode medir mil unidades, e um piso absoluto seria
/// generoso nele e apertado numa miniatura. A 230× o resíduo medido e a um
/// milionésimo do modelo, ele fica **muito abaixo do que um pincel consegue
/// autorar** — o menor dab move o vértice por uma fração do raio dele, que é
/// pelo menos um pixel de tela.
const STAMP_FLOOR: f32 = 1e-6;

impl Multires {
    /// Uma pilha de um nível só — o estado de toda malha que ninguém subdividiu.
    #[must_use]
    pub fn new(base: Mesh) -> Self {
        Self {
            levels: vec![base],
            details: vec![Details::default()],
            sel: 0,
        }
    }

    /// Em que nível o artista está.
    #[must_use]
    pub fn level(&self) -> usize {
        self.sel
    }

    /// Quantos níveis existem.
    #[must_use]
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// A malha do nível atual — a que o artista vê e esculpe.
    #[must_use]
    pub fn mesh(&self) -> &Mesh {
        &self.levels[self.sel]
    }

    /// A malha de um nível QUALQUER — para quem precisa dizer alguma coisa sobre
    /// um nível que não é o selecionado (um log, uma sonda).
    ///
    /// ⚠️ Só leitura, e de propósito: escrever num nível que o artista não está
    /// vendo pularia o `lower`/`higher`, que são quem mantém o detalhe coerente.
    #[must_use]
    pub fn level_mesh(&self, k: usize) -> Option<&Mesh> {
        self.levels.get(k)
    }

    /// A malha do nível atual, para esculpir.
    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.levels[self.sel]
    }

    /// Troca a malha do nível atual — a porta do undo, que devolve uma malha
    /// inteira.
    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.levels[self.sel] = mesh;
    }

    /// **Acrescenta um nível acima**, subdividindo o atual. Devolve `false` se
    /// não estamos no TOPO.
    ///
    /// ⚠️ **Só do topo, e a recusa é estrutural.** Subdividir do meio criaria um
    /// segundo nível `n + 1` sem dizer o que fazer com o que já existe lá — e a
    /// resposta honesta (*jogar fora o detalhe acima*) é destruição silenciosa
    /// de trabalho. O original recusa igual.
    pub fn add_level(&mut self) -> bool {
        if self.sel + 1 != self.levels.len() {
            return false;
        }
        let up = subdivide(&self.levels[self.sel]);
        let n = up.vert_count();
        self.levels.push(up);
        // O nível novo nasce SEM detalhe: ele é exatamente a subdivisão do de
        // baixo, então a diferença é zero em todo vértice.
        self.details.push(Details {
            xyz: vec![[0.0; 3]; n],
            colors: None,
            masks: None,
        });
        self.sel += 1;
        true
    }

    /// **Desce um nível**, carimbando na base o que o artista MOVEU em cima e
    /// guardando o detalhe. Devolve o carimbo (para poder desfazê-lo), ou `None`
    /// se já estava no nível 0.
    ///
    /// ⚠️ **O que desce é a DIFERENÇA, nunca a posição.** A versão anterior — e o
    /// original — copiavam `topo[..V]` verbatim para a base, e o preço foi
    /// MEDIDO: um `K` seguido de um `,`, **sem esculpir nada**, encolhia a base
    /// de raio médio 1,000 para **0,972** e a deslocava 0,038, porque `topo[i]`
    /// para um vértice par é a *regra par* aplicada à base — um alisamento. E
    /// **compunha**: 2,81% num ciclo, 3,32% em dois, 3,45% em três, convergindo
    /// para a superfície-limite. O artista perdia o modelo dele por ter olhado.
    ///
    /// ⚠️ **A régua da diferença é a SÍNTESE, não a previsão.** Comparar contra
    /// `predict` faria a base absorver o DETALHE a cada volta (subir e descer
    /// carimbaria a pele na forma grande); a pergunta certa é *o que mudou desde
    /// que cheguei aqui*, e a resposta é `topo − sintetizado(base, detalhe)` —
    /// que é exatamente zero quando ninguém tocou no topo.
    pub fn lower(&mut self) -> Option<Stamped> {
        if self.sel == 0 {
            return None;
        }
        let up = self.sel;
        let down = up - 1;

        // 1. O que o topo SERIA se o artista não o tivesse tocado desde que
        //    chegou. A diferença contra ele é o trabalho dele, e só ela desce.
        //
        // ⚠️ **Isto só funciona porque a subdivisão põe os vértices PARES em
        //    `[0, V)`** — o vértice `i` de baixo É o vértice `i` de cima. É uma
        //    das três divergências de forma que o `subdivide` tomou (o original
        //    aloca por ordem de visita e precisa de um mapa), e é aqui que ela
        //    se paga. Há gate afirmando a identidade.
        let predicted = predict(&self.levels[down]);
        let expected = synthesize(&self.levels[up], &self.details[up], &predicted);
        let mut stamped = self.stamp_down(up, down, &expected);
        if stamped.before.is_some() {
            stamped.details = Some(self.details[up].clone());
        }
        self.levels[down].rebuild();

        // 2. O que a subdivisão da base PORIA agora, e a diferença.
        let predicted = predict(&self.levels[down]);
        self.details[up] = encode(&self.levels[up], &predicted);
        self.sel = down;
        Some(stamped)
    }

    /// **Sobe um nível**, devolvendo o detalhe. `false` no topo.
    pub fn higher(&mut self) -> bool {
        if self.sel + 1 >= self.levels.len() {
            return false;
        }
        let down = self.sel;
        let up = down + 1;
        let predicted = predict(&self.levels[down]);
        let synth = synthesize(&self.levels[up], &self.details[up], &predicted);
        install(&mut self.levels[up], &synth);
        // ⚠️ O `rebuild` vem DEPOIS do decode, nunca antes: é ele que troca as
        // normais do topo, e o decode precisa das que codificaram.
        self.levels[up].rebuild();
        // ⚠️ **E o detalhe é RE-EXPRESSO no frame novo**, que o `rebuild` acabou
        // de trocar. Sem isto o detalhe guardado descreve o deslocamento em eixos
        // que não existem mais, e a `lower` seguinte compararia o topo com uma
        // síntese torta — inventando um carimbo grande que o artista não fez. O
        // deslocamento é o MESMO vetor; o que muda são as coordenadas dele.
        self.details[up] = encode(&self.levels[up], &predicted);
        self.sel = up;
        true
    }

    /// **Desfaz uma descida**: devolve à base o que o carimbo levou, re-encoda o
    /// detalhe e sobe a seleção. `false` se a pilha não está onde o carimbo diz.
    ///
    /// ⚠️ **Ele restaura o DETALHE junto**, e a razão está no doc do
    /// [`Stamped::details`]: re-encodar reproduz a forma e perde a distinção entre
    /// *o que o artista moveu* e *o que sempre foi detalhe*.
    pub fn undo_descent(&mut self, stamped: &Stamped) -> bool {
        if self.sel != stamped.level || self.sel + 1 >= self.levels.len() {
            return false;
        }
        let down = self.sel;
        let up = down + 1;
        if let Some(before) = &stamped.before {
            let mesh = &mut self.levels[down];
            mesh.positions_mut().copy_from_slice(&before.positions);
            // ⚠️ `None` quer dizer *não havia plano*, e restaurar é REMOVÊ-LO —
            // não zerá-lo. Uma descida pode ser o gesto que dá cor ou máscara à
            // base pela primeira vez (o artista pintou só lá em cima), e desfazer
            // tem de devolver a malha ao estado de não pagar por esse canal.
            match &before.colors {
                Some(c) => mesh.colors_mut().copy_from_slice(c),
                None => {
                    mesh.take_colors();
                }
            }
            match &before.masks {
                Some(m) => mesh.masks_mut().copy_from_slice(m),
                None => {
                    mesh.take_masks();
                }
            }
            mesh.rebuild();
        }
        if let Some(d) = &stamped.details {
            self.details[up] = d.clone();
        }
        self.sel = up;
        true
    }

    /// **Destaca o nível do TOPO** e desce a seleção — o desfazer do
    /// [`Multires::add_level`]. Devolve o nível, ou `None` se não havia o que
    /// destacar.
    ///
    /// ⚠️ Só do topo, e só se houver mais de um: descartar do meio deixaria uma
    /// pilha cujos detalhes descrevem um nível que não existe mais.
    ///
    /// ⚠️ **Ele ENTREGA o que tira, em vez de deixar cair** — e é o que torna o
    /// refazer exato. A alternativa (redo = subdividir de novo) só acerta
    /// enquanto o nível de baixo estiver byte-a-byte como estava, e **`lower`
    /// escreve nele** (o carimbo): depois de descer uma vez COM trabalho, uma
    /// subdivisão recomputada não é a mesma malha. Entregar o nível não custa
    /// memória de PICO — ele é MOVIDO para fora e MOVIDO de volta, nunca
    /// clonado; o que muda é só quanto tempo ele vive.
    pub fn drop_top(&mut self) -> Option<DetachedLevel> {
        if self.levels.len() < 2 || self.sel + 1 != self.levels.len() {
            return None;
        }
        let mesh = self.levels.pop()?;
        let details = self.details.pop()?;
        self.sel -= 1;
        Some(DetachedLevel { mesh, details })
    }

    /// **Recoloca no topo** um nível destacado, e o seleciona — a inversa exata
    /// de [`Multires::drop_top`].
    ///
    /// ⚠️ Só do topo, pela mesma razão do [`Multires::add_level`]: um nível que
    /// entrasse no meio deixaria o detalhe de cima descrevendo outro pai.
    /// Devolve `false` (e **consome** o nível) se a pilha não estiver no topo —
    /// o chamador sobe primeiro, como já faz para descartar.
    pub fn push_level(&mut self, level: DetachedLevel) -> bool {
        if self.sel + 1 != self.levels.len() {
            return false;
        }
        self.levels.push(level.mesh);
        self.details.push(level.details);
        self.sel += 1;
        true
    }

    /// Vai para o nível `target`, subindo ou descendo o que for preciso.
    ///
    /// ⚠️ **Ela DESCARTA o carimbo de cada descida**, e é seguro porque descer
    /// sem ter esculpido não carimba nada — a diferença contra a síntese é
    /// exatamente zero. Quem oferece o gesto ao artista chama a [`Multires::lower`]
    /// e guarda o que ela devolve; esta é a rota de conveniência, para saltar a
    /// um nível quando o documento já está consistente.
    pub fn select(&mut self, target: usize) {
        while self.sel > target && self.lower().is_some() {}
        while self.sel < target && self.higher() {}
    }

    /// Bytes segurados pela pilha inteira — os detalhes, que a malha não conta.
    #[must_use]
    pub fn detail_bytes(&self) -> usize {
        self.details
            .iter()
            .map(|d| {
                d.xyz.capacity() * size_of::<[f32; 3]>()
                    + d.colors.as_ref().map_or(0, |c| c.capacity() * 12)
                    + d.masks.as_ref().map_or(0, |m| m.capacity() * 4)
            })
            .sum()
    }

    /// Carimba na base a DIFERENÇA entre o topo e a síntese dele — o que o
    /// artista moveu desde que chegou lá, e nada mais.
    fn stamp_down(&mut self, up: usize, down: usize, expected: &Synth) -> Stamped {
        let n = self.levels[down].vert_count();
        let (lo, hi) = self.levels.split_at_mut(up);
        let (dst, src) = (&mut lo[down], &hi[0]);

        // ⚠️ Um canal que a base ainda NÃO TEM está NASCENDO — o artista pintou
        // uma máscara só lá em cima, e não há síntese de que subtrair. Ele desce
        // COPIADO, e é o único caso em que a base recebe uma posição em vez de
        // uma diferença. Acontece uma vez por canal, na vida do documento.
        let newborn_colors = src.colors().is_some() && expected.colors.is_none();
        let newborn_masks = src.masks().is_some() && expected.masks.is_none();

        // ⚠️ **Abaixo do piso, o vértice não se moveu** — ver [`STAMP_FLOOR`]. O
        // piso mede contra a DIAGONAL do modelo, e é o que faz um passeio ocioso
        // carimbar exatamente nada em vez do ulp que a régua recomputada erra.
        let floor = STAMP_FLOOR * dst.bounds().longest_edge();
        let dpos: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                let d = sub(src.positions()[i], expected.positions[i]);
                if norm(d) <= floor { [0.0; 3] } else { d }
            })
            .collect();
        let dcol: Option<Vec<[f32; 3]>> = expected
            .colors
            .as_ref()
            .zip(src.colors())
            .map(|(e, s)| (0..n).map(|i| sub(s[i], e[i])).collect());
        let dmask: Option<Vec<f32>> = expected
            .masks
            .as_ref()
            .zip(src.masks())
            .map(|(e, s)| (0..n).map(|i| s[i] - e[i]).collect());

        // ⚠️ **Andar entre níveis sem esculpir não carimba NADA**, e o `None` que
        // sai daqui é o que torna a entrada de desfazer de um passeio ocioso
        // custar dezesseis bytes em vez de uma cópia da base.
        let quiet = !newborn_colors
            && !newborn_masks
            && dpos.iter().all(|d| *d == [0.0; 3])
            && dcol
                .as_ref()
                .is_none_or(|d| d.iter().all(|x| *x == [0.0; 3]))
            && dmask.as_ref().is_none_or(|d| d.iter().all(|x| *x == 0.0));
        if quiet {
            return Stamped {
                level: down,
                before: None,
                details: None,
            };
        }

        let before = SharedBefore {
            positions: dst.positions().to_vec(),
            colors: dst.colors().map(<[[f32; 3]]>::to_vec),
            masks: dst.masks().map(<[f32]>::to_vec),
        };

        for (p, d) in dst.positions_mut().iter_mut().zip(&dpos) {
            for k in 0..3 {
                p[k] += d[k];
            }
        }
        if newborn_colors {
            let s = src.colors().expect("o canal nasce COM o topo");
            dst.colors_mut().copy_from_slice(&s[..n]);
        } else if let Some(d) = &dcol {
            for (c, d) in dst.colors_mut().iter_mut().zip(d) {
                for k in 0..3 {
                    c[k] = (c[k] + d[k]).clamp(0.0, 1.0);
                }
            }
        }
        if newborn_masks {
            let s = src.masks().expect("o canal nasce COM o topo");
            dst.masks_mut().copy_from_slice(&s[..n]);
        } else if let Some(d) = &dmask {
            for (m, d) in dst.masks_mut().iter_mut().zip(d) {
                *m = (*m + d).clamp(0.0, 1.0);
            }
        }

        Stamped {
            level: down,
            before: Some(before),
            details: None,
        }
    }
}

/// **O que uma descida carimbou na base** — o suficiente para desfazê-la.
#[derive(Clone, Debug)]
pub struct Stamped {
    level: usize,
    before: Option<SharedBefore>,
    /// O detalhe do nível de cima como estava ANTES da descida.
    ///
    /// ⚠️ **Ele é guardado, e a primeira versão dizia que não precisava.** O
    /// raciocínio era que re-encodar contra a base restaurada devolve um detalhe
    /// que sintetiza o topo — verdade, e insuficiente: o re-encode faz o detalhe
    /// ABSORVER a escultura, e a partir daí ela deixa de ser *o que o artista
    /// moveu desde que chegou*. O sintoma, achado por gate: desfazer uma descida
    /// e descer de novo à mão não carimbava mais a escultura na base. Ele acompanha
    /// o `before` — as duas metades são `None` juntas, num passeio ocioso.
    details: Option<Details>,
}

impl Stamped {
    /// O nível que recebeu o carimbo.
    #[must_use]
    pub fn level(&self) -> usize {
        self.level
    }

    /// **A descida não moveu nada** — o passeio entre níveis de quem só foi
    /// OLHAR. Desfazê-la é subir, e mais nada.
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.before.is_none()
    }
}

/// Os canais compartilhados da base, como estavam antes do carimbo.
///
/// ⚠️ Valores, não diferenças: `(b + d) − d` não é `b` em ponto flutuante, e um
/// desfazer que erra um ulp por viagem é a escultura escorregando.
#[derive(Clone, Debug)]
struct SharedBefore {
    positions: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
}

/// O topo que `(base, detalhe)` determinam — a SÍNTESE.
struct Synth {
    positions: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
}

/// **O frame local de um vértice** — a porta única de que encode e decode são as
/// duas metades.
///
/// `normal` é a do vértice no nível de cima; `at` e `neighbour` são a posição
/// PREVISTA dele e a do primeiro vizinho do anel. Devolve `None` quando o frame
/// é degenerado (normal nula, ou o vizinho exatamente sobre a normal): sem eixos
/// não há como escrever o deslocamento, e o original também desiste.
#[must_use]
fn local_frame(
    normal: [f32; 3],
    at: [f32; 3],
    neighbour: [f32; 3],
) -> Option<([f32; 3], [f32; 3], [f32; 3])> {
    let n = normalize(normal)?;
    let mut t = [
        neighbour[0] - at[0],
        neighbour[1] - at[1],
        neighbour[2] - at[2],
    ];
    // Projeta a corda no plano da normal — é o que torna o frame ortonormal.
    let along = t[0] * n[0] + t[1] * n[1] + t[2] * n[2];
    for k in 0..3 {
        t[k] -= n[k] * along;
    }
    let t = normalize(t)?;
    let bi = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    Some((n, t, bi))
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len2 == 0.0 {
        return None;
    }
    let inv = 1.0 / len2.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}

/// A diferença entre onde o topo ESTÁ e onde a subdivisão o poria.
fn encode(up: &Mesh, predicted: &Predicted) -> Details {
    let n = up.vert_count();
    let adj = up.adjacency();
    let mut xyz = vec![[0.0f32; 3]; n];
    for (v, out) in xyz.iter_mut().enumerate().take(n) {
        let Some(&first) = adj.vert_verts.neighbours(v).first() else {
            continue;
        };
        let Some((nrm, tan, bi)) = local_frame(
            up.normals()[v],
            predicted.positions[v],
            predicted.positions[first as usize],
        ) else {
            continue;
        };
        let d = sub(up.positions()[v], predicted.positions[v]);
        *out = [dot(nrm, d), dot(tan, d), dot(bi, d)];
    }
    Details {
        xyz,
        colors: up
            .colors()
            .zip(predicted.colors.as_ref())
            .map(|(a, b)| (0..n).map(|i| sub(a[i], b[i])).collect()),
        masks: up
            .masks()
            .zip(predicted.masks.as_ref())
            .map(|(a, b)| (0..n).map(|i| a[i] - b[i]).collect()),
    }
}

/// **A previsão mais o detalhe, no MESMO frame** — o topo que `(base, detalhe)`
/// determinam, sem escrevê-lo.
///
/// ⚠️ **Uma porta, DOIS consumidores.** Subir INSTALA a síntese; descer a usa
/// como RÉGUA (*o que o artista moveu além dela?*). Escritas duas vezes, as duas
/// respostas divergem — e a divergência aparece como a base absorvendo um
/// carimbo que ninguém fez.
fn synthesize(up: &Mesh, details: &Details, predicted: &Predicted) -> Synth {
    let n = up.vert_count();
    // Os canais primeiro: eles não participam do frame.
    let colors = predicted
        .colors
        .as_ref()
        .zip(details.colors.as_ref())
        .map(|(src, d)| {
            (0..n)
                .map(|i| {
                    let mut o = [0.0f32; 3];
                    for k in 0..3 {
                        o[k] = (src[i][k] + d[i][k]).clamp(0.0, 1.0);
                    }
                    o
                })
                .collect()
        });
    let masks = predicted
        .masks
        .as_ref()
        .zip(details.masks.as_ref())
        .map(|(src, d)| (0..n).map(|i| (src[i] + d[i]).clamp(0.0, 1.0)).collect());

    // ⚠️ O frame lê a PREVISÃO (não as posições vivas, que ainda são as da
    // viagem anterior) e as NORMAIS do topo, que ninguém recomputou desde o
    // encode. As duas metades da inversa perguntam à mesma porta.
    let mut positions = vec![[0.0f32; 3]; n];
    let adj = up.adjacency();
    let normals = up.normals();
    for (v, out) in positions.iter_mut().enumerate() {
        *out = predicted.positions[v];
        let Some(&first) = adj.vert_verts.neighbours(v).first() else {
            continue;
        };
        let Some((nrm, tan, bi)) = local_frame(
            normals[v],
            predicted.positions[v],
            predicted.positions[first as usize],
        ) else {
            continue;
        };
        let d = details.xyz.get(v).copied().unwrap_or([0.0; 3]);
        for k in 0..3 {
            out[k] += nrm[k] * d[0] + tan[k] * d[1] + bi[k] * d[2];
        }
    }
    Synth {
        positions,
        colors,
        masks,
    }
}

/// Escreve a síntese no topo.
fn install(up: &mut Mesh, synth: &Synth) {
    if let Some(c) = &synth.colors {
        up.colors_mut().copy_from_slice(c);
    }
    if let Some(m) = &synth.masks {
        up.masks_mut().copy_from_slice(m);
    }
    up.positions_mut().copy_from_slice(&synth.positions);
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A reversão mora num filho porque mexe nos campos privados da pilha — ver o
/// cabeçalho dele.
#[path = "multires_reverse.rs"]
mod reverse;

pub use reverse::Reversal;

/// A metade que fala com um DOCUMENTO — filho porque `levels`/`details`/`sel`
/// são privados, e é assim que se quer (ver o cabeçalho dele).
#[path = "multires_persist.rs"]
mod persist;

#[cfg(test)]
#[path = "multires_tests.rs"]
mod tests;

fn norm(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
