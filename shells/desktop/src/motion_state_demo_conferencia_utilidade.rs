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
