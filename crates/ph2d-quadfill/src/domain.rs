//! ⭐⭐ **A FORMA DO DOMÍNIO de um patch** — o polígono para onde a fronteira vai.
//!
//! ⚠️ **Irmão do [`crate::param`] pelo teto de LOC (HR-18, 700) e por ASSUNTO:** lá
//! *como* o patch é achatado e amostrado; aqui **para onde**. ⭐ A constante deste
//! ficheiro está **desligada com a tabela ao lado**, e a tabela é o produto: ela diz o
//! que já foi tentado, com que régua, e por que não é o que dói.

/// **OS CANTOS DO POLÍGONO** — o regular de `n` lados, inscrito no círculo unitário.
///
/// ⭐ **Ela é a fonte única dos cantos** — o achatamento e o `uv` dos pontos de
/// saída ([`crate::stitch`]) chamam-na com o mesmo `n`. *Duas contas do mesmo canto
/// dariam um ponto de fronteira cujo domínio discorda do da triangulação.*
///
/// ⛔⛔ **O polígono PROPORCIONAL AO COMPRIMENTO DOS LADOS foi construído, medido e
/// REJEITADO** (2026-08-21). O raciocínio era bom — um patch de lados `1` e `5`
/// achatado num quadrado tem um esticão embutido, e um passo igual no domínio vale
/// cinco vezes mais de um lado que do outro. Mas inscrever os cantos no círculo com
/// **ângulo proporcional ao comprimento** dá um polígono que pode ser quase
/// degenerado, e o embutimento esmaga triângulos até a localização de ponto falhar:
///
/// | fixtura | regular | proporcional |
/// |---|---|---|
/// | esfera 48×72 | **2,1 %** dobras, `0` pontos fora | ⛔ 13,2 %, **313** fora |
/// | esfera 24×36 | 5,5 %, `0` fora | 5,4 %, **62** fora |
/// | esfera esculpida | **0,9 %**, `0` fora | ⛔ 1,7 %, **95** fora |
///
/// *Um domínio sem esticão que não se consegue amostrar é pior que um esticado que
/// se consegue.* ⚠️ **A distorção continua a ser um problema real** — ela é o que
/// sobra na coluna `raio`/`grade` da proveniência —, mas a cura tem de manter os
/// triângulos amostráveis. *A próxima hipótese não é o polígono; é o mapa.*
#[must_use]
pub(crate) fn corners_for(n: usize) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            [a.cos(), a.sin()]
        })
        .collect()
}

/// ⛔⛔ **DESLIGADO — MEDIDO E REJEITADO** (2026-08-23). A construção fica, e o
/// interruptor está aqui em vez de num `if` no chamador **porque os DOIS sítios que
/// leem o polígono têm de concordar**: se o achatamento e o [`crate::patch::side_uv`]
/// discordarem, a malha sai rasgada ao longo de toda fronteira de patch, com um erro
/// pequeno demais para se ver num render. *Um interruptor num sítio não pode divergir.*
///
/// ⚠️ **Medida TRÊS vezes, cada uma com uma régua melhor** — e só a terceira vale,
/// porque separa o domínio por **valência** (esfera lisa, `d = 0,55`):
///
/// | | regular | ∝ a todo `n` | ⭐ ∝ só no `n = 4` |
/// |---|---|---|---|
/// | superfície dos **rectângulos** | `12°` | `10°` | ⛔ **`12°`** |
/// | domínio dos **leques** | `18,7°` | ⛔ **`28,6°`** | `18,7°` |
///
/// ⛔⛔ **A coluna do meio parecia uma cura e não era.** Restringi-la ao caso onde
/// ela é **exacta** — `n = 4`, onde a quantização garante lados opostos iguais e o
/// rectângulo fecha sem resíduo — devolve os rectângulos a `12°`. ⇒ *os `10°` não
/// vinham de os rectângulos melhorarem; vinham do lado que piorava os leques.*
///
/// ⛔⛔⛔ **A LINHA DO DOMÍNIO DOS RECTÂNGULOS SAIU DESTA TABELA, e o porquê importa
/// mais que a linha:** as três colunas mediam `0,0°`, e esse zero era a mediana de um
/// **balde vazio** — o caminho do rectângulo saía do laço dos patches por `continue`
/// antes da escrituração. Com a régua corrigida (ver [`crate::FillReport::domain_cells`]
/// e `tests/rulers.rs`) o número é **`1,0°`**, não `0,0°`, e a superfície é **`16°`**,
/// não `12°`.
///
/// ⚠️ **A conclusão desta tabela sobrevive à correcção** — a grade do rectângulo nasce
/// quase recta no domínio, logo mudar a FORMA do domínio não lhe podia tocar —, mas
/// ela passou a ser uma conclusão medida em vez de uma leitura de um zero mudo.
/// *O que a entorta está depois do domínio, e não é o mapa*: trocar o operador dá
/// `12° → 12°`, e o mapa **conforme** — que muda a condição de fronteira e não só o
/// operador — dá `16° → 14°` pagando as esculturas ([`crate::rectangle`]).
const PROPORTIONAL_DOMAIN: bool = false;

/// ⭐ **O POLÍGONO DO DOMÍNIO com os lados na PROPORÇÃO DOS SEGMENTOS.**
///
/// # ⛔ O que estava errado, e é a causa dominante do enviesamento
///
/// O [`corners_for`] devolve um polígono **regular**: todo lado tem o mesmo
/// comprimento no domínio, **independentemente de quantos quads ele carrega**. ⚠️ A
/// grade é construída no domínio, então um patch de 4 lados com `13 × 6` segmentos
/// recebe `13` divisões numa direcção e `6` na outra sobre um quadrado `1 × 1` — e
/// **toda célula nasce com aspecto `2,17` antes de tocar na superfície.**
///
/// ⇒ *Nenhuma regra sobre o INTERIOR pode consertar isso*, e foi por isso que ligar
/// o campo alinhado (2026-08-22) moveu o enviesamento mediano da orelha de `27°`
/// para… `27°`. **A distorção era imposta pela FRONTEIRA.**
///
/// # ⭐ A lei
///
/// O lado `i` mede, no domínio, o que ele carrega em segmentos. Assim uma célula da
/// grade é um **quadrado unitário no domínio**, e se o achatamento for próximo de
/// uma similaridade a célula sai quadrada na superfície também. *É a construção da
/// referência, e a razão de ela produzir `1,08` de aspecto.*
///
/// | `n` | construção | porquê |
/// |---|---|---|
/// | **4** | ⭐ **rectângulo EXACTO** `a × b` | a quantização garante lados opostos iguais, então ele fecha sem resíduo |
/// | outro | vértices no círculo em ângulo ∝ ao segmento acumulado | fecha sempre e é convexo — o que o teorema de Tutte exige |
///
/// ⚠️ **Com todos os lados iguais isto devolve o polígono regular de sempre**, e é
/// isso que torna a inércia demonstrável: um patch cujos lados carregam a mesma
/// contagem tem o domínio byte-idêntico ao de antes.
///
/// ⛔ **O `n`-gono não-rectângulo é APROXIMADO de propósito.** A corda de um sector
/// é `2·sin(π·sᵢ/S)`, não `sᵢ` — para `13 × 6` num quadrilátero isso daria razão
/// `1,83` em vez de `2,17`. Só o caso `n = 4` tem solução exacta com convexidade
/// garantida, e é o caso que domina a malha; construir um polígono convexo com
/// comprimentos de lado exactos para `n` qualquer é um problema com escolhas, e
/// nenhuma delas está medida.
pub(crate) fn corners_for_sides(seg: &[u32]) -> Vec<[f32; 2]> {
    let n = seg.len();
    if !PROPORTIONAL_DOMAIN || n < 3 {
        return corners_for(n.max(3));
    }
    let first = seg[0];
    if seg.iter().all(|&s| s == first) {
        return corners_for(n);
    }
    if n == 4 {
        // ⭐ Rectângulo exacto, normalizado para a diagonal do círculo unitário —
        // o mesmo alcance que o polígono regular usa, para o balde de [`cell`] não
        // mudar de escala.
        #[allow(clippy::cast_precision_loss)]
        let (a, b) = (seg[0].max(1) as f32, seg[1].max(1) as f32);
        let k = 1.0 / a.hypot(b);
        let (x, y) = (a * k, b * k);
        return vec![[x, y], [-x, y], [-x, -y], [x, -y]];
    }
    // ⛔⛔ **SÓ O `n = 4` LEVA A PROPORÇÃO, e a restrição é MEDIDA** (2026-08-23).
    //
    // A versão que dava a proporção a todo `n` punha os vértices no círculo em ângulo
    // ∝ ao segmento acumulado — uma **aproximação**, porque a corda de um sector é
    // `2·sin(π·sᵢ/S)` e não `sᵢ`. Medido na esfera lisa, com o domínio separado por
    // valência:
    //
    // | `d = 0,55` | regular | proporcional a todo `n` |
    // |---|---|---|
    // | ⭐ domínio dos **rectângulos** | `0,0°` | `0,0°` |
    // | ⭐ superfície dos **rectângulos** | `12°` | **`10°`** |
    // | ⛔ domínio dos **leques** | `18,7°` | ⛔ **`28,6°`** |
    //
    // ⚠️ **Os dois efeitos cancelavam-se no agregado** — foi por isso que a primeira
    // medição desta ideia (sobre a orelha, sem separar por valência) disse «não move
    // nada» e ela foi desligada inteira. *Uma régua que soma duas populações opostas
    // esconde as duas.*
    //
    // ⇒ fica onde é **exacto** e sai de onde era palpite.
    corners_for(n)
}
