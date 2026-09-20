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
