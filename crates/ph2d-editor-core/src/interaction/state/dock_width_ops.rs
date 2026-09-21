//! ⭐ **A LARGURA DAS DUAS COLUNAS** — a divisória que o artista arrasta (decisão **D4**).
//!
//! ⚠️ **Cortado do `chrome_ops.rs` em 2026-08-30 pelo tecto de LOC (706/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro é o saco do chrome (cor de widget, raio, vsync, tamanho de
//! botão) e isto é **uma** pergunta com dois lados — *quanto ela mede* e *o artista escolheu?*.
//!
//! ⛔ **As duas leituras não são a mesma, e a diferença decide o que se GRAVA:** a
//! [`WidgetStore::dock_width`] devolve sempre um número (o default quando ninguém arrastou) e a
//! [`WidgetStore::dock_width_choice`] devolve **a escolha**. Persistir a primeira escreveria o
//! default como se fosse uma decisão do artista.

use super::WidgetStore;

impl WidgetStore {
    /// **A largura AUTORADA de uma coluna docada** — a de fábrica até alguém arrastar a borda.
    ///
    /// ⚠️ Clampada na PORTA e não em cada leitor: uma coluna que possa encolher a zero ou comer a
    /// janela é estado inalcançável de volta (não sobra borda para agarrar).
    ///
    /// ⭐⭐⭐ **E a de FÁBRICA depende da JANELA desde 2026-09-20** — ver
    /// [`ChromeBands::default_dock_w`]. ⛔ **O `janela_w` é um argumento e não um campo do
    /// `WidgetStore`, de propósito:** a largura da janela é um facto do QUADRO e o store é o
    /// estado AUTORADO; guardá-la ali poria duas respostas à mesma pergunta, e a que o layout usa
    /// seria a do quadro anterior.
    ///
    /// ⚠️ **A escolha do artista NÃO é escalada** — só a base. Ver o doc da lei.
    pub fn dock_width(&self, side: crate::screens::layout::DockSide, janela_w: f32) -> f32 {
        use crate::screens::layout::{ChromeBands, DockSide};
        let stored = match side {
            DockSide::Left => self.dock_w_left,
            DockSide::Right => self.dock_w_right,
        };
        let base = ChromeBands::default_dock_w(side, janela_w);
        // ⚠️ **O piso é o de uma ESCOLHA** ([`Self::DOCK_W_MIN`]), e ele é mais BAIXO que o de
        //    fábrica desde 2026-09-20 — ver o doc dela. ⭐ Aqui isso é inerte no caminho de
        //    omissão: o `base` nunca desce do mínimo do painel, logo quem nunca arrastou lê
        //    exactamente o mesmo número de antes.
        crate::math::safe_clamp(stored.unwrap_or(base), Self::DOCK_W_MIN, Self::DOCK_W_MAX)
    }

    /// ⭐ **A ESCOLHA do artista, ou `None`** — o irmão de [`Self::dock_width`], que devolve
    /// sempre um número (o default quando ninguém arrastou).
    ///
    /// ⚠️ **A distinção decide o que se GRAVA.** Persistir o valor de `dock_width` escreveria o
    /// default como se fosse uma escolha — e no dia em que o default mudasse, toda arrumação
    /// gravada continuaria a prender a coluna no número velho, sem ninguém ter pedido nada.
    #[must_use]
    pub fn dock_width_choice(&self, side: crate::screens::layout::DockSide) -> Option<f32> {
        match side {
            crate::screens::layout::DockSide::Left => self.dock_w_left,
            crate::screens::layout::DockSide::Right => self.dock_w_right,
        }
    }

    /// Escreve a largura de uma coluna, já clampada.
    ///
    /// ⭐⭐⭐ **O PISO É O DE UMA ESCOLHA ([`Self::DOCK_W_MIN`]) — a borda encolhe até ali e PARA.**
    ///
    /// ⚠️⚠️ **Ele DEIXOU de ser o mínimo do painel em 2026-09-20** (ordem do dono: *«permita que
    /// manualmente o usuário consiga estreitar o painel»*). O parágrafo abaixo continua a
    /// descrever por que ele voltou ao mínimo do painel em 2026-09-09 — e essa premissa era
    /// **verdade sobre o que o piso protege e falsa sobre quem o pede**: o mínimo do painel é a
    /// largura em que o app deve NASCER, não o tecto do que o artista pode pedir.
    ///
    /// > *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra
    /// > superior.»* — Enio, 2026-09-09.
    ///
    /// ⛔⛔ **Isto REVERTE o piso de 2026-09-08, e a razão é que a premissa dele dissolveu.** Ele
    /// tinha descido para o degrau do fecho ([`Self::DOCK_W_COLLAPSE`]) por um motivo bom: com o
    /// piso no mínimo havia **22 px de arrasto MUDO** entre o mínimo e o degrau — a borda parava
    /// de seguir o rato e nada mudava no ecrã, e *o único sinal daquele gesto era a coisa que
    /// tinha deixado de se mexer*. Esses 22 px só existiam **porque o arrasto podia fechar**. Sem
    /// o fecho, a faixa não é «a parte muda do gesto»: é **largura que o painel não sabe
    /// desenhar** (abaixo do mínimo o cabeçalho e uma linha deixam de caber juntos). ⇒ o piso
    /// volta a ser o do painel, e o `dock_seam_up` deixa de precisar de devolver ninguém a lado
    /// nenhum. *`CLAUDE.md` §0.0: quem tira o consumidor de um número tem de reconferir o número.*
    ///
    /// ⚠️ **Um só piso, e é o mesmo na leitura e na escrita** — dois pisos diferentes eram o que
    /// deixava uma largura de GESTO chegar ao disco.
    ///
    /// ⭐⭐⭐ **`None` APAGA a excepção, e é isso que faz «sem escolha» ser exprimível por quem
    /// escreve.** Antes de 2026-09-20 esta porta só sabia gravar, e o caminho do ARRASTO usava-a
    /// para escrever a largura de FÁBRICA como se fosse uma decisão do artista — o defeito do
    /// report *«não diminuiu os paineis»*, com `dock_w_right=220` no ficheiro de arrumação depois
    /// de um gesto que não mexeu um pixel. ⚠️ Quem DECIDE é a
    /// [`ChromeBands::escolha_de_um_arrasto`], porque a decisão precisa da largura da JANELA e
    /// este store é estado AUTORADO, que não a conhece.
    pub fn set_dock_width(&mut self, side: crate::screens::layout::DockSide, w: Option<f32>) {
        let w = w.map(|w| crate::math::safe_clamp(w, Self::DOCK_W_MIN, Self::DOCK_W_MAX));
        match side {
            crate::screens::layout::DockSide::Left => self.dock_w_left = w,
            crate::screens::layout::DockSide::Right => self.dock_w_right = w,
        }
    }

    /// ⭐⭐⭐ **O PISO DE UMA ESCOLHA — e ele NÃO é o piso de fábrica.**
    ///
    /// > *«não funciona. pare de tentar. permita que manualmente o usuário consiga estreitar o
    /// > painel.»* — Enio, 2026-09-20.
    ///
    /// # ⛔⛔ O report reproduzido, com o número
    ///
    /// Ele estava numa janela de `647 px` (o fim do log de `PH2D_DOCK_LOG` que ele colou). Ali a
    /// lei da fracção já entrega o mínimo nos dois lados, logo as duas colunas mediam `220` e
    /// **ocupavam `440` de `647`** — e arrastar a borda **não fazia absolutamente nada**, porque
    /// o piso da escrita era o mesmo `220`. Medido pela rota do produto:
    ///
    /// | janela | coluna | arrastar `60 px` para estreitar |
    /// |---:|---:|---|
    /// | `1 920` | `308,0` | `308,0 → 245,0` ✅ |
    /// | `1 024` | `230,9` | `230,9 → 220,0` (pára no piso) |
    /// | `640` | `220,0` | **`220,0 → 220,0`** ⛔ inerte |
    ///
    /// *Um gesto que existe, arma, segue o dedo e não muda um pixel lê-se como um gesto partido* —
    /// e é o mesmo modo de falha que o arrasto fantasma de 2026-09-20 (o `set_dock_width`
    /// `Option`) já tinha pago do outro lado.
    ///
    /// # ⭐⭐⭐ A LEI: o piso de FÁBRICA protege quem não escolheu; um arrasto É uma escolha
    ///
    /// São **dois** pisos de propósito, e a diferença é quem os produz:
    ///
    /// - a largura de FÁBRICA ([`crate::screens::layout::ChromeBands::default_dock_w`]) continua
    ///   a parar em [`ph2d_tokens::PANEL_MIN_W_PX`] (`220`) — ninguém pediu para o app **nascer**
    ///   com uma coluna ilegível;
    /// - a largura que o artista **arrasta** pára aqui, mais abaixo, porque ele pediu.
    ///
    /// ⚠️ **Isto não muda nada no caminho de omissão:** `dock_width` clampa
    /// `stored.unwrap_or(base)`, e o `base` nunca desce dos `220` por construção ⇒ **uma
    /// arrumação sem escolha nenhuma lê exactamente o mesmo número de ontem.**
    ///
    /// # ⚠️ O NÚMERO é medido, e diz de que recurso é
    ///
    /// `84` é a largura em que o **corpo de um painel docado** ainda cabe na coluna: o controlo de
    /// largura fixa mais largo que ele pinta mede `80`, e o recuo do painel é `4`. Varrido pixel a
    /// pixel pela rota real (pintar quatro quadros e ler o índice de toque):
    ///
    /// | coluna | controlos que saem dela |
    /// |---:|---:|
    /// | `86` · `85` · **`84`** | `0` |
    /// | `83` | `1` (excesso `1,0 px`) |
    /// | `80` | `2` |
    /// | `64` | `3` |
    ///
    /// ⭐ **E a varredura correu com CADA um dos 19 painéis docáveis à frente, não só os de
    /// fábrica:** descer de `220` para `84` acrescenta no máximo **`1,0 px`** de transbordo, e num
    /// só painel (`physics`). Os outros dezoito acrescentam **zero**.
    ///
    /// ⛔⛔ **PRÉ-EXISTENTE e NOMEADO, não curado aqui:** há um controlo de `36 × 36` px que
    /// transborda a coluna **`7,0 px` já na largura de fábrica de `220`** — ele não é um problema
    /// de piso (a essa largura o produto de hoje shipa igual) e a posição dele não é monótona na
    /// largura da coluna (`x` lê `191` a `220`, `50` a `84` e `90` a `90`). *Uma régua de
    /// transbordo ABSOLUTO neste app mede esse widget; a que decide aqui é o **acréscimo** contra
    /// a largura de fábrica.*
    ///
    /// ⛔ **O piso NÃO é o da faixa de abas, e ela foi medida:** o `tab_plan` ainda entrega uma aba
    /// a `32 px` e só desiste a `24` (o glifo, `TAB_BAR_H = 22`, mais o respiro de `8`). Parar aí
    /// entregaria uma coluna cujo corpo pinta por cima da área de desenho — *o piso é do CORPO,
    /// que é o que falha primeiro.*
    ///
    /// # ⭐⭐⭐ E o número que SHIPA é `2,5 ×` esse, por DUAS ordens do dono
    ///
    /// > *«a largura mínima precisa ser no mínimo o dobro que a largura mínima que vc definiu.»*
    /// > — Enio, 2026-09-20, depois de ver os `84`. ⇒ `168`.
    ///
    /// > *«ainda muito estreito. aumente 25%.»*
    /// > — Enio, 2026-09-20, depois de ver os `168`. ⇒ **`210`**, que é `2,5 ×` o piso medido.
    ///
    /// ⚠️⚠️ **Isto separa duas coisas que estavam coladas, e a separação é o que torna o número
    /// honesto:** [`Self::PISO_DO_CORPO_PX`] é o **RECURSO** (o que o produto CONSEGUE fazer,
    /// medido) e o `DOCK_W_MIN` é a **DECISÃO** (o que ele DEVE fazer). *Um piso no recurso
    /// entrega uma coluna que cabe e não serve* — o dono viu-a e devolveu-a com um número.
    ///
    /// ⭐ **E a ordem dele é uma CERCA de compilação, não um comentário** (logo abaixo): o valor
    /// é explícito porque é uma escolha de produto, e quem o descer abaixo do dobro do recurso
    /// **não compila**.
    ///
    /// ⭐⭐ **Isto aperta a faixa legal nos DOIS lados:** `210 ≤ DOCK_W_MIN < 220`. Antes da
    /// 1.ª ordem dele só havia cerca por baixo e um piso solto era inatacável em toda a recta;
    /// depois da 2.ª a janela em que ele pode mentir tem **`10 px`**.
    ///
    /// ⚠️⚠️ **E esses `10 px` são também o que o gesto compra numa janela ESTREITA, que é onde o
    /// report nasceu.** A `640 px` a lei de fábrica já entrega `220`, logo arrastar leva a coluna
    /// de `220` a `210` e mais nada; o curso real do gesto está nas janelas LARGAS (a `1 920` a
    /// coluna abre a `308` e desce a `210`, que são `98 px`). *A decisão é do dono e está tomada
    /// com o número à frente — o que fica escrito aqui é a consequência dela, não uma objecção.*
    ///
    /// O máximo é medido pelo mesmo critério do `clamp_panel_rect`: 70 % de uma janela de
    /// referência, para uma coluna nunca comer a área de desenho inteira.
    pub const DOCK_W_MIN: f32 = 210.0; // LITERAL-PX-OK: ordem do dono -- 2,5 x o piso medido

    /// ⭐ **O RECURSO, medido — e ele NÃO é o que shipa.** Ver o doc de [`Self::DOCK_W_MIN`]: esta
    /// é a largura em que o corpo de um painel docado ainda cabe na coluna (a tabela está lá), e
    /// o que shipa é **`2,5 ×`** ela, por decisão do dono. ⛔ Ela fica porque é a proveniência do
    /// número que shipa: sem ela, *«o dobro»* e *«mais 25 %»* deixam de ter de quê.
    pub const PISO_DO_CORPO_PX: f32 = 84.0; // LITERAL-PX-OK: piso MEDIDO do corpo de um painel

    /// ⛔⛔ **DORMENTE desde 2026-09-09 — ela já não tem consumidor no produto.**
    ///
    /// > *«Vamos retirar a opção de colapsar arrastando.»* — Enio, 2026-09-09.
    ///
    /// ⚠️ **Fica escrita, e a medição com ela, porque o que morreu foi o GESTO e não o
    /// número.** O dia em que o fecho de uma coluna voltar — a morada dele é o menu, e ali
    /// ele é um verbo sem largura — quem o construir precisa de saber por que o degrau era
    /// **uma linha** abaixo do mínimo, e não meia nem duas. ⛔ Se ao fim de uma jornada nada
    /// a ler, ela é lixo e apaga-se: um `const` sem leitor é a espécie que esta casa varre.
    ///
    /// **A largura abaixo da qual o arrasto deixava de ser «encolher» e passava a ser «fechar».**
    ///
    /// Medido em `docs/UI_New_and_Simple/medicoes/06`: fechar as duas colunas devolve **89 a 92 %**
    /// do ecrã em qualquer dos três tablets — mais do que todas as faixas de chrome somadas valem.
    /// E até 2026-09-07 isso custava **dois passeios ao menu** *Ver*, um por coluna, num aparelho
    /// sem teclado.
    ///
    /// ⭐ **O gesto é o do Blender**, que é a referência que o dono nomeou: arrastar a borda de uma
    /// região para dentro **fecha-a**, e uma alça na margem trá-la de volta. Escolhido por três
    /// razões medidas: o artista **já arrasta esta borda** (a costura shipou em 2026-08-30 e hoje
    /// apenas trava no mínimo); funciona **sem teclado**, que é a condição num tablet — o
    /// `Ctrl+Space` do Blender e o modo sem distracções do Godot não servem lá; e não custa chrome
    /// permanente, porque a alça só existe enquanto a coluna está fechada.
    ///
    /// ⚠️ **O degrau é UMA LINHA de folga abaixo do mínimo, e o recurso tem nome:** chegar ao
    /// mínimo é um objectivo legítimo do artista, logo tocar-lhe não pode fechar nada. Uma linha
    /// (`ROW_H_PX`) é a menor coisa que a coluna sabe mostrar — pedir menos do que *o mínimo menos
    /// aquilo que ela mostraria* é pedir para não haver coluna. ⛔ Não é folga de dedo: `22 px` é
    /// dez vezes o tremor de um toque, e a distinção tem de ser deliberada.
    pub const DOCK_W_COLLAPSE: f32 = Self::DOCK_W_MIN - ph2d_tokens::ROW_H_PX;
}

// ⭐⭐ **As duas leis do degrau são erro de COMPILAÇÃO, não teste.**
//
// ⚠️ Elas nasceram como asserções num gate e o clippy recusou-as — *«this assertion has a constant
// value»*. Ele tinha razão, e a recusa aponta para cima: uma propriedade que o compilador consegue
// decidir não devia esperar por uma corrida de testes. *Um teste que o clippy chama de constante é
// um teste que queria ser uma cerca.*
// ⭐⭐⭐ **E a lei de 2026-09-20 é a terceira, pela mesma razão.**
//
// ⚠️ Ela nasceu como asserção no `o_piso_desta_lei_e_o_piso_do_store` e o clippy recusou-a
// (*«this assertion has a constant value»*) — a mesma recusa que pôs as duas de baixo aqui, e ela
// aponta para cima outra vez. *Um teste que o clippy chama de constante é um teste que queria ser
// uma cerca.*
// ⭐⭐⭐ **E as ORDENS DO DONO são a quarta cerca** (*«no mínimo o dobro»* e, a seguir, *«ainda
// muito estreito. aumente 25%»* — 2026-09-20). ⚠️ O valor é escrito à mão de propósito, e não
// `2.5 * PISO_DO_CORPO_PX`: assim escrito, esta asserção seria verdadeira por construção — *uma
// linha que a mutação não consegue matar é comentário com sintaxe de código*. Com o valor
// explícito, quem o descer não compila.
//
// ⚠️ A cerca leva a ordem MAIS RECENTE (`2,5 ×`) e não a primeira: as duas são do mesmo dia e a
// segunda subsume a primeira, logo manter as duas escritas daria a UMA delas o poder de aprovar
// um valor que a outra recusa.
const _: () = assert!(
    WidgetStore::DOCK_W_MIN >= 2.5 * WidgetStore::PISO_DO_CORPO_PX,
    "o piso de uma escolha desceu abaixo de 2,5 x o piso medido do corpo -- sao as duas ordens \
     do dono de 2026-09-20, dadas depois de ele ver a coluna a 84 px e depois a 168"
);

const _: () = assert!(
    WidgetStore::DOCK_W_MIN < ph2d_tokens::PANEL_MIN_W_PX,
    "o piso de uma ESCOLHA subiu ate' ao de FABRICA: numa janela estreita a lei ja' entrega o \
     minimo, logo arrastar a borda deixa de mudar um pixel -- e' o report de 2026-09-20 de volta"
);

const _: () = assert!(
    WidgetStore::DOCK_W_COLLAPSE < WidgetStore::DOCK_W_MIN,
    "o degrau de fechar tem de estar ABAIXO do minimo, senao arrastar ate' ao fim fecha por acidente"
);
const _: () = assert!(
    WidgetStore::DOCK_W_MIN - WidgetStore::DOCK_W_COLLAPSE > 10.0,
    "a folga entre o minimo e o fecho entrou no tremor de um toque: fechar deixou de ser deliberado"
);

#[allow(dead_code)]
impl WidgetStore {
    const DOCK_W_MAX: f32 = 720.0; // LITERAL-PX-OK: teto de largura de coluna docada
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐ **A FAIXA DO FUNDO** — o irmão VERTICAL das duas colunas (Enio, 2026-08-31: *«em nodes,
// arrastar a timeline na vertical deve ajustar o canvas dos nós e não deixar espaços vazios nem
// sobrepor os nodes»*).
//
// ⛔⛔ **O que ele arrastava não era uma banda: era o painel a SOLTAR-SE dela.** A costura do
// timeline escrevia um rect livre (`TimelinePanelState::rect`), e a partir daí o painel ignorava a
// faixa que o layout lhe dava — daí o espaço vazio por cima dele na foto, e a sobreposição no
// outro sentido. *Uma borda de painel docado que devolve um rect livre é um painel que deixa de
// estar docado quando se lhe toca.*
//
// ⇒ o topo da faixa passa a ser uma **costura**, exactamente como a borda interior de uma coluna:
// ela escreve uma MEDIDA, e quem partilha a banda (o grafo de nós, por `dock_timeline_into_motion`)
// segue por construção.
// ─────────────────────────────────────────────────────────────────────────────

impl WidgetStore {
    /// **A altura AUTORADA da faixa do fundo** — a de fábrica até alguém arrastar o topo dela.
    ///
    /// ⚠️ Clampada na PORTA, pela mesma razão da [`Self::dock_width`]: uma faixa que possa encolher
    /// a zero não deixa borda para agarrar de volta.
    #[must_use]
    pub fn dock_bottom_h(&self) -> f32 {
        crate::math::safe_clamp(
            self.dock_h_bottom
                .unwrap_or(crate::screens::layout::TIMELINE_DOCK_H),
            Self::DOCK_H_MIN,
            Self::DOCK_H_MAX,
        )
    }

    /// ⭐ **A ESCOLHA do artista, ou `None`** — o irmão de [`Self::dock_bottom_h`]. A distinção
    /// decide o que se GRAVA; ver [`Self::dock_width_choice`].
    #[must_use]
    pub fn dock_bottom_h_choice(&self) -> Option<f32> {
        self.dock_h_bottom
    }

    /// Escreve a altura da faixa do fundo, já clampada.
    pub fn set_dock_bottom_h(&mut self, h: f32) {
        self.dock_h_bottom = Some(crate::math::safe_clamp(
            h,
            Self::DOCK_H_MIN,
            Self::DOCK_H_MAX,
        ));
    }

    /// ⚠️ **O mínimo é o do painel do timeline** (`geom::MIN_H`, privado dele): abaixo de 120 px o
    /// cabeçalho, a fila de transporte e a régua deixam de caber juntos. ⛔ O número está aqui e
    /// não lá porque quem clampa é a PORTA da medida, e a faixa pode um dia ter outro inquilino —
    /// mas os dois têm de concordar, e há gate a exigi-lo.
    const DOCK_H_MIN: f32 = 120.0; // LITERAL-PX-OK: piso da faixa do fundo (= `timeline::geom::MIN_H`)
    /// ⚠️ O tecto é o mesmo critério da largura de coluna: uma faixa nunca come a área de desenho
    /// inteira. Numa janela baixa o layout aperta-o ainda mais contra a banda de chrome.
    const DOCK_H_MAX: f32 = 720.0; // LITERAL-PX-OK: tecto da faixa do fundo
}

impl WidgetStore {
    /// ⭐ **Publica o que a fila de ferramentas não coube** — ver o campo.
    pub fn set_tool_overflow(&mut self, entries: Vec<crate::widget::ToolRailEntry>) {
        self.tool_overflow = entries;
    }

    /// O que ficou atrás do `⋯` neste quadro.
    #[must_use]
    pub fn tool_overflow(&self) -> &[crate::widget::ToolRailEntry] {
        &self.tool_overflow
    }

    /// ⭐⭐⭐ **Publica o que o módulo com o canvas contribui neste quadro** — ver os dois campos.
    ///
    /// `menus` são os **pulldowns da fila** (a metade 2 da D2); `contrib` é o que ele acrescenta a
    /// menus que **já existem** (a metade 1 — hoje o *File*).
    ///
    /// ⚠️ **UMA porta para as duas metades, de propósito.** Elas são a mesma pergunta da D2 lida
    /// duas vezes (*este comando é do app ou do editor?*), e duas portas dariam duas leis do
    /// «escrito em todo quadro» a envelhecer em separado.
    ///
    /// ⚠️ **Chamado em todo quadro, vazio incluído.** Escrever só quando há módulo armado deixaria
    /// o chip do 3D na fila depois de o módulo fechar — pintado, e a despachar para um painel que
    /// já não existe — e uma linha *Export Draft* no menu *File* que não exporta nada.
    pub fn set_area_commands(
        &mut self,
        menus: Vec<crate::interaction::AreaMenu>,
        contrib: Vec<(
            crate::interaction::ContextMenuKind,
            Vec<crate::widget::ToolRailEntry>,
        )>,
    ) {
        self.area_menus = menus;
        self.menu_contrib = contrib;
    }

    /// Os pulldowns que a área contribui neste quadro — um chip por cada, na ordem.
    #[must_use]
    pub fn area_menus(&self) -> &[crate::interaction::AreaMenu] {
        &self.area_menus
    }

    /// O corpo do pulldown `slot`, ou vazio se não há tal pulldown neste quadro.
    ///
    /// ⚠️ Vazio e não `panic`: o `slot` vem de um `ContextMenuRequest` que sobrevive ao quadro em
    /// que foi aberto, e fechar o módulo com o menu aberto é um gesto legítimo.
    #[must_use]
    pub fn area_menu_rows(&self, slot: u8) -> &[crate::widget::ToolRailEntry] {
        self.area_menus
            .get(usize::from(slot))
            .map_or(&[], |m| &m.rows)
    }

    /// O que um módulo acrescenta a `kind` neste quadro — vazio quando ninguém contribui.
    #[must_use]
    pub fn menu_contrib(
        &self,
        kind: crate::interaction::ContextMenuKind,
    ) -> &[crate::widget::ToolRailEntry] {
        self.menu_contrib
            .iter()
            .find(|(k, _)| *k == kind)
            .map_or(&[], |(_, rows)| rows)
    }
}
