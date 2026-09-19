//! ⭐⭐⭐⭐ **A MEMÓRIA DO TRAÇO** — as sondas que mediram a fase da retícula a
//! atravessar os carimbos, e as duas alavancas dela.
//!
//! Ordem do dono (21/09), depois de eu lhe ter posto na mesa a troca de `−26 %`
//! de relógio por `0,7` pontos de regularidade: ***«quero o melhor possível, o
//! padrão ouro»*** — ou seja **não** a troca, a obra que dá o múltiplo.
//!
//! A tabela que estas sondas produziram vive onde as constantes vivem
//! ([`ph2d_quadflow::regiao::CampoDoTraco`] e
//! [`crate::dyntopo::RONDAS_DA_GRELHA`]); aqui fica o instrumento.
//!
//! ⚠️ Ele saiu do irmão do relógio por **TECTO DE LOC**, e o corte é por
//! assunto: ali mora *quanto custa o que se faz*, aqui *quanto se pode deixar
//! de fazer por se ter lembrado*.

use super::*;

/// ⭐⭐⭐⭐ **SONDA — a memória do traço contra a escada das rondas.**
///
/// A hipótese, com o mecanismo do §86 escrito por trás dela: o campo de posição
/// é semeado com *cada vértice é a própria origem* — a semente menos coerente
/// que existe —, e é por isso que ele custa `2,797 ms` contra `0,553` do de
/// orientação, que nasce de UMA direcção do mundo. Guardando o nó de cada
/// vértice entre carimbos, a **franja** (que está pregada) passa a impor ao dab
/// seguinte a retícula que a zona já penteada tem.
///
/// Imprime, por célula `(rondas, alternâncias, memória, anéis, orientação)`: as
/// quatro colunas que o dono julga, as lascas e o relógio do traço.
#[test]
#[ignore = "sonda: imprime a escada da memoria, nao afirma nada"]
fn diag_a_memoria_do_traco_contra_as_rondas() {
    use std::time::Instant;
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("rondas alt mem aneis ori  grade%  vinco90  fil50  fil90  4bracos%  lascas  ms/traco");
    for (rondas, alt, com_memoria, aneis, ori) in CELULAS {
        crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(rondas));
        crate::dyntopo::ALTERNANCIAS_DO_TESTE.with(|c| c.set(alt));
        crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(!com_memoria));
        crate::dyntopo::ANEIS_NO_TESTE.with(|c| c.set(aneis));
        crate::dyntopo::ORIENTACAO_NO_TESTE.with(|c| c.set(ori));
        let (mut g, mut v90, mut f50, mut f90) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut quatro, mut n, mut lascas_tot) = (0usize, 0usize, 0usize);
        let mut relogio = f64::INFINITY;
        for (_, e) in RUMOS.iter() {
            let t = Instant::now();
            let (m, c) = super::super::super::traco_com(1.0, *e, raio, alvo);
            relogio = relogio.min(t.elapsed().as_secs_f64() * 1000.0);
            let (bal, nb) = grade_da_faixa(&m, &c, raio);
            g += 100.0 * bal[0] as f64 / nb.max(1) as f64;
            v90 += vinco_da_faixa(&m, &c, raio).1;
            let (x, y, _, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
            f50 += x;
            f90 += y;
            lascas_tot += lascas(&m, &c, raio, LIMIAR_DA_LASCA).0;
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            for (v, q) in m.positions().iter().enumerate() {
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                n += 1;
                if grau[v] == 4 {
                    quatro += 1;
                }
            }
        }
        limpa_os_interruptores();
        println!(
            "{rondas:6} {alt:3} {:3} {aneis:5} {:3}  {:6.2}  {:7.3}  {:5.1}  {:5.1}  {:8.2}  {lascas_tot:6}  {relogio:8.1}",
            if com_memoria { "sim" } else { "NAO" },
            if ori == 1 { "sim" } else { "nao" },
            g / 4.0,
            v90 / 4.0,
            f50 / 4.0,
            f90 / 4.0,
            100.0 * quatro as f64 / n.max(1) as f64
        );
    }
}

/// As células da escada: `(rondas, alternâncias, memória, anéis, orientação)`.
///
/// ⚠️ `ori`: `1` = lembra, `2` = não lembra (o `0` seria *«o que shipa»*, e uma
/// sonda que lê o valor que shipa não mede a alavanca — mede o produto).
const CELULAS: [(usize, usize, bool, usize, u8); 12] = [
    // O CONTROLO — o que shipava antes desta wave.
    (4, 2, false, 0, 2),
    // A ESCADA DOS ANÉIS, com as rondas cortadas ao meio.
    (2, 2, true, 0, 1),
    (2, 2, true, 1, 1),
    (2, 2, true, 2, 1),
    (2, 2, true, 3, 1),
    (2, 2, true, 4, 1),
    // A ATRIBUIÇÃO da orientação, nos dois melhores anéis.
    (2, 2, true, 2, 2),
    (2, 2, true, 3, 2),
    // O que as rondas ainda compram POR CIMA da memória.
    (4, 2, true, 2, 1),
    (4, 2, true, 3, 1),
    // E os controlos das rondas cortadas, sem memória nenhuma.
    (2, 2, false, 0, 2),
    (1, 1, false, 0, 2),
];

fn limpa_os_interruptores() {
    crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(0));
    crate::dyntopo::ALTERNANCIAS_DO_TESTE.with(|c| c.set(0));
    crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(false));
    crate::dyntopo::ANEIS_NO_TESTE.with(|c| c.set(usize::MAX));
    crate::dyntopo::ORIENTACAO_NO_TESTE.with(|c| c.set(0));
}

/// ⭐⭐⭐⭐ **SONDA — a CONFIRMAÇÃO das candidatas sobre OITO rumos.**
///
/// ⚠️ **A escada da irmã corre quatro rumos e a coluna da FILEIRA salta entre
/// células vizinhas** — o §86 desta linha já o registou: o percurso dela é
/// guloso, e uma aresta a entrar ou a sair do balde alinhado funde ou parte duas
/// cadeias longas de uma vez. *Uma coluna de alta variância lida numa amostra só
/// fabrica uma tendência*, e a escada tem doze células.
///
/// Esta corre as candidatas sobre **oito** rumos (os quatro do gate mais os
/// quatro intermédios) e imprime também o **DESVIO entre rumos**, que é a coluna
/// que diz se a diferença entre duas candidatas é maior do que o ruído.
#[test]
#[ignore = "sonda: confirma as candidatas em 8 rumos, nao afirma nada"]
fn diag_as_candidatas_em_oito_rumos() {
    use std::time::Instant;
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    println!("candidata            grade%±dp   vinco90   fil50±dp     fil90±dp    4bracos%±dp  ms");
    for (nome, rondas, alt, com_memoria, aneis, ori) in CANDIDATAS {
        crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(rondas));
        crate::dyntopo::ALTERNANCIAS_DO_TESTE.with(|c| c.set(alt));
        crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(!com_memoria));
        crate::dyntopo::ANEIS_NO_TESTE.with(|c| c.set(aneis));
        crate::dyntopo::ORIENTACAO_NO_TESTE.with(|c| c.set(ori));
        let (mut gs, mut v9, mut f5, mut f9, mut b4) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut relogio = f64::INFINITY;
        for e in OITO_RUMOS {
            let t = Instant::now();
            let (m, c) = super::super::super::traco_com(1.0, e, raio, alvo);
            relogio = relogio.min(t.elapsed().as_secs_f64() * 1000.0);
            let (bal, nb) = grade_da_faixa(&m, &c, raio);
            gs.push(100.0 * bal[0] as f64 / nb.max(1) as f64);
            v9.push(vinco_da_faixa(&m, &c, raio).1);
            let (x, y, _, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
            f5.push(x);
            f9.push(y);
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            let (mut quatro, mut n) = (0usize, 0usize);
            for (v, q) in m.positions().iter().enumerate() {
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                n += 1;
                if grau[v] == 4 {
                    quatro += 1;
                }
            }
            b4.push(100.0 * quatro as f64 / n.max(1) as f64);
        }
        limpa_os_interruptores();
        let m = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let dp = |v: &[f64]| {
            let a = m(v);
            (v.iter().map(|x| (x - a) * (x - a)).sum::<f64>() / v.len() as f64).sqrt()
        };
        println!(
            "{nome:20} {:5.2}±{:4.2}  {:7.3}  {:5.1}±{:4.1}  {:5.1}±{:4.1}  {:6.2}±{:4.2}  {relogio:6.1}",
            m(&gs),
            dp(&gs),
            m(&v9),
            m(&f5),
            dp(&f5),
            m(&f9),
            dp(&f9),
            m(&b4),
            dp(&b4)
        );
    }
}

/// `(nome, rondas, alternâncias, memória, anéis, orientação)`.
const CANDIDATAS: [(&str, usize, usize, bool, usize, u8); 6] = [
    ("CONTROLO 4/2 s/mem", 4, 2, false, 0, 2),
    ("2/2 mem an=2", 2, 2, true, 2, 2),
    ("2/2 mem an=3", 2, 2, true, 3, 2),
    ("2/2 mem an=2 +ori", 2, 2, true, 2, 1),
    ("4/2 mem an=2", 4, 2, true, 2, 2),
    ("2/1 mem an=2", 2, 1, true, 2, 2),
];

/// Os quatro rumos do gate mais os quatro intermédios.
const OITO_RUMOS: [[f32; 2]; 8] = [
    [1.0, 0.0],
    [0.923_879_5, 0.382_683_4],
    [0.866_025_4, 0.5],
    [
        std::f32::consts::FRAC_1_SQRT_2,
        std::f32::consts::FRAC_1_SQRT_2,
    ],
    [0.5, 0.866_025_4],
    [0.382_683_4, 0.923_879_5],
    [0.0, 1.0],
    [-0.382_683_4, 0.923_879_5],
];

/// ⭐⭐⭐⭐ **GATE — A MEMÓRIA DO TRAÇO É O QUE FAZ DUAS VARREDURAS CHEGAREM.**
///
/// Ordem do dono (21/09): ***«quero o melhor possível, o padrão ouro»***, em
/// resposta à troca de `−26 %` de relógio por qualidade que eu lhe pusera na
/// mesa. O produto passou a correr **metade** das varreduras
/// ([`crate::dyntopo::RONDAS_DA_GRELHA`], `4 → 2`) — e só as pode correr porque
/// a fase da retícula atravessa os carimbos.
///
/// ⛔⛔ **É por isso que este gate tem de existir e o da cena não chega:** aquele
/// mede *«a grade tem os quatro braços»* contra a malha por pentear, e com a
/// memória desligada o produto **também** passaria nele com folga menor. O que
/// esta metade afirma é outra coisa — ***nas varreduras que o produto hoje
/// gasta, tirar a memória derruba as duas colunas que o dono julga***.
///
/// **Medido pela porta do produto, quatro rumos, na configuração que shipa:**
///
/// | | 4 braços % | fil p90 |
/// |---|---|---|
/// | **com memória** (shipa) | **`95,61`** | **`72,5`** |
/// | sem memória, mesmas varreduras | `88,46` | `57,2` |
/// | sem memória, o DOBRO das varreduras (o que shipava) | `93,01` | `70,8` |
///
/// ⭐ A terceira linha é a que dá o tamanho do ganho: **a memória a metade do
/// preço entrega mais do que o dobro do preço entregava.**
///
/// ⚠️ **As barras saem do VALE medido**, e ele é largo (`88,5` contra `95,6`):
/// *uma barra no meio de um vazio de sete pontos não é um número escolhido.*
#[test]
fn a_memoria_do_traco_e_o_que_faz_duas_varreduras_chegarem() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mede = |com_memoria: bool| -> (f64, f64) {
        crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(!com_memoria));
        let (mut b4, mut f90, mut n_total) = (0usize, 0.0f64, 0usize);
        for (_, e) in RUMOS.iter() {
            let (m, c) = super::super::super::traco_com(1.0, *e, raio, alvo);
            let (_, y, _, _) = ph2d_sculpt3d::medida_da_fileira::fileira_da_faixa(&m, &c, raio);
            f90 += y;
            let grau = ph2d_mesh_render::bracos_na_vista_da_grade(&m);
            for (v, q) in m.positions().iter().enumerate() {
                let mut d2 = f32::INFINITY;
                for t in &c {
                    let w = [q[0] - t[0], q[1] - t[1], q[2] - t[2]];
                    d2 = d2.min(w[0].mul_add(w[0], w[1].mul_add(w[1], w[2] * w[2])));
                }
                if d2.sqrt() > raio * 0.5 {
                    continue;
                }
                n_total += 1;
                if grau[v] == 4 {
                    b4 += 1;
                }
            }
        }
        crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(false));
        (
            100.0 * b4 as f64 / n_total.max(1) as f64,
            f90 / RUMOS.len() as f64,
        )
    };

    let (b4_com, f90_com) = mede(true);
    let (b4_sem, f90_sem) = mede(false);
    assert!(
        b4_com > 93.0 && f90_com > 65.0,
        "o produto COM memoria caiu: {b4_com:.2} % de quatro bracos e fileira p90 {f90_com:.1} \
         (medido 95,61 e 72,5)"
    );
    assert!(
        b4_sem < 91.0 && f90_sem < 62.0,
        "o CONTROLO nao reproduz o defeito: sem memoria leu {b4_sem:.2} % e {f90_sem:.1} \
         (medido 88,46 e 57,2) — uma fixtura que deixou de conter o fenomeno nao afirma nada"
    );
}

/// ⭐⭐⭐⭐ **GATE — A MEMÓRIA ATRAVESSA A TOPOLOGIA DINÂMICA SEM SER DEITADA
/// FORA.**
///
/// ⛔⛔⛔ **Ele existe porque uma MUTAÇÃO SOBREVIVEU:** apagar a chamada
/// `pente_campo.encolheu(&remap)` do produto deixava o gate da qualidade
/// **VERDE**. A causa é a rede que a própria porta tem — o
/// [`ph2d_quadflow::regiao::CampoDoTraco::acomoda`] **esquece** quando a malha
/// encolheu sem a renumeração chegar, porque uma memória que já não sabe de quem
/// fala é pior que nenhuma. ⇒ *a rede torna o produto robusto e torna o defeito
/// INOBSERVÁVEL*, que é a mesma forma do §25 desta linha (*«um recorte que
/// desiste em silêncio não recorta nada»*).
///
/// ⇒ a régua deixa de ser o barro e passa a ser **a própria memória**: ela
/// acompanha a malha nos dois sentidos e **nunca é esquecida a meio do traço**.
///
/// ⚠️ **As três metades, e cada uma reprova por um motivo diferente:**
///
/// 1. o **CONTROLO POSITIVO** — a topologia tem de mexer de facto (senão o gate
///    mede um traço sobre uma malha parada e fica verde por vácuo);
/// 2. o tamanho acompanha a malha a cada carimbo;
/// 3. a memória **nunca cai a zero** depois do primeiro carimbo — que é
///    exactamente o que a mutação provoca.
#[test]
fn a_memoria_atravessa_a_topologia_dinamica() {
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let mut malha = peca_uma_vez();
    malha.triangulate();
    let brush = ph2d_sculpt3d::Brush {
        verb: ph2d_sculpt3d::Verb::Draw,
        radius: raio,
        strength: 0.25,
        pente: 0.0,
        ..ph2d_sculpt3d::Brush::default()
    };
    let mut stroke = ph2d_sculpt3d::SculptStroke::default();
    stroke.begin(&malha);
    let mut campo = ph2d_quadflow::regiao::CampoDoTraco::default();
    let (mut births, mut region) = (Vec::new(), ph2d_mesh::RegionScratch::default());
    let mut remap = ph2d_mesh::Remap::default();
    let (mut cortou, mut refinou, mut minimo) = (0usize, 0usize, usize::MAX);
    let passo = raio * 0.15;
    for k in 0..24 {
        let u = -passo * 12.0 + passo * k as f32;
        let centro = [u.sin() * RUMOS[0].1[0], u.sin() * RUMOS[0].1[1], u.cos()];
        let direccao = stroke.direccao_do_traco(centro);
        let (cut, done, _) = crate::dyntopo::passe_nos_motores(
            &mut malha,
            brush.verb,
            alvo,
            centro,
            raio,
            crate::dyntopo::Rascunho {
                remap: &mut remap,
                births: &mut births,
                region: &mut region,
            },
            Some(crate::dyntopo::Pente {
                direccao,
                forca: 1.0,
                queda: brush.falloff,
                campo: Some(&mut campo),
            }),
        );
        if cut {
            stroke.shrink_with(&remap);
            campo.encolheu(&remap);
            cortou += 1;
        }
        if done {
            stroke.grow_with(&malha, &births);
            campo.cresceu(&malha, &births);
            refinou += 1;
        }
        stroke.dab(
            &mut malha,
            &brush,
            &ph2d_sculpt3d::Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            ph2d_sculpt3d::Symmetry::default(),
        );
        assert_eq!(
            campo.len(),
            malha.vert_count(),
            "no carimbo {k} a memoria ({}) deixou de descrever a malha ({})",
            campo.len(),
            malha.vert_count()
        );
        if k > 0 {
            minimo = minimo.min(campo.lembrados());
        }
    }
    // (1) O CONTROLO POSITIVO: sem topologia a mexer este gate nao afirma nada.
    assert!(
        cortou > 0 && refinou > 0,
        "a fixtura nao contem o fenomeno: {cortou} colapsos e {refinou} refinos"
    );
    // (3) E a memoria nunca foi deitada fora.
    assert!(
        minimo > 200,
        "a memoria foi esquecida a meio do traco (minimo {minimo} lembrados em {cortou} \
         colapsos) — a renumeracao nao esta a chegar-lhe"
    );
}

/// ⭐⭐⭐⭐ **CENSO — AS DUAS PORTAS DO TAMANHO VIAJAM COM AS DO TRAÇO, E O
/// PEN-DOWN ESQUECE.**
///
/// ⛔⛔⛔ **Ele existe porque TRÊS mutações sobreviveram ao gate de
/// comportamento**, e a razão é estrutural: o
/// [`ph2d_quadflow::regiao::CampoDoTraco::acomoda`] é uma **rede** — quando a
/// malha encolhe sem a renumeração chegar, ele **esquece** em vez de mentir. ⇒
/// *o produto continua correcto e o defeito fica INOBSERVÁVEL no barro*; o que
/// se perde é a fase, e a fase perdida lê-se como *«hoje o pente esteve pior»*.
///
/// ⚠️ **E a 1.ª tentativa de gate NÃO PODIA apanhá-las**: ela montava o laço do
/// traço à mão e chamava as três portas ela própria — *um gate que reconstrói a
/// fiação afirma que as leis existem, nunca que o produto as usa*, que é a
/// mesma frase que o §24 desta linha já tinha escrito e que eu voltei a violar.
///
/// ⇒ a régua é **DERIVADA do ficheiro que declara a fiação**: cada porta de
/// tamanho do `SculptStroke` tem de ter a irmã da memória ao lado, e o `begin`
/// do pen-down tem de ter o `esquece`. *Uma porta nova do traço reprova aqui até
/// alguém dizer o que a memória faz com ela.*
///
/// ⚠️ **O piso de população é obrigatório**: sem ele, um ficheiro que deixasse
/// de conter o laço inteiro passaria com `0 == 0`.
#[test]
fn as_duas_portas_do_tamanho_viajam_com_as_do_traco() {
    let passe = include_str!("dyntopo.rs");
    let pen_down = include_str!("input_down.rs");
    for (fonte, onde, do_traco, da_memoria) in [
        (
            passe,
            "dyntopo.rs",
            "self.stroke.shrink_with(",
            "self.pente_campo.encolheu(",
        ),
        (
            passe,
            "dyntopo.rs",
            "self.stroke.grow_with(",
            "self.pente_campo.cresceu(",
        ),
        (
            pen_down,
            "input_down.rs",
            "stroke.begin(",
            "pente_campo.esquece(",
        ),
    ] {
        let n = fonte.matches(do_traco).count();
        assert!(
            n > 0,
            "o `{do_traco}` desapareceu de {onde} — o censo passaria a medir o vazio"
        );
        assert_eq!(
            fonte.matches(da_memoria).count(),
            n,
            "em {onde} há {n} `{do_traco}` e {} `{da_memoria}`: a memória do pente \
             tem de viajar pelos MESMOS canais que o traço, senão ela é esquecida \
             pela rede do `acomoda` e ninguém vê",
            fonte.matches(da_memoria).count()
        );
    }
}

/// ⭐⭐⭐⭐ **SONDA — O RELÓGIO QUE O DONO SENTE: o passe de topologia por dab.**
///
/// Report dele (21/09): *«algoritmo mais lento que o modo padrão. tem que
/// otimizar»*. A régua é a **porta do produto**, na peça que a cena abre e no
/// raio que o app dá, e o número é o **MÍNIMO de cinco corridas** — ⚠️ *nenhuma
/// leitura de relógio desta máquina vale nada acima de `load ~5`*, e ela imprime
/// o `loadavg` ao lado para a régua não ser a própria flake.
///
/// ⚠️⚠️ **O MÍNIMO de VINTE E CINCO corridas, e não de cinco:** a linha «sem
/// pente» mede `~0,1 ms` por dab, e a `load 16` uma leitura dessa ordem lê
/// **`0,571`** — *contaminação, não custo*. A linha de `7,7 ms` reproduz ao
/// milésimo em qualquer carga, que é o controlo de que o mínimo é robusto onde
/// a grandeza é grande.
///
/// As três linhas são as três perguntas: *quanto custa o passe SEM pente*
/// (a linha de base do «modo padrão» de que ele se queixa), *quanto custava com
/// o pente de ontem* e *quanto custa com o de hoje*.
#[test]
#[ignore = "sonda: imprime o relogio do passe por dab, nao afirma nada"]
fn diag_o_relogio_do_passe_por_dab() {
    use std::time::Instant;
    let (raio, alvo) = (raio_do_app(), alvo_do_refino());
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "configuracao                     ms/dab   ms/traco(24)   loadavg: {}",
        carga.trim()
    );
    for (nome, com_pente, rondas, com_memoria) in [
        ("sem pente (o «modo padrao»)", false, 0usize, false),
        ("pente de ONTEM  4/2 s/memoria", true, 4, false),
        ("pente de HOJE   2/2 c/memoria", true, 2, true),
    ] {
        crate::dyntopo::RONDAS_DO_TESTE.with(|c| c.set(rondas));
        crate::dyntopo::SEM_MEMORIA_NO_TESTE.with(|c| c.set(!com_memoria));
        let mut melhor = f64::INFINITY;
        for _ in 0..25 {
            let mut malha = peca_uma_vez();
            malha.triangulate();
            let mut stroke = ph2d_sculpt3d::SculptStroke::default();
            stroke.begin(&malha);
            let mut campo = ph2d_quadflow::regiao::CampoDoTraco::default();
            let (mut births, mut region) = (Vec::new(), ph2d_mesh::RegionScratch::default());
            let mut remap = ph2d_mesh::Remap::default();
            let passo = raio * 0.15;
            let mut gasto = 0.0f64;
            for k in 0..24 {
                let u = -passo * 12.0 + passo * k as f32;
                let centro = [u.sin() * RUMOS[0].1[0], u.sin() * RUMOS[0].1[1], u.cos()];
                let direccao = stroke.direccao_do_traco(centro);
                let t = Instant::now();
                let (cut, done, _) = crate::dyntopo::passe_nos_motores(
                    &mut malha,
                    ph2d_sculpt3d::Verb::Draw,
                    alvo,
                    centro,
                    raio,
                    crate::dyntopo::Rascunho {
                        remap: &mut remap,
                        births: &mut births,
                        region: &mut region,
                    },
                    com_pente.then_some(crate::dyntopo::Pente {
                        direccao,
                        forca: 1.0,
                        queda: ph2d_sculpt3d::Falloff::default(),
                        campo: crate::dyntopo::campo_do_teste(&mut campo),
                    }),
                );
                gasto += t.elapsed().as_secs_f64() * 1000.0;
                if cut {
                    stroke.shrink_with(&remap);
                    campo.encolheu(&remap);
                }
                if done {
                    stroke.grow_with(&malha, &births);
                    campo.cresceu(&malha, &births);
                }
                stroke.dab(
                    &mut malha,
                    &ph2d_sculpt3d::Brush {
                        verb: ph2d_sculpt3d::Verb::Draw,
                        radius: raio,
                        strength: 0.25,
                        pente: 0.0,
                        ..ph2d_sculpt3d::Brush::default()
                    },
                    &ph2d_sculpt3d::Dab::at(centro, raio, [0.0, 0.0, -1.0]),
                    ph2d_sculpt3d::Symmetry::default(),
                );
            }
            melhor = melhor.min(gasto);
        }
        limpa_os_interruptores();
        println!("{nome:32} {:6.3}   {melhor:10.1}", melhor / 24.0);
    }
}
