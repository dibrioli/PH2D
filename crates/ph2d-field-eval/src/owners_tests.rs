//! Os gates da lei do dono. Ver [`super`].

use super::Owners;
use crate::hybrid::Registry;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

/// Uma esfera posta em `x`, como um documento de um nó — a forma que o [`Owners`] recebe.
fn ball_at(x: f32, radius: f32) -> FieldDoc {
    FieldDoc::new(
        vec![crate::leaf(
            Primitive::Sphere { radius },
            Xform::at(x, 0.0, 0.0),
        )],
        NodeId(0),
    )
    .expect("a esfera posta")
}

/// A margem com que estes gates trabalham — a mesma ordem de grandeza da tolerância de acerto da
/// marcha (`2e-4`), e não um número escolhido para o teste passar.
const MARGEM: f32 = 1.0e-3;

/// ⭐⭐ **Um ponto sobre a superfície de uma folha é DAQUELA folha** — as três, uma a uma.
///
/// ⚠️ **O ponto é posto sobre a superfície, não procurado:** é assim que ele chega do traçado (o
/// `Gbuffer::point`), e é o único sítio onde a pergunta tem uma resposta certa. No meio de uma união
/// qualquer das duas é plausível, e um gate que apontasse ali passaria com a resposta errada.
#[test]
fn a_point_on_a_leaf_belongs_to_that_leaf() {
    let reg = Registry::new();
    let docs = [ball_at(-0.5, 0.2), ball_at(0.0, 0.2), ball_at(0.5, 0.2)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    assert_eq!(owners.len(), 3);
    for (i, centro) in [-0.5f32, 0.0, 0.5].into_iter().enumerate() {
        // O pólo de cada esfera: só ela existe ali.
        assert_eq!(
            owners.at([centro, 0.0, 0.2]),
            Some(i),
            "o pólo da esfera {i} foi dado a outra"
        );
        assert_eq!(
            owners.at([centro - 0.2, 0.0, 0.0]),
            Some(i),
            "a ponta esquerda da esfera {i} foi dada a outra"
        );
        // ⭐⭐ **E custa UMA folha, não três** — é aqui que a bola à frente tem de funcionar, porque
        // é aqui que o ponto de facto chega (sobre a superfície). *Um gate sobre a resposta é cego
        // ao preço: sem esta linha, apagar a margem do filtro deixa tudo verde pelo caminho caro.*
        assert_eq!(
            owners.at_counting([centro, 0.0, 0.2]).1,
            1,
            "o pólo da esfera {i} pagou mais do que uma folha"
        );
    }
}

/// ⭐⭐⭐ **A MARGEM é obrigatória** — a 1.ª redacção da sonda que mediu esta lei testava `d² ≤ r²` e
/// a rede disparou em `26 216` de `26 216` pixels.
///
/// A marcha pára quando o campo desce abaixo de uma tolerância, isto é **ligeiramente FORA** da
/// superfície. Aqui isso é encenado: o ponto está a `margem/2` de fora, que é exactamente o que o
/// traçador entrega.
///
/// ⛔⛔ **E ele mede o PREÇO, não só a resposta — porque a 1.ª redacção media a resposta e a
/// mutação SOBREVIVEU.** Sem margem nenhuma bola contém o ponto, a **rede** dispara, e a resposta
/// sai **certa pelo caminho caro**: os quatro gates ficavam verdes sobre a optimização apagada.
/// *Uma optimização cuja ausência não se vê na saída precisa de um gate sobre o TRABALHO.*
///
/// **Mutação que deve sangrar:** `let r = b.radius;` — `visitadas` salta de `1` para `2`.
#[test]
fn a_point_just_outside_the_surface_still_finds_its_leaf_without_paying_for_the_others() {
    let reg = Registry::new();
    let docs = [ball_at(-0.5, 0.2), ball_at(0.5, 0.2)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    // `margem/2` para FORA do pólo da segunda esfera — é assim que o ponto chega da marcha.
    let fora = [0.5f32, 0.0, 0.2 + MARGEM * 0.5];
    let (dono, visitadas) = owners.at_counting(fora);
    assert_eq!(
        dono,
        Some(1),
        "um ponto a meia margem da superfície perdeu o dono"
    );
    assert_eq!(
        visitadas, 1,
        "a margem não segurou o filtro: {visitadas} folhas visitadas em vez de 1 — a resposta está \
         certa e veio pelo caminho CARO"
    );
    // E o controlo da REDE: muito fora de tudo, ela dispara e paga as duas — de propósito.
    let (longe, pagas) = owners.at_counting([0.5, 0.0, 5.0]);
    assert_eq!(longe, Some(1), "a rede não devolveu a folha mais próxima");
    assert_eq!(
        pagas, 2,
        "a rede tem de perguntar a TODAS — é isso que ela é"
    );
}

/// ⭐ **A bola à frente não muda a RESPOSTA** — ela muda o preço.
///
/// ⚠️ Este é o gate que separa *«mais rápido»* de *«mais rápido e errado»*: a sonda que mediu o
/// ganho (`pick_tests::measure_what_a_material_per_object_would_cost`, `8,3×`) traz o mesmo assert,
/// e aqui ele corre **sem** relógio nenhum — logo não flaka sob carga.
///
/// ⚠️ **Este varrimento afirma a RESPOSTA e mais nada, e é de propósito:** a maior parte destes
/// pontos está longe de toda a peça, e ali a rede **deve** pagar as oito. O preço afirma-se onde o
/// filtro tem de funcionar — sobre a SUPERFÍCIE —, e isso é o
/// [`a_point_on_a_leaf_belongs_to_that_leaf`].
#[test]
fn the_ball_in_front_changes_the_price_never_the_answer() {
    let reg = Registry::new();
    let docs: Vec<FieldDoc> = (0..8)
        .map(|i| ball_at((i as f32 - 3.5) * 0.25, 0.1))
        .collect();
    let com_bola = Owners::new(&docs, &reg, MARGEM);
    // A MESMA lei com a bola desligada: uma margem enorme faz toda bola conter todo ponto, logo o
    // filtro nunca elimina ninguém. ⛔ Não é uma segunda implementação — é a mesma, sem o corte.
    let sem_bola = Owners::new(&docs, &reg, 1.0e6);
    let mut vistos = [false; 8];
    for k in 0..200 {
        let t = k as f32 / 199.0;
        let p = [(t - 0.5) * 2.2, (t - 0.5) * 0.3, 0.05];
        let a = com_bola.at(p);
        assert_eq!(
            a,
            sem_bola.at(p),
            "a bola à frente mudou a resposta em {p:?}"
        );
        if let Some(i) = a {
            vistos[i] = true;
        }
    }
    // ⛔ **O piso de população:** sem ele um varrimento que só tocasse uma folha passaria o assert
    // acima sem ter comparado nada.
    assert!(
        vistos.iter().filter(|v| **v).count() >= 6,
        "o varrimento só alcançou {} das 8 folhas — o gate não comparou nada",
        vistos.iter().filter(|v| **v).count()
    );
}

/// Uma peça sem folhas não tem dono, e a porta diz isso em vez de entrar em pânico.
#[test]
fn a_piece_without_leaves_has_no_owner() {
    let reg = Registry::new();
    let owners = Owners::new(&[], &reg, MARGEM);
    assert!(owners.is_empty());
    assert_eq!(owners.at([0.0; 3]), None);
}

/// Um ponto **SOBRE a superfície** da esfera da esquerda, ao ângulo `phi` (rad) medido do eixo `x`.
///
/// ⛔⛔ **Ele existe porque a 1.ª redacção dos dois gates abaixo escrevia pontos à mão e NENHUM caía
/// na superfície** — e as duas mutações que eles deviam matar **sobreviveram**. A razão é o filtro:
/// fora da superfície, a bola à frente deixa passar **uma** folha só, o `mix_at` devolve *«não há
/// rival»* antes de chegar à conta, e o gate mede um caminho que o produto nunca percorre.
///
/// *É a lei que o topo deste ficheiro já escrevia, à letra: **o ponto é posto sobre a superfície,
/// não procurado** — e a 1.ª redacção destes dois violou-a.*
fn on_left_sphere(phi: f32, x: f32, r: f32) -> [f32; 3] {
    [x + r * phi.cos(), 0.0, r * phi.sin()]
}

/// ⭐⭐⭐ **A MISTURA É `½` NA FRONTEIRA E `0` LONGE DELA** — a lei do [`Owners::mix_at`].
///
/// # ⚠️ As três coisas que ela tem de fazer, e nenhuma basta sozinha
///
/// | afirmação | o que cai sem ela |
/// |---|---|
/// | **empate ⇒ `½`** | a fronteira não é suavizada de todo |
/// | **longe ⇒ `0`** | a peça inteira sai misturada, e cada material perde a cor |
/// | **monótona** | a rampa vai e vem, e o degrau reaparece noutro sítio |
///
/// ⚠️ **O ponto é posto NO PLANO MÉDIO das duas esferas**, e não procurado: é lá que a lei diz `½`, e
/// uma varredura que não lá caísse mediria outra coisa.
#[test]
fn the_mix_is_half_on_the_boundary_and_zero_away_from_it() {
    let reg = Registry::new();
    // Duas esferas que se tocam: o plano médio é `x = 0`.
    let docs = [ball_at(-0.3, 0.35), ball_at(0.3, 0.35)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    const LARGURA: f32 = 0.01;

    // O vinco: onde a superfície da esquerda toca a da direita. `cos φ = 0,3/0,35`.
    let phi_vinco = (0.3f32 / 0.35).acos();
    // ⭐ **No vinco, empate** — as duas distâncias são iguais por simetria.
    let (a, b, t) = owners.mix_at(on_left_sphere(phi_vinco, -0.3, 0.35), LARGURA);
    assert_ne!(a, b, "no plano médio há duas folhas em disputa");
    assert!(
        (t - 0.5).abs() < 1.0e-3,
        "a fronteira tem de dar meio a meio e deu {t}"
    );

    // ⛔ **Longe dela, zero** — senão a peça inteira sai misturada. O pólo da esquerda.
    let (_, _, t) = owners.mix_at(
        on_left_sphere(std::f32::consts::FRAC_PI_2, -0.3, 0.35),
        LARGURA,
    );
    assert!(
        t <= 0.0,
        "um ponto no pólo de uma esfera não pode ter mistura nenhuma e teve {t}"
    );

    // ⭐⭐ **E é MONÓTONA a afastar-se** — sem isto a rampa vai e vem e o degrau só muda de sítio.
    let mut anterior = f32::INFINITY;
    // ⛔⛔ **A rampa TEM de chegar a zero com o rival ainda em jogo** — é isso que separa «a mistura
    // desce» de «o filtro deixou de ver o rival». ⚠️ **E numa ESFERA as duas coisas coincidem por
    // geometria**: a bola envolvente de uma esfera **é** a superfície dela, logo `|f_rival| ≤ width`
    // e *«dentro da bola mais width»* são a MESMA condição, e a asserção seria insatisfazível. ⇒ ela
    // vive na fixtura da CAIXA, cuja bola é folgada — ver
    // [`the_ramp_reaches_zero_while_the_rival_is_still_in_play`].
    let mut houve_rival_com_zero = false;
    for k in 0..=20 {
        // ⚠️ **Ao longo da SUPERFÍCIE**, afastando-se do vinco — é por lá que os pixels andam.
        let phi = phi_vinco + k as f32 * 0.01;
        let (a, b, t) = owners.mix_at(on_left_sphere(phi, -0.3, 0.35), LARGURA);
        assert!(
            t <= anterior + 1.0e-6,
            "a mistura subiu ao afastar-se do vinco (φ = {phi})"
        );
        if a != b && t <= 0.0 {
            houve_rival_com_zero = true;
        }
        anterior = t;
    }
    assert!(
        anterior <= 0.0,
        "a rampa não chegou a zero dentro de duas larguras"
    );
    let _ = houve_rival_com_zero; // ver a nota acima: numa esfera isto não é observável.
    // ⛔⛔ **E o mesmo do lado da OUTRA esfera** — o que mata a mutação «o segundo melhor é o
    // primeiro». Ali o dono é a folha `1`, que é visitada **depois** da `0` no percurso: quem
    // escrever o desempate sem guardar o anterior perde o rival exactamente neste lado, e no outro
    // não. *Uma lei de ordem tem de ser medida nas duas ordens.*
    // ⚠️ **Um pouco PASSADO o vinco**, e não em cima dele: no vinco as duas empatam e o desempate
    // devolve a primeira nos dois lados — o gate não distinguiria as ordens. Aqui a folha `1` ganha
    // de facto, e ela é visitada **depois** da `0`.
    let espelhado = on_left_sphere(phi_vinco + 0.02, -0.3, 0.35);
    let (a, b, t) = owners.mix_at([-espelhado[0], 0.0, espelhado[2]], LARGURA);
    assert_eq!(a, 1, "do lado direito o dono é a folha da direita");
    assert_ne!(
        b, a,
        "⛔ do lado direito o rival desapareceu — quem ganha é visitado DEPOIS, e um desempate que \
         não guarde o anterior perde-o exactamente aqui (e não do outro lado)"
    );
    assert!(
        t > 0.0,
        "a um passo do vinco a mistura ainda tem de estar viva e deu {t}"
    );
}

/// ⭐⭐ **A LARGURA MANDA NA RAMPA** — dobrar a largura dobra o alcance da mistura.
///
/// ⛔⛔ **E a MARGEM DA BOLA segue a largura, não a da marcha** — foi isto que fez a 1.ª versão desta
/// porta ser **muda** numa união dura: o rival vinha filtrado pela bola à frente (cuja margem é a
/// tolerância do ponto, uns `2e-4`) e a mistura respondia *«não há rival»* a um pixel da fronteira.
/// *A pergunta do filtro aqui não é «quem pode GANHAR?», é «quem pode estar a menos de uma LARGURA
/// de ganhar?».*
///
/// **Mutação que deve sangrar:** `self.visit(p, self.margin, …)` no `mix_at`.
#[test]
fn the_width_drives_the_ramp_and_the_ball_filter_follows_it() {
    let reg = Registry::new();
    let docs = [ball_at(-0.3, 0.35), ball_at(0.3, 0.35)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    // ⚠️ **Um ponto SOBRE a superfície**, um pouco depois do vinco: ali ele já está **fora** da bola
    // da esfera rival por mais do que a tolerância da marcha — que é exactamente a configuração em
    // que o filtro decide, e a que a 1.ª redacção deste gate não tinha.
    let p = on_left_sphere((0.3f32 / 0.35).acos() + 0.035, -0.3, 0.35);
    let fina = owners.mix_at(p, 0.002).2;
    let larga = owners.mix_at(p, 0.02).2;
    assert!(
        fina <= 0.0,
        "com a rampa fina este ponto já devia estar fora dela e deu {fina}"
    );
    assert!(
        larga > 0.1,
        "⛔ com a rampa larga o ponto tem de estar dentro dela e deu {larga} — quase sempre é o \
         FILTRO da bola, que precisa de seguir a largura e não a tolerância da marcha"
    );
    // ⚠️ E o dono nunca muda com a largura: ela mexe no PESO, nunca na resposta.
    assert_eq!(owners.mix_at(p, 0.002).0, owners.mix_at(p, 0.02).0);
    assert_eq!(owners.at(p), Some(owners.mix_at(p, 0.02).0));
}

/// ⭐⭐⭐ **A RAMPA CHEGA A ZERO COM O RIVAL AINDA EM JOGO** — o que separa *«a mistura desce»* de
/// *«o filtro deixou de ver o rival»*.
///
/// # ⛔⛔ Porque ela precisa de uma CAIXA e não de uma esfera
///
/// A bola envolvente de uma **esfera** é a própria superfície dela: `|f_rival| ≤ width` e *«dentro da
/// bola mais `width`»* são a **mesma condição**, e a asserção seria insatisfazível — a mutação «a
/// mistura nunca desce» passaria, porque o `mix_at` sai por *«não há rival»* **antes** da conta.
///
/// ⭐ Numa **caixa** a bola é folgada (ela envolve os cantos), logo o rival continua a ser visitado
/// muito depois de a rampa ter acabado — e é aí que a conta é observável.
///
/// *A forma da fixtura não é um detalhe: ela decide QUE CAMINHO do produto o gate percorre.*
///
/// **Mutação que deve sangrar:** `(bi, si, 0.5)` — uma mistura presa a meio.
#[test]
fn the_ramp_reaches_zero_while_the_rival_is_still_in_play() {
    let reg = Registry::new();
    let caixa = |x: f32| {
        FieldDoc::new(
            vec![crate::leaf(
                Primitive::Box {
                    half: [0.3; 3],
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform::at(x, 0.0, 0.0),
            )],
            NodeId(0),
        )
        .expect("a caixa posta")
    };
    let docs = [caixa(-0.3), caixa(0.3)];
    let owners = Owners::new(&docs, &reg, MARGEM);
    const LARGURA: f32 = 0.01;

    // Percorre a face de cima (`z = 0,3`) da caixa da esquerda, a afastar-se do plano `x = 0`.
    let mut houve = false;
    let mut anterior = f32::INFINITY;
    for k in 0..=40 {
        let x = -(k as f32) * 0.005;
        let (a, b, t) = owners.mix_at([x, 0.0, 0.3], LARGURA);
        assert!(
            t <= anterior + 1.0e-6,
            "a rampa subiu ao afastar-se (x = {x})"
        );
        if a != b && t <= 0.0 {
            houve = true;
        }
        anterior = t;
    }
    assert!(
        houve,
        "⛔ o varrimento nunca passou por um ponto COM rival e SEM mistura — sem ele, uma mistura \
         presa em `½` passa neste gate"
    );
}
