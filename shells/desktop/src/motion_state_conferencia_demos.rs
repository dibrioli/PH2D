//! **As cenas da CONFERÊNCIA DOS NÓS** (doc 89) — `PH2D_GPU_COOK_DEMO=32..35`.
//!
//! As sete waves que fecharam P0 do doc 89 são quase todas **params de nós que já
//! existiam**, e um param não se julga sozinho. Estas quatro cenas cobrem as
//! metades que **só o olho decide**; as outras três (o blend por-elemento do
//! `mixer`, a variante do `duplicator`, o predicado do `cull`) são fechadas por
//! gates que medem o conjunto EXATO, e uma cena para elas seria uma pergunta que
//! o gate já responde melhor.
//!
//! ⚠️ **Duas delas pedem um GESTO, não uma foto.** O write-on do `spline_wrap` é
//! uma animação e o pivô é uma comparação: a cena monta o grafo no estado em que
//! arrastar UM slider é a demonstração, que é o que separa *"a feature existe"* de
//! *"a feature está ao alcance da mão"*.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

/// Espalha os nós numa linha legível no editor.
fn lay(g: &mut ph2d_nodegraph::graph::Graph, row: f32, nodes: &[NodeId]) {
    use ph2d_nodegraph::graph::Pos;
    for (i, n) in nodes.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "poucos nós")]
        let x = 80.0 + i as f32 * 200.0;
        g.set_pos(*n, Pos { x, y: row });
    }
}

fn wire(g: &mut ph2d_nodegraph::graph::Graph, from: NodeId, to: NodeId, port: u16) -> Option<()> {
    use ph2d_nodegraph::graph::Edge;
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed: false,
    })
    .ok()
}

/// Um `motion.tint` sólido, para as duas metades de uma comparação se separarem.
fn tint(g: &mut ph2d_nodegraph::graph::Graph, rgb: [f32; 3]) -> NodeId {
    let t = g.add_node("motion.tint");
    g.set_param(t, "mode", 0.0);
    g.set_param(t, "r", rgb[0]);
    g.set_param(t, "g", rgb[1]);
    g.set_param(t, "b", rgb[2]);
    g.set_param(t, "a", 1.0);
    t
}

/// **`=32` — A CURVA REVELA, E A GEOMETRIA SEGUE** (folha 04, os dois P0).
///
/// Uma FITA de quadrados embrulhada num S. Com `follow_rotation` ligado cada um
/// **gira com a tangente** (um quadrado a 45° lê como losango, então a rotação é
/// visível sem carregar sprite nenhum); a fita de baixo é o **CONTROLE**, com o
/// toggle desligado — os quadrados ficam todos alinhados ao eixo enquanto a
/// posição segue a mesma curva.
///
/// ⚠️ **O `x` do layout percorre o arco; o `y` é o que a curva desloca pela
/// NORMAL** — e é por isso que a fita tem três linhas. O `Height` multiplica esse
/// `y`, então uma FILA o deixa inerte por multiplicação (`p.y = 0`), que foi o que
/// o Enio encontrou: *"height serve para que? pois não vejo mudar nada"*. Medido,
/// a espessura da fita é linear no knob: **0,700 / 0,350 / 0,000** para Height
/// 1 / 0,5 / 0 (= `2 · gap_y · height`).
///
/// ⚠️ **O write-on é um GESTO:** `from`/`to` são params, e nenhum nó do grafo
/// anima um param — então a cena não pode *mostrar* a revelação, ela põe o
/// artista em posição de a fazer. Arrastar o **To** de 0 a 1 no painel abre a fila
/// ao longo da curva; é o knob mais usado do Spline Wrap da referência, e é a
/// razão de a folha marcar isto P0.
///
/// ⚠️ **E a curva pode ser a que o artista DESENHOU.** O smoke de 2026-08-12
/// reprovou o MODELO do nó, não um número dele (*"pontos e alças em sliders num
/// painel. Absurdo!"*), e a row **Shape** no topo do painel é a resposta: escolha
/// uma forma que você desenhou com as ferramentas do Vector e a fila embrulha
/// nela — os oito sliders de polígono de controle **somem**, porque deixam de ser
/// lidos. A cena abre nos oito params de propósito: ela é o CONTROLE, o estado a
/// partir do qual o gesto se faz.
pub(crate) fn build_write_on_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;

    // Uma FITA (três linhas, muitas colunas). ⚠️ **O `x` é o que percorre o arco e
    // o `y` é o que a curva desloca pela NORMAL**, então um layout de UMA linha
    // deixa o `Height` inerte por multiplicação — `p.y` é zero e zero vezes
    // qualquer coisa é zero. A primeira versão desta cena era uma fila, e o Enio
    // perguntou, com razão, para que serve o Height: *"não vejo mudar nada"*. Três
    // linhas dão à curva o que deslocar, e o knob passa a ter o que fazer.
    let mut chain = |follow: f32, py: f32, rgb: [f32; 3], row: f32| -> Option<Vec<NodeId>> {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 3.0);
        g.set_param(grid, "cols", 16.0);
        g.set_param(grid, "gap_x", 0.12);
        g.set_param(grid, "gap_y", 0.35);
        let scale = g.add_node("motion.scale");
        // ⚠️ MEDIDO, e a primeira versão reprovou aqui: com 48 quadrados de 0,09
        // o passo ao longo do arco era 0,143 ⇒ razão tamanho/passo **0,63**, os
        // vizinhos quase se tocando, e a fila lia como uma FITA contínua em vez
        // de dezesseis quadrados com orientações diferentes. A rotação estava lá
        // o tempo todo (`basis` com 55,6° de leque, contra 0,0 do controle) — o
        // que faltava era poder VÊ-LA. Dezesseis a 0,22 dão passo 0,449 e razão
        // **0,49**: vão entre vizinhos, e cada peça com quase o dobro da área da
        // cena `=35`, que é a que o smoke aprovou.
        g.set_param(scale, "amount", 0.22);
        let sw = g.add_node("motion.spline_wrap");
        g.set_param(sw, "follow_rotation", follow);
        // Um S pronunciado: a tangente varre um bom ângulo do começo ao fim, que é
        // o que torna o `follow_rotation` legível em vez de sutil.
        for (k, v) in [
            ("p0x", -3.0),
            ("p0y", py - 1.1),
            ("p1x", -1.0),
            ("p1y", py + 1.4),
            ("p2x", 1.0),
            ("p2y", py - 1.4),
            ("p3x", 3.0),
            ("p3y", py + 1.1),
        ] {
            g.set_param(sw, k, v);
        }
        let t = tint(g, rgb);
        lay(g, row, &[grid, scale, sw, t]);
        wire(g, grid, scale, 0)?;
        wire(g, scale, sw, 0)?;
        wire(g, sw, t, 0)?;
        Some(vec![grid, scale, sw, t])
    };

    let follows = chain(1.0, 1.6, [0.30, 0.85, 0.95], 120.0)?;
    let control = chain(0.0, -1.6, [0.45, 0.45, 0.50], 340.0)?;

    let comb = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    lay(g, 560.0, &[comb, out]);
    wire(g, *follows.last()?, comb, 0)?;
    wire(g, *control.last()?, comb, 1)?;
    wire(g, comb, out, 0)?;

    doc.graph.validate(reg).ok()?;
    // ⚠️ O retorno é a lista de SINKS, não de nós: devolver a cadeia inteira faria
    // o editor tratar cada nó como uma saída.
    Some(vec![out])
}

/// **`=33` — O PIVÔ DA ESCALA DE LAYOUT** (folha 05, o P0).
///
/// Duas grades **idênticas**, as duas centradas longe da origem, as duas com
/// `scale = 2`. A de cima pivota na **origem do mundo** — o que o nó sempre fez —
/// e por isso a escala também a **TRANSLADA** para longe; a de baixo pivota no
/// **centroide** e apenas se **espalha onde está**.
///
/// ⚠️ É a cena descrita pela própria célula (*"um grid centrado em (5,0) com
/// `scale=2` também translada para (10,0)"*), e ela existe porque o sintoma é
/// **espacial**: um gate mede as coordenadas e diz o número certo, mas *o layout
/// fugiu de onde eu o pus* é uma frase sobre o que se vê.
pub(crate) fn build_pivot_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;

    let mut chain = |mode: f32, oy: f32, rgb: [f32; 3], row: f32| -> Option<Vec<NodeId>> {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 6.0);
        g.set_param(grid, "cols", 6.0);
        g.set_param(grid, "gap_x", 0.16);
        g.set_param(grid, "gap_y", 0.16);
        let scale = g.add_node("motion.scale");
        g.set_param(scale, "amount", 0.1);
        // Desloca o layout para LONGE da origem: com o pivô na origem é este
        // deslocamento que a escala multiplica, e é isso que se vê.
        let place = g.add_node("motion.transform");
        g.set_param(place, "offset_x", 2.2);
        g.set_param(place, "offset_y", oy);
        let grow = g.add_node("motion.transform");
        g.set_param(grow, "scale", 2.0);
        g.set_param(grow, "pivot_mode", mode);
        let t = tint(g, rgb);
        lay(g, row, &[grid, scale, place, grow, t]);
        wire(g, grid, scale, 0)?;
        wire(g, scale, place, 0)?;
        wire(g, place, grow, 0)?;
        wire(g, grow, t, 0)?;
        Some(vec![grid, scale, place, grow, t])
    };

    // 0 = World Origin (o nó que shipava) · 2 = Centroid (o modo novo).
    let origin = chain(0.0, 1.4, [0.95, 0.45, 0.20], 120.0)?;
    let centroid = chain(2.0, -1.4, [0.30, 0.85, 0.55], 340.0)?;

    let comb = g.add_node("motion.combine");
    let out = g.add_node("motion.output");
    lay(g, 560.0, &[comb, out]);
    wire(g, *origin.last()?, comb, 0)?;
    wire(g, *centroid.last()?, comb, 1)?;
    wire(g, comb, out, 0)?;

    doc.graph.validate(reg).ok()?;
    Some(vec![out])
}

/// **`=34` — QUALQUER FÓRMULA É UMA FORÇA** (folha 08, o P0 do §0).
///
/// Nenhuma `force.*` nesta cena. Quatro fórmulas de texto e dois
/// `motion.make_point` põem a nuvem em **ÓRBITA**: uma escreve a velocidade
/// inicial, a outra a aceleração, o `motion.integrate` faz o resto.
///
/// ⚠️ **A primeira versão desta cena estava ERRADA e o smoke a pegou** (*"a nuvem
/// gira e os pontos se afastam uns dos outros"*). Ela usava `a = k·perp(P)`,
/// descrito como *"um campo rotacional puro"* — e não é: em forma complexa isso é
/// `z'' = i·k·z`, cujas raízes são `±√k·e^{iπ/4}`, **com parte real positiva**.
/// É uma espiral que cresce como `e^{√(k/2)·t}` — a nuvem dobra de raio a cada
/// 0,62 s, e o que o Enio viu foi exactamente isso. *Uma cena que se destrói não
/// demonstra nada.*
///
/// **O campo que ORBITA precisa de duas metades, e nenhuma delas sozinha basta:**
/// a aceleração **centrípeta** `a = −ω²·P` (que sozinha, a partir do repouso,
/// colapsa tudo pela origem) e a velocidade inicial **tangencial** `v₀ = ω·perp(P)`
/// (que sozinha manda todo mundo embora em linha reta). Juntas, a solução é
/// `P(t) = P₀·cos(ωt) + perp(P₀)·sen(ωt)` — uma **rotação rígida exacta**, com o
/// raio de cada ponto constante para sempre. E o Euler semi-implícito do
/// integrador é **simplético**, então a órbita não deriva com o tempo.
///
/// ⚠️ **A semente entra pelo stream `rest`, e é o próprio integrador que diz
/// isso:** ele copia `vel` do `rest` no tique em que ainda não conhece o
/// elemento — *"a velocidade de boca do emissor é o que o lança"*. Não há
/// primitivo novo aqui; há a rota que já existia, usada pelo que ela é.
///
/// ⚠️ **Ela exercita TRÊS waves de uma vez, e honestamente:** as lanes `x`/`y` da
/// `motion.expression` (sem elas a fórmula lia a posição como **zero**, em
/// silêncio, e a nuvem ficaria parada) e os **dois** alvos novos do
/// `make_point` — `Velocity` na semente, `Acceleration` na força. Se a nuvem
/// girar **sem se abrir**, as três estão vivas.
///
/// ⚠️ **A aresta de realimentação é escrita à MÃO** — o editor a plumba ao soltar
/// um nó, e um documento montado por `add_node` não a ganha.
pub(crate) fn build_formula_force_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    use ph2d_nodegraph::graph::Edge;
    let g = &mut doc.graph;

    // ω = 2,5 rad/s ⇒ uma volta a cada 2,51 s. O número aparece em DOIS sítios
    // (ω na semente, ω² na aceleração) porque são duas grandezas diferentes da
    // mesma órbita; o gate afirma a propriedade que uma discordância quebra —
    // *todo ponto guarda o seu raio*.
    let (omega, omega_sq) = ("2.5", "6.25");

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 64.0);
    g.set_param(grid, "cols", 64.0);
    g.set_param(grid, "gap_x", 0.09);
    g.set_param(grid, "gap_y", 0.09);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.05);
    let t = tint(g, [0.35, 0.75, 0.95]);
    let ig = g.add_node("motion.integrate");
    let out = g.add_node("motion.output");

    // A SEMENTE — `v₀ = ω · perp(P)`, lida das posições de repouso.
    let sx = g.add_node("motion.expression");
    g.set_text_param(sx, "expr", format!("0 - y * {omega}"));
    let sy = g.add_node("motion.expression");
    g.set_text_param(sy, "expr", format!("x * {omega}"));
    let seed = g.add_node("motion.make_point");
    g.set_param(seed, "target", 1.0); // Velocity

    // A FORÇA — `a = −ω² · P`, lida do estado do tique anterior.
    let ax = g.add_node("motion.expression");
    g.set_text_param(ax, "expr", format!("0 - x * {omega_sq}"));
    let ay = g.add_node("motion.expression");
    g.set_text_param(ay, "expr", format!("0 - y * {omega_sq}"));
    let mp = g.add_node("motion.make_point");
    g.set_param(mp, "target", 2.0); // Acceleration

    lay(g, 120.0, &[grid, scale, t, seed, ig, out]);
    lay(g, 340.0, &[sx, sy]);
    lay(g, 560.0, &[ax, ay, mp]);

    wire(g, grid, scale, 0)?;
    wire(g, scale, t, 0)?;
    // A semente roda no ramo VIVO (arestas para a frente): ela descreve a
    // condição inicial, não o estado.
    wire(g, t, sx, 0)?;
    wire(g, t, sy, 0)?;
    wire(g, t, seed, 0)?;
    wire(g, sx, seed, 1)?;
    wire(g, sy, seed, 2)?;
    wire(g, seed, ig, 0)?; // `rest`

    // A realimentação que o artista nunca desenha: o estado do tique anterior.
    for (to, port) in [(ax, 0u16), (ay, 0), (mp, 0)] {
        g.connect(Edge {
            from: (ig, 0),
            to: (to, port),
            delayed: true,
        })
        .ok()?;
    }
    wire(g, ax, mp, 1)?;
    wire(g, ay, mp, 2)?;
    wire(g, mp, ig, 1)?; // `forces`
    wire(g, ig, out, 0)?;

    doc.graph.validate(reg).ok()?;
    Some(vec![out])
}

/// **`=35` — QUEM MIRA, E QUANTO** (folha 08, o P0 do `motion.look_at`).
///
/// Uma grade de quadrados que miram um alvo. Um `field.box` estreito escreve o
/// `falloff`, então **só a faixa do meio mira** — e mira **por inteiro** nos
/// texels de peso 1 e **pela metade** nas bordas macias do campo, que é o
/// contrato de família de que este nó era a exceção.
///
/// ⚠️ **Um quadrado a 45° lê como losango**, então a rotação é visível sem sprite
/// nenhum, e o gradiente da borda do campo aparece como uma **torção progressiva**
/// entre a faixa que mira e a que não mira. Um gate mede o ângulo; o que ele não
/// sabe dizer é se a transição parece contínua.
pub(crate) fn build_partial_aim_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;

    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.2);
    g.set_param(grid, "gap_y", 0.2);
    let scale = g.add_node("motion.scale");
    g.set_param(scale, "amount", 0.12);
    // Uma faixa HORIZONTAL larga e baixa, de borda macia: quem cai fora dela tem
    // `falloff = 0` e fica exactamente como estava.
    let field = g.add_node("field.box");
    g.set_param(field, "width", 6.0);
    g.set_param(field, "height", 1.1);
    g.set_param(field, "soft", 0.9);
    let look = g.add_node("motion.look_at");
    g.set_param(look, "strength", 1.0);
    let t = tint(g, [0.95, 0.80, 0.25]);
    let out = g.add_node("motion.output");

    lay(g, 120.0, &[grid, scale, field, look, t, out]);
    wire(g, grid, scale, 0)?;
    wire(g, scale, field, 0)?;
    wire(g, field, look, 0)?;
    wire(g, look, t, 0)?;
    wire(g, t, out, 0)?;

    doc.graph.validate(reg).ok()?;
    Some(vec![out])
}

/// **=36 — O MODO DE COMPOSIÇÃO É DO SINK** (doc 89, folha 17).
///
/// Três `motion.output` sobre a MESMA nuvem sobreposta, cada um num modo: o da
/// esquerda `Normal`, o do meio `Add`, o da direita `Multiply`. A shell compõe
/// vários sinks numa lista só, então os três desenham no mesmo quadro e a
/// pergunta que a cena faz é a única que um gate não sabe fazer: **os três
/// parecem três coisas diferentes?**
///
/// ⚠️ **A sobreposição é a cena.** Blend é o que acontece quando tinta cai sobre
/// tinta — num campo de quads disjuntos `Add` e `Normal` desenham o mesmo pixel
/// e a foto não separa os dois. Daí `gap` menor que o `amount`: cada cópia cobre
/// a vizinha, e é NA sobreposição que `Add` clareia e `Multiply` escurece.
pub(crate) fn build_sink_blend_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    // ⚠️ Meia-opacidade: em alfa 1,0 o `Normal` de cima TAPA o de baixo e os três
    // modos ficariam indistinguíveis — um `Add` só tem o que somar se houver o
    // que atravessar.
    for (k, (tag, rgb)) in [
        (0.0_f32, [0.85, 0.30, 0.30_f32]), // Normal
        (1.0, [0.30, 0.55, 0.85]),         // Add
        (3.0, [0.85, 0.75, 0.35]),         // Multiply
    ]
    .into_iter()
    .enumerate()
    {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 5.0);
        g.set_param(grid, "cols", 5.0);
        // Passo MENOR que o tamanho de uma cópia ⇒ elas se cobrem.
        g.set_param(grid, "gap_x", 0.30);
        g.set_param(grid, "gap_y", 0.30);
        let mv = g.add_node("motion.move");
        g.set_param(mv, "dx", (k as f32 - 1.0) * 2.6);
        let scale = g.add_node("motion.scale");
        g.set_param(scale, "amount", 0.55);
        let t = g.add_node("motion.tint");
        g.set_param(t, "r", rgb[0]);
        g.set_param(t, "g", rgb[1]);
        g.set_param(t, "b", rgb[2]);
        g.set_param(t, "a", 0.5);
        let out = g.add_node("motion.output");
        // A wave inteira numa linha: o modo é param do SINK.
        g.set_param(out, ph2d_eval_motion::SINK_BLEND_PARAM, tag);

        lay(g, 120.0 + 130.0 * k as f32, &[grid, mv, scale, t, out]);
        wire(g, grid, mv, 0)?;
        wire(g, mv, scale, 0)?;
        wire(g, scale, t, 0)?;
        wire(g, t, out, 0)?;
        sinks.push(out);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

/// **O CATÁLOGO de kernels do `value.noise`** (doc 89 folha 15) — quatro grades
/// idênticas, cada uma com o TAMANHO dos elementos dirigido pelo MESMO campo de
/// ruído lido no ESPAÇO, e só o `kernel` a diferir.
///
/// ⚠️ **A pergunta é de OLHO e as quatro têm de parecer quatro coisas:** manchas
/// macias (Value) · fluxo orgânico sem eixos (Perlin) · bolhas com um miolo
/// pequeno por célula (Cellular/Cells) · e uma teia de linhas escuras
/// (Cellular/Cracks). Se as quatro parecerem a mesma, o `kernel` não está a
/// chegar à rota que este build usa — bissecte com `PH2D_GPU_COOK=0`.
///
/// ⚠️ **`space = World` é o que torna a cena legível:** lido pela FILA o campo
/// varia com a ordem do elemento na lista e a grade sai listrada; lido no espaço
/// ele é uma imagem do campo, que é o que se quer julgar.
pub(crate) fn build_noise_kernel_demo_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let g = &mut doc.graph;
    let mut sinks = Vec::new();

    for (k, (kernel, feature)) in [
        (0.0_f32, 0.0_f32), // Value
        (1.0, 0.0),         // Perlin
        (2.0, 0.0),         // Cellular / Cells
        (2.0, 1.0),         // Cellular / Cracks
    ]
    .into_iter()
    .enumerate()
    {
        let grid = g.add_node("motion.grid");
        g.set_param(grid, "rows", 14.0);
        g.set_param(grid, "cols", 14.0);
        g.set_param(grid, "gap_x", 0.34);
        g.set_param(grid, "gap_y", 0.34);
        let mv = g.add_node("motion.move");
        g.set_param(mv, "dx", (k as f32 - 1.5) * 5.6);
        let scale = g.add_node("motion.scale");
        g.set_param(scale, "amount", 0.10);

        let vn = g.add_node("value.noise");
        g.set_param(vn, "kernel", kernel);
        g.set_param(vn, "feature", feature);
        g.set_param(vn, "jitter", 1.0);
        g.set_param(vn, "space", 1.0); // o eixo ESPACIAL — a cena inteira
        g.set_param(vn, "frequency", 0.55);
        g.set_param(vn, "speed", 0.25);
        g.set_param(vn, "octaves", 2.0);
        g.set_param(vn, "roughness", 0.5);
        // UNIPOLAR de proposito: o campo nasce em [-1, 1] e o canal que esta cena
        // dirige e o TAMANHO, que nao tem metade negativa. A primeira versao saia
        // com tamanhos ate -0.26 (a sonda apanhou-o) e a grade do Cracks -- cuja
        // media crua e a mais baixa das quatro (0.27 contra 0.43 do Cells) --
        // ficava quase invisivel.
        g.set_param(vn, "amplitude", 0.5);
        g.set_param(vn, "offset", 0.5);
        g.set_param(vn, "seed", 7.0);

        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", 3.0); // Size
        g.set_param(drive, "mode", 0.0); // Add
        g.set_param(drive, "scale", 0.40);
        let out = g.add_node("motion.output");

        lay(
            g,
            120.0 + 150.0 * k as f32,
            &[grid, mv, scale, vn, drive, out],
        );
        wire(g, grid, mv, 0)?;
        wire(g, mv, scale, 0)?;
        // O ruído lê a geometria (para ter `P` e contagem) e o drive recebe-a
        // na porta 0 e o VALOR na porta 1 — o encaminhamento do domínio de valor.
        wire(g, scale, vn, 0)?;
        wire(g, scale, drive, 0)?;
        wire(g, vn, drive, 1)?;
        wire(g, drive, out, 0)?;
        sinks.push(out);
    }

    doc.graph.validate(reg).ok()?;
    Some(sinks)
}

#[cfg(test)]
#[path = "motion_state_conferencia_demos_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_state_conferencia_demos_blend_tests.rs"]
mod blend_tests;

#[cfg(test)]
#[path = "motion_state_conferencia_demos_noise_tests.rs"]
mod noise_tests;
