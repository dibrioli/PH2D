//! As cenas de smoke da **folha 08** (stream & utilidade) do doc 89 — `=62` a `=65`.
//!
//! ⚠️ **O corte é pela FOLHA, não pela data.** O irmão
//! [`motion_state_demo_conferencia`](super) passou dos 600 do HR-18 ao ganhar a quarta destas,
//! e a cura de um teto é um split — mas um split por *"as cenas novas"* envelhece na semana
//! seguinte. Estas quatro respondem à mesma folha, e é isso que as mantém juntas: quem abrir a
//! próxima célula de utilidade sabe onde a cena dela vai.

use super::*;

/// **A CENA `=62` — OS DOIS DEFEITOS DE JUNÇÃO** (doc 89, folha 08).
///
/// Dois pares, cada um com o seu CONTROLE: o carimbo que deitava fora a escala do ponto, e
/// a junção cujas colunas de identidade mentem.
pub(crate) fn join(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_join::build_join_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[join-demo] DOIS PARES ({} bandas). Cada par e' um DEFEITO, com o controle ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_join::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) 1-2, A ESCALA DO PONTO: a MESMA fileira de pontos nas duas, e o tamanho de cada
  ponto cresce da esquerda para a direita. Na banda 1 os carimbos saem TODOS IGUAIS -- o
  duplicator somava P e rot e deitava fora toda outra coluna do ponto. Na 2 eles crescem.
  SE AS DUAS FOREM IGUAIS, PARE. O knob e' `Point Scale`, e ele e' um PESO: 0,5 da' metade
  da variacao.
  (!) 3-4, A JUNCAO: as MESMAS duas grelhas (9 + 4 = 13 pecas) tingidas por um degrade que
  le' a coluna `Index`. Na banda 3 a cor REINICIA no meio -- cada grelha escreveu o seu
  proprio `Index = 0..n` e a juncao copiou os dois verbatim, entao as 13 pecas dizem ser
  duas listas. Na 4 o degrade corre UMA vez sobre as 13. O knob e' `Reindex`.
  (!) ⚠️ O `Reindex` nasce DESLIGADO, e essa e' uma pergunta PARA VOCE: a referencia de onde
  o no' veio (Cavalry `combineStreams`) tem o contrario -- ligado por omissao. Liga-lo aqui
  por omissao mudaria arte ja' feita, entao a escolha e' sua, nao minha.
  (!) DEU ERRADO se as bandas de cada par parecerem iguais, ou se a banda 4 mostrar duas
  passagens de cor."
    );
    sinks
}

/// **A CENA `=63` — A ORDEM** (doc 89, folha 08: a direção arbitrária e a chave como campo).
///
/// Ordenar não muda ONDE as peças estão; muda QUEM é a primeira. As três bandas passam o
/// mesmo grid pelo mesmo gradiente, que lê as colunas de identidade — a COR é a ordem.
pub(crate) fn sortkey(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_sortkey::build_sortkey_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[sort-demo] TRES bandas ({}). O MESMO grid nas tres; so' a CHAVE de ordenacao muda.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_sortkey::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) A cor E' a ordem: um gradiente le' as colunas de identidade, entao a peca mais
  escura e' a primeira e a mais clara e' a ultima. As pecas NAO se movem entre as bandas --
  se elas mudarem de lugar, algo alem da ordem mexeu, e ai' PARE.
  (!) Banda 2: `Axis Angle {:.0}` -- a mesma chave `X`, girada. Antes isto custava TRES nos
  (`rotate -> sort -> rotate` de volta). Clique no no' Sort e arraste o `Axis Angle`: a
  diagonal da cor gira ao vivo, e em 90 ela vira o `Y`.
  (!) Banda 3: a chave e' um CAMPO -- um `value.noise` ligado na porta `Weight`, que e' a
  entrada nova. A cor serpenteia, porque a ordem segue o ruido e nao a geometria. Nenhum dos
  cinco modos do dropdown sabe fazer isto.
  (!) DEU ERRADO se as tres bandas tiverem o mesmo padrao de cor, ou se a 3 ficar igual a 1
  (a porta `Weight` nao chegou).",
        conferencia_demos_sortkey::diagonal(),
    );
    sinks
}

/// **A CENA `=64` — A FILA E A MISTURA** (doc 89, folha 08: o taper por cópia do
/// `motion.clone` e o peso por entrada do `motion.mixer`).
///
/// Dois pares, o mesmo grafo dos dois lados de cada par, um número diferente.
pub(crate) fn taper(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_taper::build_taper_demo_document(doc, registry).unwrap_or_default();
    let (ts, tr, hv) = conferencia_demos_taper::authored();
    eprintln!(
        "[taper-demo] DOIS pares, {} bandas. Esquerda: a fila clonada. Direita: a mistura.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_taper::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Banda 2: `Scale Taper {ts}` diz QUANTO A ULTIMA COPIA MEDE -- e o caminho ate' la'
  e' uma rampa, entao a copia do meio fica a meio (nao a um quarto). `Rot Taper {tr:.0}` e' a
  volta que a ultima leva. Clique no no' Clone e arraste os dois: a fila afunila e gira ao vivo.
  (!) Bandas 3 e 4: as MESMAS duas fontes (uma fileira deitada e uma coluna em pe'), fundidas
  ponto a ponto. Com pesos iguais o resultado e' a diagonal exacta; com `Weight 1 = {hv:.0}` a
  coluna puxa, e a linha fica mais em pe'.
  (!) DEU ERRADO se as duas bandas de um par sairem iguais, se a fila 2 nao afunilar, ou se as
  pecas se taparem umas as outras."
    );
    sinks
}

/// **A CENA `=65` — O CAMPO SEGUE O RATO** (doc 89, folha 08: a célula do `followMouse`).
///
/// Duas fileiras iguais; só o CENTRO do campo vem de sítios diferentes. A de cima é dirigida
/// pelo `value.cursor`, a de baixo tem o centro autorado.
pub(crate) fn cursor(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_cursor::build_cursor_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[cursor-demo] DUAS fileiras ({}). Mexa o rato sobre o canvas.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_cursor::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) A fileira de CIMA incha onde o rato esta', e o inchaco anda com ele. A de BAIXO
  tem um inchaco parado, sempre no mesmo sitio -- e' o controle.
  (!) NAO existe um botao `Follow Mouse`: o que existe e' um no' `Cursor` cujas DUAS saidas
  guiam dois parametros do campo. A mesma rota liga o rato a QUALQUER numero de QUALQUER no'
  -- um raio, um angulo, uma cor.
  (!) DEU ERRADO se as duas fileiras andarem juntas, se nenhuma reagir, ou se o inchaco
  aparecer longe do ponteiro."
    );
    sinks
}

/// **A CENA `=66` — A FAMÍLIA DOS CAMPOS** (doc 89, folha 10: o anel, a força com sinal e o
/// truncamento). Três pares; só um número muda em cada.
pub(crate) fn field_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_field::build_field_demo_document(doc, registry).unwrap_or_default();
    let (inner, growth) = conferencia_demos_field::authored();
    eprintln!(
        "[field-demo] TRES pares, {} bandas. Esquerda = como era; direita = o numero novo.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_field::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) O campo e' invisivel -- o que se ve e' o TAMANHO das pecas, que ele comanda.
  (!) Par 1 (azul): `Inner Radius {inner}` abre um buraco no meio do disco. Clique no no'
  `Radial Sweep` da direita e arraste-o: o anel engorda e emagrece ao vivo.
  (!) Par 2 (laranja): `Strength -1` nao e' o `Invert`. A caixa para de crescer e o resto
  do quadro sobe ACIMA do que a caixa media -- o campo passa a empurrar em vez de mascarar.
  (!) Par 3 (verde): duas caixas somadas. Com `Clamp` o cruzamento delas nao passa de 1 e
  fica igual a cada metade; sem `Clamp` ele passa de 1 e o cruzamento cresce o dobro
  (a escala le' a mascara como `1 + ({growth} - 1) x mascara`).
  (!) DEU ERRADO se as duas bandas de um par sairem iguais, ou se as pecas se taparem."
    );
    sinks
}

/// **A CENA `=67` — A CHUVA RALA** (doc 89, folha 01: a `probability` do `motion.emitter`).
///
/// Dois jactos com o MESMO `rate` e o MESMO `seed`; só a fracção que nasce muda.
pub(crate) fn drizzle(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_drizzle::build_drizzle_demo_document(doc, registry).unwrap_or_default();
    let (rate, thin) = conferencia_demos_drizzle::authored();
    eprintln!(
        "[drizzle-demo] DOIS jactos ({}). O mesmo rate ({rate:.0}/s) nos dois.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_drizzle::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Isto NAO e' o rate mais baixo. Baixar o rate afasta as particulas
  REGULARMENTE -- o jacto fica ralo e certinho. A probabilidade deixa o ritmo intacto e tira
  particulas ONDE CALHA: os buracos sao de tamanhos diferentes. E' a diferenca entre um
  chuveiro e uma chuva.
  (!) Clique no no' Emitter da direita e arraste o `Probability` de {thin} ate' 1: o jacto
  ralo enche ate' ficar igual ao da esquerda, e nenhuma gota SALTA de sitio no caminho -- as
  que ja' estavam continuam onde estavam, e as outras aparecem entre elas.
  (!) DEU ERRADO se as gotas piscarem (aparecer/desaparecer sozinhas), se os dois jactos
  sairem iguais, ou se mexer o Probability fizer o jacto inteiro mudar de forma."
    );
    sinks
}

/// **A CENA `=68` — OS DEFORMADORES** (doc 89, folha 04: cinco células, quatro nós).
///
/// Cinco pares; o mesmo grafo dos dois lados de cada um.
pub(crate) fn deform(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_deform::build_deform_demo_document(doc, registry).unwrap_or_default();
    let (dir, rim) = conferencia_demos_deform::authored();
    eprintln!(
        "[deform-demo] CINCO pares, {} bandas. Esquerda = como era; direita = o numero novo.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_deform::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Par 1 (azul): `Direction {dir:.0}` roda o EIXO da dobra, nao o layout. Clique no
  no' Bend da direita e arraste-o: a grelha roda o lado por onde ela curva, ao vivo.
  (!) Par 2 (laranja): `Radius {rim}` diz ate' ONDE a torcao vai. Fora desse aro tudo leva a
  volta inteira -- o miolo torce e a borda fica rigida.
  (!) Par 3 (verde): o `Profile` muda o CAMINHO ate' a volta, nao a volta. A peca mais
  externa fica exactamente no mesmo sitio nos dois lados; e' o miolo que se reparte diferente.
  (!) Par 4 (rosa): `Radius Y` achata a lente. Ela incha na largura e nao na altura -- e o
  contorno dela deixa de ser um circulo, entao pontos que estavam dentro passam a estar fora.
  (!) Par 5 (lilas): `Keep Length` poe a fileira na curva SEM a esticar. O `Fit` leva-a sempre
  ate' as duas pontas do S, seja qual for o tamanho dela.
  (!) DEU ERRADO se as duas bandas de um par sairem iguais, ou se no par 3 a peca do aro
  mudar de lugar."
    );
    sinks
}

/// **A CENA `=69` — A FAMÍLIA TRANSFORM** (doc 89, folha 05: cinco células, cinco nós).
///
/// Cinco pares; o mesmo grafo dos dois lados de cada um.
pub(crate) fn transform_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_transform::build_transform_demo_document(doc, registry)
        .unwrap_or_default();
    let (step, turn) = conferencia_demos_transform::authored();
    eprintln!(
        "[transform-demo] CINCO pares, {} bandas. Esquerda = como era; direita = o controle novo.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_transform::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Par 1 (azul): o MESMO passo de {step} nos dois lados. `Space = Local` le' a
  orientacao de cada peca, entao o anel abre em vez de deslizar.
  (!) Par 2 (laranja): `Separate Y Mask` faz a altura seguir um SEGUNDO campo. Sem ele os
  nove crescem por igual; com ele a fileira vai de alto-e-magro a baixo-e-largo.
  (!) Par 3 (verde): DUAS fileiras de barras inclinadas. `Flip Orientation` espelha a
  ORIENTACAO da copia: sem ele as duas inclinam para o MESMO lado (/// ///), com ele a
  segunda inclina ao contrario (/// \\\\\\) e as duas fecham como um V deitado.
  (!) Par 4 (rosa): `Reindex` diz que as seis fatias sao UMA lista. Sem ele o degrade
  recomeca em cada fatia e as seis saem iguais.
  (!) Par 5 (lilas): a mesma volta de {turn} graus nos dois. `Carry Rotation` faz o sprite
  virar com a orbita, entao os raios continuam a apontar para fora.
  (!) DEU ERRADO se as duas bandas de um par sairem iguais, ou se no par 1 o anel da
  esquerda mudar de tamanho -- ele so' pode deslizar."
    );
    sinks
}

/// **A CENA `=70` — A FAMÍLIA `fx.*`** (doc 89, folha 11: cinco células, três nós).
///
/// ⚠️ Não é uma cena de pares — ver o cabeçalho de
/// [`conferencia_demos_fx`](super::super::conferencia_demos_fx): o glow é um passe
/// da imagem inteira, e um segundo nó dele seria inerte.
pub(crate) fn fx_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_fx::build_fx_demo_document(doc, registry).unwrap_or_default();
    let (softness, stretch, firefly) = conferencia_demos_fx::authored();
    eprintln!(
        "[fx-demo] {} bandas. As sombras vem em PAR; o glow vem em UM estado (ver abaixo).",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_fx::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Em cima (par): a MESMA sombra, so' que a da direita tem `Softness {softness}`.
  A borda ganha penumbra e o miolo continua com a MESMA densidade -- ligar a maciez nao
  pode clarear a sombra.
  (!) No meio (roxo): o halo esta' com `Anamorphic {stretch}`. Clique no no' Glow e
  arraste esse controle ate' 1: o halo volta a ser redondo. O `Streak Angle` gira o
  risco -- e ele so' aparece quando o Anamorphic sai de 1, porque num circulo ele
  nao faria nada.
  (!) Embaixo a` direita: UMA peca {firefly}x mais brilhante que o branco. Ela lava a
  tela de proposito. Arraste o `Clamp` do no' Glow para cima de 1 e o estouro cede,
  sem a cena inteira apagar.
  (!) Embaixo de tudo (verde): uma FORMA -- vetor vivo, nao uma peca comum. Ela e' o
  caso que voce reportou: ate' hoje o brilho nao a alcancava. Ela tem de ter halo
  como as outras, e o `Anamorphic` tem de estica-lo tambem.
  (!) DEU ERRADO se as duas sombras de cima sairem iguais, se a de baixo ficar mais
  CLARA que a de cima, se mexer o Clamp apagar tambem o halo roxo do meio, ou se o
  halo da FORMA aparecer DESLOCADO dela (ao lado, em vez de em volta)."
    );
    sinks
}

/// **A CENA `=71` — A FAMÍLIA `force.*`** (doc 89, folha 02: três células, três nós).
///
/// ⚠️ **Só se julga com o PLAY** — uma força acumula em `accel` e é o integrador que
/// a aplica; parada, as seis bandas são seis nuvens idênticas.
pub(crate) fn force_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_force::build_force_demo_document(doc, registry).unwrap_or_default();
    let (drag_y, cork) = conferencia_demos_force::authored();
    eprintln!(
        "[force-demo] TRES pares, {} bandas. Esquerda = como era; direita = o numero novo.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_force::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY. Uma forca nao move nada sozinha -- ela acumula e o integrador aplica.
  Paradas, as seis bandas sao seis nuvens iguais: a leitura e' o CAMINHO de cada uma.
  (!) Par 1 (azul): as duas comecam a cair na diagonal. A da direita tem `Drag Y {drag_y:.0}` e
  o `Drag X` em 1 -- so' o vertical e' freado, entao a queda dela CURVA. A da esquerda desce
  reta. Clique no no' Drag da direita e arraste o `Drag Y`: a curva abre e fecha ao vivo.
  (!) Par 2 (laranja): o MESMO aro e a MESMA forca. So' o `Curve` muda. Olhe o MIOLO --
  ele gira mais a` direita -- e a BORDA, que solta antes. Se os dois redemoinhos forem
  iguais, o perfil nao chegou.
  (!) Par 3 (verde): a mesma fileira submersa. A` esquerda todas tem a densidade {cork},
  entao sobem juntas. A` direita cada peca tem a sua, e a fileira sobe em RAMPA.
  (!) DEU ERRADO se as duas bandas de um par fizerem o mesmo caminho, ou se alguma nuvem
  ficar parada (a forca nao chegou ao integrador)."
    );
    sinks
}

/// **A CENA `=72` — A FAMÍLIA DA COR + O PAREAMENTO DO `motion.step`**
/// (doc 89: folha 09 inteira, três células; folha 07, a célula do pareamento).
///
/// ⚠️ **O par 3 é o único cujos dois lados têm de ficar IGUAIS** — ver o doc do
/// módulo da cena. E a quarta célula (o kernel de GPU do `motion.color_array`) não
/// tem lado: prova-se pela AUSÊNCIA de diferença contra `PH2D_GPU_COOK=0`.
pub(crate) fn color_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_color::build_color_demo_document(doc, registry).unwrap_or_default();
    let (beat, steps) = conferencia_demos_color::authored();
    eprintln!(
        "[color-demo] TRES pares, {} bandas. Esquerda = como era; direita = o que mudou.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_color::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Par 1 (as duas grades de cima): as duas tem o MESMO degrade' por baixo e o
  MESMO laranja por cima. A` esquerda o laranja substitui tudo e a grade fica chapada;
  a` direita ele multiplica, e o degrade' continua la' dentro, agora quente. Clique no
  segundo no' Tint e mude o `Blend` -- Mix, Add, Subtract, Multiply, Divide.
  (!) Par 2 (as duas grades do meio): a MESMA paleta de quatro cores. A` esquerda as
  listras sao regulares (a fatia sai do indice da peca). A` direita um campo dirige o
  `Offset`, e cada peca escolhe a sua -- confete, nao listra. Ate' hoje esse campo era
  jogado fora: o valor da PRIMEIRA peca valia por todas.
  (!) Par 3 (as duas fileiras de baixo): DE' PLAY. As duas sobem e descem em degraus de
  {beat:.1} s, {steps:.0} degraus ate' virar. A de cima recebe um batimento por peca; a de
  BAIXO recebe UM batimento so', para o conjunto todo. As duas tem de subir em BLOCO.
  (!) DEU ERRADO se, na fileira de baixo, so' a PRIMEIRA peca andar e as outras cinco
  ficarem paradas -- era esse o defeito. Ou se a grade da direita do par 1 sair chapada,
  ou se a do par 2 sair em listras regulares.
  (!) A PARIDADE: rode outra vez com `PH2D_GPU_COOK=0` na frente do comando. A imagem
  tem de ser a MESMA -- a paleta passou a correr na placa, e o caminho de referencia
  continua a dar a mesma resposta."
    );
    sinks
}

/// **A CENA `=73` — quatro exemplos, um por linha** (doc 89: folha 08 inteira, duas
/// células; folha 10 inteira, duas células).
///
/// ⚠️ **A prosa aqui é para o ENIO, e a v1 dela foi reprovada** (*"tudo misturado e
/// bagunçado sem explicação simples"*). O que mudou: cada linha tem o nome escrito
/// **no canvas**, a leitura de cada uma é UMA pergunta de sim/não, e o texto abaixo
/// não nomeia um nó sem dizer onde clicar.
pub(crate) fn rank_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_rank::build_rank_demo_document(doc, registry).unwrap_or_default();
    let (shift, _) = conferencia_demos_rank::authored();
    eprintln!(
        "[cena 73] Quatro exemplos, um por linha -- {} blocos.
  ESQUERDA = como era. DIREITA = o que mudou. O nome de cada linha esta' escrito
  no meio da tela, entre os dois blocos.

  CORTE  As duas fileirinhas tem o MESMO numero de pecas e acendem da esquerda
         para a direita. A da esquerda APAGA antes do fim; a da direita acende
         ate' a ultima peca.
         > clique no no' Cull e ligue/desligue `Renumber Survivors`.

  BANDA  Os dois quadrados acendem a MESMA quantidade de pecas. A` esquerda elas
         ficam todas JUNTAS, numa faixa. A` direita ficam ESPALHADAS -- e repare
         que nenhuma peca mudou de lugar.

  RAMPA  As duas fileiras acendem da esquerda para a direita. A de baixo
         RECOMECA no meio do caminho: apaga de repente e volta a acender.
         > clique no no' Remap e arraste o `Curve Offset` ({shift:.2} hoje).

  FORMA  E' o NO' NOVO. Um pentagono decide quem acende. A` esquerda ele acende
         CHEIO, como um bloco. A` direita so' a BORDA dele acende, e o miolo apaga.
         > clique no no' Shape Field e troque o `Path Mode`.

  DEU ERRADO se as duas metades de qualquer linha ficarem IGUAIS, ou se algum
  bloco sair de cima do outro.",
        sinks.len(),
    );
    sinks
}

/// **A CENA `=74` — dois exemplos, um por linha** (doc 89, folha 02: as duas células
/// que sobravam, e as duas são do `force.attractor`).
///
/// ⚠️ **Só se julga com o PLAY** — uma força acumula em `accel` e é o integrador que a
/// aplica; parada, as quatro nuvens são quatro nuvens iguais.
pub(crate) fn goal_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_goal::build_goal_demo_document(doc, registry).unwrap_or_default();
    let (goals, lead) = conferencia_demos_goal::authored();
    eprintln!(
        "[cena 74] Dois exemplos, um por linha -- {n} blocos (as nuvens + os alvos brancos).
  ESQUERDA = como era. DIREITA = o que mudou. O nome da linha esta' escrito no meio.

  >>> DE' PLAY. Uma forca nao move nada sozinha; parada, a cena nao diz nada.

  ALVO   As duas nuvens sao puxadas por um atrator. A` esquerda o alvo e' UM ponto
         (o branco no meio), e a nuvem junta-se toda nele. A` direita ha' {goals:.0}
         pontos brancos, e cada peca vai ao MAIS PROXIMO dela -- a nuvem parte-se
         em {goals:.0} grupos.
         > clique no no' Attractor e troque o `Target` entre Point e Stream.

  MIRA   As duas fileiras perseguem o MESMO ponto branco, que sobe e desce. A de
         cima mira onde ele ESTA' e chega sempre atrasada, fazendo zigue-zague
         atras dele. A de baixo antecipa {lead:.1} s e corta caminho -- ela encontra
         o alvo em vez de o seguir.
         > clique no no' Attractor de baixo e arraste o `Predict` ate' 0: ela volta
           a atrasar-se.

  DEU ERRADO se as duas metades de qualquer linha fizerem o MESMO caminho, se
  alguma nuvem ficar parada (a forca nao chegou ao integrador), ou se os pontos
  brancos nao aparecerem.",
        n = sinks.len(),
    );
    sinks
}

/// **A CENA `=75` — dois exemplos, um por linha** (doc 89, folha 03: o pin que rasga e
/// o bando que desvia).
///
/// ⚠️ **Só se julga com o PLAY** — as duas linhas são simulação.
pub(crate) fn sim_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_sim::build_sim_demo_document(doc, registry).unwrap_or_default();
    let (limiar, pedras) = conferencia_demos_sim::authored();
    eprintln!(
        "[cena 75] Dois exemplos, um por linha -- {n} blocos. ESQUERDA = como era,
  DIREITA = o que mudou. O nome da linha esta' escrito no meio da tela.

  >>> DE' PLAY. As duas linhas sao simulacao; paradas nao dizem nada.

  SOLTA  Dois PANOS iguais (tecido de verdade), pregados pela fileira de cima. O
         vento comeca em ZERO e vai SUBINDO -- os dois ficam pendurados, depois
         comecam a levantar. La' pelos 2 segundos a forca passa do que o prego da
         DIREITA aguenta ({limiar:.1}) e ele ROMPE: aquela folha solta-se e vai
         embora INTEIRA, enquanto a da esquerda continua presa.
         (!) O que rompe e' o PREGO, nao o pano -- as pecas nao se separam umas
         das outras. Partir o tecido seria outra coisa, e este tecido nao tem
         ligacoes uma-a-uma que se possam quebrar.
         > espere os 2 segundos. Depois clique no no' Pin Constraint da esquerda
           e ponha o `Break Above` em {limiar:.1}: ela solta tambem.

  DESVIA Os dois bandos correm para o meio, onde ha' {pedras:.0} pedras brancas em
         anel. A` esquerda eles ATRAVESSAM as pedras como se nao existissem. A`
         direita eles CONTORNAM e param a` volta delas.
         > clique no no' Boids e arraste o `Avoid` ate' 0: eles voltam a passar
           por dentro.

  DEU ERRADO se as duas metades de qualquer linha fizerem o mesmo, se alguma
  nuvem ficar parada, ou se as pedras brancas nao aparecerem.",
        n = sinks.len(),
    );
    sinks
}

/// **A CENA `=76` — três exemplos, um por linha** (doc 89, folha 14: as duas células que a
/// fecharam, mais o preenchimento que o traço apagava).
///
/// ⚠️ **Só a linha do meio precisa de PLAY** — o `trim_offset` dela é conduzido pelo relógio
/// por um fio. As outras duas leem-se paradas.
pub(crate) fn style_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_style::build_style_demo_document(doc, registry).unwrap_or_default();
    let (span, lap) = conferencia_demos_style::authored();
    eprintln!(
        "[cena 76] Tres exemplos, um por linha -- {n} formas. ESQUERDA = como era,
  DIREITA = o que mudou. O nome da linha esta' escrito no meio da tela.

  BORDA    A MESMA estrela azul duas vezes. A` esquerda ela e' so' azul chapado. A`
           direita ela tem uma BORDA laranja a` volta -- e o miolo continua azul.
           (Antes, por o traco a mais de zero a estrela ficava OCA: a cor de dentro
           desaparecia.)
           > clique no no' Shape da direita e arraste o `Stroke Width` ate' 0: a
             borda some e a estrela fica igual a` da esquerda.

  APARADO  Um anel branco, so' o contorno. A` esquerda ele esta' inteiro. A` direita
           so' {span:.0}% dele aparece -- e esse trecho DA' A VOLTA ao anel, uma volta
           a cada {lap:.0} segundos.
           >>> DE' PLAY para ver o trecho correr.
           > clique no no' Shape da direita e arraste o `Trim End` de 0 ate' 1: o
             anel desenha-se sozinho, como uma caneta a correr.

  PICOTADO O mesmo retangulo verde, so' o contorno. A` esquerda a linha e' continua;
           a` direita ela e' PICOTADA.
           > clique no no' Shape da direita e arraste o `Dash Gap`: os buracos abrem
             e fecham.

  (!) OS TRES CONTROLES NOVOS SO' APARECEM COM BORDA. Ponha o `Stroke Width` em 0 e
      o `Trim`, o `Dash` e a cor da borda somem do painel -- sem borda eles nao
      fazem nada, e aparar uma forma sem borda faria a forma DESAPARECER.

  DEU ERRADO se as duas metades de qualquer linha ficarem iguais, se a estrela da
  direita sair oca, ou se o trecho branco nao andar com o Play.",
        n = sinks.len(),
    );
    sinks
}

/// **A CENA `=77` — dois exemplos, um por linha** (doc 89, folha 07: o *Echo Operator* e o
/// *Strobe Operator*, que a folha dizia serem **um conserto só**).
///
/// ⚠️ **Só se julga com o PLAY** — as duas linhas são temporais.
pub(crate) fn operator_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_operator::build_operator_demo_document(doc, registry).unwrap_or_default();
    let (len, beat) = conferencia_demos_operator::authored();
    eprintln!(
        "[cena 77] Dois exemplos, um por linha -- {n} bandas. ESQUERDA = como era,
  DIREITA = o que mudou. O nome da linha esta' escrito no meio da tela.

  >>> DE' PLAY. As duas linhas sao temporais; paradas, as metades sao iguais.

  RASTRO  Uma bolinha azul percorre um OITO deitado, deixando {len:.0} ecos atras dela. Como
          o caminho se CRUZA, a cauda passa por cima de si mesma.
          A` esquerda a cauda TAPA o que esta' atras -- no cruzamento fica tudo igual.
          A` direita ela SOMA: o cruzamento ACENDE, mais claro que os dois lados.
          > clique no no' Trail da direita e troque o `Echo Operator` para `Normal`:
            o cruzamento apaga e ela fica igual a` da esquerda.

  FLASH   A mesma bolinha, piscando a cada {beat:.1} s.
          A` esquerda o flash branco TAPA a bolinha. A` direita ele SOMA -- o pico
          estoura de branco e transborda.
          > clique no no' Strobe da direita e troque o `Flash Operator` para `Normal`.

  (!) O `Sink` (o primeiro item dos dois dropdowns) NAO e' um modo: quer dizer
      \"o mesmo do Output\". E' o default, e e' por isso que nada muda ate' escolher.

  DEU ERRADO se as duas metades de qualquer linha ficarem iguais com o Play rodando,
  ou se o cruzamento do oito da direita nao acender.",
        n = sinks.len(),
    );
    sinks
}
