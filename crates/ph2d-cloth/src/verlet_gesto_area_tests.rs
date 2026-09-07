//! **OS GATES DA NORMAL E DO CENTRO DA ÁREA** (espec §4.2-bis e §4.4) — irmão
//! (`#[path]`) do [`super::verlet_gesto_tests`], cortado pelo MESMO assunto que
//! separa o [`super::verlet_gesto_area`] do ficheiro do gesto.
//!
//! ⭐ Uma varredura, duas grandezas: a normal da área é a direcção do Push e o
//! `ẑ` do referencial local; o centro da área é o PONTO por onde o plano de
//! queda passa. Os dois saem do mesmo disco de meio raio e do mesmo desempate
//! por baldes — e é por saírem juntos que os gates deles vivem juntos.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O `verlet_gesto_tests.rs`
//! cruzou os 700 do `architecture_workspace_file_loc_cap` na wave da colisão, e o
//! que saiu foi a metade com fronteira própria.

/// ⭐⭐⭐ **A NORMAL DA ÁREA só vê METADE do raio** (espec §4.2-bis (3)).
///
/// ⛔ **É a diferença que deixou o `plano_empurrar_plano_local` a errar `0,944`**:
/// a nossa lei somava as normais do disco INTEIRO, e o alvo amostra num disco de
/// `R · 0,5`. Num plano em repouso as duas leituras coincidem — *e é por isso
/// que a fixtura de um passo saía ao bit e o traço inteiro não*.
#[test]
fn a_normal_da_area_so_ve_metade_do_raio() {
    let raio = 1.0;
    let cursor = [0.0, 0.0, 0.0];
    let vista = [0.0, 0.0, 1.0];
    // Dois anéis: um DENTRO de `R/2` e outro fora, com normais bem diferentes.
    let pos = vec![[0.1, 0.0, 0.0], [0.9, 0.0, 0.0]];
    let dentro_do_raio = [0.0, 0.0, 1.0];
    // ⚠️ **O de fora tem de estar no MESMO balde**, senão a regra dos baldes
    // mascara a do raio e o gate sobrevive a trocar `R/2` por `R` — foi o que
    // aconteceu na primeira redacção, e a mutação passou.
    let fora = [0.6, 0.0, 0.8];
    let normais = vec![dentro_do_raio, fora];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    // O de fora nao pode ter entrado: a resposta e' a normal do de dentro.
    for c in 0..3 {
        assert!(
            (n[c] - dentro_do_raio[c]).abs() < 1e-12,
            "a normal da area leu o vertice a {:.2}R do cursor: {n:?}",
            0.9
        );
    }
    // Controlo: com o de dentro FORA da lista, a resposta muda -- senao este
    // gate estaria verde por o segundo vertice nunca contar para nada.
    let so_o_de_fora =
        crate::verlet_gesto::normal_da_area(&pos, &normais, &[1], cursor, raio * 4.0, vista);
    assert!(
        so_o_de_fora != n,
        "o vertice de fora nao conta nem quando o raio o alcanca -- gate vacuo"
    );
}

/// ⭐⭐ **UM vértice virado para a vista apaga todos os virados ao contrário**
/// (espec §4.2-bis (4)) — e a resposta NÃO é uma média nem «o balde com mais».
#[test]
fn um_vertice_virado_para_a_vista_apaga_os_de_costas() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // Três de costas e UM de frente, todos dentro de `R/2`.
    let pos = vec![
        [0.05, 0.0, 0.0],
        [0.0, 0.05, 0.0],
        [-0.05, 0.0, 0.0],
        [0.0, -0.05, 0.0],
    ];
    let costas = [0.0, 0.0, -1.0];
    let frente = [0.0, 0.0, 1.0];
    let normais = vec![costas, costas, costas, frente];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1, 2, 3], cursor, raio, vista);
    assert!(
        n[2] > 0.9,
        "tres de costas ganharam a UM de frente: {n:?} -- a regra nao e' a maioria"
    );
    // Controlo: sem o de frente, a resposta e' a dos de costas.
    let so_costas =
        crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1, 2], cursor, raio, vista);
    assert!(so_costas[2] < -0.9, "sem o de frente: {so_costas:?}");
}

/// ⭐⭐ **Sem balde válido a normal é NULA**, e o teste é *não-vazio E soma
/// não-nula*, balde a balde (espec §4.2-bis (4) e (5)).
///
/// ⚠️⚠️ **E há uma coisa que a redacção da espec não deixa ver, e que escrever
/// este gate revelou: o balde da FRENTE, se não estiver vazio, NUNCA tem soma
/// nula.** A soma dele é `Σ wᵢ n̂ᵢ` com todos os `wᵢ > 0` e todos os `n̂ᵢ · v̂ > 0`,
/// logo `(Σ wᵢ n̂ᵢ) · v̂ > 0` e o vector não pode ser zero. ⇒ *a cláusula «e soma
/// não-nula» só é observável no balde de TRÁS*, onde `n̂ · v̂ ≤ 0` admite o zero e
/// duas normais perpendiculares à vista podem cancelar-se. É por isso que a
/// primeira redacção deste gate era IMPOSSÍVEL de satisfazer.
#[test]
fn sem_balde_valido_a_normal_da_area_e_nula() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // (a) Ninguém dentro do alcance ⇒ nulo.
    let pos = vec![[0.9, 0.0, 0.0]];
    let normais = vec![[0.0, 0.0, 1.0]];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0], cursor, raio, vista);
    assert_eq!(
        n, [0.0; 3],
        "sem vertice no alcance a normal tem de ser nula"
    );

    // (b) Balde da frente VAZIO e o de trás com soma não-nula ⇒ ganha o de trás.
    let pos = vec![[0.05, 0.0, 0.0], [0.0, 0.05, 0.0]];
    let atras = [0.0, 0.0, -1.0];
    let normais = vec![atras, atras];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    assert!(n[2] < -0.9, "o balde de tras devia ter respondido: {n:?}");

    // (c) ⭐ O caso que a cláusula «soma não-nula» existe para apanhar: os dois
    // vértices são PERPENDICULARES à vista (`n̂ · v̂ = 0` ⇒ balde de trás) e
    // cancelam-se ⇒ nenhum balde passa, e a resposta é o vector NULO.
    let normais = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    assert_eq!(
        n, [0.0; 3],
        "o balde de tras cancelou-se e a resposta nao foi nula: {n:?} -- sem esta \
         clausula o `unit` de um vector nulo devolveria NaN"
    );
}

/// ⭐ **A normal da área PESA cada vértice por `3p² − 2p³`**, com
/// `p = 1 − d/(R·0,5)` — o de perto do cursor conta mais (espec §4.2-bis (3)).
///
/// ⛔⛔ **Este gate existe porque a mutação que apaga o peso SOBREVIVEU à
/// paridade inteira**, e não por ser inofensiva: sem peso, cinco traços do
/// oráculo mudam (`plano_empurrar_radial_local` `0,214 → 0,194`), e todos ficam
/// **ligeiramente melhores**. ⚠️ *Isso não absolve a soma crua — absolve o peso
/// de ser a causa do resíduo do Push, e diz que há outra coisa a compensá-lo.*
/// A lei que shipa é a da espec, que foi lida no fonte e atestada; a medição
/// fica registada no plano do que falta.
///
/// A fixtura é a mais simples que os separa: duas normais diferentes a
/// distâncias diferentes, dentro do meio-raio e no MESMO balde.
#[test]
fn a_normal_da_area_pesa_o_vertice_pela_distancia() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    // `d = 0,05` ⇒ `p = 0,9` ⇒ peso `0,972`; `d = 0,45` ⇒ `p = 0,1` ⇒ `0,028`.
    let pos = vec![[0.05, 0.0, 0.0], [0.45, 0.0, 0.0]];
    let perto = [0.0, 0.0, 1.0];
    let longe = [1.0, 0.0, 0.2];
    let normais = vec![perto, longe];
    let n = crate::verlet_gesto::normal_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    // Com o peso, o de longe quase não conta: a resposta fica muito perto do
    // de perto. Sem peso ela cairia a meio caminho entre os dois.
    let sem_peso = {
        let s = [
            perto[0] + longe[0],
            perto[1] + longe[1],
            perto[2] + longe[2],
        ];
        crate::verlet::unit(s)
    };
    assert!(
        n[0] < 0.15,
        "o vertice a 0,45 pesou como o de 0,05: {n:?} -- a soma e' crua"
    );
    assert!(
        (n[0] - sem_peso[0]).abs() > 0.3,
        "a resposta com peso ({n:?}) nao se distingue da soma crua ({sem_peso:?}) \
         -- a fixtura nao separa as duas leis"
    );
}

/// ⭐⭐⭐ **O CENTRO DA ÁREA não é o centroide do disco: cada vértice entra na
/// média já PUXADO PARA O CURSOR** (espec §4.4).
///
/// ```text
/// contribuição(v) = c + (p_v − c) · (1 − a_v)      a_v = 3p² − 2p³
/// ```
///
/// ⇒ o peso `1 − a` vale **zero no cursor** e cresce para a borda: um vértice
/// colado ao cursor é quase inteiramente **substituído** por ele, e um vértice
/// na borda entra quase como ele próprio. *É a mistura das duas metades que
/// separa esta lei de um centroide, e nenhuma delas sozinha o faz.*
///
/// ⚠️⚠️ **Este gate existe porque a medição de 06/09 respondeu à pergunta
/// errada:** «o plano pelo cursor reproduz o alvo e o plano pelo centro da área
/// afasta-o» foi medido com um CENTROIDE (`empurrar 0,944 → 1,250`), e o alvo
/// não usa um centroide. O plano pelo cursor é a aproximação de **primeira
/// ordem** desta lei — é por isso que ele passava quase.
#[test]
fn o_centro_da_area_puxa_cada_vertice_para_o_cursor() {
    let (raio, cursor, vista) = (1.0, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let frente = [0.0, 0.0, 1.0];
    let alcance = raio * crate::verlet_gesto::RAIO_DA_NORMAL;

    // (a) COLADO ao cursor (`d = 0,1·alcance` ⇒ `a = 0,972`): o vértice é
    // substituído pelo cursor a 97,2 %. Um centroide leria `0,05`.
    let pos = vec![[0.05, 0.0, 0.0]];
    let (_, c) =
        crate::verlet_gesto::normal_e_centro_da_area(&pos, &[frente], &[0], cursor, raio, vista);
    assert!(
        c[0] < 0.05 * 0.05,
        "o centro da area leu a POSICAO do vertice colado ao cursor ({c:?}) -- \
         um centroide daria 0,05 e a lei da' 0,0014"
    );

    // (b) NA BORDA do disco (`d = 0,98·alcance` ⇒ `a = 0,0012`): ele entra
    // quase como ele próprio. ⛔ Sem esta metade, devolver sempre o cursor
    // passaria em (a).
    let borda = 0.98 * alcance;
    let pos = vec![[borda, 0.0, 0.0]];
    let (_, c) =
        crate::verlet_gesto::normal_e_centro_da_area(&pos, &[frente], &[0], cursor, raio, vista);
    assert!(
        (c[0] - borda).abs() < borda * 0.01,
        "o vertice da BORDA devia entrar quase como ele proprio: {c:?} contra {borda}"
    );

    // (c) ⭐ O centro sai do MESMO balde que a normal (§4.2-bis (4)), e não de
    // um desempate proprio: o de tras esta' mais perto e mesmo assim nao conta.
    let pos = vec![[0.45, 0.0, 0.0], [0.05, 0.0, 0.0]];
    let normais = vec![frente, [0.0, 0.0, -1.0]];
    let (_, c) =
        crate::verlet_gesto::normal_e_centro_da_area(&pos, &normais, &[0, 1], cursor, raio, vista);
    assert!(
        (c[0] - 0.4374).abs() < 1e-4,
        "o centro misturou os dois baldes: {c:?} -- so' o da frente conta, e ele \
         da' 0,4374 (a media dos dois daria 0,2194)"
    );
}

/// ⭐ **Sem vértice nenhum no disco, o centro da área é o CURSOR** (espec §4.4)
/// — e não a origem, nem o último centro, nem `NaN`.
#[test]
fn sem_vertice_no_disco_o_centro_da_area_e_o_cursor() {
    let (raio, vista) = (1.0, [0.0, 0.0, 1.0]);
    let cursor = [0.7, -0.2, 0.3];
    // O único vértice está muito além do meio-raio.
    let pos = vec![[9.0, 9.0, 9.0]];
    let (n, c) = crate::verlet_gesto::normal_e_centro_da_area(
        &pos,
        &[[0.0, 0.0, 1.0]],
        &[0],
        cursor,
        raio,
        vista,
    );
    assert_eq!(n, [0.0; 3], "sem vertice no disco a normal e' nula");
    assert_eq!(
        c, cursor,
        "sem vertice no disco o centro da area tem de ser o CURSOR, nao {c:?}"
    );
}
