//! **A TABELA DOS KNOBS MORTOS CONHECIDOS** — cada entrada com a razão que a
//! torna inofensiva, que é o que separa uma DÍVIDA de uma isenção.
//!
//! ⚠️ **Ela é uma CATRACA que só encolhe**, e as duas metades do censo irmão
//! ([`super`]) defendem-na: um morto novo que ninguém nomeie REPROVA, e uma
//! entrada que já não descreve nada reprova a pedir para sair.

use ph2d_sculpt3d::Verb;

/// ⛔⛔⛔ **A CATRACA DOS KNOBS MORTOS — e é ela que faz disto um PORTÃO e não um
/// relatório.**
///
/// Cada entrada é um `(verbo, knob)` que o painel **PINTA** e que o barro
/// **não sente**, com o motivo ao lado. As duas metades obrigatórias:
///
/// * um morto **NOVO** reprova ⇒ um verbo não pode nascer a oferecer um knob
///   que ele não lê;
/// * um morto **CURADO** reprova ⇒ a lista só desce, e uma entrada que já não
///   descreve nada é a catraca a virar **licença** (§5.0).
///
/// ⚠️ **O motivo é o que separa uma DIVERGÊNCIA de uma DÍVIDA**, e as duas
/// entradas que sobram são **divergências medidas**: as duas são a fileira da
/// CURVA, que o painel pinta **sempre** por uma cerca de produto escrita e
/// gateada (ver [`pintado`] e o cabeçalho do `paint/brush.rs`), sobre verbos que
/// a leem noutro regime ou não a leem de todo.
///
/// ⭐⭐⭐ **A lista desceu de `5` para `2` em 2026-09-15, e NENHUMA das três que
/// saíram saiu por ser reclassificada** — cada uma teve uma causa medida:
/// - `Pose × radius` era **a régua**: este ficheiro carregava um raio próprio ao
///   lado do pincel e o produto tem **um** ([`o_raio_do_dab_sai_do_pincel`]);
/// - `Pose × hardness` era **a lente do painel**, mais larga que a do
///   consumidor: a fileira já tinha a porta certa (`shapes_the_distance`) e
///   faltava-lhe o lado do VERBO
///   ([`ph2d_sculpt3d::Verb::a_lei_le_a_distancia_ao_cursor`]);
/// - `Pose × auto_smooth` era **dívida real e foi CONSTRUÍDA**: a espec do
///   pincel prescreve a lei (§15 e item 18 — *«ela segue os pesos, não o
///   raio»*), e hoje ela corre em `ph2d_sculpt3d::stroke_pose::alisa_a_pose`.
///
/// ⚠️⚠️ **E na MESMA jornada o arnês acordou cinco verbos e as `25` células
/// novas acusaram DOIS mortos — os dois curados por ESCONDER, não por ligar:**
/// `Cloth × auto_smooth` e `Boundary × auto_smooth` liam `0,000e0` porque os
/// dois desviam antes do laço por-vértice onde o passe corre. ⛔ Nenhuma das
/// duas especs prescreve auto-suavização para aquele pincel, e *inventar uma lei
/// para um pincel de clean-room sem referência é o que a parede existe para
/// impedir* ⇒ o painel deixa de a pintar
/// ([`ph2d_sculpt3d::Verb::o_auto_smooth_chega`], com as duas saídas nomeadas lá
/// dentro). *Um censo que mede mais encontra mais, e é por isso que acordar um
/// verbo vale mais do que curar um knob.*
pub(super) const MORTOS_CONHECIDOS: &[(Verb, &str, &str)] = &[
    (
        Verb::Density,
        "panel.sculpt3d.falloff",
        "ELE NÃO TEM LEI POR-VÉRTICE: o efeito inteiro dele é sobre a \
         TOPOLOGIA, e o `dab` sai antes de a cadeia de peso existir. A fileira \
         da curva é a única que o painel pinta SEMPRE (cerca de produto medida \
         e gateada), logo ela não pode ser escondida como o `Strength` e o \
         `Auto-Smooth` dele foram — e é por isso que ela leva a RAZÃO À VISTA \
         (`ph2d_sculpt3d::Brush::curva_inerte`), que a terceira metade do gate \
         abaixo exige de toda entrada desta lista. ⚠️ Ele só entrou aqui em \
         2026-09-15 porque até então era ADORMECIDO — e um verbo que o censo \
         não acorda tem os cinco knobs por medir, não zero mortos",
    ),
    (
        Verb::Mask,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada: o canal tem a SEGUNDA curva da referência, e a \
         do carimbo não o alcança (recusa medida, com dois gates a defendê-la)",
    ),
    (
        Verb::Paint,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada, e é a MESMA da máscara: os dois verbos de canal \
         têm a curva DELES (`paint_hardness` · `mask_hardness`), que é a mesma \
         fórmula da referência com durezas de fábrica diferentes (`0,75` contra \
         `0,25`) — e a curva que o artista escolhe no pincel governa a \
         GEOMETRIA. ⚠️ A fileira é pintada sempre pela cerca de produto MEDIDA \
         que a §39 desta linha registou; o `Brush::curva_inerte` já diz a razão \
         na tela",
    ),
    (
        Verb::Blur,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada, e é a MESMA dos outros dois verbos de canal: a          cadeia de peso de um canal tem a curva DELE          (`Brush::channel_weight`, a fórmula da referência com a dureza da          pintura), e a do carimbo governa a GEOMETRIA. ⚠️ Ele entrou aqui em          2026-09-19 e NÃO é um morto novo — é a mesma lei a alcançar a terceira          e a quarta ferramenta dela: o `Brush::curva_inerte` passou a DERIVAR a          resposta do `Verb::escreve_um_canal` em vez de enumerar os dois          primeiros, e a razão aparece na tela sem uma linha nova",
    ),
    (
        Verb::SmearColor,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada, letra por letra a do `Blur` — os dois leem o          ANEL de cor e partilham a cadeia de peso do canal",
    ),
    (
        Verb::Pose,
        "panel.sculpt3d.falloff",
        "DIVERGÊNCIA declarada, e o knob NÃO está morto: a espec dele (§1.2) diz \
         que só o modo de TORÇÃO lê a curva, e este censo mede o modo de \
         OMISSÃO. Onde ela é lida, ela chega — gate \
         `a_curva_do_pincel_chega_ao_modo_de_torcao`, que foi escrito porque ela \
         NÃO chegava (a ponte entre as duas convenções não invertia o argumento, \
         e a torção com o valor de fábrica era inerte). ⚠️ A fileira é pintada \
         sempre por cerca de produto MEDIDA, e esconder um knob vivo noutro modo \
         seria o defeito oposto",
    ),
];
