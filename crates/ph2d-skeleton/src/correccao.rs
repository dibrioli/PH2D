//! ⭐⭐⭐ **A CORRECÇÃO DE PESO À MÃO** — a mancha que o artista pinta onde a conta automática
//! errou, e as DUAS leis que ela pode carregar ([`Especie`]).
//!
//! ⚠️ **Módulo irmão do [`crate`] pelo tecto de 700 LOC, e o corte é por RESPONSABILIDADE:** ali
//! vive *o que é um osso e como um ponto se mistura*; aqui vive *o que o artista faz quando a
//! conta automática erra*. ⛔ A cura de um tecto vermelho é o CORTE, nunca uma entrada numa lista
//! de dívida (`CLAUDE.md` §5.0).
//!
//! ⚠️ **A porta que o [`crate::Skin::weights_corrected`] chama sobe a `pub(super)`**: uma função
//! privada declarada num módulo filho é invisível ao pai, e é isso — e só isso — que a
//! visibilidade aqui diz. Nada nesta lei é público para fora da crate.

use crate::{Skin, SkinBone, bend, project_to_segment};

/// ⭐⭐⭐ **O QUE UMA MANCHA FAZ AO PESO** — e são DUAS perguntas diferentes, não um número com dois
/// significados (ordem do dono, 2026-09-19: *«precisamos de 2 modos de atribuir peso aos pontos»*).
///
/// ⚠️⚠️ **Ela CARREGA o número de propósito.** A 1.ª forma deste desenho tinha um `especie: Especie`
/// ao lado de um `delta: f64`, e ali o mesmo campo queria dizer *«quanto SOMAR»* num modo e *«que
/// valor PÔR»* no outro — que é à letra a armadilha do
/// [`project-memory`](../../../project-memory/feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting.md):
/// *chave e texto do mesmo tipo é um defeito à espera*. Com o número **dentro** da variante, todo
/// leitor tem de dizer qual lê, e o compilador cobra-o.
///
/// ⚠️ **Ela GRAVA-SE, e é por isso que deriva `serde` aqui e não num enum-espelho da
/// `ph2d-skeleton-ecs`** — é o precedente que o [`reach::BendSide`] já abriu, com a razão escrita no
/// `Cargo.toml` desta crate: *duas definições do mesmo conceito divergem em silêncio*.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Especie {
    /// **CUMULATIVO** — a mancha SOMA (ou tira) este valor ao peso do osso, no centro. O sinal é a
    /// direcção. É a lei de sempre, e a que o dono aprovou em smoke.
    ///
    /// ⚠️ Duas manchas destas sobrepostas **acumulam-se por construção**, e isso é o modo.
    Soma(f64),
    /// **ABSOLUTO** — a mancha PÕE o peso do osso neste valor, no centro, e reparte o que sobra
    /// (`1 − v`) pelos **outros** ossos daquele ponto, **mantendo a proporção entre eles**.
    ///
    /// ⭐⭐⭐ **Duas destas sobrepostas NÃO se somam: vence a ÚLTIMA que o artista pintou** (ordem do
    /// dono, 2026-09-19: *«a última manda — é o comportamento normal de um pincel absoluto»*). ⚠️ E
    /// isso não pede bookkeeping nenhum: a lista aplica-se **por ordem** e fixar é idempotente, logo
    /// no centro da última o valor dela é exactamente o que fica. *A ordem da lista passou a ter
    /// significado, e é só isso.*
    ///
    /// ⛔ **O caso degenerado tem resposta do DONO** (2026-09-20): num ponto que **só** o osso em
    /// mãos governa não há por onde repartir o `1 − v` — *«o osso fica com 100% independente do
    /// valor»*. Ver [`Skin::fixa`].
    Alvo(f64),
}

/// ⭐⭐⭐ **UMA CORRECÇÃO DE PESO FEITA À MÃO** — uma mancha no espaço que corrige o peso de um osso,
/// onde a conta automática errou.
///
/// ⚠️ Ver [`Skin::corrige`] para o mecanismo, o bump e a razão de ela ser uma MANCHA e não uma
/// tabela por vértice; e [`Especie`] para as duas leis que ela pode carregar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correccao {
    /// ⭐ **O TENDÃO** — o índice do osso **que o artista desenhou** na lista que o chamador
    /// resolveu, o mesmo espaço do [`SkinBone::tendon`].
    ///
    /// ⚠️ **E não o sub-osso:** o artista corrige o osso que ele vê, e a repartição por sub-ossos
    /// de um osso que dobra é feita pela lei, não por ele.
    pub tendon: u32,
    /// O centro da mancha, **no espaço da coisa deformada** (o mesmo dos eixos de repouso) — logo
    /// ela fica onde o artista a pôs, mesmo que ele mexa no desenho depois.
    pub centro: [f64; 2],
    /// O raio, nas unidades da coisa deformada. `<= 0` ⇒ a mancha não alcança nada.
    pub raio: f64,
    /// ⭐⭐⭐ **O QUE ELA FAZ, com o número dentro** — ver [`Especie`].
    ///
    /// ⛔⛔ **O campo que estava aqui era um `delta: f64` com a frase *«não há um segundo modo a
    /// lembrar»*, e a premissa morreu DUAS vezes:** primeiro na TELA (2026-09-19, os botões
    /// *Add*/*Subtract* — o dado não mudou um bit), e depois na LEI (2026-09-19/20, os dois modos
    /// de atribuir peso — ⚠️ e essa mudou o dado). *A segunda morte está à vista neste tipo.*
    pub especie: Especie,
}

impl Skin {
    /// ⭐⭐⭐ **A CORRECÇÃO À MÃO** — o artista soma (ou tira) peso a um osso, num sítio.
    ///
    /// # ⚠️ Porque ela é uma MANCHA no espaço e não uma tabela por vértice
    ///
    /// *Uma tabela indexada por ordem de varredura é o vector paralelo que o
    /// `VecVertex::corner_radius` proíbe por escrito*: dezenas de operações inserem, apagam,
    /// invertem e soldam vértices, e cada uma teria de se lembrar de mexer nela. Uma mancha é
    /// **ancorada na geometria** — ela diz *«aqui»*, e continua a dizer «aqui» depois de o artista
    /// mexer no desenho.
    ///
    /// # ⚠️ O bump é o MESMO da lei euclidiana
    ///
    /// `(1 − x²)²` com `x = d/raio`: `1` no centro, **`0` E derivada `0`** na borda. ⛔ Uma queda
    /// linear deixaria uma aresta visível no sítio exacto onde o artista pintou — o estalo que a
    /// continuidade C¹ da casa existe para não ter.
    ///
    /// # ⚠️ O sujeito é o TENDÃO e não o sub-osso
    ///
    /// O artista corrige *o osso que ele desenhou*; um osso que dobra tem `N` poses, e a correcção
    /// reparte-se por elas pela **mesma** lei que já reparte o peso ([`bend::share`]). ⛔ Corrigir
    /// um sub-osso seria expor ao artista uma divisão que ele não fez.
    ///
    /// ⚠️ **Renormaliza no fim, e só se alguma mancha alcançou o ponto** — senão isto não seria um
    /// no-op sobre a lei que já normalizou.
    ///
    /// # ⭐⭐⭐ A ORDEM da lista passou a ter significado (F29, 2026-09-20)
    ///
    /// As manchas aplicam-se **em sequência**, e não como um conjunto comutativo. Para as
    /// [`Especie::Soma`] isso não muda um bit (elas somam num acumulador e a normalização vem no
    /// fim, exactamente como antes); para as [`Especie::Alvo`] é **a lei inteira** — fixar é
    /// idempotente, logo no centro da última a pintar é o valor dela que fica, que é o que o dono
    /// pediu (*«a última manda»*).
    ///
    /// ⚠️ **Uma `Alvo` precisa que `Σw = 1` para o `1 − v` querer dizer alguma coisa**, e uma `Soma`
    /// que a preceda deixa a soma fora de `1` ⇒ normaliza-se **antes** dela, e só quando há algo
    /// pendente. ⛔ Numa lista só de `Soma` esse ramo nunca corre, e é isso que mantém o caminho de
    /// sempre **byte-idêntico**.
    pub(super) fn corrige(&self, p: [f64; 2], w: &mut [f64], correcoes: &[Correccao]) {
        if correcoes.is_empty() {
            return;
        }
        let mut mexeu = false;
        let mut pendente = false;
        for c in correcoes {
            if c.raio <= 0.0 {
                continue;
            }
            let d2 = (p[0] - c.centro[0]).powi(2) + (p[1] - c.centro[1]).powi(2);
            let x2 = d2 / (c.raio * c.raio);
            if x2 >= 1.0 {
                continue;
            }
            let t = 1.0 - x2;
            let bump = t * t;
            match c.especie {
                Especie::Soma(delta) => {
                    if !delta.is_finite() {
                        continue;
                    }
                    for (i, b) in self.bones.iter().enumerate() {
                        if b.tendon != c.tendon {
                            continue;
                        }
                        w[i] = (delta * bump)
                            .mul_add(self.quota(b, p), w[i])
                            .clamp(0.0, 1.0);
                        mexeu = true;
                        pendente = true;
                    }
                }
                Especie::Alvo(valor) => {
                    if !valor.is_finite() {
                        continue;
                    }
                    if pendente {
                        Self::normaliza(w);
                        pendente = false;
                    }
                    if self.fixa(p, w, c.tendon, valor, bump) {
                        mexeu = true;
                        pendente = true;
                    }
                }
            }
        }
        if mexeu {
            Self::normaliza(w);
        }
    }

    /// ⛔ **Soma zero deixa `w` como está** — o artista tirou tudo, e a mistura devolve o ponto
    /// INTACTO (a lei do [`Skin::blend`]). ⚠️ Dividir por zero daria `NaN` em toda a arte.
    fn normaliza(w: &mut [f64]) {
        let soma: f64 = w.iter().sum();
        if soma > 0.0 {
            for v in w.iter_mut() {
                *v /= soma;
            }
        }
    }

    /// ⭐⭐⭐ **A LEI ABSOLUTA — pôr o peso do tendão em `valor` e repartir o resto pelos OUTROS.**
    ///
    /// Devolve `true` se este tendão existe nesta pele (senão não há nada a fixar, e o chamador não
    /// deve marcar a correcção como tendo mexido).
    ///
    /// # ⚠️ O bump entra como uma MISTURA, nunca como um multiplicador
    ///
    /// `novo = lerp(w_actual, valor, bump)`: no centro (`bump = 1`) o peso **é** o valor pedido, e
    /// na borda (`bump → 0`) ele volta ao que a lei automática dava, **com derivada zero**. ⛔ Um
    /// `valor · bump` daria peso ZERO na borda da mancha — o artista pediria `0,8` e receberia um
    /// buraco à volta.
    ///
    /// # ⚠️ A repartição NÃO é a normalização que já existe
    ///
    /// O [`Skin::normaliza`] divide **todos** pela soma, incluindo o osso em mãos. Aqui o osso em
    /// mãos fica **preso** em `novo` e só os outros escalam por `(1 − novo)/Σoutros` — que é,
    /// à letra, *«o resto reparte-se pelos outros mantendo a proporção entre eles»*.
    ///
    /// # ⛔ O caso degenerado é DECISÃO DO DONO (2026-09-20)
    ///
    /// Quando `Σoutros == 0` não há por onde repartir. Ele decidiu: ***«o osso fica com 100%
    /// independente do valor»*** ⇒ o alvo local passa a `1`, e não ao `valor` pedido.
    ///
    /// ⭐ **No caso que ele nomeou isto é um NO-OP**, e é o que o torna seguro: um ponto que só este
    /// osso governa já tem `w = 1` depois da normalização da lei de base, logo `lerp(1, 1, bump)`
    /// é `1`. ⚠️ O outro caso de `Σoutros == 0` é o ponto que **ninguém** governa (a tabela guardada
    /// está vazia ⇒ `Σw = 0`), e ali esta lei dá ao osso a posse dele — que é **exactamente** o que
    /// a [`Especie::Soma`] já fazia no mesmo arranjo (ela soma num vector de zeros e a
    /// normalização final leva-o a `1`). *As duas espécies concordam ali, e há gate a dizê-lo.*
    ///
    /// ⭐⭐⭐ **E o «independente do valor» é observável em `v = 0` E SÓ ALI — medido por uma mutação
    /// SOBREVIVENTE.** Com `Σoutros == 0` a renormalização final devolve `1` a **qualquer** valor
    /// positivo (o vector `[v, 0, 0]` soma `v` e divide-se por si mesmo), logo apagar esta regra não
    /// muda um bit em todo o curso menos um ponto. No **zero** as duas leituras são opostas: com a
    /// regra o osso fica com tudo; sem ela o ponto fica **sem dono** e a arte congela ali. *A frase
    /// do dono escolhe exactamente a célula que o resto da lei não decide.*
    ///
    /// # ⚠️ Os sub-ossos são reescritos pela QUOTA, e isso é exacto
    ///
    /// As duas leis de base já escrevem `w[i] = w_tendão · quota(i)` (a partição de
    /// [`bend::share`], que soma `1` sobre os sub-ossos de um osso), e uma [`Especie::Soma`]
    /// anterior soma `delta · bump · quota(i)` — logo o vector continua **proporcional à quota** e
    /// reescrevê-lo como `novo · quota(i)` não perde informação nenhuma.
    fn fixa(&self, p: [f64; 2], w: &mut [f64], tendao: u32, valor: f64, bump: f64) -> bool {
        let mut do_tendao = 0.0;
        let mut outros = 0.0;
        let mut existe = false;
        for (i, b) in self.bones.iter().enumerate() {
            if b.tendon == tendao {
                do_tendao += w[i];
                existe = true;
            } else {
                outros += w[i];
            }
        }
        if !existe {
            return false;
        }
        let alvo = if outros > 0.0 {
            valor.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let novo = (alvo - do_tendao).mul_add(bump, do_tendao);
        let escala = if outros > 0.0 {
            (1.0 - novo) / outros
        } else {
            1.0
        };
        for (i, b) in self.bones.iter().enumerate() {
            if b.tendon == tendao {
                w[i] = novo * self.quota(b, p);
            } else {
                w[i] *= escala;
            }
        }
        true
    }

    /// A fracção deste sub-osso no osso autorado a que ele pertence — `1.0` ao bit num osso recto.
    ///
    /// ⚠️ **Uma função e não três cópias:** a mesma conta vive no [`Skin::weights_at`], no
    /// [`Skin::weights_from`] e agora na correcção, e três cópias divergiriam no dia em que a
    /// repartição mudasse — com o sintoma a ser um osso curvo a corrigir-se de outra maneira do
    /// que se pesa.
    fn quota(&self, b: &SkinBone, p: [f64; 2]) -> f64 {
        if b.sub.1 <= 1 {
            return 1.0;
        }
        let (u, _) = project_to_segment(p, b.rest_a, b.rest_b);
        bend::share(b.sub.0, b.sub.1, u)
    }
}
