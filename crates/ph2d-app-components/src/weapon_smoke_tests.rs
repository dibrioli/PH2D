//! Os gates da CENA da arma — as peças, o enquadramento, e o jogo a jogar-se quadro a quadro.

use super::{
    ACCAO, CACADEIRA_RGBA, CADENCIA_MS, CARTUCHAS, CARTUCHAS_N, CHUMBOS, COLUNA_X, GATILHO, PENTE,
    PENTE_N, TIRO, TORRETA_Y, montar,
};
use ph2d_ecs::{
    Counter, CounterRuntime, Entity, Factory, Name, SignalOnAction, SimWorld, Transform, WeaponFire,
};

const DT: u64 = 1_000_000 / 60;

/// ⭐⭐⭐ **A banda visível, MEDIDA NA FOTO desta cena** (`1930×1012`, 100 % de zoom, com a régua do
/// transporte aberta): a régua do canvas põe o `0` do mundo em `y = 491 px` de ecrã e o `−100` em
/// `588`, logo `1 m` mede `98 px`, o topo do canvas (`y = 90`) é `+4,09 m` e o pé dele (`608`) é
/// `−1,19 m`.
///
/// ⛔⛔ **A 1.ª redacção escreveu `4,2`/`1,3` DE MEMÓRIA** (os números da cena do FIM DE JOGO) e as
/// três torretas saíram cortadas na foto com este gate VERDE — *um número herdado de outra cena é
/// um palpite com cara de medição*.
const BANDA_ACIMA: f32 = 4.09;
/// …e abaixo.
const BANDA_ABAIXO: f32 = 1.19;
/// A meia-largura visível, da mesma foto (a régua horizontal: `−7,4` a `+6,2`; fica o menor).
const BANDA_LADO: f32 = 6.2;

fn montada() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1);
    (sim, m.escolhido)
}

fn por_nome(sim: &SimWorld, nome: &str) -> Entity {
    let mundo = sim.world();
    // ⚠️ `try_query` e não `query`: a leitura corre sobre um `&World`, e um mundo sem `Name` nenhum
    // devolve `None` em vez de pedir `&mut`.
    let mut q = mundo
        .try_query::<(Entity, &Name)>()
        .expect("a cena tem nomes");
    q.iter(mundo)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena tem de ter «{nome}»"))
}

/// **As TRÊS colunas ouvem o MESMO gatilho, na mesma tecla, com a mesma aresta.**
///
/// ⚠️ É isto que torna a cena legível: *uma tecla, três comportamentos, e a única diferença entre
/// colunas é o que cada uma tem a seguir ao gatilho*.
#[test]
fn as_tres_colunas_ouvem_o_mesmo_gatilho() {
    let (sim, _) = montada();
    for nome in ["Arma", "Sem arma (controlo)", "Cacadeira"] {
        let e = por_nome(&sim, nome);
        let g = sim
            .world()
            .get::<SignalOnAction>(e)
            .unwrap_or_else(|| panic!("«{nome}» tem de ter gatilho"));
        assert_eq!(g.0.len(), 1);
        assert_eq!(g.0[0].action, ACCAO);
        assert_eq!(g.0[0].signal, GATILHO);
        assert_eq!(
            g.0[0].edge,
            ph2d_ecs::ActionEdge::Hold,
            "`Hold` e nao `Press`: a cadencia so' se ve' a SEGURAR"
        );
    }
}

/// ⭐⭐ **O CONTROLO não tem arma, e a sua fábrica ouve o GATILHO direto.**
///
/// ⛔ Sem esta metade a cena teria três armas, e o dono não teria como saber o que o componente
/// comprou — *a lei do controlo ao lado, desde o `#13`*.
#[test]
fn o_controlo_nao_tem_arma_e_ouve_o_gatilho_direto() {
    let (sim, _) = montada();
    let ctrl = por_nome(&sim, "Sem arma (controlo)");
    assert!(
        sim.world().get::<WeaponFire>(ctrl).is_none(),
        "o CONTROLO e' a coluna SEM arma"
    );
    let f = sim.world().get::<Factory>(ctrl).expect("fabrica");
    assert_eq!(
        f.on_signal, GATILHO,
        "sem arma, a fabrica ouve o gatilho direto — e' isso que da' a mangueira"
    );

    let arma = por_nome(&sim, "Arma");
    let fa = sim.world().get::<Factory>(arma).expect("fabrica");
    assert_eq!(
        fa.on_signal, TIRO,
        "com arma, a fabrica ouve a ARMA e nunca o gatilho"
    );
}

/// **Os dois pentes têm nomes DIFERENTES.**
///
/// ⚠️ Com o mesmo nome as duas armas partilhariam munição, e o dono veria a esquerda ficar seca por
/// causa de um tiro da direita — um defeito de cena que nenhum gate de lei vê.
#[test]
fn os_dois_pentes_sao_contadores_diferentes() {
    let (sim, _) = montada();
    let a = sim
        .world()
        .get::<Counter>(por_nome(&sim, "Arma"))
        .expect("pente");
    let c = sim
        .world()
        .get::<Counter>(por_nome(&sim, "Cacadeira"))
        .expect("cartuchas");
    assert_eq!(a.name, PENTE);
    assert_eq!(c.name, CARTUCHAS);
    assert_ne!(a.name, c.name, "dois pentes com o mesmo nome sao UM pente");
    assert_eq!((a.start, c.start), (PENTE_N, CARTUCHAS_N));
}

/// **A caçadeira tem rajada E leque** — as duas metades, porque nenhuma sozinha é uma caçadeira.
///
/// ⛔⛔ **A 1.ª redacção era AUTO-REFERENTE e uma mutação SOBREVIVEU:** ela comparava o campo da cena
/// com a MESMA const que a cena lê, logo pôr `LEQUE_GRAUS = 0` movia os dois lados e o gate ficava
/// verde sobre uma caçadeira que cospe cinco chumbos EMPILHADOS. *Um gate auto-referente afirma que
/// a cena concorda consigo mesma, nunca que ela ensina alguma coisa.*
///
/// ⇒ a barra passa a ser **DERIVADA da geometria da própria cena**: no fim do voo, dois chumbos
/// vizinhos têm de estar mais afastados do que a LARGURA de um deles — senão o leque desenha uma
/// linha grossa e não um cone.
#[test]
fn a_cacadeira_tem_rajada_e_leque() {
    let (sim, _) = montada();
    let e = por_nome(&sim, "Cacadeira");
    let f = sim.world().get::<Factory>(e).expect("fabrica");
    assert_eq!(f.burst, CHUMBOS, "sem rajada sai um chumbo so'");

    // A largura de um chumbo, lida da RECEITA — nunca escrita à mão aqui.
    let largura = sim
        .world()
        .get::<ph2d_render::Sprite>(por_nome(&sim, "Chumbo"))
        .expect("a receita do chumbo e' pintada")
        .size[0];
    // O passo angular entre dois chumbos vizinhos, se eles se repartissem pela abertura.
    let passo = f.spread_deg.to_radians() / f32::from(u8::try_from(CHUMBOS).unwrap_or(1));
    let afastamento = super::ALCANCE * passo;
    assert!(
        afastamento > largura,
        "com {} graus de leque, dois chumbos acabam o voo a {afastamento:.3} m um do outro e cada          um mede {largura:.3} m de largo — isso desenha uma LINHA GROSSA, nao um cone",
        f.spread_deg
    );
    // ⭐ O CONTROLO da própria caçadeira: as outras duas não têm leque nenhum.
    for nome in ["Arma", "Sem arma (controlo)"] {
        let o = sim
            .world()
            .get::<Factory>(por_nome(&sim, nome))
            .expect("fabrica");
        assert!(
            o.spread_deg.abs() < f32::EPSILON,
            "so' a cacadeira espalha — «{nome}» tem de sair a direito"
        );
    }
}

/// ⭐⭐⭐ **CADA PEÇA cabe na banda visível.**
///
/// ⚠️ **Por PEÇA e não pela envergadura**, e a razão é medida: o gate do `#25` comparou a ALTURA do
/// conteúdo com a da banda e a foto apanhou o herói fora do ecrã — *uma altura não diz ONDE*.
#[test]
fn cada_peca_cabe_na_banda_visivel() {
    let (sim, _) = montada();
    for nome in ["Arma", "Sem arma (controlo)", "Cacadeira"] {
        let e = por_nome(&sim, nome);
        let t = sim.world().get::<Transform>(e).expect("pose");
        let sp = sim
            .world()
            .get::<ph2d_render::Sprite>(e)
            .expect("toda peca desta cena e' pintada");
        // ⚠️⚠️ **A CAIXA e não o CENTRO, e a rotação entra:** as três estão a `90°`, logo a meia
        // altura no MUNDO é metade da LARGURA do sprite. *Medir o centro deixou a 1.ª redacção
        // verde sobre três torretas cortadas pela borda de baixo* — o defeito que a foto apanhou.
        let meia_alt = sp.size[0] * 0.5;
        let meia_larg = sp.size[1] * 0.5;
        let (x, y) = (t.translation.x, t.translation.y);
        assert!(
            x.abs() + meia_larg <= BANDA_LADO,
            "«{nome}» ocupa ate' x = {}, e a banda acaba em {BANDA_LADO} m",
            x.abs() + meia_larg
        );
        assert!(
            y - meia_alt >= -BANDA_ABAIXO && y + meia_alt <= BANDA_ACIMA,
            "«{nome}» ocupa y de {} a {}, e a banda e' [{}, {BANDA_ACIMA}]",
            y - meia_alt,
            y + meia_alt,
            -BANDA_ABAIXO
        );
    }
    // ⭐ E as BALAS, que sobem: o topo do voo tem de caber também, senão o dono ve' so' a partida.
    let topo = TORRETA_Y + super::ALCANCE;
    assert!(
        topo <= BANDA_ACIMA,
        "uma bala chega a y = {topo} e a banda acaba em {BANDA_ACIMA}"
    );
}

// ⚠️⚠️ **As três colunas são DISTINTAS, e isto é ERRO DE COMPILAÇÃO e não um gate:** um
// `assert!` de teste sobre três constantes é **dobrado pelo compilador** antes de correr, e o
// clippy di-lo em voz alta (`this assertion has a constant value`). *A lei do #25, honrada.*
const _: () = assert!(
    COLUNA_X[0] < COLUNA_X[1] && COLUNA_X[1] < COLUNA_X[2],
    "as tres colunas tem de estar lado a lado — empilhadas, a cena nao ensina nada"
);

/// ⭐⭐⭐ **O JOGO JOGA-SE: segurar a tecla dá RITMO à esquerda e MANGUEIRA ao meio.**
///
/// Este é o gate que percorre a corrida quadro a quadro pelo caminho do PRODUTO — a ponte da arma
/// e a lei da fábrica, na ordem em que o quadro as corre.
///
/// ⚠️ **Ele conta NASCIMENTOS e não sinais:** o que o dono vê são balas, e um sinal que ninguém
/// consome lê-se exactamente como um tiro.
///
/// ⚠️⚠️ **A janela é DERIVADA, e a 1.ª redacção estava curta:** o ciclo inteiro é `6` tiros a
/// `250 ms` (`75` tiques) **mais a cadência do último** (`15`) **mais a recarga** (`800 ms`, `48`)
/// = `138`. Com `90` a arma nem chegava a ficar seca — *e o gate acusava a arma de não recarregar
/// sobre uma arma que ainda estava a disparar*.
#[test]
fn segurar_a_tecla_da_ritmo_a_uma_e_mangueira_a_outra() {
    let (mut sim, _) = montada();
    let arma = por_nome(&sim, "Arma");
    let tags = ph2d_tags::TagTree::default();

    /// Três segundos — o ciclo inteiro cabe em `138` tiques (ver o cabeçalho), e sobra para a
    /// segunda salva começar.
    const QUADROS: u32 = 180;

    let mut tiques_da_arma: Vec<u32> = Vec::new();
    let mut do_ctrl = 0usize;
    let mut ficou_seca = false;
    let mut recarregou = false;
    for q in 0..QUADROS {
        // (a) o gatilho: a tecla está em BAIXO, logo a linha `Hold` fala em todo tique.
        let fired = vec![GATILHO];
        // (b) as armas — a ponte.
        let t = crate::weapon_bridge::frame(&mut sim, true, DT, &fired);
        ficou_seca |= sim
            .world()
            .get::<CounterRuntime>(arma)
            .is_some_and(|r| r.value == 0);
        recarregou |= sim
            .world()
            .get::<ph2d_ecs::WeaponRuntime>(arma)
            .is_some_and(|r| r.reloading);
        let mut nomes: Vec<String> = fired.iter().map(|s| (*s).to_string()).collect();
        nomes.extend(t.disparos.into_iter().map(|(_, n)| n));
        // (c) as fábricas — a lei.
        let refs: Vec<&str> = nomes.iter().map(String::as_str).collect();
        for b in ph2d_ecs::tick_factories(sim.world_mut(), &tags, &refs).births {
            if b.factory == arma {
                tiques_da_arma.push(q);
            } else if sim.world().get::<WeaponFire>(b.factory).is_none() {
                do_ctrl += 1;
            }
        }
    }

    // ── O RITMO ───────────────────────────────────────────────────────────────
    //
    // ⚠️ **O intervalo, e não a contagem:** uma arma que cuspisse tudo de uma vez e parasse daria a
    // mesma contagem no fim, e é exactamente o defeito que a cadência existe para não ter.
    // ⚠️⚠️ **O passo é `div_ceil` e não `250/16,67`, e a diferença é UM TIQUE inteiro:** o passo
    // fixo mede `16 666 µs`, logo `15` tiques dão `249 990` e a cadência de `250 000` **ainda não
    // passou**. *A 1.ª redação escreveu `15` de cabeça e o gate acusou o produto de disparar tarde*
    // — a aritmética de um relógio inteiro arredonda para CIMA, sempre.
    let passo = u32::try_from((CADENCIA_MS * 1_000).div_ceil(DT)).unwrap_or(0);
    assert_eq!(passo, 16, "250 ms sao 16 tiques de 16 666 us");
    assert!(
        tiques_da_arma.len() >= usize::try_from(PENTE_N).unwrap_or(0),
        "a arma tem de cuspir o pente inteiro em 3 s — saiu {}",
        tiques_da_arma.len()
    );
    for par in tiques_da_arma.windows(2).take(5) {
        assert_eq!(
            par[1] - par[0],
            passo,
            "os tiros de um pente tem de sair a CADENCIA ({passo} tiques), e sairam {:?}",
            tiques_da_arma
        );
    }

    // ── O PENTE ───────────────────────────────────────────────────────────────
    assert!(
        ficou_seca,
        "o pente tem de CHEGAR a zero — senao nao ha' pente"
    );
    assert!(recarregou, "e ela tem de recarregar sozinha");

    // ── A MANGUEIRA (o CONTROLO) ──────────────────────────────────────────────
    assert_eq!(
        do_ctrl,
        usize::try_from(QUADROS).unwrap_or(0),
        "o CONTROLO nao tem cadencia nem pente: uma bala por tique, que e' a mangueira"
    );
    assert!(
        do_ctrl > tiques_da_arma.len() * 10,
        "a diferenca entre as duas colunas tem de ser GRITANTE, senao a cena nao ensina nada"
    );
}

/// **A cena declara uma cena só** — o `max_level` do roteador é CONTADO, nunca escrito.
#[test]
fn o_roteador_serve_uma_cena() {
    assert_eq!(super::CENAS, 1);
    let (_, escolhido) = montada();
    assert_ne!(escolhido, 0, "alguem tem de nascer escolhido");
}

/// ⭐ **A ARMA nasce ESCOLHIDA**, e é a FOTO que o exige: o passo (5) manda ver a secção *Weapon*
/// no painel da direita, e com ninguém escolhido o Inspector diz *«Select an entity in the
/// Hierarchy»*.
#[test]
fn a_arma_nasce_escolhida() {
    let (sim, escolhido) = montada();
    assert_eq!(
        Entity::from_bits(escolhido),
        por_nome(&sim, "Arma"),
        "quem nasce escolhido tem de ser a coluna de que o roteiro fala"
    );
    // ⭐ E ela tem CORPO pintado — sem `Sprite` o pick nunca a devolve (a lei do `#15`).
    assert!(
        sim.world()
            .get::<ph2d_render::Sprite>(por_nome(&sim, "Arma"))
            .is_some(),
        "um objecto sem sprite e' inalcancavel pelo dedo do dono"
    );
    let _ = CACADEIRA_RGBA;
}
