//! ⭐ **A FORÇA POR PASSO DAS ÂNCORAS É ZERADA EM TODA A MALHA** (espec §4.3).
//!
//! ⚠️ **Este ficheiro existe porque a lei sobreviveu a uma mutação.** Apagar o
//! zeramento não mexeu em NENHUM dos 50 traços do oráculo, e a razão é
//! estrutural: a relaxação não filtra por vértice activo — quem apaga a
//! contribuição de um vértice longe é o `φ`, que já é zero fora da banda. As
//! duas leis tapam-se uma à outra em todo o corpus.
//!
//! ⚠️ **Mas elas medem coisas DIFERENTES, e o caso que as separa existe:** a
//! área é decidida sobre as posições **ACTUAIS** e o `φ` sobre as de
//! **REPOUSO**. Um vértice bastante deformado pode sair da área (posição actual
//! longe do cursor) e continuar com `φ > 0` (repouso perto) — e aí a marca
//! velha dele seria aplicada outra vez, contra a espec. *Um corpus que nunca
//! deforma o bastante para separar duas leis não testa nenhuma das duas.*

use crate::V3;
use crate::verlet_gesto::{Area, Modo, Passo, Pincel, PincelTecido};

/// Uma grelha `n × n` no plano `z = 0`, com passo `h`.
fn grelha(n: usize, h: f64) -> (Vec<V3>, Vec<Vec<u32>>) {
    let meio = (n - 1) as f64 * h * 0.5;
    let mut p = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            p.push([i as f64 * h - meio, j as f64 * h - meio, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            faces.push(vec![
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]);
        }
    }
    (p, faces)
}

fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(u32::try_from(q).unwrap_or(u32::MAX));
            a[q].push(u32::try_from(p).unwrap_or(u32::MAX));
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// **GATE — nenhum vértice guarda a força por passo de um passo anterior.**
///
/// O arnês monta o caso que o corpus não tem: área *Dynamic* (o disco anda com
/// o cursor), modo de âncora, e um traço longo o bastante para que vértices
/// puxados no início fiquem para trás. Depois do último passo, todo vértice
/// fora do disco DESTE passo tem de ter `σ = 0`.
///
/// ⚠️ **A régua é a espec, não a nossa aritmética:** `σ` é *«a força por passo,
/// zerada em todo o objecto antes de ser reescrita»*, então o que se afirma é
/// uma propriedade do estado no fim do passo, e não um número.
///
/// ⛔⛔ **E o CENTRO da régua é o do MODO, não o cursor** (corrigido em 07/09).
/// A 1.ª redacção media a distância ao **cursor deste passo** nos dois modos, e
/// para o Agarrar isso é a pergunta errada: a §4.3 diz que ali `σ` é reescrito
/// *«só para quem o raio INICIAL alcança, que é o conjunto fixo em que a âncora
/// nasceu»* — o disco do Agarrar fica no pen-down.
///
/// ⚠️⚠️ **Ela passava por um ACIDENTE que a lei do mesmo dia removeu:** enquanto
/// a área *Dynamic* seguia o cursor, os vizinhos do pen-down **caíam fora do
/// conjunto simulado** à medida que a mão se afastava, e o `σ.fill(0)` deixava-os
/// a zero por não haver quem os reescrevesse. Com a área do Agarrar fixa no
/// pen-down (a lei que o `esfera_agarrar_radial_dinamica` mediu) eles ficam, e
/// mantêm o `σ = 1` que a espec lhes dá. *Um gate verde pela razão errada só se
/// distingue de um verde no dia em que a razão errada é curada.*
#[test]
fn nenhum_vertice_guarda_a_forca_por_passo_de_um_passo_anterior() {
    for modo in [Modo::Agarrar, Modo::Gancho] {
        let (rest, faces) = grelha(41, 0.05);
        let an = aneis(rest.len(), &faces);
        let anel = |v: u32| an[v as usize].clone();
        let pincel = Pincel {
            modo,
            area: Area::Dinamica,
            raio: 0.20,
            ..Pincel::default()
        };
        let r = pincel.raio;
        let mut pos = rest.clone();
        let inicio = [-0.5, 0.0, 0.0];
        let mut tecido = PincelTecido::pen_down(pincel, &pos, inicio, Vec::new());
        let mut cursor = inicio;
        let normais = vec![[0.0, 0.0, 1.0]; rest.len()];
        for k in 0..24 {
            let delta = if k == 0 { [0.0; 3] } else { [0.05, 0.0, 0.10] };
            cursor = [
                cursor[0] + delta[0],
                cursor[1] + delta[1],
                cursor[2] + delta[2],
            ];
            let passo = Passo {
                cursor,
                delta,
                // A grelha do arnês vive em `z = 0` e a vista é ao longo de `z`
                // ⇒ a projecção é um no-op e os dois deltas coincidem ao bit.
                delta_3d: delta,
                parado: k == 0,
                vista: [0.0, 0.0, 1.0],
                normais: &normais,
                pressao: 1.0,
            };
            if tecido.passo(&pos, &anel, &passo) {
                for (v, act) in tecido.sim.activo.iter().enumerate() {
                    if *act {
                        pos[v] = tecido.sim.x[v];
                    }
                }
            }
        }
        // Controlo anti-vácuo: o traço tem de ter DEFORMADO, senão «σ = 0 em
        // toda a parte» é verdade num pincel morto.
        let movidos = pos
            .iter()
            .zip(&rest)
            .filter(|(a, b)| crate::verlet::norm([a[0] - b[0], a[1] - b[1], a[2] - b[2]]) > 1e-9)
            .count();
        assert!(movidos > 100, "{modo:?}: só {movidos} movidos — vácuo");
        // E tem de haver quem esteja FORA do disco deste passo com σ escrito
        // num passo anterior, senão o gate não olha para nada.
        // ⭐ **E são DUAS escolhas, não uma:** o Agarrar mede as posições de
        // **REPOUSO** contra o pen-down — é o conjunto fixo em que a âncora
        // nasceu —, e o Snake Hook mede as de **agora** contra o cursor.
        // *Perguntar a mesma coisa aos dois é perguntar pelo modo errado num
        // deles*, e foi o que esta régua fez até 07/09.
        let (centro, no_repouso) = if modo == Modo::Agarrar {
            (inicio, true)
        } else {
            (cursor, false)
        };
        let mut fora = 0usize;
        for v in 0..pos.len() {
            let p = if no_repouso { rest[v] } else { pos[v] };
            let d = crate::verlet::norm([p[0] - centro[0], p[1] - centro[1], p[2] - centro[2]]);
            if d >= r {
                fora += 1;
                assert!(
                    tecido.sim.sigma[v] == 0.0,
                    "{modo:?}: vertice {v} esta a {d:.4} do centro do modo (raio {r}) e guarda \
                     forca por passo {} -- a espec §4.3 manda zerar em TODO o objecto",
                    tecido.sim.sigma[v]
                );
            }
        }
        assert!(
            fora > 100,
            "{modo:?}: só {fora} vértices fora do disco — vácuo"
        );
    }
}

/// ⭐⭐⭐ **A ORDEM DE NASCIMENTO por vértice é LEI** (espec §5.2 nº 1, emenda
/// Q14): **corpo mole → estruturais → âncora → pino**.
///
/// ⛔⛔ **Não há «primeiro as distâncias, depois as âncoras».** As quatro
/// espécies vivem numa lista SÓ, percorrida de fio a pavio cinco vezes, e a
/// espécie só é lida DENTRO da projecção. Como Gauss–Seidel não comuta — nesta
/// bancada, inverter a lista move a nossa resposta `0,29` num traço que erra
/// `0,07` —, *onde cada restrição cai decide o resultado tanto quanto o que ela
/// diz*.
///
/// ⚠️ **O corpus quase não a vê, e é por isso que este gate existe:** só dois
/// traços ligam o pino ou a plasticidade, e adoptar a ordem certa move-os
/// `0,068 → 0,074` e `0,040 → 0,043` — dentro da barra nos dois casos, logo o
/// gate de paridade fica verde com a ordem ERRADA lá dentro.
#[test]
fn as_quatro_especies_nascem_pela_ordem_da_referencia() {
    use crate::verlet::Alvo;
    // ⚠️ A grelha é FINA de propósito: o pino só nasce nos vértices que caem na
    // janela `[R(1+L·F), R(1+L)]` da banda, e numa grelha grossa essa janela
    // pode não conter vértice nenhum — foi o que a 1.ª fixtura fez.
    let (pos, faces) = grelha(11, 0.06);
    let an = aneis(11 * 11, &faces);
    let cursor = [0.0, 0.0, 0.0];
    let mut p = Pincel {
        modo: Modo::Gancho,
        area: Area::Local,
        // ⚠️ O raio é pequeno de propósito: o pino só nasce onde a BANDA já
        // caiu, e com um raio que cubra a grelha inteira ela vale `1` em toda a
        // parte e a espécie não chega a existir.
        raio: 0.1,
        pino: true,
        ..Pincel::default()
    };
    p.solver.plasticidade = 0.5;
    let mut t = PincelTecido::pen_down(p, &pos, cursor, Vec::new());
    let passo = Passo {
        cursor,
        delta: [0.05, 0.0, 0.0],
        delta_3d: [0.05, 0.0, 0.0],
        parado: false,
        vista: [0.0, 0.0, 1.0],
        normais: &vec![[0.0, 0.0, 1.0]; pos.len()],
        pressao: 1.0,
    };
    t.passo(&pos, &|v| an[v as usize].clone(), &passo);

    // A sequência de espécies, como um código por vértice.
    let especie = |a: &crate::verlet::Restricao| match a.b {
        Alvo::Memoria => 'm',
        Alvo::Vertice(_) => 'e',
        Alvo::Ancora => 'a',
        Alvo::Repouso => 'p',
    };
    let toda: String = t.sim.restricoes.iter().map(especie).collect();
    assert!(
        !toda.is_empty(),
        "nenhuma restricao nasceu -- a fixtura nao produz o fenomeno"
    );
    // ⚠️ A área *Local* constrói a lista DUAS vezes (§5.2-bis), e a emenda entre
    // as duas passagens é uma fronteira de bloco, não uma troca de ordem. As
    // duas metades são idênticas por construção — o que também se afirma aqui.
    assert_eq!(
        toda.len() % 2,
        0,
        "a lista da Local tem de vir em duplicado"
    );
    let (a, b) = toda.split_at(toda.len() / 2);
    assert_eq!(a, b, "as duas passagens da Local nao sao identicas");
    let seq = a.to_string();
    // As quatro espécies têm de existir, senão o gate é vácuo.
    for (c, nome) in [
        ('m', "corpo mole"),
        ('e', "estrutural"),
        ('a', "ancora"),
        ('p', "pino"),
    ] {
        assert!(
            seq.contains(c),
            "a fixtura nao produz {nome} -- gate vacuo sobre essa especie"
        );
    }
    // ⭐ A lei: o bloco de cada vértice é `m` · `e`+ · `a`? · `p`?, e nunca outra
    // permutação.
    //
    // ⚠️ **A leitura tem de ser por BLOCO, não por par de letras** — a 1.ª
    // redacção proibia `am` e `pm`, que são precisamente as **fronteiras** entre
    // dois vértices, e reprovava sobre uma sequência correcta. *Um censo de
    // transições sobre uma lista de blocos acusa as fronteiras dele.*
    let mut blocos = seq.split('m');
    assert_eq!(
        blocos.next(),
        Some(""),
        "a sequencia nao comeca pelo corpo mole: {seq}"
    );
    let mut n_blocos = 0;
    for bloco in blocos {
        n_blocos += 1;
        let mut it = bloco.chars().peekable();
        let mut estruturais = 0;
        while it.peek() == Some(&'e') {
            it.next();
            estruturais += 1;
        }
        assert!(
            estruturais > 0,
            "bloco `m{bloco}`: alguma coisa nasceu ANTES das estruturais"
        );
        if it.peek() == Some(&'a') {
            it.next();
        }
        if it.peek() == Some(&'p') {
            it.next();
        }
        assert!(
            it.next().is_none(),
            "bloco `m{bloco}` fora da ordem `corpo mole -> estruturais -> ancora -> pino`"
        );
    }
    assert!(
        n_blocos > 1,
        "um bloco so' nao testa a fronteira entre vertices"
    );
}

/// ⭐⭐ **O desvio de repouso do Expand entra nas QUATRO espécies, e INTEIRO nas
/// três de alvo próprio** (espec §5.2 e §4.5, emenda Q14).
///
/// A soma é sempre «metade do desvio de cada extremo»; quando os dois extremos
/// são o **mesmo** vértice, as duas metades somam o desvio dele por completo.
///
/// ⚠️ **O corpus não o vê:** `τ` só é diferente de zero no Expand, e nenhuma
/// fixture combina Expand com pino ou com plasticidade — embora as duas
/// combinações sejam alcançáveis com o pincel de tecido (são opções
/// independentes do modo). *Uma lei que o corpus não alcança precisa de um gate
/// que a alcance.*
#[test]
fn o_desvio_de_repouso_entra_inteiro_nas_especies_de_alvo_proprio() {
    use crate::verlet::{Solver, Verlet};
    let solver = Solver {
        varreduras: 1,
        ..Solver::default()
    };
    // Um vértice deslocado de `D = 0,10` da posição de repouso, com (ou sem) um
    // pino a puxá-lo de volta.
    let correu = |tau: f64, com_pino: bool| {
        let mut v = Verlet::nascer(vec![[0.0, 0.0, 0.0]]);
        if com_pino {
            v.pregar(0, 1.0);
        }
        v.phi[0] = 1.0;
        v.activo[0] = true;
        v.x[0] = [0.10, 0.0, 0.0];
        v.tau[0] = tau;
        v.passo(&solver);
        v.x[0][0]
    };
    let inerte = correu(0.0, false);
    assert!(
        (correu(0.0, true) - inerte).abs() > 1e-9,
        "sem desvio o pino tinha de puxar -- a fixtura nao produz o fenomeno"
    );
    // ⭐ **A régua é o ponto em que a restrição fica SATISFEITA**, que é exacto e
    // não depende do resto do passo: com `ℓ' = τ` INTEIRO ela cala-se quando
    // `τ = D`. Com `τ/2` só se calaria a `τ = 2D`, e a `τ = D` ainda puxava.
    assert!(
        (correu(0.10, true) - inerte).abs() < 1e-12,
        "a `τ = D` o pino ainda puxou ({} contra {inerte}) -- o desvio nao entra \
         INTEIRO na especie de alvo proprio",
        correu(0.10, true)
    );
    // Controlo: a METADE de `D` ele ainda tem de puxar, senão o gate acima
    // estaria verde por o desvio ter apagado a restrição em toda a parte.
    assert!(
        (correu(0.05, true) - inerte).abs() > 1e-9,
        "a `τ = D/2` o pino deixou de puxar -- o desvio esta' a apagar a restricao"
    );
}

/// ⭐⭐⭐ **GATE — O AGARRAR NÃO SEGUE O CURSOR, nem sequer na área *Dynamic***
/// (espec §2.1 · §4.3; lei MEDIDA em 2026-09-07).
///
/// A área simulada do Grab fica onde o traço começou, com o raio do 1.º passo,
/// nas **três** áreas — se ela seguisse o cursor, material NOVO entraria na
/// simulação a meio do traço, que é exactamente o que *pegar num conjunto fixo*
/// exclui (§4.3).
///
/// ⚠️ **A régua é a PORTA, e não um traço:** [`PincelTecido::localizacao_da_area`]
/// tem de devolver o pen-down e o raio inicial para o Agarrar em qualquer área.
/// ⛔ **E o anti-vácuo é a outra metade:** para os outros sete modos a *Dynamic*
/// **tem** de seguir o cursor, senão este gate estaria a afirmar que a área
/// dinâmica não existe.
#[test]
fn o_agarrar_nao_segue_o_cursor_nem_na_area_dinamica() {
    let pos = vec![[0.0, 0.0, 0.0]];
    let inicio = [1.0, 2.0, 3.0];
    let cursor = [9.0, 9.0, 9.0];
    for area in [Area::Local, Area::Global, Area::Dinamica] {
        let t = PincelTecido::pen_down(
            Pincel {
                modo: Modo::Agarrar,
                area,
                raio: 0.5,
                ..Pincel::default()
            },
            &pos,
            inicio,
            Vec::new(),
        );
        let (c, r) = t.localizacao_da_area(cursor);
        assert_eq!(
            (c, r),
            (inicio, 0.5),
            "Agarrar em {area:?}: a area foi para {c:?} com raio {r} -- ela fica \
             no pen-down, e e' isso que faz o Grab pegar num conjunto FIXO"
        );
    }
    // ⛔ O anti-vácuo: nos outros modos a área *Dynamic* segue o cursor.
    let mut seguem = 0usize;
    for modo in [
        Modo::Arrastar,
        Modo::Empurrar,
        Modo::ApertarPonto,
        Modo::ApertarLinha,
        Modo::Inflar,
        Modo::Gancho,
        Modo::Expandir,
    ] {
        let t = PincelTecido::pen_down(
            Pincel {
                modo,
                area: Area::Dinamica,
                raio: 0.5,
                ..Pincel::default()
            },
            &pos,
            inicio,
            Vec::new(),
        );
        seguem += usize::from(t.localizacao_da_area(cursor).0 == cursor);
    }
    assert_eq!(
        seguem, 7,
        "so' {seguem} dos sete outros modos seguem o cursor em Dynamic -- se \
         nenhum seguisse, este gate afirmava que a area dinamica nao existe"
    );
}

/// ⭐⭐⭐ **GATE 52 — A BASE PERSISTENTE ENTRA EM QUATRO LEITURAS E EM NENHUMA
/// MAIS** (espec §14 gate 52, §6.4).
///
/// O censo mede-se pelo que MUDA quando a base difere do repouso. As quatro que
/// **têm** de mudar: o comprimento de repouso estrutural · o filtro de raio que
/// decide quem entra · o teste e o peso da âncora radial do Agarrar · a condição
/// de criação do pino. As que **têm de ficar intactas**: os **alvos** da âncora,
/// do pino e da memória de forma (que apontam para o repouso do TRAÇO) e a
/// **banda `w`** nas varreduras e na integração.
///
/// ⚠️ **Um port que faça a banda ler a base muda a fronteira do movimento sem
/// mudar o máximo** — *o defeito vive no anel de `2,875 R` a `3,5 R`*, e só uma
/// régua de malha inteira o vê (a lição do gate 43). É por isso que este gate
/// olha para os alvos e para `φ`, e não só para o resultado.
#[test]
fn a_base_persistente_entra_em_quatro_leituras_e_em_nenhuma_mais() {
    let (rest, faces) = grelha(21, 0.10);
    let an = aneis(rest.len(), &faces);
    let anel = |v: u32| an[v as usize].clone();
    // Uma base DIFERENTE do repouso: a folha inteira empurrada em `z`, o que
    // muda toda distância de vértice a vértice e ao centro da área.
    let base: Vec<V3> = rest
        .iter()
        .map(|p| [p[0] * 1.3, p[1] * 1.3, p[2] + 0.2])
        .collect();
    let pincel = Pincel {
        modo: Modo::Agarrar,
        area: Area::Local,
        raio: 0.35,
        pino: true,
        ..Pincel::default()
    };
    let normais = vec![[0.0, 0.0, 1.0]; rest.len()];
    let correr = |com_base: bool| {
        let mut pos = rest.clone();
        let mut t = PincelTecido::pen_down(pincel, &pos, [0.0; 3], Vec::new());
        if com_base {
            t.sim.base.clone_from(&base);
        }
        for k in 0..3 {
            let delta = if k == 0 { [0.0; 3] } else { [0.06, 0.0, 0.0] };
            let passo = Passo {
                cursor: [0.06 * k as f64, 0.0, 0.0],
                delta,
                delta_3d: delta,
                parado: k == 0,
                vista: [0.0, 0.0, 1.0],
                normais: &normais,
                pressao: 1.0,
            };
            if t.passo(&pos, &anel, &passo) {
                for (v, act) in t.sim.activo.iter().enumerate() {
                    if *act {
                        pos[v] = t.sim.x[v];
                    }
                }
            }
        }
        (pos, t)
    };
    let (_, sem) = correr(false);
    let (_, com) = correr(true);

    // ── O que TEM de mudar ────────────────────────────────────────────────
    // (1) o comprimento de repouso das estruturais.
    let l_de = |t: &PincelTecido| -> Vec<f64> {
        t.sim
            .restricoes
            .iter()
            .filter(|r| matches!(r.b, crate::verlet::Alvo::Vertice(_)))
            .map(|r| r.l)
            .collect()
    };
    assert_ne!(
        l_de(&sem),
        l_de(&com),
        "leitura 1: o comprimento de repouso estrutural nao leu a base"
    );
    // (2) o filtro de raio ⇒ quem tem restrições construídas.
    let construidos = |t: &PincelTecido| t.sim.construido.iter().filter(|c| **c).count();
    assert_ne!(
        construidos(&sem),
        construidos(&com),
        "leitura 2: o filtro de raio nao leu a base -- ela decide QUEM entra"
    );
    // (3) o teste e a força da âncora radial do Agarrar.
    let ancoras = |t: &PincelTecido| -> Vec<f64> {
        t.sim
            .restricoes
            .iter()
            .filter(|r| matches!(r.b, crate::verlet::Alvo::Ancora))
            .map(|r| r.s)
            .collect()
    };
    assert_ne!(
        ancoras(&sem),
        ancoras(&com),
        "leitura 3: a ancora radial do Agarrar nao leu a base"
    );
    // (4) a condição de criação do pino.
    let pinos = |t: &PincelTecido| {
        t.sim
            .restricoes
            .iter()
            .filter(|r| matches!(r.b, crate::verlet::Alvo::Repouso))
            .count()
    };
    assert_ne!(
        pinos(&sem),
        pinos(&com),
        "leitura 4: a condicao do pino nao leu a base"
    );

    // ── E o que NÃO pode mudar ────────────────────────────────────────────
    // ⛔ Os ALVOS das três espécies de alvo próprio apontam para o repouso do
    // TRAÇO — a base muda a rede e o comprimento dela, nunca o alvo.
    assert_eq!(
        com.sim.repouso, rest,
        "o repouso do traco foi contaminado pela base -- ele e' o alvo do pino e \
         da memoria de forma"
    );
    // ⛔ E a banda `φ` mede sempre sobre o repouso do traço: um port que a faça
    // ler a base muda a fronteira do movimento sem mudar o máximo.
    assert_eq!(
        sem.sim.phi, com.sim.phi,
        "a banda `phi` leu a base -- ela mede sempre sobre o repouso do traco, e \
         o defeito viveria no anel de 2,875R a 3,5R"
    );
    assert_eq!(
        sem.sim.w_repouso, com.sim.w_repouso,
        "a retencao de banda leu a base"
    );
}

/// ⛔⛔ **GATE — SEM BASE, TUDO LÊ O REPOUSO AO BIT** (espec §6.4: as três
/// maneiras de a opção ser um no-op EXACTO).
///
/// ⚠️ **É a metade que impede a leitura preguiçosa do gate acima:** ele afirma
/// que quatro coisas mudam com a base, e este afirma que **nenhuma** muda sem
/// ela. Sem os dois, uma porta que lesse a base sempre passaria o primeiro.
#[test]
fn sem_base_a_construcao_le_o_repouso_ao_bit() {
    let (rest, _) = grelha(9, 0.20);
    let mut sim = crate::verlet::Verlet::nascer(rest.clone());
    for (v, r) in rest.iter().enumerate() {
        assert_eq!(
            sim.base_de(v),
            *r,
            "sem base, o vertice {v} nao le o repouso"
        );
    }
    // ⚠️ E uma base do TAMANHO ERRADO também não conta — a porta compara os
    // comprimentos, e não a existência.
    sim.base = vec![[9.0; 3]; rest.len() - 1];
    for (v, r) in rest.iter().enumerate() {
        assert_eq!(
            sim.base_de(v),
            *r,
            "uma base truncada foi lida como base ({v})"
        );
    }
}
