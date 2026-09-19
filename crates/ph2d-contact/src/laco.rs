//! **O LAÇO DAS VARREDURAS** — quantas vezes a nuvem é varrida, o que cada passagem escreve, e as
//! DUAS saídas antecipadas.
//!
//! Irmão do [`super`] pelo tecto de LOC (HR-18) e por ASSUNTO: ali moram a FORMA de um colisor e o
//! que uma corrente declara; aqui mora o laço que os consome — a fotografia por varredura, a
//! grelha, a escrituração e as cercas que o param ([`crate::Cercas`]).
//!
//! ⚠️ A lei de UM par vive na [`crate::par`]; o que UMA peça soma dos vizinhos dela vive na
//! [`crate::varredura`]. *Três ficheiros, três perguntas.*

use super::{
    Cercas, Colisor, PECAS_PARA_PARALELIZAR, PISO_DA_TAREFA, Pecas, REPOUSO_VISIVEL, Saida, ativo,
    grelha, par_preenche_em_blocos, varredura,
};

/// O que uma varredura decidiu para uma peça: a posição nova e o giro em graus/// mais vivo que ela tocou.
pub(crate) type Nova = Option<([f32; 2], f32)>;

/// Confere que toda coluna tem o comprimento da nuvem.
pub(crate) fn confere(n: usize, saida: &Saida<'_>, pecas: &Pecas<'_>) {
    assert_eq!(saida.giro.len(), n, "um giro por peca");
    assert_eq!(pecas.colisores.len(), n, "um colisor por peca");
    assert_eq!(pecas.pesos.len(), n, "um peso por peca");
    assert_eq!(pecas.inv_inercia.len(), n, "uma inercia por peca");
    if let Some(d) = pecas.deslize {
        assert_eq!(d.antes.len(), n, "um antes por peca");
        assert_eq!(d.girou_antes.len(), n, "um giro anterior por peca");
        assert_eq!(d.material.len(), n, "um material por peca");
    }
}

/// Afasta as peças sobrepostas, `varreduras` vezes. `p` é reescrito no sítio; a [`Saida`] ACUMULA
/// em GRAUS o quanto cada peça rodou (doc 109 §6).
///
/// # Panics
///
/// Se alguma coluna da [`Saida`] ou das [`Pecas`] não tiver o comprimento de `p` — colunas de uma
/// mesma corrente com comprimentos diferentes não são uma pergunta com resposta.
pub fn separate(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
) -> usize {
    separate_com(
        p,
        saida,
        pecas,
        varreduras,
        p.len() >= PECAS_PARA_PARALELIZAR,
        REPOUSO_VISIVEL,
    )
}

pub(crate) fn separate_com(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
    paralelo: bool,
    repouso: f32,
) -> usize {
    separate_grao(
        p,
        saida,
        pecas,
        varreduras,
        &Cercas {
            paralelo,
            repouso,
            grao: PISO_DA_TAREFA,
            duas_camadas: grelha::duas_camadas_activas(),
        },
    )
}

/// Como o [`separate`], com as CERCAS por argumento — a porta pela qual um gate pede a lei sem
/// passar pelo ambiente. Ver [`Cercas`].
///
/// ⛔ **`#[cfg(test)]` porque o único consumidor dela é um gate:** o produto entra pelo
/// [`separate`], e uma segunda porta viva para a mesma lei seria a segunda maneira de a chamar.
#[cfg(test)]
pub(crate) fn separate_com_cercas(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
    cercas: &Cercas,
) -> usize {
    separate_grao(p, saida, pecas, varreduras, cercas)
}

pub(crate) fn separate_grao(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
    cercas: &Cercas,
) -> usize {
    let (paralelo, repouso, grao) = (cercas.paralelo, cercas.repouso, cercas.grao);
    let n = p.len();
    confere(n, saida, pecas);
    let ativo: Vec<bool> = (0..n)
        .map(|i| ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
    // ⭐ Os alcances saem de UMA porta, lida pelo plano da grelha e pela cerca do repouso — e eles
    // são dos COLISORES DECLARADOS, não dos girados: `alcance()` é feito das meias extensões e do
    // comprimento do desvio, e rodar preserva os dois (a menos de ULPs, como já era antes desta
    // wave).
    let alcances = grelha::alcances_de(pecas.colisores, &ativo);
    let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
    if alcance_max <= 0.0 {
        return 0;
    }
    // O repouso, na escala da PEÇA — ver [`REPOUSO_VISIVEL`]. ⭐ **É ARGUMENTO e não const lida
    // aqui** para que um gate possa pedir `0.0` e medir a lei do ponto fixo ao bit **sozinha**:
    // são duas paragens com naturezas diferentes, e uma régua que só visse a soma delas não podia
    // afirmar nada sobre nenhuma.
    let parado = repouso * alcance_max;
    // ⭐⭐ **Os buffers vivem FORA do laço** (report do dono, 18/09). Eles eram refeitos por
    // varredura, e a `1024` isso são `1024` cópias da nuvem, `1024` grelhas e `n × 1024` listas de
    // vizinhos. Os VALORES são os mesmos — o que muda é quem os aloja.
    let mut foto = p.to_vec();
    let mut girado = saida.giro.to_vec();
    let mut agora: Vec<Option<Colisor>> = (0..n)
        .map(|i| pecas.colisores[i].map(|c| c.girado(girado[i])))
        .collect();
    let mut grade = grelha::Grelha::default();
    // ⭐⭐⭐ **O PLANO da grelha corre UMA VEZ, não por varredura** — ele lê os ALCANCES, que não
    // mudam enquanto o laço corre. É ele que decide se a nuvem parte em duas camadas, e é por isso
    // que uma peça grande deixou de inflar a grelha de todas (ver o cabeçalho da [`grelha`]).
    grade.planeia_com(p, &ativo, &alcances, cercas.duas_camadas);
    // O destino de uma varredura, **fora do laço** — ver o comentário na chamada.
    let mut novas: Vec<Nova> = vec![None; n];
    for v in 0..varreduras {
        foto.copy_from_slice(p);
        // As formas COMO ESTÃO: o que as varreduras anteriores rodaram já conta.
        // ⭐ Só quem RODOU desde a varredura anterior é recalculado — `girado()` é uma função pura
        // do ângulo, logo quem não rodou tem de dar o mesmo colisor, **ao bit**. Numa cena assente
        // isto apaga duas chamadas de trigonometria por peça e por varredura.
        for i in 0..n {
            if girado[i] != saida.giro[i] {
                girado[i] = saida.giro[i];
                agora[i] = pecas.colisores[i].map(|c| c.girado(girado[i]));
            }
        }
        grade.constroi(&foto, &ativo);
        // ⭐⭐⭐ **O BUFFER É REAPROVEITADO E A PARTIÇÃO É EXPLÍCITA** (report do dono, 2026-09-18:
        // `1000 pecas x 68 varreduras x 156 vizinhos`, com o mesmo trabalho a correr em `4`–`5`
        // núcleos de 32). A rota anterior fazia `collect()` por varredura: um `Vec` novo de cada
        // vez, e a árvore de partição do rayon a descer até pedaços pequenos, com roubo de trabalho
        // e espera entre eles — **`5×` o CPU da série** para o mesmo resultado.
        //
        // ⇒ `novas` vive fora do laço, e o [`PISO_DA_TAREFA`] só impede o rayon de partir até um
        // elemento — a partição continua a ser dele.
        par_preenche_em_blocos(
            paralelo,
            &mut novas,
            grao,
            Vec::<u32>::new,
            |vizinhos, k, slot| {
                *slot = if ativo[k] {
                    grade.vizinhos_de(k, vizinhos);
                    varredura::corrigida(
                        k,
                        vizinhos.iter().map(|&j| j as usize),
                        &foto,
                        &agora,
                        pecas,
                        &ativo,
                    )
                } else {
                    None
                };
            },
        );
        let andou = aplica(p, saida, &mut novas, alcance_max);
        // ⭐⭐⭐ **O PONTO FIXO** — e ele não é uma heurística, é uma INDUÇÃO: uma varredura que não
        // mexe um bit deixa a seguinte com a MESMA entrada (a mesma foto, os mesmos ângulos, a
        // mesma grelha), logo com a mesma saída. ⇒ parar aqui é **bit-idêntico** a varrer até ao
        // fim, e é o que faz um tecto alto não se pagar numa cena que já assentou.
        //
        // ⚠️ A pergunta é *«mudou algum BIT?»* e não *«houve contacto?»*: uma nuvem assente
        // continua a ter contactos, e `corrigida` devolve `Some` com a posição inalterada.
        if andou == 0.0 {
            return v + 1;
        }
        // ⭐⭐⭐ **E O REPOUSO VISÍVEL** (report do dono, 18/09: *«centenas a milhares de objectos
        // em runtime»*). O ponto fixo ao bit quase nunca arma: com a rotação solta duas caixas
        // acertam-se por um ULP **para sempre**, e a cena paga o tecto inteiro por movimento que
        // ninguém vê.
        //
        // ⛔⛔ **Isto NÃO é o «aceita e mente» que o §18 recusou, e a distinção é a única coisa que
        // separa as duas:** aquele era um tecto que aceita `4096` e entrega MENOS TRABALHO, com um
        // resultado pior. Este pára quando **a RESPOSTA deixou de mudar** — medido, a contagem de
        // pares sobrepostos é *idêntica* à de varrer até ao fim, e a posição de cada peça difere
        // por menos de [`REPOUSO_VISIVEL`] da própria peça. *Um é cortar o trabalho; o outro é
        // reconhecer que ele acabou.*
        if andou < parado {
            return v + 1;
        }
    }
    varreduras
}

/// Escreve o que uma varredura produziu, e devolve **quanto o ponto que mais andou andou** — a
/// grandeza que decide as duas saídas antecipadas do [`separate`].
///
/// ⚠️ **Inclui a ROTAÇÃO**, majorada: um giro de `g` graus leva um ponto a `alcance` do centro a
/// andar `g·π/180·alcance`. Sem esse termo uma peça que só roda leria *«não se mexeu»*.
///
/// ⚠️ Conservador de propósito: devolve `f32::INFINITY` se algum valor escrito não for finito, para
/// que um `NaN` nunca seja lido como *«nada mudou»*.
///
/// ⚠️⚠️ **Ele LÊ o buffer e não o limpa, e isso é uma propriedade de quem o enche:** o
/// `par_preenche_em_blocos` escreve **todos** os índices em cada varredura (o braço inactivo escreve
/// `None`), logo não há resto da passagem anterior. A 1.ª redacção fazia `take()` — uma escrita por
/// peça e por varredura para nada —, e foi uma **mutação sobrevivente** que o mostrou: *uma linha
/// que a mutação não consegue matar não é lei, é comentário com sintaxe de código*.
pub(crate) fn aplica(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    novas: &mut [Nova],
    alcance: f32,
) -> f32 {
    const POR_GRAU: f32 = core::f32::consts::PI / 180.0;
    let mut maior = 0.0_f32;
    for (k, nova) in novas.iter_mut().enumerate() {
        if let Some((q, g)) = *nova {
            let (antes_p, antes_g) = (p[k], saida.giro[k]);
            p[k] = q;
            saida.giro[k] += g;
            let andou = (p[k][0] - antes_p[0]).hypot(p[k][1] - antes_p[1])
                + (saida.giro[k] - antes_g).abs() * POR_GRAU * alcance;
            maior = if andou.is_finite() {
                maior.max(andou)
            } else {
                f32::INFINITY
            };
        }
    }
    maior
}
