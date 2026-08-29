#![forbid(unsafe_code)]
//! `ph2d-quadchain` — **a ORDEM da cadeia de quads**, numa porta que qualquer módulo pode chamar.
//!
//! # Por que esta crate existe
//!
//! A cadeia que transforma uma malha de triângulos numa malha de **quads alinhados à superfície**
//! tem sete passos, e a ordem deles é load-bearing (a fase zero sozinha vale `2×` no enviesamento
//! final). Até 2026-08-24 essa ordem vivia **dentro do shell do módulo de escultura**
//! (`sculpt3d_history_retopo_extract.rs`, `pub(in crate::sculpt3d)`) — alcançável por um módulo só.
//!
//! ⚠️ **E o segundo consumidor chegou**: o modelador implícito extrai a peça por *Dual Contouring*
//! sobre grade, e o placar dele
//! (`ph2d_field_eval::tests::the_scorecard_of_the_extracted_mesh`) mediu **onde ele perde**:
//!
//! | eixo | o extractor de campo | esta cadeia | oráculo `quadwild-bimdf` |
//! |---|---|---|---|
//! | arestas não-manifold · bordo | **0 · 0** | — | — |
//! | `\|f\|` no vértice | **~0,005 célula** | — | — |
//! | 100 % quads | **sim** | sim | sim |
//! | **enviesamento mediano** | ⛔ **25–27°** | ⭐ **5,1–5,5°** | 4,8–7,1° |
//!
//! ⛔ **E o buraco é ESTRUTURAL, não de afinação** — medido: o *mesmo* cubo alinhado com a grade sai
//! a `1,00` de aspecto e `0°` de enviesamento; rodado 45° sai a **`1,41 = √2`** com cauda a `90°`.
//! *A forma de uma face dual segue a GRADE, não a superfície.* Nenhum parâmetro cura isso; o que cura
//! é outra **conectividade** — que é exactamente o que esta cadeia produz.
//!
//! # ⚠️ Ela é a ORDEM, e não o algoritmo
//!
//! Cada passo já existe e é medido na sua própria crate. O que aqui se guarda é **a sequência e as
//! duas leis que a acompanham**:
//!
//! 1. ⛔ **A FASE ZERO é obrigatória.** Sem remalhar isotropicamente à frente, a mesma cadeia dá
//!    `10–12°` em vez de `5–5,5°` — *o dobro, sem uma linha de algoritmo mudar*.
//! 2. ⭐ **O alvo sai da malha que o artista trouxe**, nunca da remalhada — derivá-lo da remalhada
//!    foi medido e mata o controle de detalhe.
//!
//! # ⚠️ Duas cópias desta ordem seria o defeito
//!
//! O shell da escultura tem hoje a dele, e esta crate nasce **sem lhe tocar**: a `line/quadextract`
//! está viva sobre aquele arquivo, e uma migração ali seria colisão de mesmo-símbolo com uma linha
//! em curso (`DIRETRIZ` §1.5.5). ⇒ ela é escrita para que aquela metade adopte esta porta **numa
//! linha**, quando aquela linha quiser. *Duas cópias de uma lei é uma lei que gate nenhum defende —
//! e esta nasce sabendo disso.*

use ph2d_mesh::Mesh;

/// Por que a cadeia recusou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChainError {
    /// A extracção das isolinhas recusou o mapa.
    Extract(ph2d_quadextract::ExtractError),
    /// A cadeia correu e não sobrou face nenhuma — o alvo é grosso demais para a peça.
    TooCoarse,
}

/// ⭐ **Quanto custou cada fase**, em milissegundos — a cadeia mede-se a si própria.
///
/// ⚠️ **Ela existe porque o custo desta cadeia é um FACTO DE PRODUTO, não um detalhe.** Medido: numa
/// peça de um milhão de faces a cadeia congela o loop por minutos, e quem a chama tem de poder dizer
/// *onde* o tempo foi sem re-executar os sete passos numa sonda à parte. ⛔ Uma sonda que repete a
/// sequência é uma **segunda cópia da ordem** — exactamente o que esta crate existe para não ter.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChainTiming {
    /// F1 — remalhar isotropicamente + triangular.
    pub remesh: f32,
    /// F2 — o campo cruzado (`Dual::build` + `solve_miq`).
    pub field: f32,
    /// F3 — o traçado dos patches.
    pub trace: f32,
    /// G1/G2 — corte e penteado.
    pub cut: f32,
    /// G3/G5 — o mapa de grade e o arredondamento.
    pub map: f32,
    /// A extracção das isolinhas inteiras.
    pub extract: f32,
    /// ⭐ O acabamento — ver [`ph2d_quadfill::finish_extracted`].
    pub finish: f32,
}

impl ChainTiming {
    /// O total, que é o que quem espera sente.
    #[must_use]
    pub fn total(self) -> f32 {
        self.remesh + self.field + self.trace + self.cut + self.map + self.extract + self.finish
    }
}

/// O que a cadeia tem a dizer sobre o que produziu.
#[derive(Clone, Debug)]
pub struct ChainReport {
    /// Quanto cada fase custou — ver [`ChainTiming`].
    pub ms: ChainTiming,
    pub quads: usize,
    pub non_quads: usize,
    pub verts: usize,
    /// Arestas de bordo — numa peça fechada tem de ser `0`.
    pub boundary_edges: usize,
    /// Faces do MAPA que se dobraram. ⚠️ **Não são as da saída**: a extracção tolera a dobra por
    /// construção, e o que ela não pode é inventar grade onde o mapa se enrola sobre si próprio.
    pub folded: usize,
    /// O arredondamento deixou toda transição **inteira**?
    pub aligned: bool,
    /// A forma de cada face — a régua que a `line/sculpt3d` calibrou contra o oráculo.
    pub shape: ph2d_quadfill::QuadShape,
    /// ⭐ O que o acabamento fez — ver [`ph2d_quadfill::FinishReport`]. ⚠️ Ele carrega a
    /// forma **antes** dele, e é isso que torna o ganho legível sem uma segunda corrida.
    pub finish: ph2d_quadfill::FinishReport,
}

/// ⭐⭐⭐ **A CADEIA, do triângulo ao quad alinhado.**
///
/// `target_edge` é o comprimento de aresta que se quer na saída, **em unidades da malha de
/// entrada**.
///
/// # Errors
/// Ver [`ChainError`].
pub fn quads_from_mesh(
    reference: &Mesh,
    target_edge: f32,
) -> Result<(Mesh, ChainReport), ChainError> {
    let (mut out, mut report) = quads_from_mesh_raw(reference, target_edge)?;
    let clock = std::time::Instant::now();
    report.finish = ph2d_quadfill::finish_extracted(&mut out, reference);
    report.ms.finish = clock.elapsed().as_secs_f32() * 1000.0;
    report.shape = ph2d_quadfill::quad_shape(&out);
    Ok((out, report))
}

/// ⭐⭐⭐ **A CADEIA SEM O ACABAMENTO** — ver [`quads_from_mesh`], de que esta é a primeira
/// metade.
///
/// # ⛔⛔ Por que ela é pública, e não um detalhe
///
/// O veto de [`quads_or_keep_from`] tem duas metades, e **só a segunda precisa do
/// acabamento**: *«a peça continua fechada?»* é uma pergunta sobre a **topologia**, e uma
/// relaxação move vértices e mais nada — a contagem de arestas de bordo e não-manifold é
/// **idêntica** antes e depois dele (há gate).
///
/// ⚠️ **Sem esta porta, uma peça DURA pagava o acabamento inteiro para ser deitada fora.**
/// Medido: no cubo subdividido — o caso em que a cadeia perde por medição — a saída abre
/// arestas de bordo, o veto recusa, e o acabamento tinha corrido até ao tecto **duas vezes**
/// (a lei alinhada e a cega) sobre uma malha que ninguém ia usar.
///
/// # Errors
/// Ver [`ChainError`].
pub fn quads_from_mesh_raw(
    reference: &Mesh,
    target_edge: f32,
) -> Result<(Mesh, ChainReport), ChainError> {
    let mut ms = ChainTiming::default();
    let mut clock = std::time::Instant::now();
    let mut lap = |slot: &mut f32| {
        *slot = clock.elapsed().as_secs_f32() * 1000.0;
        clock = std::time::Instant::now();
    };

    // ── F1 — ver [`phase_zero`].
    let work = phase_zero(reference, target_edge);
    lap(&mut ms.remesh);

    // ── F2 (campo cruzado) + F3 (traçado dos patches) + G1/G2 (corte e penteado).
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e pedir à
    // `ph2d-gridmap` que o re-derive seria reconstruir o que já existe. ⚠️ Por isso ela é contada
    // no relógio do CAMPO: um `lap` posto onde a variável é usada em vez de onde ela é calculada
    // faz a coluna acusar a fase errada.
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&work, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    lap(&mut ms.field);

    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    lap(&mut ms.trace);
    let (cut, _) = ph2d_gridmap::cut_along_patches(&work, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(&work, &layout, &cut);
    lap(&mut ms.cut);

    // ── G3 + G5. O mapa, e o arredondamento uma-a-uma que o torna inteiro.
    let opts = ph2d_gridmap::RoundOptions::default();
    let (map, round) = if ph2d_gridmap::welded_enabled() {
        ph2d_gridmap::round_welded(&work, &cut, &combed, ph2d_gridmap::Step::uniform(target_edge), opts, &singular)
    } else {
        ph2d_gridmap::round_to_integers(&work, &cut, &combed, ph2d_gridmap::Step::uniform(target_edge), opts, &singular)
    };

    lap(&mut ms.map);

    // ── A extracção das isolinhas inteiras.
    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    let cm = ph2d_quadextract::CornerMap {
        pos: work.positions(),
        tris: &tris,
        uv: &uv,
    };
    let (out, e) = ph2d_quadextract::extract(&cm, None).map_err(ChainError::Extract)?;
    if out.faces().is_empty() {
        return Err(ChainError::TooCoarse);
    }
    lap(&mut ms.extract);

    let shape = ph2d_quadfill::quad_shape(&out);
    let report = ChainReport {
        ms,
        quads: e.quads,
        non_quads: out.face_count() - e.quads,
        verts: out.vert_count(),
        boundary_edges: boundary_edges(&out),
        folded: e.folded_faces,
        aligned: round.shift_frac_max == 0.0,
        shape,
        finish: ph2d_quadfill::FinishReport::default(),
    };
    Ok((out, report))
}

/// O que a cadeia decidiu sobre a peça — ver [`quads_or_keep`].
#[derive(Clone, Debug)]
pub enum Verdict {
    /// A cadeia correu e a saída é melhor. O relatório dela vai junto.
    Adopted(Box<ChainReport>),
    /// ⛔ A cadeia correu e **abriu a peça** — bordo ou aresta não-manifold onde não havia.
    Rejected {
        boundary: usize,
        non_manifold: usize,
    },
    /// A cadeia correu e **não melhorou a forma** — não há motivo para trocar a malha.
    NoGain { before: f32, after: f32 },
    /// A cadeia recusou.
    Refused(ChainError),
    /// ⛔ **A cadeia ESTOUROU.** Ver [`quads_or_keep`] — é um defeito a jusante, e esta porta
    /// existe para que ele não derrube quem pediu uma melhoria opcional.
    Panicked,
}

/// ⭐⭐⭐ **A CADEIA COM VETO** — corre, e só troca a malha se a troca for uma melhoria.
///
/// # Por que ela não é «corre sempre»
///
/// ⛔ **Medido** (`ph2d_field_eval::tests::the_quad_chain_turns_our_mesh_into_oracle_class`), sobre
/// a malha que o modelador implícito extrai:
///
/// | peça | extraída | pela cadeia | veredito |
/// |---|---|---|---|
/// | esfera | `1,48` / `26,6°` / 120 péssimas | ⭐ **`1,08` / `6,4°` / 4** | **a classe do oráculo** (`1,08` / `4,8–7,1°`) |
/// | toro | `1,49` / `24,8°` / 16 | `1,20` / `9,0°` / 9 | melhor |
/// | ⛔ **cubo rodado 45°** | `1,00` / **`0,0°`** / 0 | `1,35` / `17,9°` / 112 | **PIOR — e abre 6 arestas de bordo** |
///
/// ⭐ **A causa é geométrica e nomeável:** numa peça *dura* (faces planas, quinas vivas) a grade
/// dual **já é** a resposta certa — o quad dela pousa na face e sai a `0°`. O campo cruzado não tem
/// nada a que se alinhar numa face plana, e o que ele inventa é pior do que o que já havia.
/// *A cadeia é para a peça orgânica; a grade é para a peça dura.*
///
/// # As duas metades do veto, e nenhuma delas é um peso arbitrário
///
/// 1. ⛔ **Uma peça fechada continua fechada.** Bordo ou aresta não-manifold onde não havia é um
///    veto **duro**, não uma penalização: nenhum ganho de forma paga um buraco.
/// 2. Depois disso, troca-se **se a forma melhorar** (o enviesamento mediano desce).
///
/// ⚠️ *Uma regra de escolha com pesos seria uma opinião com números por cima; estas duas são
/// propriedades.*
///
/// # ⛔ E há uma PRÉ-condição, medida
///
/// A peça que entra tem de ser **fechada e manifold**. Não é zelo: uma calote faz o `ph2d-gridmap`
/// entrar em `panic!`, e um `panic` de uma crate a jusante derruba quem a chamou — um `Result` não
/// a salva. *Uma porta que não pode recusar tem de saber não entrar.*
///
/// # Errors
/// Nunca — a recusa da cadeia vira [`Verdict::Refused`] e a malha de entrada volta intacta.
#[must_use]
pub fn quads_or_keep(reference: &Mesh, target_edge: f32) -> (Mesh, Verdict) {
    quads_or_keep_from(reference, reference, target_edge)
}

/// ⭐⭐⭐ **A CADEIA COME UMA MALHA E OUTRA FICA SE ELA PERDER** — e as duas não têm de ser a mesma.
///
/// `feed` é a malha que entra na cadeia; `keep` é a que sai no arquivo quando o veto recusa, e é
/// contra ela que a melhoria se mede. [`quads_or_keep`] é o caso `feed == keep`.
///
/// # ⛔⛔ Por que a mais FINA não é a melhor entrada — medido 2026-08-25
///
/// A fase zero remalha para `target_edge` **venha a entrada de que densidade vier**, então tudo o
/// que uma grade mais fina traz a mais é deitado fora pelo F1 — depois de pago. E não é só preço:
///
/// | peça | grade | cadeia ms | quads | enviesamento | `\|f\|` máx (% da diagonal) |
/// |---|---|---|---|---|---|
/// | esfera | 6 | **4 612** | 2 539 | **6,4°** | **0,043** |
/// | esfera | 7 | 9 983 | 2 471 | 6,3° | 0,087 |
/// | esfera | 8 | 41 058 | ⛔ 320 | ⛔ **55,5°** | ⛔ **11,274** |
/// | duas caixas com filete | 6 | **10 191** | 2 920 | ⭐ **5,3°** | **0,699** |
/// | duas caixas com filete | 7 | 9 125 | 2 897 | 7,6° | 0,766 |
/// | duas caixas com filete | 8 | 16 429 | 2 971 | 8,0° | 0,812 |
/// | toro | 6 | **3 957** | 2 149 | 9,0° | **0,099** |
/// | toro | 7 | 5 033 | 2 196 | ⭐ 4,6° | 0,108 |
/// | toro | 8 | 22 373 | ⛔ 841 | 7,7° | ⛔ **6,400** |
///
/// ⭐ **A grade mais fina não é mais informação: é ruído que a cadeia tem de mastigar e depois
/// segue mal.** A fidelidade — medida no CAMPO, que é exacto — **piora** em todas as peças, e na
/// esfera e no toro a profundidade 8 destrói a peça. *Uma entrada 16× maior compra uma resposta
/// pior por 4 a 9× o preço.*
///
/// # Errors
/// Nunca — ver [`quads_or_keep`].
#[must_use]
pub fn quads_or_keep_from(feed: &Mesh, keep: &Mesh, target_edge: f32) -> (Mesh, Verdict) {
    let before = ph2d_quadfill::quad_shape(keep);
    // ⚠️ **A pré-condição é sobre quem ENTRA e o veto é sobre quem FICA** — são perguntas
    // diferentes: a primeira é «isto faz a cadeia estourar?», a segunda é «a troca piora o que o
    // artista ia levar?».
    let (bound_in, non_in) = edge_census(feed);
    let (bound_keep, non_keep) = edge_census(keep);
    // ⛔ **A CADEIA É PARA PEÇA FECHADA, e a pré-condição não é zelo: sem ela ela ESTOURA.**
    // Medido: uma calote (uma esfera sem as últimas fileiras) faz o `ph2d-gridmap` entrar em
    // `panic!` no `solve.rs`. ⚠️ Um `Result` não a salva — um `panic` de uma crate a jusante derruba
    // quem a chamou. *Uma porta que não pode recusar tem de saber não entrar.*
    if bound_in > 0 || non_in > 0 {
        return (
            keep.clone(),
            Verdict::Rejected {
                boundary: bound_in,
                non_manifold: non_in,
            },
        );
    }
    // ⛔⛔ **A CADEIA ESTOURA em malhas perfeitamente válidas, e o estouro é a jusante.**
    //
    // Medido: um **cubo subdividido** — fechado, manifold, 100 % quads — faz o `ph2d-gridmap`
    // entrar em `index out of bounds: the len is 129 but the index is 157`
    // (`solve.rs:336`, ao emparelhar os lados de uma costura). ⚠️ Não é uma pré-condição que se
    // possa conferir à porta: a malha satisfaz tudo o que se sabe exigir.
    //
    // ⭐ **E por isso esta porta apanha o estouro em vez de o propagar.** Ela oferece uma
    // MELHORIA OPCIONAL: um `panic` a jusante não pode derrubar quem exportou uma peça. O veto já
    // diz *"fica com a entrada a menos que a saída seja melhor"* — um estouro é só mais uma forma
    // de não ser melhor.
    //
    // ⛔ **Isto NÃO é a cura.** O defeito é do `ph2d-gridmap` e a linha dele está viva sobre aquele
    // arquivo (`line/quadextract`); tocá-lo daqui seria colisão de mesmo-símbolo. Ele está nomeado
    // no handoff, com a fixtura que o reproduz.
    // ⭐⭐⭐ **A CADEIA CRUA PRIMEIRO, e o acabamento só depois do veto de TOPOLOGIA.**
    //
    // ⚠️ **Não é uma optimização com risco: é a ordem certa.** Uma relaxação move vértices e
    // mais nada, então a contagem de arestas de bordo e não-manifold é a **mesma** antes e
    // depois do acabamento (gate `the_finishing_cannot_change_the_edge_census`) — decidir o
    // veto com a malha crua dá **exactamente** o mesmo veredito.
    //
    // ⛔ **Medido: sem esta ordem, uma peça DURA pagava o acabamento inteiro para ser
    // deitada fora** — no cubo subdividido, o caso em que a cadeia perde por medição, ele
    // corria até ao tecto **duas vezes** (a lei alinhada e a cega) sobre uma malha que
    // ninguém ia usar.
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        quads_from_mesh_raw(feed, target_edge)
    }));
    let Ok(ran) = ran else {
        return (keep.clone(), Verdict::Panicked);
    };
    match ran {
        Err(e) => (keep.clone(), Verdict::Refused(e)),
        Ok((mut out, mut r)) => {
            let (bound_out, non_out) = edge_census(&out);
            if bound_out > bound_keep || non_out > non_keep {
                return (
                    keep.clone(),
                    Verdict::Rejected {
                        boundary: bound_out,
                        non_manifold: non_out,
                    },
                );
            }
            // ⭐ Sobreviveu à topologia: agora vale a pena acabá-la.
            //
            // ⚠️ **A superfície é o `feed`, e não o `keep`, para que esta reordenação seja
            // PROVAVELMENTE neutra** — é o que [`quads_from_mesh`] usaria. ⏳ *Qual das duas
            // é a certa é uma pergunta em aberto e NÃO se responde aqui:* o `keep` é a malha
            // do nível que o artista escolheu (mais fiel) e o `feed` é a que entrou na
            // cadeia. Trocá-las é uma mudança de comportamento que pede a sua própria
            // medição, e misturá-la com uma correcção de custo esconderia as duas.
            let clock = std::time::Instant::now();
            r.finish = ph2d_quadfill::finish_extracted(&mut out, feed);
            r.ms.finish = clock.elapsed().as_secs_f32() * 1000.0;
            r.shape = ph2d_quadfill::quad_shape(&out);
            if r.shape.skew_p50 >= before.skew_p50 {
                return (
                    keep.clone(),
                    Verdict::NoGain {
                        before: before.skew_p50,
                        after: r.shape.skew_p50,
                    },
                );
            }
            (out, Verdict::Adopted(Box::new(r)))
        }
    }
}

/// ⛔ **A FASE ZERO — e ela não se salta.** Remalha isotropicamente para `target_edge` e triangula.
///
/// Sem ela, com a triangulação crua, a mesma cadeia dá **o dobro** do enviesamento final (medido).
///
/// # ⚠️ Ela remalha para o alvo que lhe DERAM, e isso é uma correcção de 2026-08-25
///
/// Até essa data o F1 passava `ph2d_remesh_iso::ALPHA` **fixo** enquanto o resto da cadeia
/// quantizava para o `target_edge` do argumento. Com o único chamador de então os dois números
/// coincidiam **por acidente** — ele passava exactamente `target_edge(mesh, ALPHA)` —, e o primeiro
/// chamador a pedir outra densidade teria a fase zero a remalhar para uma escala e o mapa a
/// quantizar para outra. *Um parâmetro que metade da função ignora só mente para o SEGUNDO
/// chamador.*
///
/// ⚠️ **Ela é pública porque é a PORTA por onde o gate a alcança.** O gate que prova esta lei não
/// pode medi-la pela saída da cadeia inteira: a jusante, um alvo grosso faz o `ph2d-gridmap` entrar
/// em `panic!` (`solve.rs:336`, defeito nomeado no handoff), e uma régua que atravessa um estouro
/// não é uma régua. *Medir a fase zero na fase zero é o que separa gatear a lei de gatear a
/// travessia inteira.*
#[must_use]
pub fn phase_zero(reference: &Mesh, target_edge: f32) -> Mesh {
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, alpha_for(reference, target_edge));
    work.triangulate();
    work
}

/// ⭐ **O `alpha` que reproduz `target` nesta malha** — o `ph2d_remesh_iso::target_edge` é
/// `alpha · diagonal_da_caixa`, então inverter é dividir.
///
/// ⚠️ **A caixa é a da malha de ENTRADA e isso é load-bearing**: o `remesh_isotropic` recalcula o
/// alvo sobre a malha que recebe, e ela é um clone desta — triangular não move a caixa. Uma
/// diagonal degenerada (malha vazia ou um ponto) cai no `ALPHA` da casa em vez de dividir por zero.
fn alpha_for(mesh: &Mesh, target: f32) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let diag = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    if diag.is_finite() && diag > 1.0e-6 {
        target / diag
    } else {
        ph2d_remesh_iso::ALPHA
    }
}

/// Quantas arestas da malha são tocadas por **uma** face só.
#[must_use]
pub fn boundary_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).0
}

/// Quantas arestas são tocadas por um número de faces **diferente de 2** — o censo de manifold.
#[must_use]
pub fn non_manifold_edges(mesh: &Mesh) -> usize {
    edge_census(mesh).1
}

fn edge_census(mesh: &Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.0;
        let n = if v[3] == v[2] { 3 } else { 4 };
        for k in 0..n {
            let (a, b) = (v[k], v[(k + 1) % n]);
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    (
        count.values().filter(|c| **c == 1).count(),
        count.values().filter(|c| **c > 2).count(),
    )
}
