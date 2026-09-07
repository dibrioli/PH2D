//! ⭐⭐⭐ **O POLÍGONO DE `N` VÉRTICES AUTORADOS** (W132) — a lista de pontos, e a lei da contagem.
//!
//! # Por que ele é primitiva própria, tendo o campo do [`Primitive::Extrude`]
//!
//! ⚠️ **A superfície é a mesma; a AUTORIA é oposta.** O contorno de um `Extrude` é do editor
//! vetorial: o vínculo re-coze o perfil a cada quadro, então uma linha de painel sobre um daqueles
//! pontos seria escrita e **apagada no quadro seguinte** — um controlo morto, que é o defeito que
//! esta casa caça por escrito. Os pontos de um [`Primitive::Polygon`] são deste nó, e é por isso que
//! eles têm linha no painel ([`crate::dims`]).
//!
//! ⭐ **O campo continua a ser UM só** — a árvore do polígono é literalmente a
//! `profile::sd_extrude`, e ele herda de graça a especialização por ladrilho, o filete do **aro** e
//! o chanfro dele. *Duas cópias da mesma lei divergem no dia em que uma delas mudar.*
//!
//! ⏳ **E herda também o limite dela, que a W132 MEDIU e nomeou:** aquele `round` é do **aro**, não
//! das quinas do contorno — a quina que o artista digita fica viva, com qualquer filete. Num
//! `Extrude` isso não é buraco (as quinas arredondam-se no editor vetorial, de onde o contorno vem);
//! aqui **é**, porque não há editor nenhum. A tabela da medição está no doc de
//! `ph2d_field_eval::profile::sd_extrude`, e a cura não é afinação: um polígono **côncavo** não é
//! uma intersecção de semiplanos, então a receita do triângulo e do prisma não transfere.
//!
//! # ⚠️ A lei da CONTAGEM, e por que ela é reversível
//!
//! Subir a contagem **parte a aresta mais longa ao meio**: o ponto novo cai **em cima** da aresta
//! antiga, logo a peça não muda de forma nenhuma. Descer tira o vértice **mais colinear** — o que
//! menos muda a área. ⇒ o ponto acabado de nascer tem área de desvio **exactamente zero**, e é ele
//! o primeiro a sair: **subir e descer é a identidade**, e há gate a prová-lo.
//!
//! ⛔ A alternativa óbvia — *«subir regenera um polígono regular de `N` lados»* — apagaria o trabalho
//! do artista a cada clique, e a que lhe segue — *«descer tira o último»* — faria a peça saltar. As
//! duas leis desta secção são a mesma escolha: **o gesto que muda a contagem não muda a forma**.

use crate::{FillRule, Primitive, Profile};

/// O menor número de vértices de um polígono — com dois não há área nenhuma.
///
/// ⚠️ **Não é `4`**, e a sobreposição com o [`Primitive::Triangle`] é deliberada: as duas formas
/// respondem à mesma pergunta com painéis diferentes (três pares de linhas fixas contra uma
/// contagem), e proibir o triângulo aqui faria *descer a contagem até ao fundo* recusar sem motivo
/// visível — o artista veria o controlo parar antes do fim.
pub const MIN_POLYGON_VERTICES: u32 = 3;

/// ⭐⭐⭐ **O TETO de vértices de um polígono — e o recurso é a FAMÍLIA DE LINHAS DO PAINEL.**
///
/// # ⛔ NÃO é o relógio, e a medição diz porquê
///
/// A sonda `measure_polygon_vertices` mediu o preço por ponto, e ele é linear e caro:
///
/// | vértices | nós | ns/ponto | × um cilindro | linhas do painel |
/// |---:|---:|---:|---:|---:|
/// | 3 | 121 | 4,16 | 2,25× | 16 |
/// | 8 | 280 | 8,52 | 4,60× | 26 |
/// | 16 | 536 | 15,03 | 8,12× | 42 |
/// | **27** | — | — | — | **64** |
/// | 32 | 1048 | 28,14 | 15,20× | 74 |
/// | 64 | 2072 | 65,94 | 35,62× | 138 |
///
/// ⚠️ **Duas corridas a `load 9,4` e `7,6`** — as duas acima da barra de `~5` desta workstation, e
/// por isso postas lado a lado: elas concordam a `< 2,5 %` no `ns/ponto` (`4,26`/`4,16`,
/// `15,01`/`15,03`, `28,13`/`28,14`). *Um laço apertado em cache não é o quadro; a régua que a carga
/// destrói é a que mede um passe inteiro.*
///
/// ⛔ **E um teto de preço aqui poria o caminho lento a mandar no rápido** (`CLAUDE.md` §0): a única
/// alternativa a um contorno irregular é **desenhá-lo**, e a W131 mediu que o desenho custa
/// `2,6×`–`3,1×` a fórmula. *Quem pede 20 vértices não tem rota mais barata para os ter.*
///
/// # ⭐ O recurso é o REGISTO
///
/// O `populate` do painel corre antes de o documento existir e cunha ids às cegas para uma família
/// de tamanho fixo (`ph2d_panel_model3d::MAX_ROWS`, hoje `64`). Uma linha além dela fica **sem
/// controlo** — e a última de um polígono é o *Fillet*.
///
/// Medido na cena (`field3d_polygon_rows_tests`): as linhas de um nó são **`2N + 10`** — os `2N + 4`
/// da forma mais os `6` da pose. ⇒ `2N + 10 ≤ 64` dá **`N ≤ 27`**, e `27` é o número que shipa.
///
/// ⚠️ **As duas pontas têm gate**, e é isso que faz disto uma medição e não uma folga escolhida: o
/// polígono no teto **cabe** e o seguinte **não caberia**. Quem acrescentar uma linha a um nó vê o
/// gate ficar vermelho com a conta dentro.
pub const MAX_POLYGON_VERTICES: u32 = 27;

/// A tolerância de achatamento que o perfil de um polígono carrega.
///
/// ⚠️ **Ela não é lida por ninguém, e isso é a nota**: um [`Profile`] guarda a tolerância com que uma
/// curva foi achatada em polilinha, e um polígono autorado **não foi achatado de nada** — os
/// segmentos dele são a forma. O tipo exige um número positivo, e este é o que o caminho do desenho
/// usa (`1e-4`, o joelho medido na W54), para que as duas portas não digam coisas diferentes sobre a
/// mesma estrutura.
pub const POLYGON_TOLERANCE: f32 = 1e-4;

/// **Os vértices de um polígono**, ou `None` se `p` não é um.
///
/// ⚠️ O documento garante **um** contorno ([`crate::validate_primitive`]), então o `first` é a
/// resposta inteira — e não a primeira de várias.
#[must_use]
pub fn polygon_points(p: &Primitive) -> Option<&[[f32; 2]]> {
    match p {
        Primitive::Polygon { profile, .. } => profile.contours().first().map(Vec::as_slice),
        _ => None,
    }
}

/// **A única porta que constrói o perfil de um polígono.**
///
/// Devolve `None` quando a lista não faz um polígono — menos de [`MIN_POLYGON_VERTICES`] pontos,
/// pontos não-finitos, ou tudo em cima da mesma recta (o [`Profile::new`] recusa os três).
///
/// ⚠️ **A regra de preenchimento e a tolerância moram aqui, num sítio.** Um segundo chamador a
/// escolher `FillRule` dava duas respostas para a mesma pergunta — e num contorno **só** as duas
/// regras concordam, o que faria a divergência ser invisível até ao dia em que alguém permitisse o
/// segundo contorno.
#[must_use]
pub fn polygon_profile(points: Vec<[f32; 2]>) -> Option<Profile> {
    if points.len() < MIN_POLYGON_VERTICES as usize {
        return None;
    }
    Profile::new(vec![points], FillRule::NonZero, POLYGON_TOLERANCE).ok()
}

/// ⭐⭐ **A ESCRITA das linhas que só o polígono tem** — a contagem, os `2N` números e a altura.
///
/// Devolve `false` para um índice que **não** é destes (o chanfro e o filete, que entram pela porta
/// dos dois recuos do [`crate::dims`], como em toda forma).
///
/// ⚠️ **A contagem é COAGIDA, não recusada** — a mesma lei do `sides` de um prisma: a UI nunca
/// oferece fora da faixa, então um valor de fora só chega de um ficheiro estragado, e recusar ali
/// rejeitaria a peça inteira.
///
/// ⚠️ **Uma escrita que não constrói um polígono válido não escreve nada.** Arrastar um vértice para
/// cima do vizinho colapsa uma aresta, e o [`Profile`] recusa — a peça fica como estava, que é o que
/// a invariante *«um documento que existe está válido»* exige. *Foi este o defeito que a família de
/// tela em branco da W126 pagou.*
pub(crate) fn write_polygon_dim(p: &mut Primitive, index: usize, value: f32) -> bool {
    let Primitive::Polygon {
        profile,
        half_height,
        ..
    } = p
    else {
        return false;
    };
    let pontos: Vec<[f32; 2]> = profile.contours().first().cloned().unwrap_or_default();
    let n = pontos.len();
    // ⚠️ **A contagem tem de sobreviver à escrita.** O [`Profile::new`] **funde** pontos
    // consecutivos repetidos — é limpeza de entrada, e aqui seria um defeito: arrastar um vértice
    // exactamente para cima do vizinho devolveria um polígono de `N−1` pontos, e a **linha da
    // contagem saltaria sozinha** debaixo do dedo de quem estava a arrastar outra linha.
    let mantem = |profile: &mut Profile, novos: Vec<[f32; 2]>| {
        let quantos = novos.len();
        if let Some(novo) = polygon_profile(novos)
            && novo.contours().first().map_or(0, Vec::len) == quantos
        {
            *profile = novo;
        }
    };
    if index == 0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let alvo = (value.round().max(0.0) as u32).clamp(MIN_POLYGON_VERTICES, MAX_POLYGON_VERTICES)
            as usize;
        let mut novos = pontos;
        while novos.len() < alvo {
            novos = with_one_more_vertex(&novos);
        }
        while novos.len() > alvo {
            novos = with_one_fewer_vertex(&novos);
        }
        mantem(profile, novos);
        return true;
    }
    if index <= 2 * n {
        let (i, eixo) = ((index - 1) / 2, (index - 1) % 2);
        let mut novos = pontos;
        novos[i][eixo] = value;
        mantem(profile, novos);
        return true;
    }
    if index == 2 * n + 1 {
        *half_height = value * 0.5;
        return true;
    }
    false
}

/// **Esta linha é do polígono?** — a contagem, um dos `2N` números, ou a altura.
///
/// ⚠️ **Ela existe separada da escrita porque uma GUARDA de `match` não pode mutar** o que casou.
/// *Um predicado com efeito colateral não compila aqui, e a linguagem tem razão.*
#[must_use]
pub(crate) fn owns_row(p: &Primitive, index: usize) -> bool {
    match p {
        Primitive::Polygon { profile, .. } => {
            let n = profile.contours().first().map_or(0, Vec::len);
            index <= 2 * n + 1
        }
        _ => false,
    }
}

/// ⭐ **O vértice novo: a aresta mais longa partida ao meio.** A forma **não muda**.
///
/// ⚠️ O ponto médio é sempre distinto das pontas: o [`Profile::new`] já tirou as arestas de
/// comprimento zero, então toda aresta que chega aqui tem comprimento positivo.
#[must_use]
pub fn with_one_more_vertex(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let n = points.len();
    let mut melhor = 0;
    let mut maior = f64::NEG_INFINITY;
    for i in 0..n {
        let a = points[i];
        let b = points[(i + 1) % n];
        let l = f64::from(b[0] - a[0]).hypot(f64::from(b[1] - a[1]));
        if l > maior {
            maior = l;
            melhor = i;
        }
    }
    let a = points[melhor];
    let b = points[(melhor + 1) % n];
    let mut saida = points.to_vec();
    saida.insert(melhor + 1, [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    saida
}

/// ⭐ **O vértice que sai: o mais COLINEAR** — o que menos muda a área.
///
/// A medida é `|(v − anterior) × (seguinte − v)|`, que é o dobro da área do triângulo que se perde
/// ao ligar os dois vizinhos directamente. Um ponto nascido de [`with_one_more_vertex`] tem-na
/// **exactamente zero**, e por isso as duas operações são inversas.
///
/// ⚠️ **O empate resolve-se pelo índice mais baixo**, e não é cosmética: sem uma ordem escrita, dois
/// vértices igualmente colineares fariam a mesma peça encolher de maneiras diferentes entre corridas.
#[must_use]
pub fn with_one_fewer_vertex(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let n = points.len();
    let mut melhor = 0;
    let mut menor = f64::INFINITY;
    for i in 0..n {
        let a = points[(i + n - 1) % n];
        let v = points[i];
        let b = points[(i + 1) % n];
        let area2 = (f64::from(v[0] - a[0]) * f64::from(b[1] - v[1])
            - f64::from(v[1] - a[1]) * f64::from(b[0] - v[0]))
        .abs();
        if area2 < menor {
            menor = area2;
            melhor = i;
        }
    }
    let mut saida = points.to_vec();
    saida.remove(melhor);
    saida
}
