//! **O ESTILO RESOLVIDO** — o que uma forma bindada a um token desenha NESTE modo.
//!
//! ⚠️ **O tipo chamava-se `BoundPaint` e foi renomeado na W4c.4**, quando a ESPESSURA entrou: uma
//! largura não é tinta, e um `BoundPaint { width }` seria um nome a mentir em todo sítio de uso.
//!
//! É a costura *fonte autorada ≠ o que o mundo consome* do ADR-0121, agora na TINTA em vez da
//! geometria: o documento guarda o literal, e o que se desenha é derivado. Daí a forma da resposta
//! ser a MESMA — [`crate::VecPath::painted`] devolve um [`Cow`], `Borrowed` quando não há binding,
//! e foi isso que permitiu ligá-la no ponto único de desenho sem mudar um byte do que já existe.
//!
//! # Quem resolve, e por quê não é aqui
//!
//! Esta crate é o MODELO DE DOCUMENTO: ela não conhece `ph2d-tokens`, não conhece tema, e não
//! conhece o ECS onde os bindings moram. Quem resolve é a shell, uma vez por frame, e publica o
//! resultado no [`crate::VecViewState`] — exatamente como já publica *quem está escondido* e *quem
//! recorta*. O renderer consome tinta pronta.
//!
//! ⚠️ **Um número, três consumidores, UMA porta:** a shell (para desenhar), o painel (para a
//! swatch mostrar a cor do token) e o gate perguntam todos à `ph2d_tokens::ColorToken::from_key` +
//! `resolve(theme)`. Uma segunda tabela em qualquer um deles é a swatch que mostra uma cor e a arte
//! que desenha outra — divergência que só aparece num screenshot.

use serde::{Deserialize, Serialize};

use crate::{Paint, Rgba8, VecPath, VecPathId};

/// A tinta que os tokens desta forma produzem no modo vigente.
///
/// `None` num campo = aquela propriedade **não** está bindada e o literal do documento vale.
// ⚠️ **Sem `Eq`, e a ausência é o campo `width`**: um comprimento é `f64`, e `f64` não é `Eq`
// (`NaN != NaN`). Ninguém compara duas entradas destas por igualdade total — o consumidor pergunta
// campo a campo —, então o que se perde é nada e o que se ganharia seria uma promessa falsa.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundStyle {
    /// A forma a que isto se refere.
    pub path: VecPathId,
    /// Cor do preenchimento vinda de um token.
    pub fill: Option<Rgba8>,
    /// Cor do traço vinda de um token.
    pub stroke: Option<Rgba8>,
    /// **A opacidade VIVA desta forma neste frame**, `255` = opaca (plano UI/UX W8b.3).
    ///
    /// ⚠️ E ela é **VISTA, nunca documento**: quem a produz é o valor vivo de um controle (um
    /// estado de UI, uma curva da linha do tempo), e o autorado fica onde o artista o escreveu. É
    /// a costura *fonte ≠ o que o mundo consome* do ADR-0121, aqui na opacidade — arrastar até
    /// zero e voltar devolve exatamente a arte.
    ///
    /// ⭐⭐⭐ **Ela SOBREPÕE a [`crate::VecPath::opacity`], e isso mudou em 2026-09-05.** Até à v19
    /// do schema não havia opacidade de objecto nenhuma, então este campo **escalava o alfa de
    /// toda a tinta** — a única forma de desvanecer que existia. Com o objecto a ter a sua, os
    /// quatro campos desta struct passam a ser **a mesma lei**: *`None` = o literal do documento
    /// vale; `Some` = o valor vivo cobre-o*. A composição tem uma porta só,
    /// [`object_alpha`], e o desenho aplica-a como CAMADA.
    ///
    /// ⚠️ **A mudança é observável, e é uma correcção:** escalar a tinta faz o traço transparecer
    /// através do próprio preenchimento a meia-opacidade (duas marcas, cada uma a metade); a
    /// camada compõe a forma inteira uma vez. É o que o Illustrator e o Figma fazem, e é o que a
    /// palavra *opacidade do objecto* quer dizer.
    pub alpha: Option<u8>,
    /// **A ESPESSURA que um token de escala dá ao traço**, em unidades de MUNDO (W4c.4).
    ///
    /// ⚠️ Ela chega aqui **já convertida**: o token fala pixels, o documento fala mundo, e quem
    /// cruza a fronteira é a régua do projeto (`ProjectSettings::pixels_per_meter`), na shell.
    /// Esta crate é o modelo de documento e não conhece régua nenhuma — como já não conhece tema.
    ///
    /// ⚠️ E ela **pinta o traço que existe**, nunca cria um: é a mesma lei que a cor do traço já
    /// segue (ver [`VecPath::painted`]), e pela mesma razão — um traço precisa também de COR, e
    /// inventá-la seria escolher por um artista que não escolheu.
    pub width: Option<f64>,
}

impl BoundStyle {
    /// Nada resolvido ⇒ esta entrada não muda desenho nenhum.
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.fill.is_none() && self.stroke.is_none() && self.alpha.is_none() && self.width.is_none()
    }
}

impl VecPath {
    /// **A forma como ela DESENHA neste modo** — o literal, ou o token que o cobre.
    ///
    /// ⚠️ Sem binding devolve [`Cow::Borrowed`]: o mesmo ponteiro, zero cópia, e o desenho é
    /// byte-idêntico ao mundo pré-token. É a propriedade que torna seguro chamar isto no caminho
    /// quente de TODA forma da cena, e é a mesma que o [`VecPath::cooked`](crate::VecPath) usa
    /// para o raio de quina vivo.
    ///
    /// ⚠️ **O preenchimento e o traço não se comportam igual, e o motivo é geometria e não
    /// gosto:** um `Paint::Solid` descreve um preenchimento por INTEIRO, então bindar o
    /// preenchimento de uma forma sem preenchimento é autoria completa e ele nasce. Um traço
    /// precisa também de LARGURA — colori-lo numa forma sem traço obrigaria a inventar um número
    /// que o artista não escreveu —, então o token do traço **pinta o traço que existe** e não
    /// cria nenhum. Bindar sem ver mudança nenhuma seria pior do que a row não estar lá; é por
    /// isso que o painel só oferece a row do traço quando há traço.
    ///
    /// ⚠️ **A ESPESSURA segue a MESMA lei, e pela mesma metade que falta:** um token de largura
    /// numa forma sem traço teria de inventar a COR. Então ela engrossa o traço que existe, e a
    /// row dela é oferecida sob a mesma condição.
    #[must_use]
    pub fn painted<'a>(&'a self, bound: Option<&BoundStyle>) -> std::borrow::Cow<'a, VecPath> {
        let Some(b) = bound.filter(|b| !b.is_noop()) else {
            return std::borrow::Cow::Borrowed(self);
        };
        // Um traço bindado numa forma SEM traço não tem o que colorir — e se ele fosse o único
        // binding, clonar aqui seria uma cópia que não muda um pixel.
        let paints_stroke = b.stroke.is_some() && self.stroke.is_some();
        // ⚠️ A ESPESSURA entra no early-out pelo MESMO teste, e não por um `is_some()` solto: um
        // token de largura numa forma sem traço não tem o que engrossar, exactamente como a cor.
        let widens = b
            .width
            .zip(self.stroke.as_ref())
            .is_some_and(|(w, s)| (s.width - w).abs() > f64::EPSILON);
        // ⛔ **A OPACIDADE NÃO ENTRA AQUI desde 2026-09-05** (v19 do schema). Ela deixou de ser
        // um escalar da tinta e passou a ser a opacidade do OBJECTO, aplicada como CAMADA por
        // quem desenha ([`object_alpha`]) — ver o doc do campo `alpha`. ⭐ E o efeito colateral é
        // uma economia: a chave do memo de FX é feita desta forma pintada, então desvanecer uma
        // forma filtrada **acerta** o memo em vez de a re-cozinhar 60 vezes por segundo (que é o
        // que a wave de 2026-09-04 teve de fazer enquanto a opacidade vivia nas cores).
        if b.fill.is_none() && !paints_stroke && !widens {
            return std::borrow::Cow::Borrowed(self);
        }
        let mut out = self.clone();
        if let Some(c) = b.fill {
            out.fill = Some(Paint::Solid(c));
        }
        // ⚠️ Um token de cor no traço **substitui a tinta por uma cor** — a mesma lei que a linha
        // do preenchimento acima já obedece (ela troca o `Paint` inteiro por um `Solid`). Pintar só
        // a `fallback` de um padrão seria escolher uma cor que ninguém vê.
        if let (Some(c), Some(s)) = (b.stroke, out.stroke.as_mut()) {
            s.paint = crate::StrokePaint::Solid(c);
        }
        if let (Some(w), Some(s)) = (b.width, out.stroke.as_mut()) {
            s.width = w;
        }
        std::borrow::Cow::Owned(out)
    }
}

/// Multiplica o alfa de TODA tinta da forma por `a/255` — preenchimento (seja qual for a espécie)
/// e traço.
///
/// ⚠️ **A conta é `(alfa * a + 127) / 255`, não `alfa * a / 255`** — arredondamento ao mais
/// próximo, e não truncamento. ⚠️ E a razão **não** é a identidade: `255 * 255` é divisível por
/// 255, então opaco continua opaco nas duas contas (uma primeira versão deste comentário afirmava
/// o contrário, e a mutação que tirou o `+127` sobreviveu a todos os gates exactamente por isso).
/// A razão é o VIÉS: truncar erra sempre para BAIXO, e uma cadeia de desvanecimentos escureceria
/// meio nível de cada vez. O gate mede onde o erro se vê — `100 * 130/255` é 50,98, que arredonda
/// para 51 e trunca para 50.
///
/// ⭐⭐ **É `pub(crate)` desde a W6 porque ela tinha um SEGUNDO consumidor:** o
/// [`crate::brush_stroke::brush_copies`] desvanece a arte de um pincel com esta função. *Desvanecer
/// toda a tinta de um `VecPath` é uma pergunta só, e uma segunda cópia dela divergiria na primeira
/// espécie de tinta nova* — foi a razão de o `ArcPath` existir, um nível abaixo.
///
/// ⚠️ **Hoje o pincel é o ÚNICO** — a opacidade viva saiu daqui em 2026-09-05 e virou uma camada
/// ([`object_alpha`]). A função fica porque a pergunta do pincel não mudou: ali o que desvanece é
/// a ARTE de um traço, que é tinta de verdade, e não a composição de um objecto.
pub(crate) fn fade(p: &mut VecPath, a: u8) {
    let scale = |c: &mut Rgba8| c.a = ((u32::from(c.a) * u32::from(a) + 127) / 255) as u8;
    match p.fill.as_mut() {
        Some(Paint::Solid(c)) => scale(c),
        Some(Paint::Linear { stops, .. } | Paint::Radial { stops, .. }) => {
            for s in stops {
                scale(&mut s.color);
            }
        }
        Some(Paint::MultiPoint { points }) => {
            for pt in points {
                scale(&mut pt.color);
            }
        }
        // ⚠️ **Um padrão não tem cor para escalar — tem OPACIDADE.** Desvanecer aqui a `fallback`
        // sozinha faria a forma manter o ladrilho a cheio e clarear só o instante em que ele ainda
        // não resolveu, que é o contrário do que se vê. As duas descem juntas.
        Some(Paint::Pattern(pat)) => {
            pat.alpha = (pat.alpha * f32::from(a) / 255.0).clamp(0.0, 1.0);
            scale(&mut pat.fallback);
        }
        None => {}
    }
    // ⚠️ **O traço desvanece pela MESMA lei do preenchimento** (plano 35): um padrão no traço não
    // tem cor para escalar — tem OPACIDADE, e as duas descem juntas.
    match p.stroke.as_mut().map(|s| &mut s.paint) {
        Some(crate::StrokePaint::Solid(c)) => scale(c),
        Some(crate::StrokePaint::Pattern(pat)) => {
            pat.alpha = (pat.alpha * f32::from(a) / 255.0).clamp(0.0, 1.0);
            scale(&mut pat.fallback);
        }
        // ⭐⭐⭐ **UM PINCEL desvanece pela cor de recurso, e as CÓPIAS SEGUEM-NA** (plano 36, W6).
        //
        // ⚠️ **A linha não mudou; o que mudou foi quem a lê.** Até 2026-08-30 esta linha era uma
        // **dívida declarada** — *"as cópias são `VecPath` com a tinta delas, e o desvanecimento
        // delas mora em quem as desenha"*. Hoje `brush_copies` lê `fallback.a` como a opacidade do
        // pincel, então escalar a `fallback` **é** escalar as cópias. *Quando a cura de um buraco
        // apaga o comentário que o declarava sem lhe tocar, o desenho está no sítio certo.*
        //
        // ⚠️ E é por isso que um pincel **não** ganhou um `alpha` próprio como o padrão: ali o
        // campo existe porque o amostrador do Vello quer um `f32`; aqui as cópias são geometria
        // `Rgba8`, e um segundo número seria uma segunda casa para a mesma opacidade.
        Some(crate::StrokePaint::Brush(b)) => scale(&mut b.fallback),
        None => {}
    }
}

/// ⭐⭐⭐ **A OPACIDADE DO OBJECTO, do documento** (v19 do schema).
///
/// Newtype por causa do `Default`: a [`VecPath`] deriva-o, `..VecPath::default()` é o idioma de
/// centenas de sítios deste repo, e um `f32` cru nasceria a `0.0` — *toda forma invisível, sem uma
/// linha de erro*. O tipo carrega o neutro **e** a cerca (0..1), num sítio só.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Opacity(f32);

impl Default for Opacity {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Opacity {
    /// A opacidade, presa a `0..=1`.
    ///
    /// ⚠️ **`NaN` vira OPACO, não `NaN`.** Um `f32::clamp` com `NaN` devolve `NaN` (não estoura), e
    /// um `NaN` a chegar ao `push_layer` do Vello é uma camada com alfa indefinido — o modo de
    /// falha silencioso que este tipo existe para não ter. O neutro é a resposta segura: quem
    /// entrega `NaN` já perdeu o valor, e desaparecer a forma seria pior do que a ignorar.
    #[must_use]
    pub fn new(v: f32) -> Self {
        Self(if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            1.0
        })
    }

    /// O número, `1.0` = opaca.
    #[must_use]
    pub fn get(self) -> f32 {
        self.0
    }

    /// **Esta forma compõe-se como sempre?** — o caminho rápido do desenho: sem camada, byte a
    /// byte o que se desenhava antes de a v19 existir.
    #[must_use]
    pub fn is_opaque(self) -> bool {
        self.0 >= 1.0
    }
}

/// ⭐⭐⭐ **QUÃO OPACO este objecto está NESTE quadro** — a porta ÚNICA que compõe o autorado com o
/// vivo.
///
/// A lei é a dos outros três campos do [`BoundStyle`], e ela é uma só: *`None` = o literal do
/// documento vale; `Some` = o valor vivo cobre-o*. Um estado de UI ou uma curva da linha do tempo
/// **sobrepõem** a opacidade que o artista escreveu, sem lhe tocar — largar o controlo devolve a
/// arte autorada.
///
/// ⚠️ **Uma segunda porta era o defeito:** enquanto a opacidade viva escalava a tinta e a autorada
/// não existia, «quão transparente é esta forma» tinha duas respostas possíveis e nenhum sítio
/// onde elas se encontrassem. Quem desenha, quem mede o custo e quem publica o número no painel
/// perguntam todos aqui.
#[must_use]
pub fn object_alpha(path: &VecPath, bound: Option<&BoundStyle>) -> f32 {
    bound
        .and_then(|b| b.alpha)
        .map_or(path.opacity.get(), |a| f32::from(a) / f32::from(u8::MAX))
}

#[cfg(test)]
#[path = "paint_bind_tests.rs"]
mod tests;
