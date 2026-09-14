//! §5 — resolver a cadeia a cada evento do ponteiro.
//!
//! ⚠️⚠️ **O solver é INCREMENTAL, e isso é a coisa mais fácil de implementar ao
//! contrário nesta lei.** A **direcção** de cada segmento sai da origem em que o
//! **evento anterior** o deixou; só a **rotação** é medida contra o estado
//! inicial. ⇒ isto não é um solver fechado avaliado no deslocamento final: é uma
//! relaxação que dá **um passo por evento**.
//!
//! **A consequência, medida:** a pose final **depende de quantos eventos o
//! ponteiro entregou**, para cadeias de mais de um segmento — o mesmo arrasto em
//! `4`, `12` e `36` eventos difere em `5,3e-2`. ⭐ Com **um** segmento e âncora
//! ligada o deslocamento da âncora repõe a origem exactamente, o solver fica
//! **sem memória**, e as três amostragens dão o mesmo **ao bit**.
//!
//! ⛔⛔ **E é por isso que um gate «as três taxas concordam» REPROVA sobre
//! produto correcto.** O gate certo é *«a nossa saída bate a do oráculo **em
//! cada taxa**»*.

use crate::cadeia::Cadeia;
use crate::vetor::{add, escalar, normalizar, ponto, sub, Rot, V3};
use crate::{Controlos, Deformacao};

/// O que muda de um evento para o outro.
pub struct Evento {
    /// `G` — o deslocamento do arrasto em espaço de **objecto**: a diferença
    /// entre o ponteiro projectado no plano de profundidade constante que passa
    /// pelo ponto de aplicação ancorado, e esse mesmo ponto no primeiro evento.
    pub arrasto: V3,
    /// O deslocamento do ponteiro em **pixels** no eixo `x` desde o primeiro
    /// evento.
    ///
    /// ⚠️ **Lido SÓ pelo modo de torção** (§5.2) — que é o único sítio em que
    /// este pincel lê pixels, e portanto o único cuja saída depende da resolução
    /// do ecrã e do zoom. *Sem o factor pixels-por-unidade, uma fixtura de
    /// torção não é reproduzível.*
    pub dx_pixels: f32,
}

/// ⭐ `k = 0,020000` **radianos por pixel**, MEDIDO da saída do oráculo: num
/// segmento de peso `1` a deformação é rotação pura em torno do eixo do
/// segmento, e o ângulo lê-se da malha deformada. Duas fixturas a forças
/// diferentes dão o mesmo `k` — *o par mede a constante **e** confirma que a
/// força entra linearmente*. Dispersão entre vértices: `1,8e-6`.
const RAD_POR_PIXEL: f32 = 0.020_000;

/// Resolve a cadeia para o estado deste evento.
pub fn resolver(cadeia: &mut Cadeia, ctrl: &Controlos, ev: &Evento) {
    // §1.3 — a força efectiva é `força × pluma_de_simetria`, e a pluma vale `1`
    // a menos que a opção de esbatimento da simetria esteja ligada.
    // ⚠️ A **pressão da caneta NÃO entra aqui**, e está medido: a fixtura com
    // pressão `0,3` é idêntica **ao bit** à sem pressão, sobre 4 930 vértices.
    let s = ctrl.forca;
    let alvo = add(cadeia.ancora, escalar(ev.arrasto, s));

    match ctrl.deformacao() {
        Deformacao::Rodar => resolver_corrente(cadeia, alvo, ctrl.ancorado),
        Deformacao::Torcer => torcer(cadeia, ctrl, ev, s),
        Deformacao::Escalar => {
            // §5.4 passo 1 — ⚠️ com a trava **desligada** o gesto roda **e**
            // escala; com ela ligada, escala sem rodar.
            if !ctrl.trava_rotacao {
                resolver_corrente(cadeia, alvo, ctrl.ancorado);
            }
            let k = quociente_de_escala(cadeia, alvo);
            for seg in cadeia.segmentos.iter_mut() {
                seg.escala = [k, k, k];
            }
        }
        Deformacao::Transladar => {
            let g = escalar(ev.arrasto, s);
            for seg in cadeia.segmentos.iter_mut() {
                seg.origem = add(seg.origem_inicial, g);
                seg.rot = Rot::IDENTIDADE;
            }
        }
        Deformacao::Espremer => espremer(cadeia, alvo),
    }
}

/// §5.1 — a cadeia de cinemática inversa.
///
/// Os segmentos resolvem-se **do mais próximo do cursor para o mais distante**,
/// cada um contra um alvo que **cada segmento reescreve para o seguinte**.
fn resolver_corrente(cadeia: &mut Cadeia, alvo_inicial: V3, ancorado: bool) {
    let mut alvo = alvo_inicial;
    for seg in cadeia.segmentos.iter_mut() {
        let anterior = seg.origem;
        let Some(d) = normalizar(sub(alvo, anterior)) else {
            // O alvo caiu em cima da origem: a direcção é indeterminada.
            seg.rot = Rot::IDENTIDADE;
            alvo = seg.origem;
            continue;
        };
        // ⚠️ A rotação mede-se contra o estado **INICIAL**, nunca contra `O⁻` —
        // é isto que faz a deformação ser a pose acumulada desde o princípio do
        // traço, e não um incremento por evento.
        // ⭐ E a direcção inicial vem pela porta que sabe distinguir uma
        // direcção de ruído (§11.1): sem ela esta linha roda a região por um
        // ângulo arbitrário quando o pivô cai em cima do cursor.
        seg.rot = match seg.direccao_inicial() {
            Some(inicial) => Rot::entre(inicial, d),
            None => Rot::IDENTIDADE,
        };
        // ⭐ A origem é posta de modo que um segmento do comprimento original,
        // apontado ao longo de `d`, tenha a ponta distante exactamente no alvo.
        seg.origem = sub(alvo, escalar(d, seg.comprimento));
        alvo = seg.origem;
    }

    // Epílogo da âncora: a cadeia inteira desloca-se pelo vector que devolve a
    // origem do **último** segmento ao sítio onde nasceu.
    // ⭐ **Com UM segmento e âncora ligada a origem volta sempre ao lugar**, logo
    // a translação do mapa (§6) é nula e a deformação é **rotação pura em torno
    // do pivô** — que é o que o nome do pincel promete.
    if ancorado {
        let Some(ultimo) = cadeia.segmentos.last() else {
            return;
        };
        let delta = sub(ultimo.origem_inicial, ultimo.origem);
        for seg in cadeia.segmentos.iter_mut() {
            seg.origem = add(seg.origem, delta);
        }
    }
}

/// §5.2 — a torção. As posições de cabeça e origem **não** se mexem.
fn torcer(cadeia: &mut Cadeia, ctrl: &Controlos, ev: &Evento, s: f32) {
    let n = cadeia.segmentos.len().max(1) as f32;
    // `(x_no_primeiro_evento − x_agora)` ⇒ o simétrico do deslocamento.
    let angulo = -ev.dx_pixels * s * RAD_POR_PIXEL;
    for (i, seg) in cadeia.segmentos.iter_mut().enumerate() {
        let Some(eixo) = seg.direccao_inicial() else {
            seg.rot = Rot::IDENTIDADE;
            continue;
        };
        // A curva de atenuação avaliada no **índice do segmento**, não numa
        // distância — este é o único controlo do pincel que a lê.
        let atenuacao = (ctrl.curva)(1.0 - i as f32 / n);
        // ⚠️ A rotação guardada é a **INVERSA** da rotação do segmento.
        seg.rot = Rot::eixo_angulo(eixo, -(angulo * atenuacao));
    }
}

/// §5.4 passos 2–3 — o quociente de escala.
///
/// ⚠️⚠️ **Tem um POLO em `δ = comprimento₀`** e muda de sinal ao atravessá-lo.
/// Não há saturação neste modo, e isso é o alvo: uma fixtura atinge
/// deslocamento máximo `11,2` numa peça de extensão `2,0`. ⇒ perto do polo **a
/// paridade não é asserível** e a barra passa a ser relativa (§11.2, §12.3).
fn quociente_de_escala(cadeia: &Cadeia, alvo: V3) -> f32 {
    let Some(primeiro) = cadeia.segmentos.first() else {
        return 1.0;
    };
    let Some(normal) = primeiro.direccao_inicial() else {
        return 1.0;
    };
    let delta = ponto(sub(alvo, primeiro.cabeca_inicial), normal);
    primeiro.comprimento / (primeiro.comprimento - delta)
}

/// §5.5 — espremer / esticar.
///
/// ⚠️⚠️ **Este modo NÃO resolve a cadeia.** Ele toma **só** o quociente de
/// escala; cabeça, origem e rotação ficam nos valores **iniciais** o traço todo.
/// ⛔ Importar para aqui o passo de resolução do §5.4 torna real a divergência
/// que o §5.1 mede como invisível — é o item **13** da lista de verificação.
fn espremer(cadeia: &mut Cadeia, alvo: V3) {
    let z = quociente_de_escala(cadeia, alvo);
    let escala = if z.abs() < 1e-5 {
        // ⭐ A guarda que existe, e a razão pública de ela existir: sem ela a
        // malha ia a `NaN` e o **desfazer não a recuperava**. A nossa tem de
        // existir *e* ser testada (item 9 da lista de verificação).
        [0.0, 0.0, 0.0]
    } else {
        // Conserva o volume: `x·y·z = ±1` em módulo.
        let xy = z.signum() * (1.0 / z.abs()).sqrt();
        [xy, xy, z]
    };
    for seg in cadeia.segmentos.iter_mut() {
        seg.escala = escala;
    }
}
