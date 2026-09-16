//! Gates da LIMPEZA DA COSTURA — ver [`super::super::costura`].
//!
//! Filho (`#[path]`) do [`super`], e os ajudantes de fixtura são os dele.

use super::*;

/// O aspecto de cada triângulo de uma malha, ordenado.
fn aspectos(m: &Mesh) -> Vec<f32> {
    let p = m.positions();
    let mut tris = Vec::new();
    for f in m.faces() {
        f.triangles(&mut tris);
    }
    let mut v: Vec<f32> = tris
        .iter()
        .map(|t| aspecto(p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]))
        .collect();
    v.sort_by(f32::total_cmp);
    v
}

/// ⭐⭐⭐ **A COSTURA NÃO TEM TRIÂNGULOS IMPOSSÍVEIS** — report do dono
/// (2026-09-15): *«o algoritmo remesh produz bordas mais corretas que o Box
/// Trim; melhore a topologia das bordas do corte»*.
///
/// Medido na saída CRUA do motor: aspecto **`2 573 809`** e aresta mínima
/// **`8,74e-9`** numa malha cuja aresta é `0,0242` — vértices **duplicados** na
/// curva de interseção. *Um triângulo assim não tem normal utilizável, e é isso
/// que a borda mostra.*
///
/// ⚠️ **As duas metades medem coisas diferentes:** a aresta mínima apanha o
/// vértice duplicado (o defeito que a SOLDA cura) e o aspecto máximo apanha a
/// lasca (o que o COLAPSO cura). Uma só deixaria metade da cura sem régua.
#[test]
fn a_costura_nao_tem_triangulos_impossiveis() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let out = corta(&bola, &lamina(1.0, 0.6), Op::Subtrair).expect("o corte");
    let e = arestas(&out);
    let a = aspectos(&out);

    // ⭐⭐ **A BARRA É A PRÓPRIA PEÇA, e não um número escolhido:** o corte não
    // pode introduzir uma aresta pior do que a mais curta que a peça já tinha.
    //
    // ⚠️⚠️ **A 1.ª redacção usava a cerca da limpeza (`alvo × 0,2 × 0,5`) e
    // reprovava sobre produto CERTO:** a menor aresta da saída é `7,906e-4`, que
    // é **a menor aresta da esfera** — protegida de propósito pela cerca dos
    // dois vértices antigos. *Uma barra derivada do limiar da cura mede a cura;
    // a barra da PEÇA mede o que o artista recebe.*
    let da_peca = arestas(&bola)[0];
    assert!(
        e[0] >= da_peca * 0.99,
        "o corte introduziu uma aresta de {:.3e} numa peça cuja mais curta é \
         {da_peca:.3e} — a solda deixou passar um vértice duplicado",
        e[0]
    );
    // ⭐⭐⭐ **O CONTROLO é o mesmo corte SEM a limpeza**, e sem ele esta régua
    // seria um número solto: *uma barra sobre a saída curada não diz que a cura
    // fez alguma coisa.*
    let cru = corta_cru(&bola, &lamina(1.0, 0.6), Op::Subtrair).expect("o corte cru");
    let (pior, pior_cru) = (
        a.last().copied().unwrap_or(f32::INFINITY),
        aspectos(&cru).last().copied().unwrap_or(f32::INFINITY),
    );
    // ⚠️⚠️ **A barra ABSOLUTA é desta FIXTURA, que é a mais dura que esta crate
    // consegue montar:** aqui a lâmina é o cubo de SEIS faces, e o caminho do
    // produto entra sempre com a lâmina TESSELADA (a `ph2d-trim` pede
    // `Resolucao::Ate`), onde o mesmo corte mede **`33`**. *A régua do caminho
    // real vive onde a lâmina real é construída.*
    assert!(
        pior < 500.0,
        "a costura deixou um triângulo de aspecto {pior:.0} nesta fixtura \
         (medido `256` com a lâmina grossa) — antes da limpeza o pior era \
         `14 331`, e a régua existe para ele não voltar"
    );
    // ⚠️ **A barra do controlo é `10 ×` e a medição deu `56 ×`** (`14 331` contra
    // `256`) — ⛔ ela não é apertada até ao valor de hoje, porque o que ela
    // afirma é *«a limpeza faz alguma coisa»*, não *«faz exactamente isto»*.
    assert!(
        pior_cru > pior * 10.0,
        "CONTROLO: o corte CRU tinha de trazer um triângulo muito pior ({pior_cru:.0} \
         contra {pior:.0}) — se ele já não traz, a limpeza deixou de ser \
         necessária e pode sair"
    );
    assert!(
        arestas(&cru)[0] < da_peca * 0.5,
        "CONTROLO: o corte CRU tinha de trazer uma aresta muito mais curta que a \
         da peça ({:.3e} contra {da_peca:.3e})",
        arestas(&cru)[0]
    );
}

/// ⛔⛔ **A LIMPEZA NÃO ABRE A PEÇA NEM A TORNA NÃO-MANIFOLD.**
///
/// ⚠️ **Um colapso de aresta é exactamente a operação que pode fazer as duas
/// coisas** — ele funde dois vértices, e se eles partilharem mais do que a
/// aresta que os une nasce uma aresta com três faces. *Medir só o bordo deixaria
/// metade do risco por medir.*
#[test]
fn a_limpeza_nao_abre_nem_torna_a_peca_nao_manifold() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    for (nome, l) in [
        ("lâmina a meio", lamina(1.0, 0.6)),
        ("lâmina rasante", lamina(1.55, 0.6)),
        ("lâmina funda", lamina(0.4, 0.6)),
    ] {
        let out = corta(&bola, &l, Op::Subtrair).expect("o corte");
        assert_eq!(
            ph2d_mesh::border_edges(&out),
            0,
            "{nome}: a limpeza ABRIU a peça"
        );
        assert_eq!(
            ph2d_mesh::non_manifold_edges(&out),
            0,
            "{nome}: a limpeza deixou aresta com três faces"
        );
    }
}

/// ⛔⛔⛔ **A CERCA QUE TORNA A LIMPEZA SEGURA: uma aresta entre DOIS vértices
/// antigos é da PEÇA, e não nossa para tocar.**
///
/// ⚠️ **Sem ela a limpeza varre a malha inteira.** Medido: ela colapsava arestas
/// curtas naturais da esfera e **`1 251` de `14 136`** vértices longe do corte
/// deixavam de ser bit-idênticos — isto é, ela quebrava sozinha a propriedade
/// que decide a arquitectura desta linha, e o gate irmão
/// [`longe_do_corte_nenhum_vertice_se_move_um_bit`] reprovaria.
///
/// ⭐ Esta metade afirma a OUTRA ponta: a peça **tem** arestas abaixo do limiar,
/// logo a cerca não é decorativa — sem ela haveria o que colapsar.
#[test]
fn a_peca_tem_arestas_curtas_que_a_limpeza_nao_pode_tocar() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    let tris: usize = bola
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    let alvo = ph2d_mesh::edge_for_tri_count(bola.surface_area(), tris as f32);
    let limiar = alvo * costura::FRACCAO_DA_ARESTA;
    let curtas = arestas(&bola).into_iter().filter(|e| *e < limiar).count();
    assert!(
        curtas > 0,
        "a fixtura deixou de ter arestas abaixo do limiar ({limiar:.4}) — sem \
         elas a cerca dos DOIS antigos não tem o que defender, e este gate e o \
         irmão do bit passam por vácuo"
    );
}

/// ⛔⛔⛔ **TODO VÉRTICE DA PEÇA QUE SOBREVIVE AO CORTE ESTÁ NO SÍTIO, AO BIT.**
///
/// # Ele nasceu de DUAS mutações SOBREVIVENTES, e a causa era a FIXTURA
///
/// As duas cercas da limpeza da costura — *«uma aresta entre dois vértices
/// antigos é da PEÇA»* e *«quando um extremo é antigo, o sobrevivente é ELE»* —
/// passaram a suíte inteira ao serem apagadas. ⚠️ O gate irmão
/// [`longe_do_corte_nenhum_vertice_se_move_um_bit`] corre numa `uv_sphere(24,32)`,
/// **cuja aresta mais curta está ACIMA do limiar do colapso** ⇒ sem nada para
/// colapsar, apagar a cerca não é observável. *A fixtura não continha o
/// fenómeno* — a sexta vez que este módulo o escreve.
///
/// ⭐ **Esta corre na peça que TEM arestas curtas** (`sphere_with_triangles`), e
/// afirma a propriedade inteira em vez de metade dela: não é *«longe do corte
/// nada se move»*, é *«o que era da peça e ficou, ficou onde estava»* — o que
/// inclui os vértices **encostados à costura**, que é exactamente onde a
/// segunda cerca trabalha.
#[test]
fn todo_vertice_antigo_que_sobrevive_esta_no_sitio_ao_bit() {
    let bola = shapes::sphere_with_triangles(50_000, 1.0);
    // ⛔⛔ **FORA DA LÂMINA são as TRÊS dimensões, e a 1.ª redacção deste gate
    // filtrava só por `x < 0,4`.** A lâmina é o cubo `x ∈ [0,4; 1,6]`,
    // `|y| ≤ 0,6`, `|z| ≤ 0,6` — e os vértices antigos que a limpeza de facto
    // toca vivem em `x ≈ 0,64`, **fora do cubo pelas paredes de `y`/`z`**.
    // Medido com a mutação na mão: são `14` colapsos de par misto, e nenhum
    // deles caía no filtro antigo ⇒ o gate media a metade errada da peça e a
    // mutação da segunda cerca SOBREVIVIA.
    let dentro =
        |v: &[f32; 3]| (0.4..=1.6).contains(&v[0]) && v[1].abs() <= 0.6 && v[2].abs() <= 0.6;
    let fora: std::collections::BTreeSet<[u32; 3]> = bola
        .positions()
        .iter()
        .filter(|v| !dentro(v))
        .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
        .collect();
    assert!(
        fora.len() > 10_000,
        "a fixtura tem só {} vértices fora da lâmina — o gate mede quase nada",
        fora.len()
    );
    let out = corta(&bola, &lamina(1.0, 0.6), Op::Subtrair).expect("o corte");
    let saida: std::collections::BTreeSet<[u32; 3]> = out
        .positions()
        .iter()
        .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
        .collect();
    let perdidos = fora.difference(&saida).count();
    assert_eq!(
        perdidos,
        0,
        "{perdidos} de {} vértices da peça mudaram de sítio — a limpeza da \
         costura saiu da costura",
        fora.len()
    );
}

/// ⛔⛔ **UM DUPLICADO DE MESMO ENROLAMENTO NÃO É UMA ALMOFADA, e a limpeza não
/// lhe toca.**
///
/// # Ele nasceu de uma MUTAÇÃO SOBREVIVENTE
///
/// Trocar a detecção de *«pares ESPELHADOS»* por *«qualquer trio repetido»*
/// passava a suíte inteira — nenhuma fixtura do corte produz um duplicado de
/// mesmo enrolamento, logo a distinção não era observável. *Uma linha que a
/// mutação não consegue matar não é lei, é comentário com sintaxe de código.*
///
/// ⚠️ **E a distinção é load-bearing:** dois triângulos espelhados encerram
/// volume **ZERO** e saem os DOIS; dois com o mesmo enrolamento são a MESMA
/// face escrita duas vezes, e descartar ambos **abre um buraco**. *Curar o
/// segundo é outra lei, e ela não vive aqui.*
#[test]
fn um_duplicado_de_mesmo_enrolamento_nao_e_uma_almofada() {
    // Um tetraedro fechado, com UMA das faces escrita duas vezes.
    let p = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let base = [
        Face::tri(0, 2, 1),
        Face::tri(0, 1, 3),
        Face::tri(1, 2, 3),
        Face::tri(0, 3, 2),
    ];
    let limpo = corta_dupla(&p, &base, Face::tri(0, 2, 1));
    assert_eq!(
        limpo, 5,
        "a limpeza mexeu num duplicado de MESMO enrolamento — ele não é uma \
         almofada, e descartar os dois lados abriria a peça"
    );
    // ⭐ E o CONTROLO: o ESPELHO da mesma face sai, e saem os dois.
    let espelhada = corta_dupla(&p, &base, Face::tri(0, 1, 2));
    assert_eq!(
        espelhada, 3,
        "o par espelhado tinha de sair INTEIRO (as duas faces)"
    );
}

/// Monta `base + extra` e devolve quantas faces a limpeza deixou.
fn corta_dupla(p: &[[f32; 3]], base: &[Face], extra: Face) -> usize {
    let mut faces = base.to_vec();
    faces.push(extra);
    let suja = Mesh::from_parts(p.to_vec(), faces).expect("a fixtura");
    let peca = Mesh::from_parts(p.to_vec(), base.to_vec()).expect("a peça");
    limpa_a_costura(&suja, &peca).faces().len()
}
