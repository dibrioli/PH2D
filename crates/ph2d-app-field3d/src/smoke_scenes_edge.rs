//! ⭐⭐ **AS CENAS DOS DOIS RECUOS DE UMA ARESTA** (Enio, 2026-08-30) — o chanfro, a costura entre
//! cópias, e o prisma do report.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::field3d_smoke_scenes`] é o **roteador**; estas três são um assunto fechado dentro dele
//! (*«o que os dois recuos fazem, e onde eles falharam»*), e o arquivo passou as `600` linhas do
//! gate de LOC do shell. ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **O gate que o apanhou vive em `shells/desktop/tests/`** — e o `cargo test --bins` **não lhe
//! toca**. É a mesma cegueira que o §5 do `CLAUDE.md` já nomeia.

// ⚠️ Módulo-filho do roteador: o `use super::*` traz os construtores (`leaf`, `combine`) que ele
// já tem, e que continuam a existir **uma vez**.
use super::*;

/// ⭐⭐⭐ **AS CINCO JUNTAS NOVAS, lado a lado sobre a MESMA peça** (W145, pedido do Enio de 09/09).
///
/// # Por que um SALIENTE SOBRE UMA CHAPA, e não duas caixas a cruzar
///
/// A costura de um saliente sobre uma chapa é um **anel fechado**, e é a figura em que estas juntas
/// existem para trabalhar: um cordão de solda corre à volta da base, uma linha de painel contorna-a,
/// um friso reforça-a. ⚠️ **Duas caixas a cruzar dariam uma costura RECTA**, e uma decoração recta
/// lê-se como um bisel — *a metade que interessa é a costura VIRAR, e só uma costura fechada a
/// mostra*.
///
/// ⭐ E as seis peças são a **mesma geometria**: o que muda de uma para a outra é uma palavra.
pub fn cena_32() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 32 — AS JUNTAS NOVAS, da esquerda para a direita: \
         Fillet (referência) · Soft · Bead · Groove · Ridge · Chamfer desigual"
    );
    println!(
        "[field-smoke]            a peça é a MESMA nas seis; o que muda é o carácter da junta \
         entre o saliente e a chapa."
    );
    const PASSO: f32 = 0.62;
    const R: f32 = 0.06;
    // ⚠️⚠️⚠️ **A PENETRAÇÃO É LOAD-BEARING, e foi um report que o ensinou** (Enio, 09/09): o
    // saliente entra `0,09` numa chapa de `0,18`, e é essa sobreposição que a guarda do sulco
    // protege. Com uma penetração RASA (a 1.ª versão desta cena entrava `0,04`) o vinco fica a menos
    // de um raio de canal de toda a face de baixo do saliente, e nem a guarda o salva.
    //
    // ⚠️⚠️⚠️ **E A ESPESSURA TEM UM NÚMERO DERIVADO, não escolhido:** o sulco localiza-se por
    // `‖(a,b)‖`, e entre duas faces PARALELAS sem relação (a base do saliente e o fundo da chapa) o
    // mínimo dessa distância é `h/√2`, onde `h` é o vão entre elas. ⇒ para o canal não morder ali é
    // preciso **`h > R·√2`** — aqui `h = 0,12` contra `R·√2 = 0,085`. *Com `h = 0,08` ele mordia, e
    // foi o gate da perfuração que o disse.*
    //
    // ⚠️ A espessura também importa pelo óbvio: o sulco escava `R` a partir da superfície, e numa
    // chapa fina ele **perfura** — o que se veria como um rasgo à volta da base e se leria como
    // um defeito da peça, não como a feição. Com `0,14` de espessura e `R = 0,06` sobram `0,08`, e o
    // gate `the_new_junctions_scene_does_not_perforate_the_plate` mede-o.
    //
    // ⚠️ **E o saliente ENTRA na chapa** (`z` de `−0,04` a `0,44`) em vez de pousar nela: duas faces
    // exactamente coincidentes são o caso degenerado de toda booleana, e a costura de uma união
    // assim é uma REGIÃO em vez de uma curva.
    let chapa = |x: f32| {
        leaf(
            Primitive::Box {
                half: [0.26, 0.26, 0.11],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, -0.11],
                ..Xform::IDENTITY
            },
        )
    };
    let saliente = |x: f32| {
        leaf(
            Primitive::Box {
                half: [0.12, 0.12, 0.28],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.18],
                ..Xform::IDENTITY
            },
        )
    };
    let juntas = [
        Blend::Exact { radius: R },
        Blend::Soft { radius: R },
        Blend::Bead { radius: R },
        Blend::Groove { radius: R },
        Blend::Ridge {
            radius: R,
            width: R * Blend::SEAM_WIDTH_RATIO,
        },
        Blend::Bevel {
            radius: R,
            bias: 3.0,
        },
    ];
    let mut nodes = Vec::new();
    let mut grupos = Vec::new();
    for (i, b) in juntas.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = (i as f32 - 2.5) * PASSO;
        let base = NodeId(nodes.len() as u32);
        nodes.push(chapa(x));
        nodes.push(saliente(x));
        grupos.push(NodeId(nodes.len() as u32));
        nodes.push(combine(Op::Union(b), vec![base, NodeId(base.0 + 1)]));
    }
    // ⚠️ **O topo é `Sharp`**, e é load-bearing: com uma mistura aqui as seis peças derretiam umas
    // nas outras e nenhuma das seis leituras seria a da junta que ela nomeia.
    let raiz = NodeId(nodes.len() as u32);
    nodes.push(combine(Op::Union(Blend::Sharp), grupos));
    FieldDoc::new(nodes, raiz)
}

/// ⭐⭐⭐ **A LUZ QUE ATRAVESSA A PEÇA** (17/09, `docs/Render3d/10`) — a PAREDE FINA e a MACIÇA, lado
/// a lado, sobre a mesma forma.
///
/// # ⚠️ A cena é DUAS peças e não uma, e a razão é que são DOIS fenómenos
///
/// A subsuperfície do OpenPBR não é um grau de um efeito — é uma escolha entre dois:
///
/// | a peça | o que se vê |
/// |---|---|
/// | **parede fina** (uma folha, uma pétala, um abajur) | com a luz ATRÁS ela acende inteira |
/// | **maciça** (jade, cera, mármore fino) | a luz CONTORNA a quina e o terminador amacia |
///
/// ⛔ Uma cena com uma peça só ensinaria metade e deixaria a outra a parecer um knob sem efeito.
///
/// # ⚠️ A ESPESSURA da fina é load-bearing, e ela não entra na lei
///
/// A lei da parede fina não lê espessura nenhuma — ela é a lambertiana do lado de lá. Mas o que o
/// artista VÊ depende da peça parecer fina: a mesma lei numa bola maciça lê-se como *«a bola ficou
/// clara»*, e numa lâmina lê-se como *«a luz passa através»*. ⇒ a fina é uma **lâmina** de `0,03`
/// contra `0,90` de largura, que é a proporção de uma folha.
///
/// ⚠️ **E a maciça é uma esfera**, pela razão oposta: a lei dela mede a CURVATURA, e uma lâmina é
/// plana — ali o piso do GLSL entregaria o raio de `100` e a lei ficaria indistinguível de uma
/// difusa. *Cada metade tem a forma que a lei dela precisa.*
pub fn cena_33() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 33 — A LUZ QUE ATRAVESSA A PECA. A' esquerda uma LAMINA (a folha), a' \
         direita uma ESFERA (o jade). As duas abrem OPACAS de proposito."
    );
    println!(
        "[field-smoke]            (1) MODEL · Shading · Render — (2) escolha a LAMINA e suba \
         `Subsurface` ao maximo, e ligue `Thin Walled` — (3) arraste a LUZ para TRAS da peca: ela \
         acende inteira."
    );
    println!(
        "[field-smoke]            (4) escolha a ESFERA, suba `Subsurface`, deixe `Thin Walled` em \
         `Solid` e baixe `Subsurface Radius` — a luz contorna a quina em vez de parar nela."
    );
    let lamina = leaf(
        Primitive::Box {
            half: [0.45, 0.42, 0.015],
            round: 0.012,
            chamfer: 0.0,
        },
        Xform {
            translation: [-0.55, 0.0, 0.0],
            ..Xform::IDENTITY
        },
    );
    let bola = leaf(
        Primitive::Sphere { radius: 0.42 },
        Xform {
            translation: [0.55, 0.0, 0.0],
            ..Xform::IDENTITY
        },
    );
    FieldDoc::new(
        vec![
            lamina,
            bola,
            combine(
                Op::Union(ph2d_field::Blend::Sharp),
                vec![NodeId(0), NodeId(1)],
            ),
        ],
        NodeId(2),
    )
}

/// ⭐⭐⭐ **A CENA DA COR: quatro esferas, a MESMA tinta, QUATRO profundidades** (`docs/Render3d/10`
/// §§17–18; ordem do dono, 2026-09-18: *«faça a cena»*).
///
/// # ⛔⛔⛔ Por que esta cena existe, e é um report do dono que a paga
///
/// A lei da matiz foi construída, medida e ligada a um interruptor — e o dono correu as duas
/// metades lado a lado e viu **a mesma imagem**. A causa não era a lei: a `=33` abre com o material
/// de omissão, que é **CINZENTO**, e *uma lei que redistribui saturação ENTRE canais não tem o que
/// fazer num material onde os três já são iguais*. Medido no quadro inteiro: `1,00` byte de
/// diferença média em cinzento, contra **`23,19`** num jade.
///
/// ⇒ é a espécie que o `CLAUDE.md` §5.0 chama de **pior que uma cena ausente**, um andar acima: não
/// era a cena a ensinar o contrário — era o SMOKE a prometer o que aquela cena não podia mostrar.
///
/// # O desenho, e cada escolha responde a uma medição
///
/// | escolha | porquê |
/// |---|---|
/// | **quatro** esferas | são as **quatro profundidades** do oráculo convergido (`0,03 · 0,10 · 0,30 · 1,00`) |
/// | a **mesma** tinta nas quatro | assim a única coisa que varia entre elas é a PROFUNDIDADE |
/// | raios **IGUAIS** nos três canais | é a família que **isola** a pergunta — sem ela, a matiz também muda por cada canal viajar o seu |
/// | especular a **zero** | é o que a medição usou, e um realce branco por cima lava o que se quer ver |
///
/// ⭐ **O fenómeno vê-se DENTRO de uma imagem** (as quatro deviam escurecer de tom da esquerda para
/// a direita) **e entre as duas corridas** (com o interruptor, a da direita lava para o neutro).
///
/// ⚠️ **Sem `PH2D_SSS_DEPTH_HUE=1` as quatro saem com EXACTAMENTE o mesmo tom** — e isso não é a
/// cena partida, é o defeito que ela existe para mostrar.
pub fn cena_34() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 34 — A COR QUE A PROFUNDIDADE DEIXA. Quatro esferas, a MESMA tinta, e a \
         luz a viajar 0,03 · 0,10 · 0,30 · 1,00 dentro de cada uma."
    );
    println!(
        "[field-smoke]            (1) MODEL · Shading · Render — (2) olhe o TOM das quatro: elas \
         saem TODAS IGUAIS, e e' esse o defeito."
    );
    println!(
        "[field-smoke]            (3) feche, e corra outra vez com PH2D_SSS_DEPTH_HUE=1 — agora a \
         tinta tem de LAVAR para o neutro da esquerda para a direita."
    );
    println!(
        "[field-smoke]            (4) como saber que falhou: se as duas corridas ficarem iguais, o \
         interruptor nao chegou."
    );
    let esferas: Vec<Node> = PROFUNDIDADES_DA_COR
        .iter()
        .enumerate()
        .map(|(i, _)| {
            #[allow(clippy::cast_precision_loss)]
            let x = -1.05 + 0.70 * i as f32;
            leaf(
                Primitive::Sphere { radius: 0.30 },
                Xform {
                    translation: [x, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            )
        })
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    let n = esferas.len() as u32;
    let mut nodes = esferas;
    nodes.push(combine(
        Op::Union(ph2d_field::Blend::Sharp),
        (0..n).map(NodeId).collect(),
    ));
    FieldDoc::new(nodes, NodeId(n))
}

/// As quatro profundidades da cena da cor, em unidades do MUNDO.
///
/// ⛔ **São as do oráculo** (`docs/Render3d/10` §17.3) e não números escolhidos: é contra estas
/// quatro que a lei foi calibrada, logo é sobre estas quatro que a cena pode prometer o que mostra.
pub const PROFUNDIDADES_DA_COR: [f32; 4] = [0.03, 0.10, 0.30, 1.00];

/// ⭐⭐⭐ **A CAMADA DE ESTILO** (`docs/Render3d/03`, a `W8`) — a peça onde os quatro botões têm o que
/// morder.
///
/// # ⚠️ Porque a peça é ESTA e não uma bola
///
/// A tinta por curvatura precisa das DUAS coisas — uma **aresta** e uma **cova com ÁREA** —, e um
/// vinco vivo é um conjunto de **medida nula**: a grelha de píxeis nunca lá cai (medido: `negativos
/// = 0` sobre `3 456` píxeis numa peça sem filete). ⇒ a peça leva crateras cavadas e quinas
/// arredondadas, e o filete é o que dá largura à cova.
///
/// ⛔ E ela abre em **Matcap**, como todo o módulo: o passo `(1)` é ligar o *Render*, que é onde o
/// artista aprende que a camada de estilo **é do Render** — no matcap ela nem sequer aparece.
///
/// # ⭐⭐⭐ O ENQUADRAMENTO foi FOTOGRAFADO, e a foto apanhou três coisas que a suíte não vê
///
/// (`docs/Components/ferramentas/fotografa_cena.sh`, a lei que o TOP-20 #16 pagou.)
///
/// 1. **As covas ficavam de lado.** A 1.ª redacção cavava-as em `+z`, e a câmera de omissão é uma
///    três-quartos (`Orbit::default`: guinada `0,72`, arfagem `0,52` ⇒ o olho em
///    `(0,572, 0,497, 0,652)`). ⇒ as posições são **derivadas desse olho**, não escolhidas.
/// 2. **A barra saía do ecrã.** Pendurada por baixo da bola ela era cortada pela barra de estado;
///    atravessada e mais curta, cabe inteira — e o encontro com a bola dá mais uma banda côncava.
/// 3. **E o que se fotografa com o `$HOME` do dono é a BANCADA dele, não a cena:** a arrumação vive
///    em `~/.ph2d/layout.txt`, e uma janela flutuante deixada aberta noutra sessão tapava a peça nas
///    duas primeiras fotos. ⇒ *a foto corre com um `HOME` limpo*, e o roteiro avisa o dono.
pub fn cena_35() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 35 — O ESTILO: os botoes para MENTIR DE PROPOSITO por cima da fisica."
    );
    println!(
        "[field-smoke]            (1) MODEL · no canto superior direito carregue no separador \
         Model · no painel do topo, Shading · Render — a seccao STYLE aparece no FIM do painel da \
         direita. No Matcap ela nao existe, e isso e' de proposito."
    );
    println!(
        "[field-smoke]            (2) suba Rim Strength ate' ~1,5: a peca ganha um FIO de luz na \
         beirada e DESCOLA do fundo. Rim Falloff aperta ou alarga esse fio."
    );
    println!(
        "[field-smoke]            (3) Cavity Tint para um AZUL escuro: so' o fundo das crateras \
         muda. Edge Tint para um LARANJA: so' as quinas mudam."
    );
    println!(
        "[field-smoke]            (3b) a BORDA DURA e' o Curvature Softness: suba-o e a passagem \
         entre a tinta e a peca alarga; desca-o e ela volta a ser um corte seco. Edge Sharpness e \
         Cavity Sharpness decidem, cada um por sua conta, quanta curvatura ja' conta como tinta \
         cheia numa quina e no fundo de uma cratera."
    );
    println!(
        "[field-smoke]            (4) Shadow Tint azul + Highlight Tint quente: a imagem inteira \
         ganha a grade de cor de um filme. Zone Pivot escolhe onde e' a fronteira."
    );
    println!(
        "[field-smoke]            (5) uma fileira APAGADA nao e' um defeito: ela diz porque e' que \
         nao faz nada agora (por exemplo Zone Pivot, enquanto as duas tintas de zona forem a mesma \
         cor). Faca o que a frase manda e ela acende."
    );
    println!(
        "[field-smoke]            (6) como saber que falhou: se mexer num controlo ACESO e a peca \
         NAO mudar nada, o botao nao chegou. Se a seccao STYLE nao aparecer no Render, PARE. E se \
         mexer numa cor e OUTRA cor mudar junto, PARE."
    );
    // ⚠️ **A arrumação dos painéis vive FORA do repositório** (`~/.ph2d/layout.txt`) — uma janela
    // flutuante deixada aberta noutra sessão aparece por cima da peça, e o dono leria isso como
    // defeito desta cena. *Dizê-lo é mais barato do que ele descobrir.*
    println!(
        "[field-smoke]            (⚠️) se uma janela flutuante estiver por cima da peca, feche-a no \
         X dela: a arrumacao dos paineis fica gravada entre sessoes, fora do projecto."
    );
    // ⚠️⚠️ **A ORDEM dos nós é a da TRAVESSIA, e o gate cobra-a:** um combine vem logo a seguir aos
    // filhos dele, senão a peça muda de forma ao virar objectos — e *o que se vê na tela é o
    // DEPOIS*. (A 1.ª redacção desta cena declarava a caixa antes do corte e reprovou.)
    //
    // A bola grande — a superfície de fundo, toda ela ARESTA suave.
    let mut nodes = vec![leaf(Primitive::Sphere { radius: 0.45 }, Xform::IDENTITY)];
    // ⭐ **Três crateras**, de raios diferentes: elas dão as COVAS, e raios diferentes dão
    // curvaturas diferentes — é isso que faz o `Curvature Sharpness` ter o que separar.
    //
    // ⚠️⚠️ **As posições são DERIVADAS do olho da câmera de omissão** (`Orbit::default`: guinada
    // `0,72`, arfagem `0,52` ⇒ o olho em `(0,572, 0,497, 0,652)`), e não escolhidas: a 1.ª redacção
    // cavava-as em `+z` e a **FOTO** mostrou-as de lado, quase na silhueta. *Uma cova que o artista
    // não vê de frente é um botão que parece morto.*
    for (raio, pos) in [
        (0.20_f32, [0.065_f32, 0.452, 0.348]),
        (0.13, [0.343, 0.372, 0.158]),
        (0.16, [0.364, -0.056, 0.407]),
    ] {
        nodes.push(leaf(
            Primitive::Sphere { radius: raio },
            Xform {
                translation: pos,
                ..Xform::IDENTITY
            },
        ));
    }
    // ⚠️ **O filete é o que dá LARGURA à cova** — com aresta viva ela é medida nula e o botão da
    // tinta parece morto.
    nodes.push(combine(
        Op::Difference(ph2d_field::Blend::Exact { radius: 0.05 }),
        vec![NodeId(0), NodeId(1), NodeId(2), NodeId(3)],
    ));
    // ⭐ E uma CAIXA arredondada atravessada, que traz quinas de verdade — uma esfera sozinha não
    // tem aresta nenhuma para o `Edge Tint` morder.
    nodes.push(leaf(
        Primitive::Box {
            half: [0.60, 0.085, 0.085],
            round: 0.02,
            chamfer: 0.0,
        },
        // ⚠️ **ATRAVESSADA e não pendurada:** a 1.ª redacção punha-a por BAIXO da bola e a FOTO
        // mostrou-a cortada pela barra de estado. Atravessada, ela mostra as quinas nos dois lados e
        // o encontro com a bola dá mais uma banda CÔNCAVA de graça.
        Xform {
            translation: [0.0, 0.02, 0.0],
            ..Xform::IDENTITY
        },
    ));
    nodes.push(combine(
        Op::Union(ph2d_field::Blend::Exact { radius: 0.06 }),
        vec![NodeId(4), NodeId(5)],
    ));
    FieldDoc::new(nodes, NodeId(6))
}

/// ⭐⭐⭐ **OS BRILHOS da cena `=36`** — três luzes, e o que as separa é UM número: quão fortes são.
///
/// ⚠️⚠️ **A escada é de `4×` e não de `2×`, e isso é MEDIDO:** o limiar de fábrica é `1` e a lei
/// corta na luminância de **pico**, logo duas luzes a `2` e a `4` acendem-se as duas e o artista vê
/// *«o limiar apagou tudo de uma vez»*. Com `2 · 8 · 32` cada volta do limiar apaga **uma**, e é
/// isso que faz a cena ENSINAR o botão em vez de o demonstrar.
///
/// ⭐ **E o `32` é o número que a nota do [`ph2d_field_ecs::FieldMaterial::emission`] mede:** acima
/// dele a peça sai **bit a bit a mesma** na tela. *Ele satura a IMAGEM e continua a alimentar o
/// BRILHO* — que é a melhor demonstração de que este passe lê o quadro ANTES do olhar, e o roteiro
/// di-lo por extenso.
pub const BRILHOS_DA_CENA: [f32; 3] = [2.0, 8.0, 32.0];

/// ⭐⭐⭐ **A cena `=36` — O BRILHO** (`docs/Render3d/12`, a `W7`).
///
/// # ⚠️ Porque três luzes numa fileira, e uma peça ESCURA por baixo
///
/// Um halo só se vê contra o que ele **não** é: uma cena toda acesa não tem onde o mostrar, e uma
/// toda apagada não o produz. ⇒ três esferas acesas sobre uma **barra escura** que as atravessa —
/// ela é o controlo, e é nela que se lê se o halo derramou para fora das luzes.
///
/// ⚠️ **As posições e a barra são as da [`cena_35`]**, e de propósito: aquela disposição foi
/// corrigida por FOTO (as covas de lado, a barra cortada pela barra de estado), e *reescrever uma
/// disposição já fotografada é pagar as mesmas três correcções outra vez*.
pub fn cena_36() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!("[field-smoke] cena 36 — O BRILHO: a luz forte DERRAMA para fora da peca.");
    println!(
        "[field-smoke]            (0) ⚠️ ESTA CENA PRECISA DE UMA BANDEIRA. Feche o app e abra-o \
         com PH2D_FIELD_GPU=0 (esta' no comando que lhe foi dado). Sem ela as fileiras do brilho \
         aparecem APAGADAS e dizem porque'."
    );
    println!(
        "[field-smoke]            (1) MODEL · separador Model, em cima · painel do topo, Shading · \
         Render. A seccao BLOOM e' a ULTIMA do painel da direita, a seguir a STYLE."
    );
    println!(
        "[field-smoke]            (2) ponha Bloom em ON. As tres bolas ganham um HALO que derrama \
         para fora delas; a barra escura nao ganha nada."
    );
    println!(
        "[field-smoke]            (3) suba Threshold devagar. As bolas apagam-se UMA DE CADA VEZ, \
         da mais fraca para a mais forte — e' isso que o numero faz: escolhe quao forte uma luz \
         tem de ser para brilhar."
    );
    println!(
        "[field-smoke]            (4) Intensity muda a FORCA do halo; as fileiras Size 1..7 mudam o \
         TAMANHO dele (cada uma e' o dobro da anterior). Suba Size 5 e o halo abre-se pelo ecra."
    );
    println!(
        "[field-smoke]            (5) Knee arredonda a passagem: com ele a zero uma bola acende \
         DE REPENTE ao cruzar o Threshold; com ele alto ela acende aos poucos."
    );
    println!(
        "[field-smoke]            (6) uma fileira APAGADA nao e' um defeito: ela diz porque e' que \
         nao faz nada agora. Faca o que a frase manda e ela acende."
    );
    println!(
        "[field-smoke]            (7) como saber que falhou: se pos Bloom em ON e NENHUMA bola \
         ganhou halo, PARE. Se o halo aparecer com Bloom em OFF, PARE. E se a BARRA escura ganhar \
         halo proprio, PARE — ela nao emite luz."
    );
    println!(
        "[field-smoke]            (⚠️) se uma janela flutuante estiver por cima da peca, feche-a no \
         X dela: a arrumacao dos paineis fica gravada entre sessoes, fora do projecto."
    );
    // ⚠️ **A ORDEM é a da TRAVESSIA** — os filhos, depois o combine. A `cena_35` pagou esta.
    let mut nodes: Vec<Node> = BRILHOS_DA_CENA
        .iter()
        .enumerate()
        .map(|(i, _)| {
            #[allow(clippy::cast_precision_loss)]
            let x = -0.70 + 0.70 * i as f32;
            leaf(
                Primitive::Sphere { radius: 0.22 },
                Xform {
                    translation: [x, 0.12, 0.0],
                    ..Xform::IDENTITY
                },
            )
        })
        .collect();
    // ⭐ A BARRA escura, atravessada — o CONTROLO da cena: ela não emite, logo não pode brilhar.
    nodes.push(leaf(
        Primitive::Box {
            half: [1.05, 0.06, 0.06],
            round: 0.03,
            chamfer: 0.0,
        },
        Xform {
            translation: [0.0, -0.26, 0.0],
            ..Xform::IDENTITY
        },
    ));
    #[allow(clippy::cast_possible_truncation)]
    let n = nodes.len() as u32;
    nodes.push(combine(
        Op::Union(ph2d_field::Blend::Sharp),
        (0..n).map(NodeId).collect(),
    ));
    FieldDoc::new(nodes, NodeId(n))
}
