//! ⭐⭐⭐ **A FORMA ACESA COMO MATÉRIA** — o laço por texel que acende um objecto 2D com a lei do
//! OpenPBR, a partir das normais que uma malha 3D lhe doou.
//!
//! # Porque esta crate existe, com o número
//!
//! A rota A do [`02.2`](../../../docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md) está
//! construída: uma malha 3D assa-se num sprite e deixa lá `base` (o albedo), `form` (as normais) e
//! `form_occ` (a oclusão), e o objecto fica **re-iluminável**. ⛔ **Mas quem o acende é o
//! `ImpastoLightPass`** — o passe da TINTA do Painter, emprestado através de um adaptador que
//! *neutraliza* os planos dele (`baked_form::neutral_planes`).
//!
//! Medido no [`impasto_light.wgsl`](../../ph2d-render/src/shaders/impasto_light.wgsl), esse passe
//! é um modelo de tinta: difuso envolvido mais um especular lido de uma tabela, **sem GGX e sem
//! conservação de energia**. Ao lado, o modelador acende com o OpenPBR inteiro.
//!
//! ⇒ *o objecto 3D do jogo é aceso pela lei errada, e é essa a razão de ele não parecer o alvo.*
//!
//! # ⛔⛔ Porque NÃO se muda o passe que já existe
//!
//! O cabeçalho do `impasto_light.wgsl` declara duas propriedades **load-bearing**, e as duas são
//! sobre TINTA: *«tinta plana é byte-idêntica»* (o sombreamento é RELATIVO — divide pelo que uma
//! superfície plana do mesmo material devolve) e *«papel nu recebe exactamente nada»*. Mudar a lei
//! ali parte o Painter.
//!
//! ⇒ **passe NOVO, escolhido por objecto.** O antigo fica, porque é a lei certa para a tinta.
//!
//! # ⚠️ A diferença que o artista VAI ver, e ela é deliberada
//!
//! O passe da tinta é **relativo**; este é **absoluto**, como manda a física — é o que o
//! [`01` §1](../../../docs/Render3d/01_o_alvo_decomposto.md) chama de *«um pipeline fisicamente
//! correcto, com a direcção de arte a mentir por cima DE PROPÓSITO»*. Um objecto aceso por aqui
//! responde à luz como matéria: escurece onde a luz não chega e tem destaque especular com a forma
//! do GGX, em vez de um brilho tabelado.
//!
//! ⛔ **É por isso que a escolha é POR OBJECTO e não global**: um projecto gravado tem de continuar
//! a abrir com a aparência com que foi gravado (`baked_form` guarda o `rig` exactamente por essa
//! razão), e trocar a lei por baixo mudaria a arte em silêncio.
//!
//! # A vista é CONSTANTE, e isso não é uma simplificação
//!
//! Num objecto 2D o observador olha o canvas de frente: `v = (0, 0, 1)`. ⚠️ Não é uma aproximação
//! — é o que a projecção do canvas É. Uma vista por texel só faria sentido com câmera perspectiva,
//! e a engine é 2D **por desenho** ([`14` §5](../../../docs/Render3d/14_a_ordem_de_superar.md)).

#![forbid(unsafe_code)]

pub use ph2d_material::{OpenPbr, Rgb, Surface};

pub mod imagem;
pub mod wgsl;

#[cfg(test)]
mod tests;

/// **Uma lâmpada, já resolvida** — direcção e radiância, nada mais.
///
/// ⚠️ **Ela não é a `ph2d_light::Lamp`, e a diferença é a razão de esta crate ser folha.** Aquela
/// carrega o vocabulário do rig do artista (o `half` do modelo de tinta, o `tint` antes da
/// intensidade); esta carrega o que a **óptica** precisa. Quem converte é o chamador, que já tem o
/// rig — e é lá que a conversão tem de morar, porque ela é uma decisão sobre o RIG.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lampada {
    /// Da superfície PARA a luz, **normalizada**. ⚠️ O sentido é o que a
    /// [`Surface::direct`] espera; invertê-lo apaga a peça em vez de a acender.
    pub para_a_luz: [f32; 3],
    /// A radiância que ela entrega, já com intensidade e cor.
    pub radiancia: Rgb,
}

/// **O que um texel traz** — o que a forma doou, num ponto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texel {
    /// A normal doada pela malha, **em espaço de CANVAS** (`y` cresce para BAIXO — o mesmo em que o
    /// rig é autorado). ⚠️ Pode vir **não normalizada** do `baked_form` (ela viaja quantizada em
    /// `rgba8`), e [`acende_texel`] normaliza-a — ver lá porque isso não é defensivo.
    ///
    /// ⛔ **Esta linha dizia «espaço de vista» e estava errada**, com duas fixturas sintéticas a
    /// segui-la: quem escreve o canal é o `canvas_normal` do `ph2d-mesh-render`, cuja última linha
    /// é `vec3(n.x, -n.y, n.z)`. O referencial decide **de que lado o céu está**.
    pub normal: [f32; 3],
    /// O albedo — os pixels **antes** da luz.
    pub albedo: Rgb,
    /// Quanto deste texel é forma (`0` = nada, `1` = forma cheia). Fora da silhueta a lei é um
    /// **no-op exacto**, que é a metade do contrato que o `baked_form` já cumpre com a tinta.
    pub cobertura: f32,
    /// A oclusão que a forma doou (`1` = aberto, `0` = fechado).
    pub oclusao: f32,
}

/// A vista de um canvas 2D. Ver o cabeçalho do módulo.
pub const VISTA: [f32; 3] = [0.0, 0.0, 1.0];

/// ⭐⭐⭐ **O CÉU, na forma que esta lei lê: uma RAMPA LINEAR na altura da TELA.**
///
/// `E(n) = base + inclinação · (−n.y)` — a irradiância normalizada (`E/π`, *a luz que uma difusa
/// branca devolveria*), na direcção da normal, com a normal no referencial do [`Texel::normal`]
/// (CANVAS, `y` para BAIXO ⇒ o topo da tela é `−y`).
///
/// # ⚠️ Porque é uma RAMPA e não um fecho nem o `Environment` inteiro
///
/// **Uma rampa porque é isso que o céu desta casa É**: a irradiância de um ambiente linear na altura
/// vale `c + (2/3)·k·(n·cima)`, e o `2/3` é o `Â₁` da convolução zonal com o lóbulo cosseno
/// (Ramamoorthi & Hanrahan 2001). *Um ambiente linear não tem termo de grau 2*, logo dois `Rgb`
/// **são** a resposta exacta — não uma amostragem dela. Guardá-la como DADOS é o que a deixa viajar
/// para o dispositivo sem uma segunda redacção da fórmula.
///
/// ⭐⭐⭐ **E ELE *É* O [`ph2d_material::Environment`] desde 2026-09-20 — a premissa da redacção
/// anterior MORREU, e o dia que ela previa foi este.** Ela dizia: *«não é o `Environment` porque
/// aquele trait tem uma segunda metade (`radiance`, a espelhada pré-filtrada) que só as closures
/// INDIRECTAS leem — e esta lei não as chama»*, e acabava com *«no dia em que a indirecta do OpenPBR
/// entrar (a coluna B3), as duas metades chegam juntas»*. Chegaram.
///
/// ⭐⭐ **E as DUAS metades saem dos MESMOS dois `Rgb`, sem um dado novo** — é a mesma álgebra da
/// rampa, lida duas vezes:
///
/// ```text
/// irradiance(n)        = base + inclinação · up(n)                    (o lóbulo COSSENO)
/// radiance(dir, α)     = base + 1,5 · inclinação · c(α) · up(dir)     (o lóbulo GGX)
/// ```
///
/// O `1,5` desfaz o `Â₁ = 2/3` que o [`Ceu::inclinacao`] carrega (a irradiância já vem convolvida
/// com o cosseno) e o `c(α)` é a [`ph2d_material::lobe_shrink`] — o coeficiente de grau `1` do
/// núcleo do pré-filtro GGX. *Um ambiente linear não tem termo de grau 2, logo isto é a resposta
/// EXACTA às duas perguntas e não uma amostragem de nenhuma delas.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ceu {
    /// A irradiância na horizontal (`n·cima = 0`).
    pub base: Rgb,
    /// Quanto ela sobe do horizonte para o topo da TELA.
    pub inclinacao: Rgb,
}

impl Ceu {
    /// **O céu que não existe** — a lei sem ambiente nenhum.
    ///
    /// ⚠️ Ele é uma CONSTANTE com nome e não um `Ceu { base: [0.0; 3], … }` escrito em cada gate: *o
    /// valor de fábrica da ausência tem de ser o mesmo em todos os sítios que a afirmam*, e é ele
    /// que torna «byte-idêntico ao que se ship antes do céu» uma frase verificável.
    pub const PRETO: Self = Self {
        base: [0.0; 3],
        inclinacao: [0.0; 3],
    };

    /// Um céu **sem direcção** — o mesmo em toda normal.
    ///
    /// ⚠️ Ele existe para os gates da composição, que medem *onde o ambiente entra na conta* e não
    /// de que lado ele vem. ⛔ O produto **não** o usa: um ambiente sem direcção deixa duas faces
    /// na sombra — uma a olhar para cima e outra para baixo — exactamente iguais, e ali a peça fica
    /// sem leitura de forma nenhuma.
    #[must_use]
    pub const fn chapado(c: Rgb) -> Self {
        Self {
            base: c,
            inclinacao: [0.0; 3],
        }
    }

    /// A irradiância normalizada na direcção de `n`. Ver o doc do tipo.
    ///
    /// ⚠️ **`mul_add` e não `a + b * c`**, e é para a paridade: o gémeo em WGSL pede `fma`, que tem
    /// **um** arredondamento. Escrito solto, a igualdade ao bit passaria a depender de o compilador
    /// do WGSL contrair a multiplicação-soma — e esta casa já mediu uma placa a contraí-la onde o
    /// fonte não o pedia.
    #[must_use]
    pub fn irradiancia(&self, n: Rgb) -> Rgb {
        let up = -n[1];
        [
            self.inclinacao[0].mul_add(up, self.base[0]),
            self.inclinacao[1].mul_add(up, self.base[1]),
            self.inclinacao[2].mul_add(up, self.base[2]),
        ]
    }

    /// ⭐⭐⭐ **A RADIÂNCIA pré-filtrada** — o céu que a espelhada vê, para um lóbulo já encolhido.
    ///
    /// ⚠️ **Ela recebe o ENCOLHIMENTO e não o `α`**, e a razão é o gémeo: no dispositivo o `c(α)`
    /// viaja **pronto** dentro do material empacotado ([`ph2d_material::wgsl::EnvLobe`]), porque ele
    /// é constante por MATERIAL e correr um logaritmo por pixel seria pôr a mesma conta a dar o
    /// mesmo número um milhão de vezes. ⇒ a porta que os dois motores partilham fala a língua do
    /// mais apertado dos dois; quem tem um `α` na mão passa por [`ph2d_material::Environment`], que
    /// é o adaptador logo abaixo.
    ///
    /// ⚠️ **`1,5` é `1/(2/3)`, não um ganho**: o [`Ceu::inclinacao`] é a inclinação da
    /// IRRADIÂNCIA, que já traz o `Â₁` do lóbulo cosseno lá dentro; a radiância quer a rampa crua.
    ///
    /// ⚠️ **A associação é a do gémeo, e não é livre**: `(1,5 · inclinação)` primeiro, e só depois o
    /// `fma` com o `up` — escrita solta, a igualdade ao bit passaria a depender de o compilador do
    /// WGSL contrair a multiplicação-soma.
    #[must_use]
    pub fn radiancia(&self, dir: Rgb, shrink: f32) -> Rgb {
        /// O `Â₁` do lóbulo cosseno, desfeito. Ver o doc.
        const CRU: f32 = 1.5;
        let up = shrink * -dir[1];
        [
            (CRU * self.inclinacao[0]).mul_add(up, self.base[0]),
            (CRU * self.inclinacao[1]).mul_add(up, self.base[1]),
            (CRU * self.inclinacao[2]).mul_add(up, self.base[2]),
        ]
    }
}

/// ⭐⭐⭐ **O céu desta lei É um [`ph2d_material::Environment`]** — e é isso que deixa a
/// [`ph2d_material::Surface::indirect`] entrar sem uma linha de óptica escrita aqui.
///
/// ⚠️ **O `α → c(α)` mora AQUI e não na [`Ceu::radiancia`]**, porque é a costura entre duas
/// convenções: o trait fala em rugosidade (é o que uma closure tem na mão) e o gémeo fala em
/// encolhimento (é o que cabe num uniform). *Pôr o logaritmo do lado da porta partilhada poria a CPU
/// a correr uma conta que o dispositivo não corre, e a paridade teria de a desfazer.*
impl ph2d_material::Environment for Ceu {
    fn radiance(&self, dir: Rgb, alpha: f32) -> Rgb {
        self.radiancia(dir, ph2d_material::lobe_shrink(alpha))
    }

    fn irradiance(&self, n: Rgb) -> Rgb {
        self.irradiancia(n)
    }
}

/// ⭐⭐⭐ **A LEI, num texel.** Ver o cabeçalho do módulo.
///
/// # ⚠️ Porque a normal é normalizada AQUI
///
/// Não é uma guarda defensiva: o `baked_form` guarda a forma em `rgba8` (medido — em `f32` custaria
/// `4×` o disco), e a viagem por oito bits **encurta o vector**. Uma normal de comprimento `0,97`
/// num `N·L` dá uma peça `3 %` mais escura em todo lado, e isso lê-se como *«o material está
/// errado»*. ⛔ Normalizar no chamador poria a mesma conta em cada consumidor novo.
///
/// # ⚠️ A COBERTURA mistura no fim, e a ordem importa
///
/// Ela mistura entre o albedo cru e o albedo aceso — **nunca entre preto e o aceso**. Misturar com
/// preto escureceria a borda da silhueta a cada re-acendida, que é exactamente o defeito que o
/// `baked_form` guarda `base` para impedir (*«re-acender a partir do que já está aceso compõe»*).
#[must_use]
pub fn acende_texel(
    s: &Surface,
    t: &Texel,
    lampadas: &[Lampada],
    ceu: Ceu,
    olhar: ph2d_view_transform::Look,
) -> Rgb {
    let n = normaliza(t.normal);
    // ⚠️ Uma normal degenerada (o texel fora da silhueta, onde a forma não escreveu nada) devolve o
    // albedo **cru**. ⛔ Devolver preto pintaria um halo no contorno de toda peça assada.
    let Some(n) = n else {
        return t.albedo;
    };

    // ⭐⭐⭐ **A COR DO TEXEL É O `base_color`, e não um factor no fim da conta.**
    //
    // ⛔⛔ A 1.ª redacção desta lei multiplicava a resposta pelo albedo depois, e isso está errado
    // com número: o `compose` **não é linear no `base_color`** — só o lóbulo difuso escala com ele,
    // o especular não escala nada. Medido no destaque de um dieléctrico de omissão, um texel
    // `[0,80 0,10 0,10]` saía com razão `R/B = 8,00` (destaque VERMELHO, que é o que um METAL faz)
    // contra `1,50` da lei. *Um plástico vermelho tem destaque branco.*
    //
    // ⚠️ A porta é da `ph2d-material`, onde a óptica mora, e é a irmã do `at_curvature`: uma
    // grandeza do PIXEL numa struct que se diz «por material». Sem verniz ela é uma troca de
    // campo — e há gate LÁ a prová-lo byte-idêntico a um `prepare()` inteiro, com e sem verniz.
    let s = s.at_base_color(t.albedo);

    let mut luz: Rgb = [0.0; 3];
    for l in lampadas {
        // ⛔⛔⛔ **A LÂMPADA ANTI-PARALELA À VISTA, e ela só é alcançável porque a vista é
        // CONSTANTE.** Medido: com `to_light = -VISTA` a lei devolve `[NaN, NaN, NaN]`, porque o
        // meio-vector `v + to_light` é o vector nulo e normalizá-lo não tem resposta.
        //
        // ⚠️ **No modelador isto tem medida nula** — ali a vista é a do raio e varia por pixel,
        // logo nenhuma lâmpada é anti-paralela em mais do que um ponto. **Num canvas 2D a vista é
        // `(0,0,1)` em TODO texel**, e uma lâmpada apontada de frente para trás cai exactamente
        // aqui, na peça inteira. *A constante que simplifica o 2D é a mesma que torna um caso
        // degenerado alcançável.*
        //
        // ⭐ E o limite é ZERO, não um valor escolhido: a `1e-3` de distância do caso a lei já lê
        // `2,6e-9` (medido). Devolver zero é continuar a curva, não inventar um número.
        //
        // ⛔ **A cura mora AQUI e não na [`ph2d_material`]**: aquela crate é o porte fiel do
        // MaterialX e o caso é do nosso CONSUMIDOR, não da óptica.
        if meio_vector_degenera(l.para_a_luz) {
            continue;
        }
        let r = s.direct(n, VISTA, l.para_a_luz, l.radiancia);
        luz = [luz[0] + r[0], luz[1] + r[1], luz[2] + r[2]];
    }

    // ⚠️ **A oclusão só pesa o AMBIENTE**, nunca a luz directa. Uma lâmpada que o artista apontou
    // tem de chegar onde ele a apontou; escurecer a directa com oclusão de forma é o que faz um
    // objecto parecer sujo em vez de ocluído — e nenhuma das cinco referências o faz.
    //
    // ⭐⭐⭐ **E O AMBIENTE É A LEI desde 2026-09-20 — a coluna B3 do plano.** A redacção anterior
    // desta linha era `albedo × E(n)` e declarava-se, por escrito, *«DECLARADAMENTE nosso, não a
    // lei … trocá-lo pela lei é a coluna B3, e é uma feature»*. Trocado.
    //
    // ⚠️ **O que ela compra não é «mais preciso»: é uma METADE QUE NÃO EXISTIA.** O lambertiano
    // respondia só *quanta luz o céu entrega a uma DIFUSA*; a [`Surface::indirect`] responde também
    // *o que o céu REFLECTE* — o especular pré-filtrado, com Fresnel e a forma do GGX.
    //
    // ⛔⛔ **E o defeito de ontem não era a quantidade, era a CLOSURE.** Medido: sobre o mesmo
    // albedo e o mesmo céu, um barro e um metal polido recebiam o **MESMO** ambiente, ao bit —
    // porque `albedo × E(n)` é um lóbulo difuso, e no OpenPBR um metal não tem nenhum. *O ambiente
    // não sabia que material estava a iluminar.*
    //
    // ⚠️ **Zero linhas de óptica, outra vez:** o `[`Ceu`]` passou a ser um
    // [`ph2d_material::Environment`] e a composição inteira (Oren-Nayar + espelhada + verniz +
    // conservação de energia) é a do port. ⛔ O albedo do texel **não** volta a multiplicar nada
    // aqui: ele já entrou pelo `at_base_color`, que é a mesma lição que a directa pagou em cima.
    //
    // ⚠️ **A oclusão pesa o termo INTEIRO**, difusa e espelhada. ⛔ **Divergência DECLARADA:** o
    // Filament e o Frostbite derivam uma *specular occlusion* separada (Lagarde), função de `AO`,
    // `α` e `N·V`, porque um AO de hemisfério não descreve o cone estreito de um espelho. Não a
    // temos, e o efeito é uma fresta espelhar um pouco mais do que devia — *acrescentá-la é uma lei
    // com oráculo próprio, e escrevê-la aqui de cabeça seria inventar o que nenhuma referência desta
    // casa mediu.*
    //
    // ⭐⭐⭐ **Com o [`Ceu::PRETO`] esta linha continua a ser `luz + 0` ao bit**: as duas metades do
    // céu devolvem zero e toda closure indirecta multiplica a resposta do ambiente.
    let indirecta = s.indirect(n, VISTA, &ceu);
    let aceso = [
        luz[0] + indirecta[0] * t.oclusao,
        luz[1] + indirecta[1] * t.oclusao,
        luz[2] + indirecta[2] * t.oclusao,
    ];

    // ⭐⭐⭐ **A VISTA entra AQUI — no ACESO, e ANTES da mistura da cobertura.**
    //
    // ⛔⛔ A 1.ª redacção aplicava-a ao resultado JÁ misturado, no corredor, com o argumento de que
    // a lei da luz devolve radiância e o ecrã é outra pergunta. O argumento está certo e o SÍTIO
    // estava errado: fora da silhueta esta lei devolve o albedo CRU — que são os pixels 2D do
    // sprite —, e uma exposição aplicada por cima mudava a ARTE que a forma não tocou. *A promessa
    // do no-op só é verdade para a identidade, e uma promessa que só vale no valor de fábrica não
    // é uma promessa.*
    //
    // ⚠️ Com o [`Look::default`] (0 stops, `Standard`) ela é a identidade para toda luz dentro do
    // branco ⇒ o caminho de omissão fica byte-idêntico, e há gate.
    let aceso = olhar.apply(aceso);

    let c = t.cobertura.clamp(0.0, 1.0);
    let n_c = 1.0 - c;
    [
        t.albedo[0] * n_c + aceso[0] * c,
        t.albedo[1] * n_c + aceso[1] * c,
        t.albedo[2] * n_c + aceso[2] * c,
    ]
}

/// ⛔⛔ **O meio-vector degenera** — `v + to_light ≈ 0`, o caso em que a lei do OpenPBR devolve
/// `NaN` porque normalizar o vector nulo não tem resposta.
///
/// ⚠️ **Só é alcançável porque a [`VISTA`] é CONSTANTE**, logo basta olhar a lâmpada: no modelador a
/// vista é a do raio e varia por pixel, e este caso tem medida nula; aqui ele é uma configuração
/// que o artista escreve, e vale para a peça inteira de uma vez.
///
/// ⚠️ O limiar é o quadrado, como na [`normaliza`], e a `1e-3` de distância deste caso a lei já lê
/// `2,6e-9` (medido) — *devolver zero continua a curva em vez de inventar um número*.
fn meio_vector_degenera(para_a_luz: [f32; 3]) -> bool {
    let h = [
        VISTA[0] + para_a_luz[0],
        VISTA[1] + para_a_luz[1],
        VISTA[2] + para_a_luz[2],
    ];
    h[0] * h[0] + h[1] * h[1] + h[2] * h[2] < 1e-12
}

/// Normaliza, ou `None` se o vector não tem direcção.
///
/// ⚠️ O limiar é o quadrado do comprimento e não o comprimento: poupa a raiz no caso que sai, e
/// `1e-12` é o quadrado de `1e-6`, que é onde um `f32` deixa de ter dígitos para uma direcção.
fn normaliza(v: [f32; 3]) -> Option<[f32; 3]> {
    let q = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if q < 1e-12 {
        return None;
    }
    let inv = 1.0 / q.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}
