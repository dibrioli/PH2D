# 11 — A fronteira dos MOTORES: 74 % dela é uma DERIVAÇÃO, não mil chaves à mão

> **Medido em 2026-09-17**, na árvore da `line/UIUX` já rebaseada no `main` da rodada de seis
> (`git merge --ff-only main` → `3090cac3f`, `0` à frente / `0` atrás).
> Instrumento: `git grep` + varredura por contagem de chavetas (scratchpad, com `assert` de
> população) e `python3 scripts/censo-texto-pintado.py`.

## §1 — Porque esta página existe

O fecho de 2026-09-16 deixou o HR-15 fechado **fora dos motores** e escreveu a fronteira seguinte
em prosa: *«os MOTORES escrevem o rótulo numa tabela de dados, e ali a chave tem de ser derivada do
id»*. **Prosa não é um plano.** Esta página põe o número em cada célula dela, e a primeira medição
já **mudou o tamanho da wave**.

⚠️ **E ela corrige uma premissa minha por escrito:** eu supunha que o rótulo de um parâmetro de nó
vivia no `ParamSpec` do manifesto. **Não vive** — o `ParamSpec` é `{ name, default }`, e o rótulo
está no `ParamUiHint`, que é *side-metadata no registry* (a lei que o `CLAUDE.md` §5 já declara
para este módulo). Quem apanhou o erro foi o `assert` de população da própria sonda, que leu `0`
onde esperava `~819`. *Uma varredura sem piso de população teria impresso «0 % deriváveis» e eu
teria acreditado.*

## §2 — A população, por motor

| Motor | população MEDIDA | forma em que o rótulo nasce |
|---|---:|---|
| manifestos de nó (135 crates `ph2d-node-*`) | **801** `ParamUiHint.label` | `ParamUiHint { param: "channel", label: "Channel", … }` |
| … mais as opções de enum dos mesmos | **398** | `ParamWidget::Enum { labels: &["X", "Y", …] }` (151 widgets) |
| `ph2d-component-desc` | **86** `display_name` + **103** `FieldDesc.name` | literal **POSICIONAL** num `const` (`D::authored("ph2d::ecs::AudioSource2D", "Audio Source 2D", …)`) |
| `ph2d-painter-brush` | **37** | braços de `pub const fn name(self) -> &'static str` |
| `ph2d-painter-effects` | **24** | braços de `pub const fn display_name` |
| `ph2d-tool-vector` | **7** | braços de `pub fn label` |
| **o censo de PORTA** (o que só o ponto-fixo vê) | **63** em **10** crates | literal que nenhum censo léxico alcança |
| **TOTAL** | **≈ 1 519** | |

⚠️ **O `63` subiu de `62` em `9` crates** desde o fecho: a `line/components` trouxe um literal novo
no `ph2d-panel-tags`. *O censo de porta não é um gate — ninguém o corre num portão —, e por isso
ele deriva sozinho.* Fechar a fronteira dos motores sem o transformar em gate deixa a mesma porta
aberta.

## §3 — O achado que muda o tamanho: a lei da derivação JÁ SHIPA

⭐⭐⭐ Os **nomes de PORTA** de um nó nunca foram texto autorado: o cartão pinta-os por
`PortLabel::of(p.name)` ([`paint_port_label.rs`](../../../crates/ph2d-panel-motion-graph/src/paint_port_label.rs)),
que **deriva** `target_x` → `Target X`, `in0` → `In 0`. Uma porta, um consumidor, **521**
`PortSpec` servidos sem um único literal de interface.

⇒ a pergunta certa deixa de ser *«quantas chaves tenho de escrever?»* e passa a ser
***«quantos rótulos JÁ SÃO a derivação do id?»***. Medido com a **mesma régua** do `PortLabel::of`:

| | contagem | fracção |
|---|---:|---:|
| `ParamUiHint` com `param` **e** `label` literais | 578 | — |
| **`label` == `derive(param)`** (a tabela não acrescenta nada) | **428** | **74,0 %** |
| divergentes (texto autorado a sério) | 150 | 26,0 % |
| blocos cujo `param` é uma `const` (`param::COUNT`) ⇒ a sonda não os lê | 223 | — |

**Amostra do que diverge de verdade** — e é boa, não é ruído:

```
soft         → "Softness"     (derivado: "Soft")
key          → "Order By"     (derivado: "Key")
mode         → "Path Mode"    (derivado: "Mode")
target_mode  → "Target"       (derivado: "Target Mode")
type         → "Noise Type"   (derivado: "Type")
gust_freq    → "Gust Frequency"
```

⇒ **três quartos da maior população da fronteira desaparecem** se a lei for *«o rótulo é derivado do
id; a tabela guarda só o que DIFERE da derivação»* — que é exactamente a forma que o `PortLabel::of`
já prova em produção, um nível acima.

## §4 — O que isto prescreve para a wave (e o que ela NÃO pode fazer)

1. **A chave sai do id, nunca de um segundo literal.** `node.<nó>.param.<param>` para o hint,
   `component.<canonical_name>.name` / `.field.<n>` para o catálogo. Um rótulo novo que ninguém
   traduziu cai na **derivação**, e a derivação é legível em inglês — *o modo de falha é um rótulo
   sem tradução, nunca um `???` no ecrã*.
2. ⚠️ **A chave derivada do `canonical_name` prende a tradução ao nome do TIPO.** Renomear
   `ph2d::ecs::AudioSource2D` muda a chave **em silêncio** ⇒ o gate obrigatório é o de **dois
   sentidos** que a crate-régua já sabe fazer (`gate::chaves`): toda chave derivada existe na tabela,
   e toda chave da tabela é derivada por alguém.
3. ⛔ **`tr()` não é `const`** e estas tabelas são `static`. O que viaja na tabela é `TextKey`
   (const-construtível) e quem chama `.tr()` é o **pintor** — o padrão já pago 10 vezes no fecho de
   16/09. Um `ComponentDesc::display_name: &'static str` que passasse a `String` reescrevia 86
   construtores `const`.
4. ⛔ **Os 63 literais do censo de porta NÃO são cobertos por nada disto** — eles vivem em sítios
   que a régua léxica já isenta, e o único instrumento que os vê é um script que **nenhum portão
   corre**. Ou ele vira gate na mesma wave, ou a fronteira fecha com um buraco que se enche sozinho.

## §5 — Como reproduzir

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX
grep -rhoE 'ParamUiHint *\{' crates/ph2d-node-*/src/ | wc -l     # 801
grep -rhoE 'D::[a-z_]+\(' crates/ph2d-component-desc/src/catalog/*.rs | wc -l   # 86
python3 scripts/censo-texto-pintado.py                            # 63 em 10 crates
```

A sonda da derivação vive no scratchpad de propósito (ela responde UMA pergunta e o `assert` dela é
o que a torna honesta); o que fica versionado é o **número**, nesta página.

---

# §6 — ⛔ RECUSA MEDIDA: a coluna do rótulo NÃO era a wave (2026-09-17)

Antes de escolher a fronteira dos motores eu preferi uma candidata **visível**: generalizar a
`every_label_this_panel_paints_fits_its_column` (que existe em **3** de **29** painéis) aos outros
26, porque havia um report do dono por trás — *«3 pontos (…) sendo usados antes de ficar estreito»*,
com foto, em 2026-09-14.

A crate-régua foi construída (`ph2d-label-fit`, quarta irmã da família `ph2d-label-*`), correu sobre
**as tabelas de texto dos 24 painéis** — possível *só porque esta linha as construiu* — e o
resultado **refutou a hipótese**:

| largura do painel | strings acima da coluna do rótulo |
|---:|---:|
| `220` (mínimo do dock) | **510** |
| `245` | 219 |
| `304` (omissão) | 59 |
| `333,8` · `369,7` (**as que o dono tem hoje**, lidas do `layout.txt`) | **37** · **24** |

⛔⛔ **E a população estava ERRADA, que é o defeito que este repo já pagou quatro vezes.** Os mais
largos não são rótulos de coluna nenhuns — são **frases de estado**:

```
physics    300,4 px   "Paused: drag carries the whole rig, anchors included"
sculpt3d   298,8 px   "Only used by Twist, and only with Segments above 1"
```

…e elas **DOBRAM**, não cortam: o `paint_hint` da física passa por `paint_text_block` com a largura
inteira da linha, e o doc dele já o diz (*«só o excedente da quebra é acrescentado»*). *Uma régua
que mede uma frase que dobra contra uma coluna que ela não usa fabrica dívida* — exactamente o que a
1.ª redacção do censo do ritmo fez ao ler `48 de 81` ficheiros.

⇒ **o que sobra à largura REAL do dono é quase nada, e nada do que sobra é um rótulo cortado.** A
wave era **insurance**, não um defeito visível — e a ordem em vigor do dono é *«se temos o manual,
precisamos converter o APP todo a ele»*, que é a §2 desta página.

⚠️ **A crate foi APAGADA no mesmo dia.** Ela compilava e fazia a coisa certa, e **não tinha
consumidor** — a lei desta casa é que uma ferramenta que nenhum passo escrito chama pelo nome morre.
*O que se guarda de uma experiência revertida é a MEDIÇÃO, e ela está nesta tabela.*

⭐ **O que ela deixa para quem voltar aqui:** a régua honesta desta pergunta **não é a tabela de
i18n de um painel** — é a tabela de LINHAS que o pintor dele lê (`IROWS` na física, `PLAYER_CARDS` no
Inspector). Medido: **11** dos 29 painéis têm uma, e **3** já têm o gate. *Quem quiser a insurance
começa nos 8 que têm tabela e nenhuma prova; os outros 18 pagam primeiro a tabela.*
