//! **A COSTURA DE LARGURA de uma coluna docada** — as medidas do chrome, o vocabulário dos lados,
//! e a geometria do agarre.
//!
//! Cortado do `layout.rs` em 2026-08-30 pelo tecto de LOC (777/700), e o corte é por
//! RESPONSABILIDADE: aquele ficheiro responde *«onde fica cada rect?»* e este responde *«como uma
//! coluna se redimensiona?»* — uma pergunta sobre um GESTO, e o único sítio deste módulo que
//! precisa de saber que existe um ponteiro.

use super::layout::{
    HERO_VIEWPORT_W, HIERARCHY_W, HeroLayout, INSPECTOR_W, TIMELINE_DOCK_H, TOPBAR_H,
};
use crate::zones::Rect;

/// **As MEDIDAS do chrome deste quadro** — larguras e alturas, nunca modos.
///
/// ⭐ *«Sem chrome legado»* é `rail_w = 0` e `top_bar_h = 0`; *«a coluna foi arrastada»* é um
/// `left_dock_w` diferente. O [`HeroLayout`] não conhece nenhuma dessas frases — ele recebe
/// números, e a aritmética dele é a mesma sempre.
///
/// ⚠️ **Uma struct e não seis argumentos:** o construtor já tinha cinco, e cada medida nova a mais
/// é uma posição a mais para trocar em silêncio — `for_viewport_bands(v, m, 57.0, 64.0, 308.0,
/// 304.0, …)` é uma linha que ninguém revê.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChromeBands {
    /// Largura do trilho de ferramentas. `0` = fora.
    pub rail_w: f32,
    /// Altura da barra de topo. `0` = fora.
    pub top_bar_h: f32,
    /// Largura da coluna da ESQUERDA.
    pub left_dock_w: f32,
    /// Largura da coluna da DIREITA.
    pub right_dock_w: f32,
    /// ⭐ Altura da **fila de ferramentas** por cima da área de desenho. `0` = fora.
    ///
    /// ⚠️ **Ela sai da ÁREA, não da janela** — ao contrário das outras quatro, que cortam a
    /// viewport. É a spec §4: a toolbar é uma REGIÃO da área, irmã da régua, e por isso vive entre
    /// as colunas em vez de atravessar o ecrã.
    pub tool_bar_h: f32,
    /// ⭐ Altura da **faixa do fundo** (o timeline, ou a tira do Flip por baixo dele).
    ///
    /// ⚠️ Ela é AUTORADA como as duas colunas (`WidgetStore::dock_bottom_h`) — o topo dela é uma
    /// costura, e quem partilha a banda segue. ⛔ Não é um interruptor: a faixa só é **desenhada**
    /// se o painel dela estiver visível, e essa pergunta é do `hero/paint.rs`.
    pub bottom_dock_h: f32,
}

impl ChromeBands {
    /// As medidas de fábrica — o trilho e a barra presentes, as colunas nas larguras dos tokens.
    pub const DEFAULT: Self = Self {
        rail_w: 57.0, // LITERAL-PX-OK: espelha `tool_rail_width_px()` no preset Small
        top_bar_h: TOPBAR_H,
        left_dock_w: HIERARCHY_W,
        right_dock_w: INSPECTOR_W,
        // ⚠️ **ZERO, e é o mockup que o pede**: o `DEFAULT` descreve a referência de desenho, que
        // tem o trilho VERTICAL. A fila horizontal é o chrome de produção, e quem a mede é o
        // `hero/paint.rs` — ela depende do preset de tamanho do chip, que é autorado.
        tool_bar_h: 0.0,
        bottom_dock_h: TIMELINE_DOCK_H,
    };

    /// ⭐⭐⭐ **A LARGURA DE FÁBRICA DE UMA COLUNA, NUMA JANELA** — a mesma decisão de desenho a
    /// custar a mesma FRACÇÃO em todo alvo.
    ///
    /// > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo
    /// > espaço.»*
    ///
    /// # ⛔⛔ O defeito que ela cura, em números
    ///
    /// As duas colunas são `308 + 304 = 612 px` **absolutos**, e o [`HERO_VIEWPORT_W`] diz contra
    /// que janela eles foram autorados. ⇒ elas custam `44,8 %` na referência, `51,3 %` no iPad 11"
    /// e **`54,0 %`** no iPad mini: *a mesma decisão custa `20 %` mais no aparelho mais pequeno, e
    /// nenhum documento dizia isso* (`medicoes/06 §1`).
    ///
    /// # ⭐ A fracção é DERIVADA, não escolhida
    ///
    /// Ela é `HIERARCHY_W / HERO_VIEWPORT_W` — dois tokens que já existem, um a dividir pela janela
    /// para que foi autorado. ⛔ Não há número novo nesta lei; há a decisão que já estava tomada,
    /// aplicada onde ela ainda não chegava. *`CLAUDE.md` §0.0: um limite legítimo diz de que
    /// recurso ele é.* O recurso aqui é a **largura da janela**, e a referência é o único sítio
    /// onde alguém escolheu quanto dela o chrome podia comer.
    ///
    /// # ⛔⛔ É um TECTO e nunca uma ESCALA, e a diferença tem número
    ///
    /// Escalar nos dois sentidos poria a coluna a **crescer** no ecrã grande: na janela de
    /// `1 930 px` da bancada do dono, `1930 × 308/1366 = 435 px` por coluna — **`870`** contra os
    /// `612` de hoje. *A cura tornaria o app pior exactamente onde ele é usado todos os dias.*
    /// ⇒ acima da referência ela devolve o token **ao bit**, e há gate a exigi-lo
    /// (`acima_da_referencia_a_lei_nao_toca_em_nada`).
    ///
    /// # ⚠️ O que ela NÃO toca, declarado
    ///
    /// A largura que o **artista arrastou** ([`crate::interaction::WidgetStore::dock_width_choice`]).
    /// Apertar uma escolha explícita seria o *«aceita e mente»* que o §0.0 proíbe — e o preço fica
    /// nomeado: uma escolha gravada num ecrã largo continua a valer o que vale num estreito.
    ///
    /// ⚠️ **E ela pára no mínimo do PAINEL** ([`ph2d_tokens::PANEL_MIN_W_PX`]): abaixo dele o
    /// cabeçalho e uma linha deixam de caber juntos, e uma coluna que não se sabe desenhar é pior
    /// do que uma coluna larga. Nos três tablets o mínimo **não** morde — se um dia morder, a
    /// fracção medida passa a ser a do clamp e não a da lei, e o gate diz isso em voz alta.
    #[must_use]
    pub fn default_dock_w(side: DockSide, janela_w: f32) -> f32 {
        let token = match side {
            DockSide::Left => HIERARCHY_W,
            DockSide::Right => INSPECTOR_W,
        };
        // ⭐⭐ **SEM RAMO NENHUM, e as três propriedades saem da aritmética:**
        //
        //  * **acima da referência é inerte AO BIT** — `1366.0 / 1366.0` é exactamente `1.0` em
        //    IEEE (um valor a dividir por si mesmo), e `token * 1.0` é o token;
        //  * **`NaN` cai do lado seguro** — `f32::min` devolve o OUTRO operando quando um é `NaN`,
        //    logo uma janela sem largura lê `1.0` e recebe o token. *Toda guarda escrita com `<`
        //    ou `>` é cega ao `NaN`, e aqui não há guarda nenhuma a ser cega.*
        //  * **uma janela degenerada (`0` ou negativa) recebe o mínimo do painel**, que é a única
        //    largura que um painel sabe desenhar.
        let escala = (janela_w / HERO_VIEWPORT_W).min(1.0);
        (token * escala).max(ph2d_tokens::PANEL_MIN_W_PX)
    }
}

impl ChromeBands {
    /// ⭐⭐⭐ **O QUE UM ARRASTO GRAVA — `None` quando ele aterra na largura de FÁBRICA.**
    ///
    /// # O defeito que ela cura, lido no readout do perfil do dono (2026-09-20)
    ///
    /// ```text
    /// [dock] janela=473  esq=220.0 (escolha -)  dir=220.0 (escolha 220)
    /// ```
    ///
    /// A `473 px` a lei **já** entrega o mínimo do painel nas duas colunas. Tocar na borda da
    /// direita ali gravava `220` **como escolha** ⇒ ⛔ *um gesto que não mudou um pixel no ecrã
    /// tirou aquela coluna da lei da fracção para sempre*, com `dock_w_right=220` no ficheiro de
    /// arrumação e sem outra saída além do *Reset Panel Layout*.
    ///
    /// # ⚠️ Não é desenho novo: é a lei que a casa JÁ declara, no caminho que a violava
    ///
    /// O doc da [`crate::interaction::WidgetStore::dock_width_choice`] diz, desde que existe:
    /// *«persistir o valor de `dock_width` escreveria o default como se fosse uma escolha — e no
    /// dia em que o default mudasse, toda arrumação gravada continuaria a prender a coluna no
    /// número velho»*. ⭐ **O default mudou** (passou a seguir a janela nesse mesmo dia), e o
    /// arrasto era exactamente quem o gravava como escolha.
    ///
    /// # ⛔ E ela mora AQUI e não no store, por duas razões
    ///
    /// Os números da decisão são os desta struct, e o `WidgetStore` é estado **autorado** que não
    /// conhece a janela (ver o doc do [`crate::interaction::WidgetStore::dock_width`]). ⚠️ A
    /// tentativa de a pôr lá fez a catraca do DAG `interaction → screens` subir de `18` para
    /// `20`, e a lei desta casa é **curar por movimento, nunca subir o número** — *a catraca
    /// apontou para onde a porta devia estar*.
    ///
    /// ⚠️ **A tolerância é MEIA UNIDADE porque o gesto é em PIXELS:** duas larguras que
    /// arredondam ao mesmo pixel são o mesmo pedido, e abaixo disso o artista não consegue pedir
    /// outra coisa. ⛔ Não é folga de conforto — é a resolução do que o dedo exprime.
    /// ⛔⛔ **E a comparação é DEPOIS do piso, não antes — o gate reprovou a 1.ª redacção.**
    ///
    /// Ela media a largura **CRUA** do gesto, e o store clampa ao escrever: arrastar a borda para
    /// lá do mínimo numa janela estreita dava `|80 − 220| = 140` ⇒ *«é uma escolha»*, e o que
    /// ficava gravado era `220` — **exactamente o caso do report**. *Uma lei que julga o pedido
    /// enquanto o consumidor guarda o pedido CLAMPADO julga um número que ninguém grava.*
    ///
    /// ⚠️⚠️ **O piso é o do STORE e já não é o token do painel — a premissa mudou em
    /// 2026-09-20.** Ele era `ph2d_tokens::PANEL_MIN_W_PX`, e com isso *arrastar a borda numa
    /// janela estreita não fazia nada*: ali a largura de FÁBRICA já é o mínimo, logo o gesto
    /// pedia um número que o piso devolvia ao ponto de partida (report do dono, *«permita que
    /// manualmente o usuário consiga estreitar o painel»*). Hoje o piso de uma ESCOLHA é mais
    /// baixo que o de FÁBRICA, e a medição inteira vive no doc do
    /// [`crate::interaction::WidgetStore::DOCK_W_MIN`].
    ///
    /// ⛔ **Continuam a não ser dois pisos:** é o mesmo número lido do mesmo sítio — este ficheiro
    /// lê-o do store —, e há gate a exigi-lo (`o_piso_desta_lei_e_o_piso_do_store`). ⭐ A direcção
    /// `screens → interaction` é a que DESCE no DAG da fundação (há sentinela a exigi-la), logo
    /// esta leitura custa **zero** à catraca `interaction → screens`, que é a dívida.
    #[must_use]
    pub fn escolha_de_um_arrasto(side: DockSide, w: f32, janela_w: f32) -> Option<f32> {
        let w = w.max(crate::interaction::WidgetStore::DOCK_W_MIN);
        // ⚠️ `>=` e não `>`: exactamente meio pixel ainda é o mesmo pixel pedido.
        ((w - Self::default_dock_w(side, janela_w)).abs() >= 0.5).then_some(w)
    }
}

/// **A faixa de agarre que redimensiona uma coluna** — a borda INTERIOR dela.
///
/// ⭐ Enio, 2026-08-30: *«os painéis devem ser redimensionáveis para esquerda e para direita e com
/// setas bidirecionais no cursor; os pontinhos de redimensionamento podem ser retirados. A borda
/// inteira serve para redimensionar»*.
///
/// ⚠️ **Ela vive DENTRO da coluna**, nos últimos [`DOCK_SEAM_PX`] px antes da área de desenho — e
/// não a cavalo da fronteira, como uma costura de divisória costuma ser. A razão é medida: desde
/// que a régua ficou **colada** à coluna, a faixa dela começa exactamente onde a coluna acaba, e
/// uma costura centrada roubaria metade do agarre à régua. *A borda é do painel.*
pub const DOCK_SEAM_PX: f32 = 6.0; // LITERAL-PX-OK: largura de agarre, irmã do GRAB_HALF_PX do 3D

/// Qual coluna, quando o ponteiro está sobre uma costura de largura.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DockSide {
    /// A coluna da esquerda.
    Left,
    /// A coluna da direita.
    Right,
}

impl HeroLayout {
    /// **As duas colunas laterais, ORDENADAS POR `x`** — `(esquerda, direita)`.
    ///
    /// ⚠️ **Existe para o `mirrored` não ser uma inversão escrita à mão em cada chamador.** Sob
    /// espelho a Hierarchy vai para a direita e o dock de *takeover* para a esquerda; pedir
    /// *«o rect da Hierarchy»* e chamar-lhe *«a coluna da esquerda»* é a forma exacta do erro
    /// que o compilador não vê. Aqui a resposta vem da **posição**, que é o que a pergunta
    /// significa.
    #[must_use]
    pub fn side_columns(&self) -> (Rect, Rect) {
        if self.hierarchy.x <= self.inspector.x {
            (self.hierarchy, self.inspector)
        } else {
            (self.inspector, self.hierarchy)
        }
    }

    /// **A faixa de agarre que redimensiona esta coluna** — os últimos [`DOCK_SEAM_PX`] px dela,
    /// do lado da área de desenho.
    ///
    /// ⚠️ Devolve um rect de largura zero quando a coluna está vazia (`w == 0`): sem painel não há
    /// borda para agarrar, e um agarre sobre o nada seria chrome vivo e invisível.
    #[must_use]
    pub fn dock_seam(&self, side: DockSide) -> Rect {
        let (left_col, right_col) = self.side_columns();
        let col = match side {
            DockSide::Left => left_col,
            DockSide::Right => right_col,
        };
        let occupied = match side {
            DockSide::Left => self.docks.left,
            DockSide::Right => self.docks.right,
        };
        if !occupied || col.w <= 0.0 || col.h <= 0.0 {
            return Rect::new(col.x, col.y, 0.0, 0.0);
        }
        let w = DOCK_SEAM_PX.min(col.w);
        match side {
            // A borda INTERIOR: à direita na coluna da esquerda, à esquerda na da direita.
            DockSide::Left => Rect::new(col.x + col.w - w, col.y, w, col.h),
            DockSide::Right => Rect::new(col.x, col.y, w, col.h),
        }
    }

    /// ⭐⭐⭐ **A ALÇA que traz de volta uma coluna FECHADA** — o espelho exacto da [`Self::dock_seam`].
    ///
    /// A costura vive na borda **interior** de uma coluna aberta; esta vive na borda **exterior**
    /// de uma coluna fechada, que é o sítio onde a borda estava antes de ela fechar. *A mão volta
    /// a puxar de onde empurrou.*
    ///
    /// ⛔⛔ **Sem ela, o gesto de fechar seria uma armadilha num tablet.** A costura não é pintada
    /// — ela vive do cursor, e num ecrã de toque não há cursor; o que a torna descobrível é a
    /// borda visível da coluna. Fechada a coluna, essa borda desaparece, e sem alça o caminho de
    /// volta seria outra vez o menu. ⇒ esta é **pintada** (`hero::dock_reopen`), ao contrário da
    /// irmã.
    ///
    /// ⚠️ A coluna fechada **mantém o rect reservado** (medido: `57,64,308,960` com e sem painel);
    /// o que muda é a ocupação. É isso que dá geometria à alça sem inventar número nenhum.
    #[must_use]
    pub fn dock_reopen(&self, side: DockSide) -> Rect {
        let (left_col, right_col) = self.side_columns();
        let col = match side {
            DockSide::Left => left_col,
            DockSide::Right => right_col,
        };
        let occupied = match side {
            DockSide::Left => self.docks.left,
            DockSide::Right => self.docks.right,
        };
        if occupied || col.w <= 0.0 || col.h <= 0.0 {
            return Rect::new(col.x, col.y, 0.0, 0.0);
        }
        let w = DOCK_SEAM_PX.min(col.w);
        match side {
            // A borda EXTERIOR — o oposto da costura, que toma a interior.
            DockSide::Left => Rect::new(col.x, col.y, w, col.h),
            DockSide::Right => Rect::new(col.x + col.w - w, col.y, w, col.h),
        }
    }

    /// **Sobre qual ALÇA de reabertura está o ponteiro, se alguma.**
    ///
    /// ⚠️ Porta separada da [`Self::dock_seam_at`] de propósito: as duas nunca podem responder ao
    /// mesmo tempo (uma exige a coluna aberta, a outra fechada), e uma porta só devolveria um
    /// `DockSide` que não diz **qual das duas coisas** o dedo pediu.
    #[must_use]
    pub fn dock_reopen_at(&self, p: (f32, f32)) -> Option<DockSide> {
        for side in [DockSide::Left, DockSide::Right] {
            let r = self.dock_reopen(side);
            if r.w > 0.0 && p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h {
                return Some(side);
            }
        }
        None
    }

    /// **Sobre qual costura de largura está o ponteiro, se alguma** — a porta única do gesto e do
    /// cursor.
    ///
    /// ⚠️ Ela existe para que *«onde a seta bidirecional aparece»* e *«onde o arrasto pega»* sejam
    /// a **mesma** pergunta: a seta a aparecer um pixel ao lado de onde o gesto agarra lê-se como
    /// «às vezes não pega», que é o defeito que o irmão dela no 3D já pagou.
    #[must_use]
    pub fn dock_seam_at(&self, p: (f32, f32)) -> Option<DockSide> {
        for side in [DockSide::Left, DockSide::Right] {
            let r = self.dock_seam(side);
            if r.w > 0.0 && p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h {
                return Some(side);
            }
        }
        None
    }

    /// A largura que a coluna passa a ter se a costura for solta em `x`.
    ///
    /// ⚠️ **A conta é do LADO**: à esquerda a largura cresce com o `x`, à direita ela decresce —
    /// é a inversão que se escreve ao contrário sem o compilador reclamar.
    #[must_use]
    pub fn dock_width_for(&self, side: DockSide, x: f32) -> f32 {
        let (left_col, right_col) = self.side_columns();
        match side {
            DockSide::Left => x - left_col.x,
            DockSide::Right => right_col.x + right_col.w - x,
        }
    }
}
