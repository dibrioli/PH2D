//! Os gates do `rig.bones`. ⚠️ **A régua de todos eles é a mesma:** o osso desenhado a partir do
//! quadro que este nó devolve tem de ir de uma junta à SEGUINTE — e o CONTROLO é a corrente crua,
//! onde essa mesma conta falha. *Sem o controlo, um nó que não fizesse nada passaria no gate de
//! ladrilhamento numa cadeia recta, onde as duas leituras coincidem.*

use super::*;

/// Graus → radianos, com a mesma aritmética dos gates da família (a lei do nó não usa trig
/// nenhuma; quem precisa dela é a RÉGUA, para andar ao longo do osso).
fn dir(graus: f32) -> [f32; 2] {
    let r = graus.to_radians();
    [r.cos(), r.sin()]
}

/// A corrente que o `rig.skeleton` emite, já RESOLVIDA — `n` juntas, cada uma a `len` da
/// anterior, virada `passo` graus em relação a ela. ⚠️ **As colunas são as do contrato do rig**
/// (`fk.rs`): `rot` é o ângulo de MUNDO do osso que CHEGA, e `P[0]` é a raiz.
fn corrente(n: usize, len: f32, passo: f32, raiz: f32) -> Stream {
    let mut p = vec![[0.0f32; 2]; n];
    let mut w = vec![0.0f32; n];
    w[0] = raiz;
    for i in 1..n {
        w[i] = w[i - 1] + passo;
        let d = dir(w[i]);
        p[i] = [p[i - 1][0] + len * d[0], p[i - 1][1] + len * d[1]];
    }
    #[expect(clippy::cast_precision_loss, reason = "Index/Count sao f32")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    Stream::new(n)
        .with(
            PARENT,
            #[expect(clippy::cast_precision_loss, reason = "o pai viaja num f32")]
            Column::Scalar((0..n).map(|i| i as f32 - 1.0).collect::<Vec<_>>()),
        )
        .with(
            "len",
            Column::Scalar(
                (0..n)
                    .map(|i| if i == 0 { 0.0 } else { len })
                    .collect::<Vec<_>>(),
            ),
        )
        .with("rot", Column::Scalar(w.clone()))
        .with(WROT, Column::Scalar(w))
        .with(LROT, Column::Scalar(vec![passo; n]))
        .with(INDEX, Column::Scalar(idx))
        .with(COUNT, Column::Scalar(vec![n as f32; n]))
        .with("P", Column::Vec2(p))
}

fn escalar(s: &Stream, nome: &str) -> Vec<f32> {
    match s.get(nome) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("sem coluna `{nome}`"),
    }
}

fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem P"),
    }
}

/// ⭐⭐⭐ **O OSSO VAI DE UMA JUNTA À SEGUINTE — e na corrente crua ele NÃO vai.**
///
/// Report do dono (2026-09-19): *«assim skeleton:bend coloca a base de um osso na ponta do outro.
/// Contudo o mais correto seria se tivesse o mesmo resultado colocando na base do osso»*.
///
/// A régua anda o comprimento do osso a partir do quadro devolvido e exige aterrar na junta
/// seguinte. ⛔ **A segunda metade é o discriminador**: a MESMA conta sobre a corrente crua erra
/// — é ela que impede que um nó que devolvesse a entrada intacta passasse aqui.
#[test]
fn o_osso_vai_de_uma_junta_a_seguinte_e_na_corrente_crua_nao_vai() {
    let (n, len, passo) = (6usize, 0.6f32, 30.0f32);
    let cru = corrente(n, len, passo, 0.0);
    let juntas = posicoes(&cru);
    let saida = bones(&cru);

    let p = posicoes(&saida);
    let rot = escalar(&saida, "rot");
    assert_eq!(
        p.len(),
        n - 1,
        "uma corrente de {n} juntas tem {} ossos",
        n - 1
    );
    for k in 0..p.len() {
        // A CABEÇA é a junta `k`.
        assert!(
            (p[k][0] - juntas[k][0]).abs() < 1e-5 && (p[k][1] - juntas[k][1]).abs() < 1e-5,
            "o osso {k} tem de pender da junta {k}: {:?} contra {:?}",
            p[k],
            juntas[k]
        );
        // E a PONTA é a junta `k + 1`.
        let d = dir(rot[k]);
        let ponta = [p[k][0] + len * d[0], p[k][1] + len * d[1]];
        assert!(
            (ponta[0] - juntas[k + 1][0]).abs() < 1e-5
                && (ponta[1] - juntas[k + 1][1]).abs() < 1e-5,
            "o osso {k} tem de acabar na junta {}: {ponta:?} contra {:?}",
            k + 1,
            juntas[k + 1]
        );
    }

    // ⛔ O CONTROLO: a mesma conta na corrente CRUA, onde `P[i]` é a PONTA e não a cabeça.
    let rot_cru = escalar(&cru, "rot");
    let d = dir(rot_cru[1]);
    let ponta_crua = [juntas[1][0] + len * d[0], juntas[1][1] + len * d[1]];
    let erro = (ponta_crua[0] - juntas[2][0]).hypot(ponta_crua[1] - juntas[2][1]);
    assert!(
        erro > 0.2 * len,
        "o controlo tem de FALHAR: sem este no' a peca erra {erro} de um osso de {len}"
    );
}

/// ⛔ **A RAIZ SAI, e a contagem é a dos OSSOS** — era ela a peça pendurada para fora da corrente
/// nas fotos do dono. Com uma ÁRVORE (duas raízes) a conta é `n − r`, que é o que prova que a
/// regra é *«quem não tem pai»* e não *«o elemento zero»*.
#[test]
fn a_raiz_sai_e_a_contagem_e_a_dos_ossos() {
    assert_eq!(bones(&corrente(6, 0.6, 30.0, 0.0)).count(), 5);
    // Uma corrente de UMA junta não tem osso nenhum — e isso é honesto, não um nó partido.
    assert_eq!(bones(&corrente(1, 0.6, 30.0, 0.0)).count(), 0);

    // Duas raízes: os elementos 0 e 3 não têm pai.
    let n = 6;
    let arvore = corrente(n, 0.6, 30.0, 0.0)
        .with(PARENT, Column::Scalar(vec![-1.0, 0.0, 1.0, -1.0, 3.0, 4.0]));
    assert_eq!(
        bones(&arvore).count(),
        n - 2,
        "duas raizes ⇒ dois ossos a menos"
    );
}

/// ⚠️ **Num stream que não é um rig ele é a IDENTIDADE** (doc 39) — sem coluna `parent` não há
/// árvore. ⛔ A leitura ingénua (*«sem pais ⇒ sem ossos»*) esvaziaria um `motion.grid`.
#[test]
fn numa_corrente_que_nao_e_um_rig_ele_e_a_identidade() {
    let nuvem = Stream::new(3)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]))
        .with("rot", Column::Scalar(vec![10.0, 20.0, 30.0]));
    let saida = bones(&nuvem);
    assert_eq!(saida.count(), 3, "uma nuvem de pontos sobrevive inteira");
    assert_eq!(posicoes(&saida), posicoes(&nuvem));
    assert_eq!(escalar(&saida, "rot"), escalar(&nuvem, "rot"));
}

/// ⛔⛔ **As três colunas da CORRENTE não viajam** — ver a decisão (2) do cabeçalho. A metade que
/// interessa é o `lrot`: com ele presente, o degrau 3 da escada do `fk::local` faria um `rig.fk` a
/// jusante reescrever `rot` com o ângulo LOCAL, e a cadeia saía desenhada com os ângulos
/// relativos, calada.
#[test]
fn as_colunas_da_corrente_nao_viajam() {
    let saida = bones(&corrente(6, 0.6, 30.0, 0.0));
    for nome in [PARENT, LROT, WROT] {
        assert!(
            saida.get(nome).is_none(),
            "a coluna `{nome}` nao pode viajar para uma lista de carimbos"
        );
    }
    // ⚠️ E o que É do osso fica: o ângulo de mundo e o comprimento.
    assert!(saida.get("rot").is_some() && saida.get("len").is_some());
}

/// ⚠️ **`Index`/`Count` são RE-CONTADOS** — a população passou de juntas a ossos, e um `Count` a
/// dizer `n` sobre `n − 1` linhas faz toda a normalização a jusante endereçar o elemento errado.
/// ⛔ E só se já existiam: escrevê-los onde não estavam cria colunas que viajam e mudam o que um
/// nó a jusante vê.
#[test]
fn o_index_e_o_count_sao_os_dos_ossos() {
    let saida = bones(&corrente(6, 0.6, 30.0, 0.0));
    assert_eq!(escalar(&saida, INDEX), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    assert_eq!(escalar(&saida, COUNT), vec![5.0; 5]);

    // Sem as colunas na entrada, elas não nascem aqui.
    let sem = Stream::new(3)
        .with(PARENT, Column::Scalar(vec![-1.0, 0.0, 1.0]))
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let saida = bones(&sem);
    assert!(saida.get(INDEX).is_none() && saida.get(COUNT).is_none());
}

/// ⚠️ **O que o artista pendurou no elemento viaja com o OSSO** (a cor, o falloff, o que for) — e
/// é o valor do FILHO, que é quem carrega o `rot` e o `len` daquele osso. *Colher pelo pai poria
/// a cor de uma junta no osso da vizinha.*
#[test]
fn as_outras_colunas_viajam_pelo_filho() {
    let cru = corrente(4, 0.6, 30.0, 0.0).with("falloff", Column::Scalar(vec![9.0, 1.0, 2.0, 3.0]));
    assert_eq!(escalar(&bones(&cru), "falloff"), vec![1.0, 2.0, 3.0]);
}

// ---------------------------------------------------------------------------
// O QUADRO DE UM OSSO — a lei de 2026-09-20 (ordem do dono sobre a corda).
// ---------------------------------------------------------------------------

/// Uma corrente CRUA: só posições e `parent`, como uma corda de Verlet as publica.
/// ⚠️ Nem `rot` nem `len` — é essa ausência que a lei nova responde.
fn corrente_crua(pos: &[[f32; 2]]) -> Stream {
    #[expect(
        clippy::cast_precision_loss,
        reason = "indice de pai num f32, como a familia"
    )]
    let parent: Vec<f32> = (0..pos.len())
        .map(|i| if i == 0 { -1.0 } else { (i - 1) as f32 })
        .collect();
    Stream::new(pos.len())
        .with("P", Column::Vec2(pos.to_vec()))
        .with("parent", Column::Scalar(parent))
}

/// ⭐⭐⭐ **O QUADRO DE UM OSSO SAI DO SEGMENTO, quando a corrente não o traz.**
///
/// Uma corda publica posições e a corrente delas, e mais nada. Sem esta lei cada peça carimbada
/// saía **sem rodar** — vinte traços deitados na horizontal onde o dono pediu um cordão. A régua
/// é exacta: uma cotovelada de `(1,0)` seguida de `(0,1)` dá `0°` e `90°`, com comprimento `1`.
///
/// ⛔⛔ **E O `90` É A CORRECÇÃO, com a premissa morta à vista:** a 1.ª redacção deste gate exigia
/// `FRAC_PI_2` — ele nasceu na mesma wave que a lei e herdou a unidade errada dela, logo os dois
/// concordavam um com o outro e discordavam do PRODUTO (o dono fotografou a corda por rodar).
/// *Um gate escrito na mesma hora que a lei que ele cobre não é uma segunda opinião.*
///
/// ⚠️ **A régua é a da FAMÍLIA** (`dir`, que converte graus): ela já existia neste ficheiro e era
/// o que qualquer fixtura daqui usava — a lei nova é que falava noutra língua.
///
/// FALSIFICADO por apagar o `derive_frame` (não há coluna `rot` nenhuma na saída) ou por devolver
/// o `atan2` cru (o segundo lê `1,5708` contra `90`).
#[test]
fn o_quadro_de_um_osso_sai_do_segmento_quando_a_corrente_nao_o_traz() {
    let s = bones(&corrente_crua(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]));
    assert_eq!(s.count(), 2, "tres juntas dao dois ossos");
    let rot = escalar(&s, "rot");
    assert!(
        (rot[0] - 0.0).abs() < 1e-4,
        "o primeiro aponta para +x: {rot:?}"
    );
    assert!(
        (rot[1] - 90.0).abs() < 1e-4,
        "o segundo aponta para +y, em GRAUS: {rot:?}"
    );
    // ⭐ A metade que fecha a unidade: o ângulo derivado anda pela MESMA régua que a corrente
    // resolvida usa, logo a direcção que ele descreve é a do segmento.
    let d = dir(rot[1]);
    assert!(
        d[0].abs() < 1e-6 && (d[1] - 1.0).abs() < 1e-6,
        "a direccao lida do angulo derivado e' a do segmento: {d:?}"
    );
    let len = escalar(&s, "len");
    assert!(
        len.iter().all(|l| (l - 1.0).abs() < 1e-6),
        "cada osso mede o proprio segmento: {len:?}"
    );
}

/// ⛔⛔ **O CONTROLO, e é ele que justifica a lei ser CONDICIONAL:** numa corrente RESOLVIDA o
/// `rot` e o `len` já existem, foram calculados pelo solver, e a saída fica com **os números
/// dele** — não com um `atan2` que diria quase o mesmo com erro de vírgula flutuante.
///
/// ⚠️ A régua é a IGUALDADE AO BIT com o que a corrente trazia: *«quase igual» é exactamente o que
/// se estaria a introduzir na família toda se a lei fosse incondicional.*
///
/// FALSIFICADO por derivar sempre (os valores passam a diferir do `wrot` do solver).
#[test]
fn uma_corrente_resolvida_mantem_o_quadro_que_o_solver_calculou() {
    let cru = corrente(4, 0.6, 30.0, 0.0);
    let antes = escalar(&cru, "rot");
    let s = bones(&cru);
    let depois = escalar(&s, "rot");
    // O `colhe` re-amostra pelo FILHO de cada osso, que são os elementos `1..n`.
    assert_eq!(
        depois,
        antes[1..].to_vec(),
        "o angulo do solver viaja ao bit, e nao um atan2 parecido"
    );
}

/// ⚠️ **As duas metades são independentes:** uma corrente que traz o comprimento e não o ângulo
/// fica com o comprimento dela e com o ângulo derivado. FALSIFICADO por escrever as duas em bloco
/// (o `len` pregado seria substituído).
#[test]
fn cada_metade_do_quadro_decide_por_si() {
    let cru = corrente_crua(&[[0.0, 0.0], [3.0, 0.0]]).with("len", Column::Scalar(vec![7.0, 7.0]));
    let s = bones(&cru);
    assert_eq!(
        escalar(&s, "len"),
        vec![7.0],
        "o comprimento pregado sobrevive"
    );
    assert!(
        escalar(&s, "rot")[0].abs() < 1e-6,
        "e o angulo em falta e' derivado na mesma"
    );
}

/// ⭐⭐⭐ **A PEÇA VESTE O OSSO** — ordem do dono (2026-09-20: *«DEVE SIM»*), e a lei está no doc
/// do [`veste`], com a tabela do defeito que ela cura.
///
/// ⚠️ **As DUAS metades medem EIXOS diferentes, e são leis diferentes:**
///
/// 1. o **comprimento** de cada osso é metade do `len` DELE (o ½ é o contrato da receita do
///    `source.shape`: toda forma é cortada de uma caixa de largura `2 × size`);
/// 2. a **espessura** é metade do MENOR `len` da cadeia — a mesma para todos.
///
/// ⛔⛔⛔ **A METADE 2 DIZIA O CONTRÁRIO ATÉ 2026-09-20, e a premissa dela está morta à vista no
/// diff.** Ela exigia *«o MESMO nos dois eixos — a peça ESCALA, e uma escala é um par»*, e o
/// report do dono no mesmo dia derrubou-a: *«porque a corda afina no final?»*. Numa corda o `len`
/// é um facto do SOLVER (o segmento de cima suporta o peso dos de baixo e estica), e com a escala
/// uniforme a espessura esticava com ele — `0,721×` do topo à ponta a `Count = 80`. *O comprimento
/// é do osso; a espessura é da cadeia.* O mecanismo e as duas tabelas estão no doc do [`veste`].
///
/// ⚠️⚠️ **E a fixtura tem `len` VARIADO de propósito:** numa cadeia uniforme — que é o que as
/// outras fixturas desta crate e **quatro das cinco cadeias do produto** têm — as duas leis dão o
/// MESMO par, e um gate escrito ali passaria verde sobre qualquer uma das duas. *Uma fixtura que
/// não contém o fenómeno não distingue a lei da lei anterior.*
///
/// ⛔ **E o CONTROLO é a nuvem:** num stream que não é um rig o nó é a identidade, logo ele **não
/// pode inventar um `size`** — *uma lei que alcança quem não é osso muda o desenho de um
/// `motion.grid` em silêncio*.
#[test]
fn a_peca_veste_o_osso() {
    // Três ossos de `1,0`, `0,5` e `1,5` — o menor é `0,5`, e nenhum dos outros lhe é igual.
    let saida = bones(&corrente_crua(&[
        [0.0, 0.0],
        [1.0, 0.0],
        [1.5, 0.0],
        [3.0, 0.0],
    ]));
    let Some(Column::Vec2(size)) = saida.get("size") else {
        panic!("a lista de ossos traz a coluna `size`")
    };
    let len = escalar(&saida, "len");
    assert_eq!(size.len(), saida.count(), "um `size` por osso");
    let menor = len.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (menor - 0.5).abs() < 1e-6 && len.iter().any(|l| (l - menor).abs() > 0.4),
        "a fixtura CONTEM o fenomeno: lens {len:?}, menor {menor:.6}"
    );
    for (i, (s, l)) in size.iter().zip(&len).enumerate() {
        assert!(
            (s[0] - l * 0.5).abs() < 1e-6,
            "o comprimento do osso {i} e' o `len` DELE: {:.6} contra {:.6}",
            s[0],
            l * 0.5
        );
        assert!(
            (s[1] - menor * 0.5).abs() < 1e-6,
            "a espessura do osso {i} e' a da CADEIA: {:.6} contra {:.6}",
            s[1],
            menor * 0.5
        );
    }
    // ⭐ E o CONTROLO da degenerescência: com `len` uniforme as duas leis coincidem ao bit, que é
    // o que faz a saída das quatro cadeias uniformes do produto ser byte-idêntica à aprovada.
    let uniforme = bones(&corrente(6, 0.6, 30.0, 0.0));
    let Some(Column::Vec2(su)) = uniforme.get("size") else {
        panic!("a cadeia uniforme tambem veste")
    };
    for (i, s) in su.iter().enumerate() {
        assert!(
            (s[0] - s[1]).abs() < 1e-6 && (s[0] - 0.3).abs() < 1e-6,
            "numa cadeia uniforme o osso {i} degenera na escala uniforme: [{:.6}, {:.6}]",
            s[0],
            s[1]
        );
    }
    // ⛔ O CONTROLO — sem `parent` não há osso, logo não há tamanho de osso a escrever.
    let nuvem = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]));
    assert!(
        bones(&nuvem).get("size").is_none(),
        "numa nuvem que nao e' um rig o no' nao inventa um `size`"
    );
}
