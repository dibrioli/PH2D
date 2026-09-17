//! ⭐⭐⭐ **OS GATES DO ASSADOR DO BIND** (F9 W1) — ver o cabeçalho do [`super`].
//!
//! ⚠️ **Eles medem a LEI ([`super::assar`]) e não a PORTA ([`super::assar_no_bind`])**, e a razão
//! está escrita lá: a env var é lida uma vez por processo (`OnceLock`), logo um gate que a
//! escrevesse mediria o que o vizinho já tinha fixado. O que se afirma da porta é a única coisa que
//! se pode afirmar sem a escrever: **ela nasce desligada**.

use super::*;

/// Uma grelha `n × n` sobre a arte de `320 × 96` — a forma de uma malha de bind.
fn grelha(n: usize) -> Mesh2d {
    let lado = n + 1;
    let mut rest = Vec::with_capacity(lado * lado);
    for j in 0..lado {
        for i in 0..lado {
            rest.push([i as f64 * 320.0 / n as f64, j as f64 * 96.0 / n as f64]);
        }
    }
    let mut tris = Vec::with_capacity(n * n * 2);
    for j in 0..n {
        for i in 0..n {
            let a = (j * lado + i) as u32;
            let (b, c, d) = (a + 1, a + lado as u32, a + lado as u32 + 1);
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    Mesh2d {
        rest,
        tris,
        size: [320, 96],
    }
}

/// Dois ossos, com a troca toda numa banda de `12 px` — a forma de uma pele de esqueleto.
fn pesos_articulados(m: &Mesh2d) -> Vec<f64> {
    let mut out = Vec::with_capacity(m.rest.len() * 2);
    for p in &m.rest {
        let u = ((p[0] - 160.0) / 12.0).clamp(-1.0, 1.0);
        let t = f64::midpoint(u, 1.0);
        let w = t * t * (3.0 - 2.0 * t);
        out.push(1.0 - w);
        out.push(w);
    }
    out
}

/// ⛔⛔ **A PORTA NASCE DESLIGADA** — é o pedido da W1 (*«sem mudar o que se vê»*), e é o que faz
/// esta wave não pôr mais vértices para a CPU deformar antes de a W2 pagar por eles.
///
/// ⚠️ **O CONTROLO está na segunda metade:** a mesma entrada, pela LEI, assa. Sem ele este gate
/// passaria sobre um assador partido — *«não fez nada» e «está desligado» são o mesmo `None`*.
#[test]
fn a_porta_nasce_desligada_e_a_lei_por_tras_dela_funciona() {
    let m = grelha(16);
    let pesos = pesos_articulados(&m);
    assert!(
        assar_no_bind(&m, &pesos, 2).is_none(),
        "a porta do assador nasceu LIGADA — a W1 nao pode mudar o que se ve'"
    );
    assert!(
        assar(&m, &pesos, 2).is_some(),
        "a LEI por tras da porta nao assa nada — o `None` acima nao prova que a porta esta' \
         desligada, so' que o assador esta' partido"
    );
}

/// ⭐⭐ **A ASSADURA PARTE ONDE O PESO CURVA e devolve os pesos re-interpolados.**
#[test]
fn a_assadura_parte_e_devolve_um_peso_por_osso_por_vertice() {
    let m = grelha(16);
    let pesos = pesos_articulados(&m);
    let (assada, novos) = assar(&m, &pesos, 2).expect("o campo articulado curva");
    assert!(
        assada.tris.len() > m.tris.len(),
        "a assadura nao partiu nada ({} pecas)",
        assada.tris.len()
    );
    assert_eq!(
        novos.len(),
        assada.rest.len() * 2,
        "a tabela de pesos tem de fechar com a malha — e' a invariante que o `SkinnedMesh::valida` \
         cobra, e uma tabela curta ali e' a forma de a arte sumir"
    );
    // ⭐ **A PARTIÇÃO DA UNIDADE sobrevive à re-interpolação** — a lei de Hermite é linear nos
    // valores e nos gradientes, logo ela conserva a soma **sem renormalizar** (o doc do
    // `attr_law` mede `|Σ − 1| ≤ 4e-16`). Um vértice novo cuja soma fugisse mudaria a escala da
    // arte naquele ponto.
    let pior = (0..assada.rest.len())
        .map(|v| (novos[v * 2] + novos[v * 2 + 1] - 1.0).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        pior < 1e-9,
        "a particao da unidade partiu-se num vertice novo (pior desvio {pior:e})"
    );
    // ⚠️ E o tecto é honrado: a escada mede `4,5 ×` a esta tolerância, contra um tecto de `8 ×`.
    assert!(
        assada.tris.len() <= m.tris.len() * CRESCIMENTO_MAX,
        "a assadura passou o tecto de crescimento"
    );
}

/// ⛔ **SEM PESOS não há campo para perseguir** — é a 1.ª mídia, que resolve pela lei derivada.
#[test]
fn sem_pesos_nao_ha_o_que_assar() {
    let m = grelha(8);
    assert!(assar(&m, &[], 0).is_none(), "assou uma malha sem pesos");
    assert!(
        assar(
            &Mesh2d {
                rest: Vec::new(),
                tris: Vec::new(),
                size: [1, 1]
            },
            &[],
            2
        )
        .is_none(),
        "assou uma malha vazia"
    );
}

/// ⛔⛔ **UM CAMPO LINEAR DEVOLVE `None`, e não uma cópia da entrada.**
///
/// ⚠️ *Devolver a entrada faria o chamador gravar bytes novos que descrevem a mesma malha* — e o
/// `SkinBind::source` é o que atravessa o ficheiro do projecto.
#[test]
fn um_campo_linear_devolve_none_em_vez_de_uma_copia() {
    let m = grelha(16);
    let pesos: Vec<f64> = m
        .rest
        .iter()
        .flat_map(|p| {
            let w = (p[0] / 320.0).clamp(0.0, 1.0);
            [1.0 - w, w]
        })
        .collect();
    assert!(
        assar(&m, &pesos, 2).is_none(),
        "um campo de pesos LINEAR nao tem curvatura, e a assadura devolveu uma malha nova"
    );
}

/// ⛔⛔ **O BIND CHAMA O ASSADOR** — e este gate é TEXTUAL de propósito, com a razão dita.
///
/// # Porque não há gate de comportamento aqui, hoje
///
/// A porta nasce desligada, logo `assar_no_bind` devolve `None` e **guardar o resultado ou ignorá-lo
/// dá exactamente os mesmos bytes**. *Uma mutação que apague a chamada não é observável enquanto a
/// porta estiver fechada* — e inventar um gate que finja o contrário seria pior que nenhum.
///
/// ⇒ o que se pode afirmar hoje é a FORMA, por [`include_str!`] (que deixa de **compilar** se o
/// ficheiro mudar de sítio). A metade de comportamento chega com a W2, quando a porta abrir.
///
/// ⚠️⚠️ **E ele existe porque o modo de falha é um CORTE**, que é a forma exacta que mordeu neste
/// repo em 2026-09-17: um corpo separado do guarda dele compila, o clippy cala-se e as suítes
/// ficam verdes.
#[test]
fn o_bind_da_imagem_chama_o_assador() {
    const BIND: &str = include_str!("skin_live.rs");
    assert!(
        BIND.contains("skin_bake::assar_no_bind("),
        "o bind da imagem deixou de chamar o assador — a W1 da F9 ficou sem chamador, e uma porta \
         sem chamador e' uma lei viva e orfa"
    );
    // Controlo positivo: é mesmo o ficheiro do bind, e ele guarda a malha.
    assert!(
        BIND.contains("SkinBind::new(bytes, tendoes)"),
        "este gate devia estar a medir o ficheiro que ESCREVE o bind — perdeu o sujeito"
    );
}
