//! **A MÁSCARA DA REFERÊNCIA** — o `Masking.paint`, e a SEGUNDA curva do
//! SculptGL.
//!
//! Filho de [`super`] (`ref_kernels`) pela mesma razão do [`super::smooth`]: é o
//! mesmo porte, e a superfície pública fica plana.
//!
//! ⚠️ **A afirmação *"a referência tem UMA curva para as dez tools"* está
//! ERRADA, e este arquivo é a correção.** Ela vale para as dez que movem
//! GEOMETRIA — as que multiplicam pela quártica de [`super::falloff`]. O
//! `Masking` (e o `Paint`, que ele chama: `Masking.js:37`) tem curva **própria**,
//! `(1 − d)^softness` com `softness = 2·(1 − hardness)`, e um knob **hardness**
//! que a molda (`Masking.js:14`, `_hardness = 0.25` ⇒ `softness = 1.5`).
//!
//! Isso importa porque o pedido do Enio — *"cada Tool deve ter seu falloff
//! apropriado"* — parecia contradizer a paridade, e não contradiz: a referência
//! **já** dá ao canal de máscara uma curva que não é a da geometria. O que ela
//! não tem é um SELETOR de falloff, que é nosso e é um superconjunto.
//!
//! # As três diferenças com a família da geometria, cada uma com consequência
//!
//! - **O `dist` é CLAMPADO, não descartado.** Os nove irmãos fazem `if dist >=
//!   1.0 { continue }`; aqui é `if dist > 1 { dist = 1 }` (`Masking.js:66`).
//!   ⚠️ **E ele é INALCANÇÁVEL pela própria referência — medido, não suposto:**
//!   a mutação que o troca pelo `continue` dos irmãos **sobreviveu ao gate**,
//!   porque quem monta a lista é o `pickVerticesInSphere`, que só admite
//!   `d² < r²` ⇒ `dist < 1` sempre. Ele é defesa em camadas, e fica documentado
//!   em vez de gateado (o precedente do ADR-0145).
//!   ⚠️ **A minha primeira versão desta nota dizia que o clamp é o que torna
//!   `hardness = 1` exprimível, e isso está ERRADO:** com `dist < 1` garantido,
//!   `(1 − d)^0 == 1` em toda a pegada com ou sem clamp. O disco duro sai do
//!   EXPOENTE ZERO, e o clamp não participa.
//! - **Ela ACUMULA no canal e satura** (`clamp(m + f, 0, 1)`), em vez de escrever
//!   um alvo. É a lei que faz esfregar construir máscara.
//! - **`_negative` nasce `true`** (`Masking.js:16`), e com a polaridade da
//!   referência (`1` = livre) isso significa que o gesto de fábrica **PROTEGE**.
//!
//! ⚠️ **Este é o único kernel do porte que paga um transcendental** (`powf`,
//! o `Math.pow` do original). Ele não é aproximável por tabela sem deixar de ser
//! um porte, e o HR-5 não o alcança: nada aqui alimenta um hash determinista
//! cross-OS — o que existe é o gate contra o JS, e é ele que mede se as duas
//! `pow` concordam ao bit em vez de eu afirmar que concordam.

/// **`Masking.paint`** — escreve o canal de máscara **na polaridade da
/// referência** (`1` = livre).
///
/// `free` é indexado por id de vértice (passo 1), como em todo este porte; a
/// posição é lida do estado **VIVO**, que é o que o original faz (`vAr`, e não
/// um proxy — a máscara não tem `accumulate`).
///
/// `negative = true` **protege** (o default de fábrica); `false` limpa.
///
/// ⚠️ **`hardness` é o knob do original, não um número que eu escolhi:** `0.25`
/// de fábrica, e ele entra como `softness = 2·(1 − hardness)`.
/// ⚠️ **Oito argumentos, e o `allow` é o mesmo dos irmãos:** eles são os
/// parâmetros do `Masking.paint` da referência, um a um. Empacotá-los num
/// struct tornaria a comparação com o original um exercício de tradução em vez
/// de leitura lado a lado, que é a única coisa que este módulo existe para
/// permitir.
#[allow(clippy::too_many_arguments)]
pub fn mask(
    free: &mut [f32],
    pos: &[f32],
    verts: &[u32],
    center: [f64; 3],
    radius_squared: f64,
    intensity: f64,
    hardness: f64,
    negative: bool,
) {
    let radius = radius_squared.sqrt();
    let softness = 2.0 * (1.0 - hardness);
    let signed = if negative { -intensity } else { intensity };
    for &v in verts {
        let ind = v as usize * 3;
        let dx = f64::from(pos[ind]) - center[0];
        let dy = f64::from(pos[ind + 1]) - center[1];
        let dz = f64::from(pos[ind + 2]) - center[2];
        let mut dist = (dx * dx + dy * dy + dz * dz).sqrt() / radius;
        if dist > 1.0 {
            dist = 1.0;
        }
        let fall = (1.0 - dist).powf(softness) * signed;
        let m = f64::from(free[v as usize]) + fall;
        free[v as usize] = m.clamp(0.0, 1.0) as f32;
    }
}
