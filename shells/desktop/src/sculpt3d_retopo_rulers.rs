//! ⭐⭐ **AS RÉGUAS DA TENTATIVA** — *«esta saída é pior que a anterior?»*
//!
//! Irmão de [`super::retopo_extract`] por RESPONSABILIDADE: ele decide **o que tentar**,
//! estas medem **o que saiu**. ⛔⛔ A chave da frente é [`open_edges`] (bordo **+**
//! não-manifold) e não só o bordo: em 2026-08-28 o ficheiro que o artista exportou tinha
//! `19 786` quads impecáveis com **`2` arestas não-manifold** num ponto só, e o veto não
//! as via — *«furo» contava metade*.

use ph2d_mesh::Mesh;

/// A aresta mediana e a mais longa da saída.
pub(super) fn edges(mesh: &Mesh) -> (f32, f32) {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    (
        e.get(e.len() / 2).copied().unwrap_or(0.0),
        e.last().copied().unwrap_or(0.0),
    )
}

/// Arestas com uma face só — a assinatura da casca aberta.
/// ⭐⭐⭐ **A ORDEM DA ESCOLHA entre duas tentativas — `true` se `a` é PIOR que `b`.**
///
/// **Furos, depois faces `>60°`, depois o enviesamento mediano.** ⚠️ Os furos vêm primeiro
/// porque são o que o artista **vê** — foi a queixa dele três vezes seguidas
/// (*«furos nas pontas»*). *Uma ordem que pusesse o enviesamento à frente escolheria a peça
/// mais bonita com um buraco na ponta.*
///
/// ⛔⛔ **E «furo» conta as DUAS formas de a casca não fechar, desde 2026-08-28.** Até essa
/// data esta ordem via só as arestas de **bordo**; uma aresta **não-manifold** — três faces a
/// tocá-la — passava invisível, e o campo alinhado produz exactamente isso (medido:
/// `sculpt_hooked`, `1` não-manifold contra `0` do liso, com o alinhado a ganhar por
/// `0,2°` de enviesamento). ⚠️ **O artista vê o mesmo entalhe escuro nos dois casos** — e o
/// ficheiro que ele exportou em 28/08 tinha `19 786` quads impecáveis com **`2` arestas
/// não-manifold** num ponto só, três vértices de valência `2`–`3`. *Uma chave de desempate
/// que não vê metade do defeito escolhe a peça furada com toda a razão do mundo.*
///
/// ⚠️ **O desempate final é por `total_cmp`** e não por `<`: um `NaN` numa das medianas
/// tornaria a comparação não-reflexiva e a escolha dependeria da ordem dos argumentos.
///
/// ⛔⛔⛔ **E DESDE 2026-08-30 A SEGUNDA CHAVE É [`components`]** — o report do artista com
/// foto (*«péssimo»*, um quad a flutuar solto ao lado de uma ponta). ⚠️ **Nenhuma das duas
/// chaves que existiam o via:** um pedaço que se desprende sai **fechado**, logo `0` arestas
/// de bordo e `0` não-manifold, e o `open_edges` — que a nota acima chama de *«as DUAS formas
/// de a casca não fechar»* — dá **zero** nas duas peças. *Uma superfície fechada pode conter
/// uma segunda superfície fechada, e contar arestas nunca o revela.*
///
/// ⚠️ **A ORDEM é `furos → peças → >60° → enviesamento`, e o lugar da chave nova é uma
/// decisão, não um acaso:** os furos ficam à frente porque *foi isso que se mediu* (a queixa
/// do artista três vezes seguidas), e ⛔ **não existe medição nenhuma que ordene um estilhaço
/// contra um furo** — inventá-la aqui seria escolher por conforto. O estilhaço não precisa
/// de ganhar a chave da frente para ser apanhado: [`shattered`] veta-o **depois** da
/// escolha, e um veto absoluto não depende de ordenação nenhuma.
#[allow(clippy::too_many_arguments)]
pub(super) fn worse(
    a_mesh: &Mesh,
    a_over60: usize,
    a_skew: f32,
    a_dev: ph2d_quadfill::TipDeviation,
    b_mesh: &Mesh,
    b_over60: usize,
    b_skew: f32,
    b_dev: ph2d_quadfill::TipDeviation,
) -> bool {
    let (a_holes, b_holes) = (open_edges(a_mesh), open_edges(b_mesh));
    if a_holes != b_holes {
        return a_holes > b_holes;
    }
    let (a_parts, b_parts) = (components(a_mesh), components(b_mesh));
    if a_parts != b_parts {
        return a_parts > b_parts;
    }
    // ⭐⭐⭐ **AS FACES QUE SE AUTO-INTERSECTAM — a 3.ª chave, e o report de 30/08 é a razão.**
    //
    // ⛔⛔⛔ **A régua que via a destruição JÁ EXISTIA e o produto não a consultava.**
    // [`ph2d_quadfill::local_shape`] vive numa crate do produto desde 30/08 e o seu único
    // leitor era a **sonda** da foto. Medido no A/B daquele dia: o caminho de omissão dá
    // **`0`** gravatas e o caminho novo dá **`125`** — e o dono descreveu a saída como
    // *«destruiu completamente a malha»*, enquanto as colunas que esta função lia diziam
    // apenas *pior* (`χ` `1 → 0`, bordo `4 → 12`, que se leem como brandos).
    //
    // ⚠️ **É a família do §5.0 do `CLAUDE.md`:** *nenhum instrumento do repo pergunta se o
    // valor chega a um consumidor.* Uma régua na prateleira não protege ninguém.
    //
    // ⚠️ **Aqui ela é ORDINAL, não veto**, e de propósito: uma tentativa com gravatas perde
    // sempre para uma sem, mas se **todas** as tentativas as tiverem ainda se escolhe a menos
    // má. *Um veto absoluto pede prova de corpus que esta linha ainda não tem* — e inventar
    // um limiar sem medir é o que o §0.0 proíbe.
    let (a_bow, b_bow) = (bowties(a_mesh), bowties(b_mesh));
    if a_bow != b_bow {
        return a_bow > b_bow;
    }
    // ⭐⭐⭐ **A AMPUTAÇÃO — e ela vem ANTES da forma, porque é o que o dono fotografou.**
    //
    // ⛔⛔⛔ **Medida em 2026-08-31, e é a razão de esta chave existir:** numa varredura do teto
    // de graduação da fase zero, a célula `ADAPT_RATIO = 8` entregou uma fase zero **perfeita**
    // (`0` de `4` pontas cortadas, pior `−0,5 %`) e a **saída** cortou a ponta mais longa em
    // ⛔ **`−43 %`**. As duas candidatas estavam limpas na topologia, e o `worse` escolheu a que
    // comia o espinho — *porque nada aqui olhava para o alcance.*
    //
    // ⛔⛔⛔ **E ATÉ 2026-08-31 ELA MEDIA O ALCANCE, QUE É UM EXTREMO GLOBAL — e sujo.**
    // Duas coisas mudaram no mesmo dia, as duas medidas:
    //
    // 1. **A régua estava contaminada.** O alcance tirava o centroide da média dos
    //    **vértices**, que é uma propriedade da amostragem: na escultura do dono o centroide
    //    derivava `0,2129` entre entrada e saída e a régua lia `−6,5 %` onde a verdade era
    //    `−0,1 %`; duas densidades da mesma peça diferiam `1,06 %` contra uma banda de `2 %`.
    //    ⚠️ E o sinal era o pior: quem **corta** a ponta perde vértices longe do corpo, o
    //    centroide afasta-se e o alcance medido **sobe**. Curado em
    //    [`ph2d_quadfill::reach`] (centroide de **área**), que o [`log_candidate`] imprime.
    // 2. **Um extremo global não conta QUANTAS pontas morreram** — é a limitação que esta
    //    linha nomeou três vezes. A régua por ponta existe agora
    //    ([`ph2d_quadfill::tip_deviation`]) e mede a distância da **escultura** à superfície
    //    de cada candidata junto de cada ápice, em unidades do quad pedido.
    //
    // ⭐⭐⭐ **E a troca MUDA uma escolha medida.** `_base_sculpt.obj` a `Detail 0,40`, onde as
    // duas primeiras candidatas **empatam** em bordo (`4`) e a chave decide:
    //
    // | candidata | alcance | **pontas acima da barra** |
    // |---|---|---|
    // | ⛔ `w = 0,000` (a que o alcance escolhia) | `2,8644` | **`2` de `4`** |
    // | ⭐ `w = 0,030` | `2,7869` | **`1` de `4`** |
    //
    // *A régua velha preferia a candidata com mais pontas partidas, porque a ponta que ela
    // media era a que sobrevivia nas duas.*
    //
    // ⚠️ **A barra é o chão da discretização** ([`ph2d_quadfill::TIP_DEVIATION_MAX`] = `1`
    // quad), não um número escolhido: medido, as pontas sãs ficam em `máximo 0,45` e a
    // partida em `p50 1,15`.
    //
    // ⛔ **A amostra vazia NÃO decide** — `tips = 0` é *«não medido»*, e lê-se igual a
    // *«perfeito»* em toda régua que devolva só a média.
    //
    // ⛔ **Depois dos FUROS e antes da forma:** um espinho cortado ao meio é mais visível que
    // uma face com canto pior que `60°`, e menos que um buraco — *que foi a queixa mais antiga
    // dele.*
    // ⛔⛔⛔ **E A CONTAGEM SOZINHA DEITA FORA A GRAVIDADE — corrigido em 2026-09-01.**
    //
    // A `over` conta **quantas** pontas passaram da barra e não diz **quão** longe. Uma
    // candidata que come uma ponta *por inteiro* (`p90 = 3,0`, que é o piso do «mais longe do
    // que eu olhei» de [`ph2d_quadfill::tip_deviation`]) e uma que a arranha (`p90 = 1,02`)
    // contam **`1` as duas**: empatam aqui, e a escolha cai para as chaves da beleza — faces
    // `>60°` e enviesamento —, que é decidir uma amputação por quão quadrados ficaram os quads.
    //
    // ⚠️ **A barra da `over` é a MEDIANA da ponta** (`TIP_DEVIATION_MAX`), logo meia ponta
    // comida não a arma sequer; a gravidade é a única coluna que a vê. *Os três números já
    // eram calculados e impressos no log — nada aqui os lia.*
    //
    // ⚠️ **`p90` e não `max`**: o `max` é o vértice mais afastado de uma amostra, e um único
    // ponto da escultura que caia numa fenda entre dois quads move-o sem que nada esteja
    // amputado. O `p90` é o mesmo extremo com a cauda de amostragem aparada, e continua a
    // separar `3,0` de `1,02` por larga margem.
    //
    // ⛔ **Depois da contagem, nunca à frente:** duas pontas partidas de raspão são um defeito
    // pior que uma partida a fundo — foi por «amputa **uma** ponta» / «amputou **2**» que o
    // dono nomeou os dois reports, nessa ordem.
    if a_dev.tips > 0 && b_dev.tips > 0 {
        if a_dev.over != b_dev.over {
            return a_dev.over > b_dev.over;
        }
        if (a_dev.p90 - b_dev.p90).abs() > 1.0e-3 {
            return a_dev.p90.total_cmp(&b_dev.p90) == std::cmp::Ordering::Greater;
        }
    }
    if a_over60 != b_over60 {
        return a_over60 > b_over60;
    }
    // ⭐⭐⭐ **A DENSIDADE DA PONTA — a chave que faltava, e a medição que a exige.**
    //
    // ⛔⛔⛔ **A cura do report de 28-29/08 já estava a ser produzida e era DEITADA FORA
    // aqui.** Medido em 2026-08-30 (`sculpt_antes.obj`, `Detail 0,85`), as candidatas do
    // caminho de omissão, **sem knob nenhum**:
    //
    // | candidata | quads | bordo | `>60°` | **`ENTREGA`** |
    // |---|---|---|---|---|
    // | campo liso | `9 484` | `28` | `2` | `1,585` |
    // | ⛔ campo alinhado (**a escolhida**) | `9 414` | `4` | `2` | `1,502` |
    // | ⭐ **campo com linhas de feição** | `9 121` | `4` | `2` | ⭐ **`0,851`** |
    //
    // ⭐⭐⭐ **A terceira EMPATA em furos, peças, gravatas e faces `>60°`** — ela perdia
    // **só** no enviesamento mediano, que era a última chave. *O eixo de que o dono se
    // queixou três vezes não estava na função que escolhe*, e o desempate era feito por uma
    // grandeza que ele não vê.
    //
    // ⚠️ **O lugar é DEPOIS de `>60°` e ANTES do enviesamento**, e isso é uma decisão: uma
    // face com canto pior que `60°` é um defeito local visível, uma ponta grosseira é um
    // defeito de **cobertura** (o dono fotografou-a), e a mediana do enviesamento é a única
    // das três que ele nunca nomeou.
    //
    // ⛔ **Nunca à frente dos FUROS.** Com `Follow Curvature` ligado, a candidata de feições
    // chega a `0,543` — o alvo é `0,59` — mas traz `6` arestas de bordo contra `4`. *Buracos
    // foram a queixa dele três vezes; esta chave não os compra.*
    //
    // ⚠️ **Menor é melhor, e sem banda** — pela mesma razão que a chave seguinte (o
    // enviesamento) não tem: inventar um limiar aqui seria escolher um número sem o medir.
    // ⛔ **A amostra vazia NÃO decide** (`0,0` de «não medido» lê-se como o melhor resultado
    // possível — é a armadilha que o doc do [`ph2d_quadfill::tip_body_ratio`] nomeia).
    //
    // ⚠️ `PH2D_RETOPO_TIPKEY=0` desliga a chave, para bissectar.
    if tip_key_on() {
        let ((a_tip, a_n), (b_tip, b_n)) = (tip_ratio(a_mesh), tip_ratio(b_mesh));
        if a_n > 0 && b_n > 0 && a_tip.total_cmp(&b_tip) != core::cmp::Ordering::Equal {
            return a_tip > b_tip;
        }
    }
    a_skew.total_cmp(&b_skew) == core::cmp::Ordering::Greater
}

/// ⭐ **O ALCANCE de uma malha** — a distância máxima ao centroide **pesado pela área**,
/// pela porta [`ph2d_quadfill::reach`].
///
/// ⚠️ É a única régua desta linha que vê **amputação**: uma ponta cortada sai com a casca
/// fechada, quads bonitos e `χ` exacta. ⛔ *Ela é um extremo GLOBAL* — não diz **quantas**
/// pontas morreram, e é por isso que o diagnóstico usa o suporte por ponta. Aqui, entre duas
/// candidatas da mesma entrada, o extremo é exactamente a comparação certa.
///
/// ⛔⛔ **E até 2026-08-31 ela tirava o centroide da média dos VÉRTICES, que é uma
/// propriedade da amostragem e não da forma** — medido na escultura do dono: `0,2129` de
/// deriva do centroide, um alcance lido a **`−6,5 %`** onde a verdade era `−0,1 %`, e
/// **`1,06 %`** de erro entre duas densidades da mesma peça contra uma banda de `2 %`.
/// ⚠️ O sinal era o pior possível: quem **corta** a ponta perde vértices longe do corpo, o
/// centroide afasta-se, e o alcance medido **sobe**. O mecanismo e a tabela vivem no
/// módulo [`ph2d_quadfill::tips`].
fn reach(mesh: &Mesh) -> f32 {
    ph2d_quadfill::reach(mesh)
}

/// ⚠️ **A chave da ponta está ligada?** — `PH2D_RETOPO_TIPKEY=0` desliga.
///
/// ⭐ **Ela nasce LIGADA**, ao contrário do costume desta linha, e a razão é medida: ela não
/// acrescenta um caminho novo — ela **escolhe entre candidatas que a cadeia já produzia**, e
/// a que ela passa a escolher empata em toda chave de topologia com a que ganhava antes.
/// *O que se liga aqui é uma decisão, não um algoritmo.*
fn tip_key_on() -> bool {
    std::env::var("PH2D_RETOPO_TIPKEY").as_deref() != Ok("0")
}

/// ⭐⭐⭐ **CADA CANDIDATA DIZ O QUE É** — e ela mora aqui, com o [`worse`], de propósito.
///
/// ⛔⛔ **Ela existe por uma medição de 2026-08-30.** Com `Follow Curvature` a `1` a saída da
/// peça do artista é **byte-idêntica** à de `0` (mesmos `9 414` quads, mesmas medianas por
/// casca, mesmas dobras) — porque a guarda `uniforme` da porta recorre e o [`worse`] escolhe a
/// corrida sem campo. ⚠️ **E não havia como o saber:** nada registava as candidatas, então um
/// knob **descartado** e um knob **fraco** liam-se exactamente igual.
///
/// ⚠️ **A `ENTREGA` sai da MESMA porta que o [`worse`] consulta** ([`tip_ratio`]): um registo
/// que medisse a grandeza de outra maneira imprimiria um número que não explica a escolha.
#[allow(clippy::too_many_arguments)]
pub(super) fn log_candidate(
    w: f32,
    features: bool,
    adaptive: f32,
    out: &Mesh,
    shape: &ph2d_quadfill::QuadShape,
    round: &ph2d_gridmap::RoundReport,
    cut_rep: &ph2d_gridmap::CutReport,
    dev: ph2d_quadfill::TipDeviation,
) {
    let (ratio, amostra) = tip_ratio(out);
    eprintln!(
        "[sculpt3d] candidata w={w:.3} feicoes={features} adapt={adaptive:.2}: {} quads | \
         bordo {} | costuras soltas {} | locais trocados {} | lados a discordar {} | >60 {} | \
         envies p50 {:.2} p99 {:.1} | aspecto p50 {:.2} | ENTREGA {ratio:.3} (ponta {amostra}) \
         | alcance {:.4} | DESVIO p50 {:.2} ({} de {} ponta(s) acima de {:.1})",
        out.face_count(),
        boundary_edges(out),
        round.solve.loose_seams,
        round.solve.mismatched_locals,
        cut_rep.side_patch_flips,
        shape.skew_over_60,
        shape.skew_p50,
        shape.skew_p99,
        shape.aspect_p50,
        reach(out),
        dev.p50,
        dev.over,
        dev.tips,
        ph2d_quadfill::TIP_DEVIATION_MAX,
    );
}

/// ⭐ **A RAZÃO PONTA/CORPO desta malha** — a mesma porta que as sondas usam
/// ([`ph2d_quadfill::tip_body_ratio`]), com a contagem da amostra ao lado.
///
/// ⚠️ **Pela porta partilhada e não recalculada aqui:** duas leis para a mesma grandeza dão
/// dois números que ninguém pode comparar, e é o defeito que esta linha já pagou três vezes.
pub(super) fn tip_ratio(mesh: &Mesh) -> (f32, usize) {
    let pos = mesh.positions();
    let mut cent: Vec<[f32; 3]> = Vec::with_capacity(mesh.face_count());
    let mut raiz: Vec<f32> = Vec::with_capacity(mesh.face_count());
    for f in mesh.faces() {
        let v = f.verts();
        let mut c = [0.0f32; 3];
        let mut s = [0.0f32; 3];
        for k in 0..v.len() {
            let a = pos[v[k] as usize];
            let b = pos[v[(k + 1) % v.len()] as usize];
            for j in 0..3 {
                c[j] += a[j];
            }
            s[0] += a[1].mul_add(b[2], -(a[2] * b[1]));
            s[1] += a[2].mul_add(b[0], -(a[0] * b[2]));
            s[2] += a[0].mul_add(b[1], -(a[1] * b[0]));
        }
        #[allow(clippy::cast_precision_loss)]
        let n = v.len() as f32;
        cent.push([c[0] / n, c[1] / n, c[2] / n]);
        raiz.push((0.5 * s[0].mul_add(s[0], s[1].mul_add(s[1], s[2] * s[2])).sqrt()).sqrt());
    }
    ph2d_quadfill::tip_body_ratio(&cent, &raiz)
}

/// ⭐⭐⭐ **EM QUANTAS PEÇAS a malha é** — componentes ligados por **ARESTA**.
///
/// ⚠️ **Por aresta e não por vértice, e a diferença é o que o artista vê:** dois sacos
/// fechados que se tocam num vértice só são, para quem olha, duas peças — a união por
/// vértice diria `1` e daria a peça partida por boa.
///
/// ⚠️ **Uma aresta não-manifold não parte nada:** as três faces que a partilham entram
/// todas no mesmo grupo (o mapa guarda a PRIMEIRA face de cada aresta, e as seguintes
/// unem-se a ela) — é por isso que esta régua e o [`open_edges`] medem coisas
/// independentes, e é por isso que as duas têm de existir.
pub(super) fn components(mesh: &Mesh) -> usize {
    use std::collections::{BTreeMap, BTreeSet};
    let n = mesh.face_count();
    if n == 0 {
        return 0;
    }
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    let mut parent: Vec<usize> = (0..n).collect();
    let mut first: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&other) = first.get(&key) {
                let (ra, rb) = (find(&mut parent, other), find(&mut parent, fi));
                if ra != rb {
                    parent[ra] = rb;
                }
            } else {
                first.insert(key, fi);
            }
        }
    }
    (0..n)
        .map(|i| find(&mut parent, i))
        .collect::<BTreeSet<_>>()
        .len()
}

/// ⭐⭐⭐ **O VETO — a retopologia ESTILHAÇOU a peça?** `Some((peças, eram))` quando sim.
///
/// ⛔⛔⛔ **Reproduzido em 2026-08-30 com a peça do artista, e é a foto dele:** ao carregar
/// no botão uma **segunda** vez, a saída vem com `2` peças — um pedaço solto de `22` faces a
/// flutuar — `χ` de `2` para `4`, e a ponta mais longa cortada de `−0,2 %` para **`−35,0 %`**.
/// ⚠️ **Um clique só não o faz**: o insumo do segundo clique é a saída do primeiro, e é a
/// re-entrada que parte a peça.
///
/// ⚠️ **É RELATIVO à entrada, nunca absoluto:** uma cena com dois objectos soltos entra com
/// `2` peças e tem todo o direito de sair com `2`. *O que o botão não pode é devolver mais
/// peças do que recebeu.*
///
/// ⚠️ **O veto é a ÚLTIMA palavra e não uma candidata:** o [`worse`] escolhe entre tentativas
/// e só sabe dizer qual é a melhor; quando **todas** estilhaçam, a melhor delas ainda é uma
/// peça partida. *Uma escada de candidatas nunca compara com o que o artista já tinha na
/// mão* — e o que ele tinha é o que fica.
pub(super) fn shattered(out: &Mesh, reference: &Mesh) -> Option<(usize, usize)> {
    let (saiu, entrou) = (components(out), components(reference));
    (saiu > entrou).then_some((saiu, entrou))
}

pub(super) fn boundary_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).0
}

/// ⭐⭐⭐ **A SAÍDA AINDA ESTÁ PARTIDA?** — a condição que ARMA as tentativas extra do botão.
///
/// ⚠️ **Ela é «as chaves da frente do [`worse`] ainda não estão satisfeitas»**, e não um limiar
/// escolhido: bordo/não-manifold (`1.ª`) e faces auto-intersectadas (`3.ª`). ⛔ A `2.ª` — a
/// contagem de peças — fica de fora **porque não é absoluta**: ela só significa alguma coisa
/// contra a entrada (ver [`shattered`]), e aqui não há entrada.
///
/// ⛔⛔ **As gravatas entram aqui em 2026-08-30 e a razão é o report do dono** (*«destruiu
/// completamente a malha»*): uma saída com faces cruzadas sobre si próprias tem de fazer o botão
/// **tentar outra vez**, não entregá-la.
///
/// ⭐⭐ **E isto é estritamente melhor que um VETO, com risco ZERO de trancar o botão:** as
/// candidatas extra entram todas pelo mesmo [`worse`], logo *só vencem onde são melhores*. Se
/// **todas** as tentativas saírem cruzadas, ainda se escolhe a menos má e o artista recebe
/// alguma coisa. *Uma recusa absoluta transformaria um defeito raro numa ferramenta inutilizável,
/// e a prova de corpus que a justificaria não existe* — medido em `5` corridas (3 peças × 3
/// níveis, `1 353`..`9 598` quads): **`0` gravatas em todas**.
///
/// ⛔⛔⛔ **E o VETO está MEDIDO E REFUTADO (30/08), com o único denominador honesto que existe:
/// os ficheiros do próprio dono.** Toda malha retopologizada da pasta dele tem faces cruzadas —
/// `Sculpt_Blender.obj`, a saída que ele **APROVOU** (*«preserva as pontas»*), tem `1` em
/// `8 291` faces; `sculpt_Depois.obj` tem `1`; e `sculpt_t003.obj`, a **entrada** dele no nosso
/// botão, tem `2`. ⇒ *«uma face cruzada é inaceitável» é uma barra que a ferramenta de
/// referência não cumpre*, e um veto teria recusado a malha que ele elogiou. ⛔ **Não volte a
/// propô-lo sem re-medir esses três ficheiros.**
///
/// ⛔⛔⛔ **E A AMPUTAÇÃO ENTRA AQUI EM 2026-09-01, porque ela era a chave da frente que esta
/// porta NÃO consultava.** A 4.ª chave do [`worse`] nasceu em 31/08 e esta condição não foi
/// actualizada com ela — resultado medido na peça do dono: uma saída **topologicamente
/// impecável** (`0` furos, `0` gravatas) com **uma ponta comida** nunca armava a 3.ª nem a 4.ª
/// tentativa. *É exactamente a forma do report* (*«amputa uma ponta»* — foto, 31/08): o botão
/// entregava a primeira candidata sem sequer tentar outra vez.
///
/// ⚠️ **A promessa de segurança é a MESMA e não enfraquece:** as candidatas extra entram todas
/// pelo mesmo [`worse`], logo só vencem onde são melhores. Armar mais vezes custa **relógio**,
/// nunca qualidade. ⛔ E o custo é limitado por construção: uma peça sã não paga nada, porque
/// as três condições são todas `0`.
pub(super) fn still_broken(mesh: &Mesh, dev: ph2d_quadfill::TipDeviation) -> bool {
    open_edges(mesh) > 0 || bowties(mesh) > 0 || dev.over > 0
}

/// ⭐⭐ **Quantas faces se AUTO-INTERSECTAM** — pela porta de [`ph2d_quadfill::local_shape`].
///
/// ⚠️ **Pela porta e não reimplementada aqui:** a lei do quad em oito vive naquela crate com
/// os gates dela, e uma segunda cópia divergiria da primeira no dia em que uma fosse
/// corrigida. *O que faltava não era a lei — era um consumidor.*
pub(super) fn bowties(mesh: &Mesh) -> usize {
    ph2d_quadfill::local_shape(mesh).0.bowties
}

/// ⭐⭐⭐ **AS DUAS FORMAS DE A CASCA NÃO FECHAR, somadas** — a chave da frente de [`worse`].
///
/// ⚠️ **Uma aresta de bordo e uma não-manifold dão o MESMO report** (*«furos»*), e nenhuma
/// régua desta linha as somava: a escolha entre tentativas via só a primeira.
pub(super) fn open_edges(mesh: &Mesh) -> usize {
    let (bordo, nm) = edge_census(mesh);
    bordo + nm
}

/// `(arestas de bordo, arestas não-manifold)` — uma face só, ou mais de duas.
pub(super) fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (
        n.values().filter(|c| **c == 1).count(),
        n.values().filter(|c| **c > 2).count(),
    )
}

/// Vértices com valência diferente de 4 — a grandeza que o pivô existiu para
/// derrubar. ⭐ Uma grade numa esfera admite **oito**.
pub(super) fn irregular(mesh: &Mesh) -> usize {
    let mut deg = vec![0usize; mesh.vert_count()];
    use std::collections::BTreeSet;
    let mut seen: BTreeSet<(u32, u32)> = BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if seen.insert(if a < b { (a, b) } else { (b, a) }) {
                deg[a as usize] += 1;
                deg[b as usize] += 1;
            }
        }
    }
    deg.iter().filter(|d| **d != 4 && **d > 0).count()
}

/// **A DIAGONAL da caixa da peça** — o denominador da fração absoluta, e a mesma
/// régua do irmão.
pub(super) fn span(mesh: &Mesh) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

#[cfg(test)]
#[path = "sculpt3d_retopo_rulers_tests.rs"]
mod tests;

/// ⭐⭐⭐ **OS GATES DA CHAVE DA PONTA** — irmão do [`tests`] pelo teto de LOC da shell
/// (HR-18, 600), cortado por RESPONSABILIDADE: aquele defende as chaves de **defeito**
/// (furos · peças · gravatas), este a chave de **cobertura** e a fronteira entre as duas.
#[cfg(test)]
#[path = "sculpt3d_retopo_tip_tests.rs"]
mod tip_tests;

/// ⭐⭐⭐ **OS GATES DA CHAVE DA GRAVATA** — irmão dos outros dois pelo teto de LOC da shell
/// (HR-18, 600), cortado por RESPONSABILIDADE: a **face que se cruza sobre si própria**, a
/// ORDEM das quatro chaves, e a guarda que a gravata armou.
#[cfg(test)]
#[path = "sculpt3d_retopo_bowtie_tests.rs"]
mod bowtie_tests;
