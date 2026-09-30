# 28 — A marca da água depois de rodar a vista: o bug e a cura

> **Módulo:** 3D / Sculpt — o Painter a pintar na peça (etapas 2 e 3a,
> [`25_avaliacao_o_painter_na_malha.md`](25_avaliacao_o_painter_na_malha.md)).
> **Estado:** ✅ **curado e com o smoke do dono aprovado (30/09)**, commit
> `ca47a273e` no `line/sculpt3d`. O diário de cada ronda, com as medições e as
> provas de mutação, está no
> [handoff da etapa 2 §9.7–§9.9](handoffs/HANDOFF_line_sculpt3d_O_PAINTER_NA_PECA_ETAPA_2_2026-09-24.md);
> este doc é o **mecanismo e a lei**, para a próxima pessoa não repetir as três
> rondas.

## 1. O sintoma

*Wet Paint* sobre a peça 3D (`IMG` → `PNTR`, cena `PH2D_SCULPT3D_SMOKE=52`):
dá-se um traço, a água escorre, **roda-se a vista** com o botão direito e dá-se
outro traço enquanto ela ainda corre. Na peça fica uma **linha fina e clara, de
1–2 px, tracejada**, a desenhar a FRENTE da água no instante do segundo traço —
e ela **fica**: os escorridos seguintes atravessam-na e ela continua lá.

*«Só acontece após rotacionar a view»* (report do dono) — sem rodar, nunca.

## 2. As três rondas (e o que cada uma não viu)

| Ronda | Report | O que se curou | O que ficou |
|---|---|---|---|
| **1** (29/09, `82b0f9ee9`) | *«rotacionar e pintar em seguida está pausando a simulação»* | a sessão da água deixa de morrer ao mudar de vista: a grade, a base congelada e a tela são **levadas** para a vista nova, ponto da superfície a ponto da superfície (`reproject_grid`, `reproject_screen_canvas`, `tela_origem::origem`) | a semente do traço novo passou a ser a **tela levada** |
| **2** (29/09, `f08ccfbdd`) | *«fica uma marca sem tinta no local onde a simulação reinicia»* | a semente passa a ser o **retrato** da peça na vista nova, e o Painter devolve o retrato a todo píxel seco | o erro passou de **um píxel inteiro** para **meio píxel interpolado** |
| **3** (30/09, `ca47a273e`) | *«melhorou mas não curou perfeitamente»* | a **cadeia molhada** (§4) — o resíduo deixa de existir por construção | — |

⚠️ **As rondas 1 e 2 atacaram o SINTOMA de uma causa que continuou lá.** A 2.ª
é a lição: trocar uma aproximação por outra melhor **reduz** o erro e não o
anula, e o artista vê a diferença entre «menor» e «zero».

## 3. O mecanismo

A peça recebe a tela do Painter por uma **lei de diferença**
([`tela_na_malha`](../../crates/ph2d-sculpt3d/src/tela_na_malha.rs)):

```
nova = base + k·(c − s)
```

`base` é a cor da amostra antes do traço, `c` a tela no ponto dela e `s` a
**semente** (o retrato com que a tela começou) no mesmo ponto. Isto devolve
`nova ≈ c` **só quando `base − s(p) ≈ 0`** — ou seja, quando a semente, lida no
ponto `p`, é a cor que a amostra de facto tem.

- **Na mesma vista** isso é verdade por construção: a amostra foi escrita pela
  mesma tela, lida no mesmo ponto.
- **Depois de rodar** não é. A amostra traz a frente da água de antes **com o
  detalhe da vista de antes** (a tinta fina é mais fina que o ecrã), e o retrato
  da vista nova é essa mesma cor **amostrada nos centros de píxel de agora** e
  interpolada. Numa frente dura as duas diferem por uma fracção de píxel.

Esse resíduo `base − s` fica **assado** na amostra: a lei soma-lhe a diferença
da tela enquanto a água passa, e quando a água sai a amostra volta a
`base − s + s = base`… mais o erro. A frente fica desenhada onde a água já não
está. Com a tela levada (ronda 1) o resíduo era de um píxel; com o retrato
(ronda 2), de meio píxel interpolado — mas **nunca zero**, porque nenhuma
amostragem nova de uma superfície com detalhe sub-píxel reproduz a antiga.

## 4. A lei que fica — a cadeia molhada

[`BaseDaCadeia`](../../crates/ph2d-sculpt3d/src/tela_na_malha_cadeia.rs):
**uma pincelada molhada e as que a continuam são uma cadeia**, e a lei passa a
ser escrita contra o que a peça era ANTES dela:

```
nova = antes + k·(c − s₀)
```

- **`antes`** — a cor de cada amostra antes da **primeira** pincelada da
  cadeia, guardada na 1.ª vez que a cadeia a toca (`BTreeMap<u32, _>` por
  amostra tocada ⇒ `O(pegada)`, nunca `O(plano)`) e levada de traço em traço na
  `TelaMolhada` da escultura.
- **`s₀`** — o retrato da peça **sem a cadeia**
  ([`semente_antes_da_cadeia`](../../crates/ph2d-sculpt3d/src/tela_semente.rs)):
  troca-se a cor de antes para dentro do destino, retrata-se, e desfaz-se a
  troca — cada troca é a sua própria inversa, logo a peça volta **ao bit**.
- **A água do Painter usa `s₀` como base congelada inteira** na vista nova
  ([`reproject.rs`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/reproject.rs)):
  a tela é o pigmento da cadeia por cima de `s₀`, logo **`c − s₀` é exactamente
  o pigmento**. A frente de antes deixa de existir como resíduo — ela só vive no
  pigmento, que corre.

⭐ **Na mesma vista a lei nova É a antiga, somada:** a antiga era
`(antes + k(c₁ − s₀)) + k(c − c₁)`, e isso é `antes + k(c − s₀)`. É por isso que
a cura não muda nada em quem nunca roda a vista (gate de igualdade `≤ 1e-5`).

### As quatro metades que a lei obrigou

1. **Uma amostra da cadeia é repintada mesmo com `c − s₀ = 0`** — é quando a
   água SAIU dali, e ela tem de voltar a `antes`. Sem isto ficava com a água que
   já não está (`na_cadeia` na pousada).
2. **Sem cor a comparar ela NÃO volta à base da cadeia, e nem é tocada**
   (`Mistura::SemCor`, separada da `Diferenca` zero): um píxel apagado pela
   borracha, ou o fundo fora da silhueta, não é informação — voltar a `antes`
   seria apagar tinta por falta dela.
3. **Rodar deixa na cadeia só o que a vista nova herdou**
   (`so_o_que_a_vista_levou`): uma amostra cujo píxel novo não tem origem na
   vista de antes (ou que a vista nova não mostra) sai. A água dela não viajou; se
   ficasse, voltaria a `antes` e **apagaria** a tinta que ali pousou.
4. **O fim do traço larga a cadeia** para a `TelaMolhada`, e uma cadeia de outro
   destino (o plano de tinta fina mudou de tamanho) é **recusada** em vez de
   indexar amostras erradas.

⚠️ O Painter **deixou de adivinhar**: as rondas 1 e 2 tentavam reconstruir,
do lado dele, qual era a peça sem a água (base levada pelo píxel mais perto,
retrato fora dela, o `b == t`). Quem sabe isso é a escultura — ela guarda a cor
de antes de cada amostra que a água tocou —, e o contrato passa a ser dela.

## 5. Os gates

| Gate | Onde | O que afirma |
|---|---|---|
| `depois_de_rodar_a_frente_de_antes_nao_fica_na_cor_por_vertice` · `…_na_tinta_fina` | [`tela_na_malha_cadeia_tests`](../../crates/ph2d-sculpt3d/src/tela_na_malha_cadeia_tests.rs) (sem placa) | frente de UM píxel, vista seguinte deslocada **meio píxel**: a lei da cadeia deixa a peça chapada a `≤ 1/255`; **o CONTROLO é a lei da ronda 2**, que desvia `> 0,1` na mesma peça |
| `na_mesma_vista_a_cadeia_e_a_lei_de_antes_*` | idem | as duas leis iguais a `≤ 1e-5` (a água espalhou · a água saiu) |
| `sem_cor_a_comparar_a_amostra_da_cadeia_fica` | idem | o apagão não devolve a tinta a `antes` **nem põe a amostra na janela do desfazer**, com o controlo de que sem apagão a cadeia é tocada |
| `rodar_deixa_na_cadeia_so_o_que_a_vista_nova_herdou` · `o_retrato_sem_a_cadeia_nao_mexe_na_peca` · `uma_cadeia_de_outro_destino_e_recusada` | idem | as metades 3 e 4, e a troca a devolver a peça ao bit |
| `depois_de_rodar_a_semente_e_a_peca_sem_a_agua` | [`tinta_no_produto_escorre`](../../crates/ph2d-app-sculpt3d/src/tinta_no_produto_escorre.rs) (placa, `#[ignore]`) | o FIO: a semente do traço depois de rodar não tem a tinta do 1.º traço (com o controlo de que a peça a tem) e a cadeia chega ao traço novo |
| `a_base_da_agua_e_a_semente_em_todo_pixel` | [`screen_canvas_reproject_tests`](../../crates/ph2d-tool-painter/src/tool/screen_canvas_reproject_tests.rs) | nenhum píxel da tela do Painter guarda a base de antes |
| censo P16 · P29 · P30 | [`painter_fiacao_tests`](../../crates/ph2d-app-sculpt3d/src/painter_fiacao_tests.rs) | os três elos de texto da costura (a cadeia no traço reaproveitado, a semente sem a cadeia, a cadeia filtrada pela vista nova) |

**Prova de mutação** ([`muta_o_painter_na_peca.sh`](ferramentas/muta_o_painter_na_peca.sh)):
**19 de 19** sangram na lei e na costura, **3 de 3** no produto com placa, o
controlo sobrevive, pré-voo `83/83`.

## 6. O que a investigação ensinou

- ⛔ **Uma bancada que não reproduz o sintoma não pode aprovar a cura pelo
  sintoma.** A água das fixturas assenta cedo e o escorrido carregado do dono
  não se forma aqui; uma contagem de «píxeis mais claros que a tela» foi
  construída na ronda 2 e **não discriminava** (`66` sem rodar, `51` com a cura,
  `48` sem ela — é resolução). ⇒ a régua passou a ser a **LEI**, numa fixtura
  construída para conter o fenómeno (frente de um píxel, meio píxel de rotação),
  com a lei antiga como controlo.
- ⛔ **Uma cura que troca uma aproximação por outra melhor não fecha um resíduo
  que é da forma da lei.** O resíduo `base − s` só desaparece se `s` for,
  **por construção**, o que a base representa — daí escrever as duas contra o
  mesmo instante (antes da cadeia) em vez de amostrar melhor.
- ⛔ **Duas cercas que se tapam uma à outra leem-se como uma cerca a
  funcionar.** A pergunta pela mistura em `na_cadeia` era mascarada pela
  `base_de` (que devolve `pre` no `SemCor`, e a cor não muda): a mutação que a
  tirava sobreviveu. O efeito real — a amostra entrar na janela do desfazer e do
  upload sem mudar — só se vê contando a **janela**, e só as amostras **da
  cadeia** (na orla do apagão a bilinear mistura píxeis apagados e vivos e ali a
  captura é legítima: `66`, medido).

## 7. Como reproduzir / smoke

```
cd <worktree> && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

`IMG` → `PNTR`, *Wet Paint*, um traço, deixar escorrer; rodar com o botão
direito e dar outro traço enquanto corre, perto de onde estava a frente;
repetir. Correcto: a tinta segue sem risco. Defeito: a linha clara tracejada
onde a frente estava.

## 8. O que fica em aberto

Nada deste bug. Da etapa, continuam por fazer a **3b** (o relevo do Impasto
lido pela luz da cena 3D) e a **4** (camadas e efeitos) — decisões e ordem do
dono.
