//! **OS DOIS GESTOS QUE TROCAM A MALHA INTEIRA** — o voxel remesh e a
//! retopologia por campo cruzado.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é de ASSUNTO: lá moram os gestos
//! que editam a malha que existe (o traço, a máscara, a pilha de níveis); aqui
//! os dois que a **substituem**.
//!
//! ⚠️ **Eles são irmãos por natureza, e o ADR-0160 §1 é a tabela que os
//! separa:** o voxel remesh re-amostra um campo (a arrumação destrutiva, os
//! quads seguindo os eixos da grade, uma alça mais fina que o voxel some); a
//! retopologia **preserva a topologia** e alinha a grade às direções principais
//! da forma. Os dois ficam, e a UI os oferece separados — fundi-los num botão só
//! seria a pergunta *"que remesh?"* respondida por omissão.
//!
//! ⚠️ **E as duas entram na história pela MESMA entrada** (`StrokeUndo::Remeshed`,
//! que carrega a malha inteira de antes): um remesh não partilha estrutura
//! nenhuma com o que estava lá — nem a contagem de vértices, nem a
//! correspondência entre eles —, então não há representação mais barata.
//!
//! ⚠️ **O corte foi FORÇADO pelo teto de LOC do shell (HR-18, 600):** o pai
//! chegou a **651**. A cura de um teto é um corte para o IRMÃO, nunca uma
//! allowlist.

use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

/// **O QUE A RETOPOLOGIA FEZ** — o relatório que o log e o smoke lêem.
///
/// ⚠️ **O `edge` viaja no relatório, e não só o `detail` que o artista pediu.**
/// O knob é uma fração do curso, e o lado do quad sai da MALHA — então sem este
/// campo ninguém consegue relacionar o que se pediu com o que saiu, nem numa
/// sessão de smoke nem num bug daqui a um mês. *Um número que só existe dentro
/// da função não é comparável entre duas corridas.*
#[derive(Clone, Copy, Debug)]
pub(in crate::sculpt3d) struct QuadRemeshReport {
    /// Quantos vértices a malha nova tem.
    pub verts: usize,
    /// Quantas faces saíram com quatro lados.
    pub quads: usize,
    /// Quantas não saíram.
    pub non_quads: usize,
    /// O lado do quad que o `detail` pedido virou nesta malha.
    pub edge: f32,
    /// O relógio do passe.
    pub ms: f64,
    /// **Quantos buracos ficaram na casca** — `0` é o que se espera.
    pub holes: usize,
    /// ⭐ **Quantos vértices IRREGULARES** (valência ≠ 4) — a grandeza que o pivô
    /// do ADR-0162 existiu para derrubar, e a que o artista de facto vê.
    ///
    /// ⚠️ **É a CONTAGEM e não uma percentagem**, e a diferença é medida: a mesma
    /// peça com o dobro da densidade tem os mesmos irregulares e metade da
    /// percentagem. Uma esfera admite **oito**; o motor local entregava milhares.
    ///
    /// ⚠️ O backend LOCAL não a mede — ele reporta `usize::MAX` como *"não sei"*,
    /// que é diferente de zero e não pode ser lido como um resultado bom.
    pub irregular: usize,
    /// ⭐⭐ **A ARESTA MAIS LONGA, em múltiplos do `edge` pedido** — a primeira
    /// grandeza GEOMÉTRICA que este relatório carrega.
    ///
    /// ⛔ **Todos os outros campos sobrevivem a posições embaralhadas.** Foi assim
    /// que o botão devolveu uma malha destruída com `100 % quads · casca fechada ·
    /// 22 irregulares` e 10.515 gates verdes (auditoria de 2026-08-21). Medido: o
    /// caminho correcto fica **≤ 4×**; o destruído deu **18×**, que era o diâmetro
    /// da peça.
    ///
    /// ⚠️ `f32::NAN` quando o backend não a mede — que é diferente de `0`.
    pub edge_max_ratio: f32,
    /// A aresta mediana em múltiplos do `edge` pedido — ⭐ esta diz se a DENSIDADE
    /// saiu no alvo. Medido: correcto ≈ **0,9×**; destruído **4,6×**.
    pub edge_median_ratio: f32,
    /// ⭐⭐ **A ARESTA MAIS LONGA em FRAÇÃO DA PEÇA** (da diagonal da caixa).
    ///
    /// ⚠️ **Ela existe porque a razão-ao-alvo NÃO é a grandeza que a asserção
    /// afirma.** O `assert` diz *"alguma coisa na malha atravessa a peça"*, e isso é
    /// um fato **absoluto**: uma aresta de metade do modelo é catastrófica quer o
    /// alvo seja grosso ou fino. Com a razão, a mesma barra aperta sozinha a cada
    /// passo do slider — medido em 2026-08-21, na mesma malha e sem defeito nenhum:
    ///
    /// | alvo | quads | `edge_max / alvo` | **`edge_max / diagonal`** |
    /// |---|---|---|---|
    /// | `3,00×` a aresta de entrada | 1 336 | 2,71× | **7,2 %** |
    /// | `1,50×` | 4 885 | 4,82× | **6,4 %** |
    /// | `0,75×` | 20 039 | 7,71× | **5,1 %** |
    /// | `0,54×` | 38 315 | 9,48× | **4,5 %** |
    ///
    /// ⭐ **A razão triplica e a fração até MELHORA** — porque não há defeito
    /// nenhum: o que muda é o denominador. E o defeito real que a barra existe para
    /// apanhar (a geometria montada sobre índices de outra malha, 2026-08-21) media
    /// **2,01 numa peça de diagonal 3,46 = 58 %**, contra os 5 a 7 % do caminho
    /// correcto. *Onze vezes de margem, e a barra deixa de apertar sozinha.*
    pub edge_max_span: f32,
    /// ⭐⭐⭐ **A FORMA DE CADA QUAD** — ver [`ph2d_quadfill::QuadShape`].
    ///
    /// ⛔ **Ela entrou em 2026-08-22, e a razão está numa foto.** Nesse dia a
    /// [`Self::edge_max_span`] da orelha caiu de `57 %` da peça para `5,5 %`, o
    /// relatório ficou verde em toda a coluna — e a foto seguinte veio com a palavra
    /// **«péssimo»**. *Todas as réguas geométricas deste struct são GLOBAIS*, e o
    /// defeito é por-face: quads esmagados e enviesados em faixas, numa malha cujos
    /// extremos estão bem.
    ///
    /// ⚠️ **O ENVIESAMENTO é a coluna que faltava**, e é a única que separa a nossa
    /// saída da do oráculo quando o aspecto já está quase certo: na `wrinkled_sphere`
    /// o aspecto p50 é `1,28` contra `1,08` dele — quase igual — e o enviesamento p50
    /// é `18°` contra `5°`. *Um losango tem as quatro arestas iguais.*
    pub shape: ph2d_quadfill::QuadShape,
    /// ⭐⭐ **QUANTAS FACES DOBRARAM contra a peça original** — a medida da fenda
    /// escura que o artista fotografa. Ver [`ph2d_quadfill::folded_against`].
    ///
    /// ⚠️ **Os DOIS backends a medem pela MESMA régua**, e é isso que os torna
    /// comparáveis. Enquanto a única contagem foi o teste radial de uma sonda, a
    /// esfera com bico — que não é um sólido estrelado — acusava os dois motores
    /// de dobrar sem que nenhum dos dois tivesse dobrado.
    pub folded: usize,
    /// ⭐⭐⭐ **ALMOFADAS que a extracção deitou fora** — ver
    /// [`ph2d_quadextract::ExtractReport::mirrored_cells`]. Uma dobra do mapa cobre a mesma
    /// região duas vezes com orientações opostas e o par sai como **duas faces coincidentes
    /// e soltas** — foi a foto do artista de 2026-08-28. ⚠️ *Ela é invisível a toda outra
    /// coluna desta linha*: `χ` conta os dois lados de uma almofada e dá `2`, o bordo é
    /// zero, o não-manifold é zero.
    pub mirrored: usize,
    /// ⭐⭐⭐ **DOUBLETS dissolvidos** — ver [`ph2d_quadextract::ExtractReport::doublets`]. Um
    /// vértice preso entre duas faces é a **mordida** que o artista fotografou nas pontas, e
    /// ela realimenta-se: a saída com doublets, ao voltar a entrar, parte a fase zero.
    pub doublets: usize,

    /// ⭐⭐⭐ **QUANTAS PONTAS A CADEIA CORTOU** — ver [`ph2d_quadfill::tip_survival`].
    ///
    /// ⛔⛔ **Ela existe por uma foto com uma seta VERDE e uma VERMELHA na mesma peça**
    /// (Enio, 2026-08-30): *«algumas pontas boas, algumas ruins, amputadas»*. O alcance
    /// global é um **extremo único** e não podia dizer isso — na peça dele ele lia
    /// `−16,2 %` enquanto **dez** das doze pontas estavam a `−0,1 %` e **duas** tinham
    /// perdido `20 %`.
    ///
    /// ⚠️ **E o artista não tinha como saber.** A amputação é a célula da grade a ser
    /// mais grossa que a ponta — *resolução, não defeito* —, e o `Detail` que a cura não
    /// se anuncia. ⇒ **o botão passa a dizê-lo**, que é a diferença entre uma ferramenta
    /// que falha em silêncio e uma que explica.
    pub tips_cut: usize,
    /// Quantas pontas foram medidas — o **denominador**, sem o qual `tips_cut` não é
    /// interpretável.
    pub tips_total: usize,
    /// A pior perda, em percentagem (negativa).
    pub tips_worst_pct: f32,
    /// ⭐⭐⭐ **QUANTO DA ESCULTURA FICOU POR COBRIR, na casca exterior** — a mediana, em
    /// fracção da diagonal da peça. Ver [`ph2d_quadfill::coverage`].
    ///
    /// ⛔⛔ **É a direcção que ninguém mede**, e a ausência está confirmada nos DOIS lados:
    /// nem as réguas desta linha nem as do padrão-ouro medem distância à ENTRADA. *Uma ponta
    /// comida sai fechada, com quads bonitos, e passa em tudo.*
    ///
    /// ⚠️ **Ela é mais geral que a [`Self::tips_cut`]:** aquela tem de ACHAR um ápice primeiro
    /// (máximo local do raio) e por isso só vê o que se parece com uma ponta; esta responde
    /// *«a escultura toda foi coberta?»* sem saber o que é uma ponta. Medido na peça do
    /// artista: `6,02 %` no `Detail` de fábrica contra `0,28 %` a `0,85` + `Follow Curvature`.
    pub coverage_shell_p50: f32,
    /// A pior falta na casca — o mesmo denominador.
    pub coverage_shell_worst: f32,
    /// Vértices medidos. ⛔⛔ **`0` = NÃO MEDIDO**, nunca «perfeito» — ver
    /// [`ph2d_quadfill::Coverage::samples`].
    pub coverage_samples: usize,
    /// ⭐⭐ **O campo desta corrida obedeceu ao RELEVO?**
    ///
    /// ⛔ **Ele existe porque a cadeia global tem uma REDE**, e uma rede silenciosa
    /// é indistinguível de uma feature que regrediu: quando o layout do campo
    /// alinhado não fecha, a porta volta a correr o campo só-suavidade e devolve uma
    /// malha perfeitamente boa por todas as outras réguas — 100 % de quads, casca
    /// fechada, contagem de irregulares na mesma ordem. *Sem este campo, «o
    /// alinhamento deixou de funcionar» leria exactamente como «funcionou».*
    ///
    /// ⚠️ **`false` no backend LOCAL**, e não é *"não sei"*: aquele motor não tem
    /// campo cruzado com termo de alinhamento nenhum, então a resposta é um facto.
    pub aligned: bool,
    /// ⭐⭐⭐ **A escolha foi por MEDIÇÃO, e não por recusa.**
    ///
    /// ⛔⛔ **Sem esta coluna o [`Self::aligned`] tem dois sentidos e o log MENTE.** Desde
    /// 2026-08-26 a cadeia da extracção corre **as duas** tentativas (alinhada e lisa) e fica
    /// com a melhor — furos, depois faces `>60°`, depois o enviesamento mediano. O irmão dela
    /// só cai para a lisa quando a alinhada **RECUSA**.
    ///
    /// ⇒ com `aligned == false`, *«o alinhado não fechou»* e *«o liso saiu melhor»* são
    /// **factos diferentes** e leriam-se igual. ⚠️ *Um log que descreve a recusa quando houve
    /// uma escolha manda o leitor procurar um defeito que não existe.*
    pub measured: bool,
}

impl Sculpt3dScene {
    /// lugar errado.
    /// **RECONSTRÓI a malha por voxelização** — o botão do W7.
    ///
    /// Recusa com a pilha de multires montada, e essa recusa é da MESMA família
    /// da de tapar buraco: um remesh troca a topologia inteira, e todo nível
    /// acima é `subdivide` da base — o detalhe deles passaria a descrever uma
    /// malha que não existe mais. ⚠️ **A alternativa seria achatar a pilha em
    /// silêncio**, e isso é destruir trabalho que o artista autorou sem dizer; o
    /// log nomeia a recusa e o conserto.
    ///
    /// ⚠️ **E ela devolve o MOTIVO, não um `None`.** Havia três causas de recusa
    /// entrando num `Option` só, e o chamador tinha de escolher UMA mensagem
    /// para todas — escolheu a da pilha, então um campo que vazou mandava o
    /// artista *"reverter os níveis"* que ele não tem. Uma recusa que nomeia a
    /// causa errada é pior que uma recusa muda: ela dirige o conserto para o
    pub(in crate::sculpt3d) fn remesh(
        &mut self,
        resolution: u32,
    ) -> Result<ph2d_sdf::RemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let (out, report) =
            ph2d_sdf::remesh(self.mesh(), resolution).map_err(RemeshRefusal::Engine)?;
        let previous = core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais,
        // e os buffers do device mudaram de tamanho.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(report)
    }

    /// **A RETOPOLOGIA por campo cruzado** (ADR-0160) — a grade corre AO LONGO
    /// da forma. Devolve o [`QuadRemeshReport`].
    ///
    /// ⚠️ **Irmã do [`Self::remesh`] e NÃO substituta**, e as duas ficam porque
    /// respondem a perguntas diferentes: o voxel remesh re-amostra um campo (a
    /// arrumação destrutiva depois de uma booleana, os quads seguindo os eixos da
    /// grade), esta **preserva a topologia** da entrada e alinha a grade às
    /// direções principais da forma. O ADR-0160 §1 traz a tabela.
    ///
    /// ⚠️ **A mesma recusa de PILHA do irmão**, e pelo mesmo motivo: a saída é
    /// uma malha com outra contagem de vértices, e um nível de multires é uma
    /// subdivisão da base — as duas coisas não coexistem.
    ///
    /// ⚠️ **E ela entra na história pela MESMA entrada** (`StrokeUndo::Remeshed`,
    /// que carrega a malha inteira de antes). Não há representação mais barata: um
    /// remesh não partilha estrutura nenhuma com o que estava lá, nem a contagem
    /// de vértices nem a correspondência entre eles.
    pub(in crate::sculpt3d) fn quad_remesh(
        &mut self,
        detail: f32,
        adaptive: f32,
    ) -> Result<QuadRemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let t = std::time::Instant::now();
        let mesh = self.mesh();
        // ⚠️ **O LADO DO QUAD SAI DA MALHA, e não do slider** — ver
        // `ph2d_quadflow::edge_for_detail`. Um tamanho absoluto vindo do painel é
        // destrutivo numa malha grossa e conservador numa fina, e foi o defeito
        // que o smoke do Enio fotografou (2026-08-19).
        // ⭐⭐⭐ **A MESMA CONTAGEM DOS IRMÃOS, e depois a CERCA deste motor.**
        //
        // ⛔ **Um slider, um significado:** o `Detail` é a mesma linha do painel para os
        // dois motores, então ele pede a mesma contagem nos dois — ver
        // [`ph2d_quadflow::MAX_QUADS`], e a medição dos três apertos que a motivou.
        //
        // ⚠️ **A cerca deste motor CONTINUA e é dele:** a extração por retícula rasga
        // quando o quad é mais fino que o triângulo de entrada (a foto de 2026-08-19, um
        // ciclo de 352 lados com 58 % do volume perdido), e é isso que
        // [`ph2d_quadflow::resolvable_edge_range`] declara. ⇒ a contagem entra e a faixa
        // **apara**. *Quando a cerca morde, este motor deixa de ser idempotente — e é a
        // cerca dele que o diz, não o slider.*
        let (floor, ceiling) = ph2d_quadflow::resolvable_edge_range(mesh);
        let edge = ph2d_quadflow::edge_for_detail_by_count(mesh, detail).clamp(floor, ceiling);
        let scale = ph2d_quadflow::ScaleField::adaptive(mesh, edge, adaptive);
        let (orient, pos) = ph2d_quadflow::solve_fields(mesh, &scale);
        // ⚠️ **NÃO HÁ passe de relaxação aqui, e a ausência é MEDIDA.** Eu
        // construí um (Laplaciano tangente + reprojeção) enquanto a extração
        // ainda era invenção minha, e ele comprava pouco. Com a extração PORTADA
        // da referência ele passou a **piorar as três fixturas em todas as
        // grandezas** — o Hausdorff da malha da cena vai de 0,60 para 1,49 quad
        // com 16 passadas. A grade que sai do campo cruzado já está alinhada; um
        // Laplaciano por cima briga com ela. Ver ADR-0160 §5-octies.
        let q = ph2d_quadflow::extract(mesh, &orient, &pos, &scale).map_err(RemeshRefusal::Quad)?;
        // ⚠️ **A recusa é NOMEADA, e não uma malha vazia.** Com o `detail` toda
        // corrida cai dentro da faixa legal, então este braço é uma rede — mas a
        // diferença entre *"a peça sumiu"* e *"ele disse por quê"* é a razão de
        // ele existir, e o custo é uma comparação.
        if q.mesh.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }
        let out = QuadRemeshReport {
            verts: q.mesh.vert_count(),
            quads: q.quads,
            non_quads: q.non_quads,
            edge,
            ms: t.elapsed().as_secs_f64() * 1000.0,
            holes: q.holes,
            // ⚠️ **`MAX` é *"não sei"*, e é a resposta honesta.** Este backend não
            // conta valências; escrever `0` seria afirmar uma grade perfeita
            // sobre um motor que entrega 21 a 49 % de irregulares.
            irregular: usize::MAX,
            // ⚠️ `NAN` é *"não sei"*, e o log escreve `?`. Este backend não mede
            // as arestas da saída, e escrever `1,0` seria afirmar uma grade
            // perfeita.
            edge_max_ratio: f32::NAN,
            edge_median_ratio: f32::NAN,
            edge_max_span: f32::NAN,
            shape: ph2d_quadfill::QuadShape::default(),
            // ⚠️ **`false` é um FACTO aqui**, não um *"não sei"*: este motor não tem
            // campo cruzado nenhum, logo não tem termo de alinhamento para ligar.
            aligned: false,
            measured: false,
            // ⭐ **A MESMA régua do outro backend, e é por isso que ela é medida
            // aqui e não estimada.** Sem esta linha o motor local aparecia como
            // *"não dobra"* por não ter quem contasse — que é o mesmo defeito do
            // `irregular: MAX` acima, só que silencioso.
            folded: ph2d_quadfill::folded_against(mesh, &q.mesh),
            // ⚠️ **`0` é um FACTO aqui**: este motor não extrai de mapa nenhum, então não há
            // dobra de mapa que possa gerar uma almofada.
            mirrored: 0,
            doublets: 0,
            // ⚠️ Este backend não mede pontas — ver o doc do campo.
            tips_cut: 0,
            tips_total: 0,
            tips_worst_pct: 0.0,
            // ⚠️ `0` = NÃO MEDIDO — ver `QuadRemeshReport::coverage_samples`.
            coverage_shell_p50: 0.0,
            coverage_shell_worst: 0.0,
            coverage_samples: 0,
        };
        let previous =
            core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, q.mesh);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(out)
    }
}
