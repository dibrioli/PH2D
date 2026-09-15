//! Os gates da POSE que vivem **nesta** crate — a fiação, não a lei.
//!
//! ⚠️ A lei está medida contra `69` traços do oráculo na bancada da
//! `ph2d-pose`. O que **só** aqui se pode afirmar é o que a ponte promete: que
//! a simetria do traço chega à lei, e que a cadeia se constrói **uma vez**.

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Pose,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    }
}

fn puxao(centro: [f32; 3], raio: f32, puxao: [f32; 3]) -> Dab {
    let l = (centro[0] * centro[0] + centro[1] * centro[1] + centro[2] * centro[2]).sqrt();
    let olho = [-centro[0] / l, -centro[1] / l, -centro[2] / l];
    Dab::pulling(centro, raio, olho, puxao)
}

/// ⭐ **A simetria do TRAÇO chega à lei.**
///
/// ⛔ Este verbo não entra no censo `every_verb_inherits_symmetry_…` porque ele
/// mede o conjunto **tocado**, um plano por-slot do laço por-vértice que a pose
/// nunca preenche (ela [`Verb::resolve_a_propria_regiao`]). *Tirar um verbo de
/// um censo sem escrever o gate que o substitui é como uma ausência vira
/// permanente* — este é o gate que o substitui, e ele mede o que a ponte de
/// facto promete: que `Symmetry` vira `Controlos::simetria`.
#[test]
fn a_pose_espelha_o_que_move() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let antes = malha.positions().to_vec();
    // Fora do plano `x = 0`, para que o espelho caia noutro sítio.
    for k in 0..2 {
        let d = 0.10 + f32::from(u8::try_from(k).unwrap_or(0)) * 0.05;
        s.dab(
            &mut malha,
            &b,
            &puxao([0.6, 0.0, 0.8], b.radius, [0.0, d, 0.0]),
            Symmetry::MIRROR_X,
        );
    }
    let movidos = s.last_moved();
    assert!(
        movidos.len() > 8,
        "a pose com simetria não moveu nada de jeito ({} vértices) — o gate mediria vácuo",
        movidos.len()
    );
    let pos = malha.positions();
    let (mut esquerda, mut direita, mut no_plano) = (0usize, 0usize, 0usize);
    for &v in movidos {
        // ⚠️ **A banda do PLANO sai da contagem, e a primeira redacção deste
        // gate esqueceu-a.** Uma esfera UV tem um anel inteiro de vértices
        // exactamente em `x = 0`; contá-los num dos lados faz o gate reprovar
        // sobre produto correcto, e foi o que ele fez. *Uma peça simétrica tem
        // três conjuntos, não dois.*
        let x = antes[v as usize][0];
        if x.abs() < 1e-4 {
            no_plano += 1;
        } else if x < 0.0 {
            esquerda += 1;
        } else {
            direita += 1;
        }
    }
    assert!(
        esquerda > 0 && direita > 0,
        "a simetria não chegou à lei: {esquerda} à esquerda e {direita} à direita"
    );
    // ⚠️ Uma esfera UV é simétrica em `x` por construção, logo os dois lados têm
    // de sair com contagens **iguais**. Uma banda larga aqui aceitaria a
    // simetria aplicada **duas** vezes, que é exactamente o defeito que o desvio
    // antes do espelho existe para impedir.
    // ⛔⛔ **A CONTAGEM é a régua FRÁGIL, e ela reprovou sobre produto correcto:**
    // `155` contra `154`. A diferença era **um** vértice que se move um ulp de um
    // lado e exactamente zero do outro — o limiar de escrita é uma igualdade
    // estrita de `f32`, não uma barra. *Uma régua discreta sobre uma grandeza
    // contínua transforma ruído de arredondamento num veredito.*
    assert!(
        esquerda.abs_diff(direita) <= 2,
        "os dois lados moveram contagens muito diferentes ({esquerda} vs {direita}, \
         {no_plano} no plano) — a simetria não é a da lei"
    );
    // ⭐ **A régua que decide é a ENERGIA de cada lado**, que é contínua e imune
    // ao limiar de escrita — e que um espelho aplicado **duas** vezes não pode
    // satisfazer: ali um dos lados receberia a deformação composta consigo
    // mesma e a razão sairia longe de `1`.
    let energia = |sinal: f32| -> f64 {
        movidos
            .iter()
            // ⚠️ **O lado é do REPOUSO, nunca da posição final** — um vértice
            // pode ATRAVESSAR o plano durante a deformação, e classificá-lo
            // onde ele acabou muda-o de balde a meio da medição. A primeira
            // redacção fez isso e leu `1,25e-2` de assimetria numa saída
            // simétrica.
            .filter(|&&v| antes[v as usize][0] * sinal > 1e-4)
            .map(|&v| {
                let (a, b) = (pos[v as usize], antes[v as usize]);
                f64::from(
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
                )
            })
            .sum()
    };
    let (e, d) = (energia(-1.0), energia(1.0));
    assert!(
        e > 0.0 && ((e - d) / e).abs() < 1e-4,
        "a energia dos dois lados difere: {e:.6} à esquerda contra {d:.6} à direita"
    );
}

/// ⭐⭐ **A CADEIA CONSTRÓI-SE UMA VEZ POR TRAÇO — e isto é a nossa vantagem
/// medida sobre o alvo, agora GATEADA em vez de afirmada.**
///
/// O alvo reconstrói a cadeia inteira a cada movimento do rato, **mesmo sem
/// traço nenhum**, só para desenhar o indicador sob o cursor: é a causa
/// registada em quatro relatos públicos de o editor engasgar em malha densa, e
/// com peças desligadas cada movimento de câmara volta a pagar um `O(V²)`.
///
/// ⚠️ *Uma vantagem escrita num cabeçalho é uma promessa; uma vantagem com
/// contador é uma propriedade.* Sem este gate, alguém que movesse a construção
/// para dentro do laço de eventos não partiria teste nenhum — a saída seria a
/// mesma, só mais lenta, e a regressão viajaria até ao dia em que o dono
/// esculpisse uma malha grande.
#[test]
fn a_cadeia_da_pose_constroi_se_uma_vez_por_traco() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    for k in 0..12 {
        let d = 0.02 * f32::from(u8::try_from(k).unwrap_or(0));
        s.dab(
            &mut malha,
            &b,
            &puxao([0.6, 0.0, 0.8], b.radius, [0.0, d, 0.0]),
            Symmetry::default(),
        );
    }
    assert_eq!(
        s.pose_construcoes, 1,
        "doze eventos construíram a cadeia {} vezes — ela é a MESMA do princípio \
         ao fim do traço (espec §10), e reconstruí-la é o defeito do alvo que \
         esta implementação existe para não ter",
        s.pose_construcoes
    );
    // E o controlo do outro lado: um traço NOVO acha o pivô outra vez.
    s.begin(&malha);
    s.dab(
        &mut malha,
        &b,
        &puxao([0.6, 0.0, 0.8], b.radius, [0.0, 0.1, 0.0]),
        Symmetry::default(),
    );
    assert_eq!(
        s.pose_construcoes, 1,
        "o `begin` tem de largar a cadeia: um traço novo constrói a dele"
    );
}

/// ⛔⛔ **O TRAÇO DE POSE TEM DE PREENCHER A JANELA DO UNDO.**
///
/// Report do dono (2026-09-14): *«undo/redo não funciona para esse pincel»*.
///
/// ⚠️ **O mecanismo, e ele é estrutural:** o `close_stroke` da cena grava
/// `StrokeUndo::Stroke { verts: touched(), positions: base_positions() }` e
/// **devolve cedo** quando `touched()` está vazio. Quem enche essa janela é o
/// `capture`, que vive no laço por-vértice do `dab_core` — e este verbo
/// [`Verb::resolve_a_propria_regiao`], logo **nunca passa por lá**. O tecido
/// desvia igual e chama o `capture` à mão; a pose não chamava.
///
/// ⇒ o traço movia a malha e **não deixava rasto nenhum** para o `Ctrl+Z`.
/// *Uma janela vazia e um gesto que não fez nada são o mesmo byte para o
/// `close_stroke`.*
#[test]
fn a_pose_enche_a_janela_do_undo() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let antes = malha.positions().to_vec();
    for k in 1..=4 {
        let d = 0.05 * f32::from(u8::try_from(k).unwrap_or(1));
        s.dab(
            &mut malha,
            &b,
            &puxao([0.6, 0.0, 0.8], b.radius, [0.0, d, 0.0]),
            Symmetry::default(),
        );
    }
    let mexidos = antes
        .iter()
        .zip(malha.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        mexidos > 8,
        "o arnes nao moveu nada de jeito ({mexidos} vertices) — o gate mediria vacuo"
    );
    assert!(
        !s.touched().is_empty(),
        "a janela do undo saiu VAZIA depois de mover {mexidos} vertices — o \
         `close_stroke` devolve cedo e o Ctrl+Z nao tem o que desfazer"
    );
    assert_eq!(
        s.touched().len(),
        s.base_positions().len(),
        "a janela e o `pre` tem de andar em par"
    );
    // ⭐⭐ **E o `pre` tem de ser o de ANTES DO TRAÇO, não o do evento em que o
    // vértice entrou na janela.** É a metade que faz o desfazer devolver a
    // forma, e não uma pose intermédia: com quatro eventos, um vértice que só
    // se mova no terceiro tem de trazer a posição do pen-down.
    for (&v, &pre) in s.touched().iter().zip(s.base_positions()) {
        assert_eq!(
            pre, antes[v as usize],
            "o vertice {v} entrou na janela com uma pose INTERMEDIA — o Ctrl+Z \
             devolveria o meio do arrasto"
        );
    }
    // E todo vértice que de facto se moveu tem de estar na janela, senão o
    // desfazer deixa parte da deformação para trás.
    let janela: std::collections::BTreeSet<u32> = s.touched().iter().copied().collect();
    let esquecidos = antes
        .iter()
        .zip(malha.positions())
        .enumerate()
        .filter(|(i, (a, b))| a != b && !janela.contains(&(*i as u32)))
        .count();
    assert_eq!(
        esquecidos, 0,
        "{esquecidos} vertices mexeram e ficaram FORA da janela — o Ctrl+Z \
         devolveria a peca pela metade"
    );
}

/// ⛔⛔⛔ **A CURVA DO PINCEL CHEGA AO MODO DE TORÇÃO — e a INVERSÃO é a ponte.**
///
/// A [`ph2d_pose::Curva`] amostra em `1 − i/n` e o argumento dela é *quanto
/// FALTA* (`1` no segmento mais perto do cursor); o [`crate::Falloff::weight`]
/// mede *quanto já se ANDOU* (`1` na borda, onde o peso é zero). Ligadas sem a
/// inversão, o 1.º segmento recebia `weight(1,0)`, que é **zero nas doze
/// curvas**, e com o valor de fábrica — **um** segmento — o pincel inteiro não
/// rodava um vértice.
///
/// ⚠️ **Nenhuma régua desta casa podia apanhá-lo:** os `69` traços do oráculo
/// correm a `ph2d-pose` **directamente**, com a convenção dela
/// ([`ph2d_pose::suave`]), logo *uma paridade medida a montante de uma conversão
/// não afirma nada sobre a conversão* — a mesma lei que o pincel de contorno
/// pagou um dia antes, no [`crate::stroke_boundary`].
///
/// As três metades, e cada uma mata uma ponte diferente:
/// 1. **com UM segmento a torção move barro** — mata a inversão em falta;
/// 2. **com UM segmento as doze curvas dão o MESMO** (todas valem `1` em
///    `curva(1)`, §5.2) — mata uma inversão a mais, que faria a curva decidir
///    onde a espec diz que ela não decide;
/// 3. **com DOIS segmentos elas divergem, e a mais afiada move MENOS** — mata
///    uma ponte que devolvesse a constante `1`.
#[test]
fn a_curva_do_pincel_chega_ao_modo_de_torcao() {
    /// O arrasto de ecrã que a torção lê, em pixels (§5.2 — é o único número
    /// deste pincel que vem do ecrã).
    const ARRASTO_PX: f32 = 40.0;

    let torcer = |falloff: crate::Falloff, segmentos: u32| {
        let mut malha = esfera();
        let mut s = SculptStroke::default();
        s.begin(&malha);
        let b = Brush {
            falloff,
            pose: crate::PoseControlos {
                // ⚠️ **Escolhida no PAINEL desde 2026-09-15** — até então a
                // torção era a metade escondida do `Rotate / Twist`, atrás do
                // `Ctrl`, e este gate armava-a com `invert: true`.
                deformacao: ph2d_pose::Deformacao::Torcer,
                segmentos,
                arrasto_x_pixels: ARRASTO_PX,
                ..crate::PoseControlos::default()
            },
            ..pincel()
        };
        let antes = malha.positions().to_vec();
        s.dab(
            &mut malha,
            &b,
            &puxao([0.0, 0.0, 1.0], b.radius, [0.0; 3]),
            Symmetry::default(),
        );
        antes
            .iter()
            .zip(malha.positions())
            .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max)
    };

    // (1) — o valor de FÁBRICA, que é onde o defeito vivia.
    for f in crate::Falloff::ALL {
        let d = torcer(f, 1);
        assert!(
            d > 1e-2,
            "com UM segmento (o valor de fabrica) a torcao moveu {d:.4e} com a \
             curva {f:?} — o 1.o segmento esta' a receber atenuacao ZERO, que e' \
             a ponte ligada sem a inversao"
        );
    }
    // (2) — ali a curva NÃO decide, e é a espec que o diz.
    let base = torcer(crate::Falloff::Constant, 1);
    for f in crate::Falloff::ALL {
        assert_eq!(
            torcer(f, 1),
            base,
            "com UM segmento a curva {f:?} mudou a saida — a espec amostra em \
             `1 - i/n`, logo o unico segmento le sempre `curva(1)`, que vale `1` \
             nas doze"
        );
    }
    // (3) — com mais segmentos ela decide, e na direcção certa.
    let (constante, afiada) = (
        torcer(crate::Falloff::Constant, 2),
        torcer(crate::Falloff::Sharper, 2),
    );
    assert!(
        afiada < constante * 0.9,
        "com DOIS segmentos a curva afiada moveu {afiada:.4e} contra {constante:.4e} \
         da constante — a curva nao esta' a chegar ao 2.o segmento"
    );
}

/// ⭐⭐⭐ **A AUTO-SUAVIZAÇÃO DESTE PINCEL SEGUE OS PESOS, E NÃO O RAIO.**
///
/// A espec manda-o com todas as letras (§15 e item 18 da lista de verificação:
/// *«**Não copiar**: se oferecermos auto-suavização aqui, ela segue os pesos,
/// não o raio»*), porque copiar o passe genérico reproduziria um defeito que os
/// próprios autores do alvo registam em público — ele alisa dentro do raio
/// inicial enquanto a deformação alcança muito mais longe, e *«o efeito dela
/// desaparece longe do cursor»*.
///
/// ⚠️ **Antes desta wave a fileira era PINTADA e INERTE:** o verbo desvia antes
/// do laço por-vértice onde o passe genérico corre, logo o artista arrastava o
/// controlo e o barro não sentia nada (medido pelo censo dos knobs,
/// `0,000e0` entre as duas pontas da faixa).
///
/// As quatro metades, e cada uma mata uma implementação diferente:
/// 1. **alisa alguma coisa** — o controlo positivo, sem o qual as outras três
///    são afirmações sobre o vácuo;
/// 2. **alcança MUITO além do raio do carimbo** — a grandeza que o dono vê;
/// 3. **não toca o outro lado da peça** — mata um alisamento global, que
///    alisaria a malha inteira a cada evento de ponteiro;
/// 4. **a MÁSCARA protege** — mata um passe que ignore o factor do §9, e é ela
///    que prende a passagem pela porta [`ph2d_pose::Fatores::de`].
///
/// ⚠️⚠️ **A metade (2) não tem mutação que a mate SOZINHA, e a razão é
/// estrutural e vale mais que uma:** a função do alisamento **não recebe o
/// `Dab`**, logo um passe preso ao carimbo não é exprimível ali sem mudar a
/// assinatura. *Uma propriedade que o compilador impede é mais forte que uma
/// que um gate mede* — o número fica na mesma, porque ele é o que separa esta
/// lei da que a referência ship.
#[test]
fn a_auto_suavizacao_da_pose_segue_os_pesos_e_nao_o_raio() {
    const CENTRO: [f32; 3] = [0.0, 0.0, 1.0];
    /// O lado OPOSTO da bola: um `z` abaixo disto está fora da região da cadeia
    /// neste arranjo por **construção geométrica**, não por medição — o cursor
    /// está no pólo `+z` e a região cresce pela ligação da malha a partir dele.
    const OUTRO_LADO: f32 = -0.5;

    let corre = |auto_smooth: f32, mascarar: bool| {
        let mut malha = esfera();
        if mascarar {
            // Metade NORTE mascarada: ela é onde a região da cadeia vive, logo
            // é exactamente onde a máscara tem de ser observável.
            let z: Vec<f32> = malha.positions().iter().map(|p| p[2]).collect();
            let m = malha.masks_mut();
            for (i, &zi) in z.iter().enumerate() {
                m[i] = if zi > 0.0 { 1.0 } else { 0.0 };
            }
        }
        let mut s = SculptStroke::default();
        s.begin(&malha);
        let b = Brush {
            auto_smooth,
            ..pincel()
        };
        s.dab(
            &mut malha,
            &b,
            &puxao(CENTRO, b.radius, [0.25, 0.0, 0.0]),
            Symmetry::default(),
        );
        malha.positions().to_vec()
    };
    let repouso = esfera().positions().to_vec();
    let sem = corre(0.0, false);
    let com = corre(1.0, false);

    let alisados: Vec<usize> = (0..sem.len()).filter(|&i| sem[i] != com[i]).collect();
    // (1) — o controlo positivo.
    assert!(
        alisados.len() > 50,
        "o auto-smooth mexeu em {} vertices — com a fileira PINTADA e o barro \
         parado, o artista arrasta o controlo e nao acontece nada",
        alisados.len()
    );
    // (2) — o alcance é o da REGIÃO, não o do carimbo.
    let raio = pincel().radius;
    let mais_longe = alisados
        .iter()
        .map(|&i| {
            (0..3)
                .map(|k| (repouso[i][k] - CENTRO[k]).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        mais_longe > raio * 1.5,
        "o vertice alisado mais distante esta' a {mais_longe:.3} de um raio de \
         {raio:.3} ({:.2}x) — isto e' o passe GENERICO preso ao carimbo, que e' \
         o defeito que a espec §15 manda nao copiar",
        mais_longe / raio
    );
    // (3) — e ele pára onde a cadeia pára.
    let fora: Vec<usize> = alisados
        .iter()
        .copied()
        .filter(|&i| repouso[i][2] < OUTRO_LADO)
        .collect();
    assert!(
        fora.is_empty(),
        "{} vertices do OUTRO LADO da peca foram alisados — o passe esta' a \
         correr sobre a malha inteira em vez de seguir os pesos da cadeia",
        fora.len()
    );
    // (4) — a máscara protege, e ela entra pela MESMA porta que atenua o
    // deslocamento (§9). ⚠️ A régua tem de comparar os dois lados COM a máscara
    // posta: comparar contra a corrida sem máscara mediria a máscara a proteger
    // do PINCEL, que já tem gate próprio, e não do alisamento.
    let (sem_mascarado, com_mascarado) = (corre(0.0, true), corre(1.0, true));
    let alisados_sob_mascara = (0..sem_mascarado.len())
        .filter(|&i| sem_mascarado[i] != com_mascarado[i])
        .filter(|&i| repouso[i][2] > 0.0)
        .count();
    assert_eq!(
        alisados_sob_mascara, 0,
        "{alisados_sob_mascara} vertices TOTALMENTE mascarados foram alisados — \
         o passe nao esta' a atenuar pelo factor do §9, e um vertice que a \
         mascara prende nao pode ser alisado por baixo dela"
    );
    // (5) — ⛔⛔ **UM `NaN` NÃO ALISA NADA**, e esta metade nasceu de uma mutação
    // SOBREVIVENTE: apagar a consulta à porta deixava as quatro anteriores
    // verdes, porque no ponto neutro o orçamento já devolve uma passada de peso
    // `0`. O que a porta de facto compra é o **peneiro do não-finito**, e a
    // primeira redacção desta metade tinha a premissa errada — eu esperava a
    // malha a virar `NaN`, e `NaN.min(1,0)` devolve **`1,0`** em Rust ⇒ o que
    // acontece sem a guarda é o contrário: um param mal carregado alisa a
    // **FORÇA CHEIA**, calado. *A saída do defeito não é lixo visível; é a
    // ferramenta a fazer o máximo onde o artista pediu nada.*
    let com_nan = corre(f32::NAN, false);
    assert_eq!(
        com_nan, sem,
        "com `auto_smooth = NaN` a peca ficou diferente da corrida SEM \
         alisamento — o passe nao esta' a passar pela porta que peneira o \
         nao-finito, e um param mal carregado alisa a forca cheia"
    );
}

/// ⛔⛔⛔ **O `Ctrl` NÃO TROCA A DEFORMAÇÃO DA POSE** — ordem do dono
/// (2026-09-15: *«não devem ser ativados com CTRL mas checando o botão no
/// painel»*).
///
/// Com as cinco deformações alcançáveis no painel, o modificador seria a
/// **segunda** maneira de dizer a mesma coisa — e uma que **compõe** com a
/// primeira: escolher `Twist` no painel e carregar `Ctrl` devolveria `Rotate`, e
/// o artista leria isso como *«o botão não funciona»*. É o argumento que o
/// [`Verb::honours_invert`] já escreve para os dois [`crate::Grip::Turn`],
/// aplicado aqui.
///
/// ⚠️ **A régua é o BARRO e é AO BIT**, sobre as **cinco** — uma barra frouxa
/// aceitaria o `Ctrl` a mudar alguma coisa de leve, e o que se afirma é que ele
/// não chega ao verbo de todo.
///
/// ⚠️ **O controlo positivo vive no `deformacao`, não aqui:** o gate irmão
/// `a_deformacao_escolhida_muda_o_gesto` prova que as cinco escolhas dão saídas
/// diferentes — sem ele, este ficaria verde sobre um pincel que não faz nada.
#[test]
fn o_ctrl_nao_troca_a_deformacao_da_pose() {
    for d in ph2d_pose::Deformacao::ALL {
        let corre = |invert: bool| {
            let mut malha = esfera();
            let mut s = SculptStroke::default();
            s.begin(&malha);
            let b = Brush {
                invert,
                pose: crate::PoseControlos {
                    deformacao: d,
                    // A torção lê pixels de ecrã; sem eles ela é inerte e o
                    // par de corridas seria idêntico por vácuo.
                    arrasto_x_pixels: 40.0,
                    ..crate::PoseControlos::default()
                },
                ..pincel()
            };
            s.dab(
                &mut malha,
                &b,
                &puxao([0.0, 0.0, 1.0], b.radius, [0.0, 0.0, 0.25]),
                Symmetry::default(),
            );
            malha.positions().to_vec()
        };
        assert_eq!(
            corre(false),
            corre(true),
            "com `{d:?}` escolhido no painel, carregar Ctrl mudou o barro — o \
             modificador voltou a chegar a este verbo, e ele COMPÕE com a \
             escolha: quem escolher `Twist` e carregar Ctrl volta a `Rotate`"
        );
    }
}

/// ⭐⭐ **A ESCOLHA DO PAINEL MUDA O GESTO** — o controlo positivo do gate acima,
/// e a prova de que as cinco deformações são cinco leis e não cinco rótulos.
///
/// ⚠️ **O arrasto tem componente nos dois eixos de propósito:** a escala e o
/// espremer lêem a componente **ao longo** da cadeia e a rotação lê a
/// **transversal** — um arrasto num eixo só deixaria duas das cinco inertes, e o
/// gate leria isso como duas leis iguais.
#[test]
fn a_deformacao_escolhida_muda_o_gesto() {
    let saidas: Vec<Vec<[f32; 3]>> = ph2d_pose::Deformacao::ALL
        .into_iter()
        .map(|d| {
            let mut malha = esfera();
            let mut s = SculptStroke::default();
            s.begin(&malha);
            let b = Brush {
                pose: crate::PoseControlos {
                    deformacao: d,
                    arrasto_x_pixels: 40.0,
                    ..crate::PoseControlos::default()
                },
                ..pincel()
            };
            s.dab(
                &mut malha,
                &b,
                &puxao([0.0, 0.0, 1.0], b.radius, [0.18, 0.0, 0.18]),
                Symmetry::default(),
            );
            malha.positions().to_vec()
        })
        .collect();
    let repouso = esfera().positions().to_vec();
    for (i, d) in ph2d_pose::Deformacao::ALL.into_iter().enumerate() {
        assert_ne!(
            saidas[i], repouso,
            "`{d:?}` não moveu um único vértice — o botão existe e a lei não"
        );
        for (j, outra) in ph2d_pose::Deformacao::ALL
            .into_iter()
            .enumerate()
            .skip(i + 1)
        {
            assert_ne!(
                saidas[i], saidas[j],
                "`{d:?}` e `{outra:?}` dão o MESMO barro — dois botões, uma lei"
            );
        }
    }
}

/// ⛔⛔ **NO `Scale`, O GESTO RODA **E** ESCALA — e a caixa que tira a rotação
/// funciona.**
///
/// Pergunta do dono (2026-09-15): *«Em Scale o osso escalona e rotaciona ao
/// mesmo tempo. Isso é o esperado?»* — **é**, e é a lei da referência: a espec
/// §5.4 passo 1 manda **resolver a cadeia primeiro** (o mesmo passo do §5.1) e
/// só depois aplicar o quociente, *«com a trava desligada o gesto roda e escala;
/// com ela ligada, escala sem rodar»*. A trava nasce **desligada** (é o valor em
/// `68` das `69` fixturas do oráculo).
///
/// ⚠️⚠️ **O discriminador é a DIRECÇÃO DO OSSO, e a primeira régua que escrevi
/// era cega:** eu media o deslocamento lateral contra o de profundidade, e leu
/// `4,36e-1` contra `4,42e-1` — *quase nada* — porque **uma escala em torno de um
/// pivô também move os vértices de lado**. A grandeza que separa as duas é a
/// direcção: uma rotação vira o osso, uma escala pura só lhe muda o comprimento.
///
/// ⚠️ **E o comprimento do osso é o MESMO nos dois lados** (`0,0687`), de
/// propósito: o quociente vive no mapa do segmento (`seg.escala`), não no
/// comprimento dele — *o indicador mostra onde o membro APONTA, não quanto ele
/// engordou*.
#[test]
fn a_trava_de_rotacao_tira_a_rotacao_da_escala() {
    let direccao_do_osso = |trava_rotacao: bool| {
        let mut malha = esfera();
        let mut s = SculptStroke::default();
        s.begin(&malha);
        let b = Brush {
            pose: crate::PoseControlos {
                deformacao: ph2d_pose::Deformacao::Escalar,
                trava_rotacao,
                ..crate::PoseControlos::default()
            },
            ..pincel()
        };
        s.dab(
            &mut malha,
            &b,
            &puxao([0.0, 0.0, 1.0], b.radius, [0.18, 0.0, 0.18]),
            Symmetry::default(),
        );
        let ctrl = b.pose.lei(&b);
        let mut ossos = Vec::new();
        s.pose_sessao()
            .expect("a sessão da pose vive durante o traço")
            .ossos(&ctrl, &mut ossos);
        let [o, c] = ossos[0];
        let v = [c[0] - o[0], c[1] - o[1], c[2] - o[2]];
        let comp = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!(comp > 1e-6, "o osso degenerou e a régua mede ruído");
        ([v[0] / comp, v[1] / comp, v[2] / comp], comp)
    };
    let (solta, comp_solta) = direccao_do_osso(false);
    let (travada, comp_travada) = direccao_do_osso(true);

    // (1) — com a trava DESLIGADA o osso RODA: ele sai do eixo em que nasceu.
    assert!(
        solta[0].abs() > 0.3,
        "com a trava desligada o osso ficou em {solta:?} — ele devia RODAR, que \
         é o passo 1 do §5.4, e é o que o dono vê"
    );
    // (2) — com a trava LIGADA ele fica no eixo, e é isso que o nome promete.
    assert!(
        travada[0].abs() < 1e-3 && travada[1].abs() < 1e-3,
        "com a trava ligada o osso ficou em {travada:?} — `Scale without \
         rotating` prometeu tirar a rotação e não tirou"
    );
    // (3) — e o COMPRIMENTO não é onde a escala vive: as duas leem igual, o que
    // impede alguém de ler a metade (2) como «a trava desligou a escala».
    assert!(
        (comp_solta - comp_travada).abs() < 1e-6,
        "o comprimento do osso mudou com a trava ({comp_solta} contra \
         {comp_travada}) — o quociente vive no mapa do segmento, e se ele \
         passou a viver aqui as duas metades acima deixaram de medir o que dizem"
    );
}
