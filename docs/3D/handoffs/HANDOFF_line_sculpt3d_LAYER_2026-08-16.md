# HANDOFF — `line/sculpt3d`: o **Verb::Layer** idêntico ao Blender

**Data:** 2026-08-16 · **Branch:** `line/sculpt3d` · **Worktree:** `Worktrees/line-sculpt3d/`
**Estado:** 112 commits, **nada pushado**, árvore limpa, gate verde (fmt · clippy · 43 suítes release · debug).

---

## §0 — A MISSÃO (ordem permanente do Enio, desde o começo da sessão)

> *"quero idêntico ao blender"* · *"paridade bit-idêntica"* · *"se aumentar
> **hardness** ou **Auto Smooth**, Layer fica muito ruim"*

O alvo é o **`Verb::Layer`** — a demão — **bit-idêntica ao `layer.cc` do Blender**,
e o defeito reportado tem **dois eixos NOMEADOS pelo Enio**: `hardness` e `auto_smooth`.

⚠️ **O agente anterior (eu) errou o alvo e gastou a sessão fora dele** — foi para o
catálogo de falloff e para a fiação do chip, e **só mediu os dois eixos do report no
fim**. As medições da §3 são o único produto útil daquela tarde; **comece por elas.**

---

## §1 — Abrir a linha (rota "linha reaberta")

A branch e a worktree **já existem**. Siga
[`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md),
rota *linha reaberta*:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
pwd && git branch --show-current      # DEVE dizer line/sculpt3d
git rebase main
```

⚠️ **Modo L: TODO comando começa com o `cd` da worktree.** A cwd do Bash **volta ao
repo primário** entre chamadas, e o mesmo path relativo existe nas duas árvores —
eu li `main` achando que lia o tip e **contradisse um agente que estava certo**.
Na dúvida, `pwd` antes de qualquer veredito.

**Referências (fora do repo, já clonadas):**
`/home/enio/Documentos/Recursos/BlenderSculpt` · `/home/enio/Documentos/Recursos/SculptGL`
⚠️ Blender é **GPL** ⇒ **só comportamento, nunca código**. SculptGL é MIT.

---

## §2 — O que JÁ EXISTE (não reconstrua)

A W8 landou **nesta linha** (`ea17b33af`). O verbo está inteiro e **a fiação está provada**:

| peça | onde | estado |
|---|---|---|
| kernel do alvo | [`stroke_target.rs:452`](../../../crates/ph2d-sculpt3d/src/stroke_target.rs) | `base + base_nrm·h`, `base` e `base_nrm` **congelados** |
| a lei que satura | `GripLaw::coat` em [`grip.rs`](../../../crates/ph2d-sculpt3d/src/grip.rs) | `disp += f·força·(1,05 − |disp|)`, o `layer.cc` verbatim |
| defaults do verbo | [`brush_verb_defaults.rs:115`](../../../crates/ph2d-sculpt3d/src/brush_verb_defaults.rs) | `coat=true`, `unit_accum=false`, `from_live=false` |
| `layer_height` | [`brush.rs:132`](../../../crates/ph2d-sculpt3d/src/brush.rs), default **0,1** | faixa `0..0,2` UI, `0..1,0` dura (RNA) |
| row do painel | [`rows.rs:329`](../../../crates/ph2d-panel-sculpt3d/src/rows.rs) | `UiLevel::Basic`, `show: verb == Layer` |
| chip (23º) | `ids::SCULPT3D_VERB[22]` | pintado, registrado, **e clicado por gate** |
| 12 gates de lei | `verb_layer_tests.rs` | **todos verdes** |

⚠️ **NÃO reabra a fiação.** O seam ([`tests/seam.rs:237-246` e `:528`](../../../crates/ph2d-panel-sculpt3d/tests/seam.rs))
já despacha `Click` em **cada um dos 23 chips** e clica o rect que o `paint` registrou.
Medido pela porta do artista, a demão **chega ao barro**:

```
esfregando sem soltar   passadas  1      2      4      8
  Layer                           0,077  0,096  0,100  0,100   <- SATURA no layer_height
  Draw                            0,087  0,173  0,341  0,656
soltando entre as passadas
  Layer                           0,077  0,154  0,309  0,618
```

Sonda: [`tests/probe_layer_product.rs`](../../../crates/ph2d-sculpt3d/tests/probe_layer_product.rs)
(`--ignored --nocapture --test-threads=1`).

---

## §3 — ⭐ O DEFEITO, MEDIDO (comece aqui)

Mesma sonda, teste `probe_hardness_and_auto_smooth_on_the_coat`.
Régua do espeto = **desvio de guarda-chuva sobre os vértices TOCADOS**, em fração de aresta.
⚠️ Varrer a malha inteira mede os **polos** da `uv_sphere` e não o traço — foi assim que
uma tabela minha saiu com seis linhas idênticas.

### (A) AUTO SMOOTH aniquila a demão — o achado principal

```
  auto_sm    relevo    espeto   espeto/relevo
  0.00       0.07707   0.12122          1.573
  0.25       0.00935   0.07721          8.260     <- 8x menos relevo
  0.50       0.00517   0.07771         15.036
  0.75       0.00261   0.08271         31.683
  1.00       0.00164   0.08735         53.375

  CONTROLE (Draw)
  0.00       0.08738   0.13844          1.584
  0.50       0.02072   0.06257          3.020
  1.00       0.00016   0.06100        392.990
```

**A leitura:** a `0,50` o Draw perde **4,2×** de relevo e a demão perde **14,9×** —
⚠️ **a demão é 3,5× mais destruída pelo mesmo knob**, e a razão espeto/relevo dela
sai de 1,57 para **15,0**.

**HIPÓTESE (não confirmada — meça antes de construir):** a demão escreve
`lerp(base, target, accum)` **absolutamente a partir do `base` CONGELADO**, e o passe
de auto-smooth roda **depois** dela, no mesmo dab
([`stroke_symmetry.rs:172`](../../../crates/ph2d-sculpt3d/src/stroke_symmetry.rs)).
O alisador achata o que a demão acabou de levantar, e como a demão **não lê o vivo**
(`from_live = false`, e é o `layer.cc` que o exige) ela **não recupera** o que foi
achatado — ela só reafirma a mesma altura, que o alisador achata de novo.
Em Blender essa composição **não colapsa**, e é aí que a paridade tem de ser lida.

### (B) HARDNESS espeta a demão mais que o Draw

```
  hardness   relevo    espeto   espeto/relevo
  0.00       0.07707   0.12122          1.573
  0.25       0.08622   0.13724          1.592
  0.50       0.09224   0.15855          1.719
  0.75       0.09632   0.26931          2.796     <- pico
  0.90       0.09737   0.22283          2.289     <- NAO-MONOTONICO

  CONTROLE (Draw)
  0.00       0.08738   0.13844          1.584
  0.50       0.11578   0.18906          1.633
  0.90       0.13420   0.23999          1.788
```

⚠️ **O `0,75 → 0,90` NÃO É MONOTÔNICO** — o espeto cai quando a dureza sobe.
Isso é assinatura de descontinuidade, não de afinação. Suspeito nomeado:
`apply_hardness_to_distances` ([`brush_scale.rs:117`](../../../crates/ph2d-sculpt3d/src/brush_scale.rs)),
que zera `t` abaixo de `hardness` — e **num verbo cuja curva é uma TAXA e não um
perfil** (gate `the_falloff_is_a_rate_and_not_a_profile`) isso faz a pegada inteira
saturar de uma vez, em vez de desenhar um ombro.

---

## §4 — A PERGUNTA DE PARIDADE que ninguém respondeu

**Nenhuma das duas composições foi conferida contra o Blender A CORRER.** Antes de
mexer no kernel, responda com a referência na mão:

1. O `SCULPT_do_layer_brush` do Blender passa pelo `apply_hardness_to_distances`?
   (Se **não**, o `hardness` não devia alcançar a demão — e o eixo (B) fecha sem
   tocar na lei.)
2. Como o Blender compõe **auto-smooth com um brush que satura**? A ordem, a força,
   e contra que posições o smooth mede (vivo ou `orig_data`).
3. O `layer.cc` mede as distâncias contra `orig_data.positions`
   **incondicionalmente** — isso já está portado e gateado. O que **não** está
   conferido é o que acontece **depois**, no passe de alisamento.

⚠️ **O oráculo EXTERNO já existe e é o padrão desta linha:**
[`docs/3D/ferramentas/blender_sculpt_oracle.py`](../ferramentas/blender_sculpt_oracle.py)
roda o **Blender 5.2 de verdade** e imprime a tabela `(r, dz)` que o gate
`the_factory_curve_is_what_blender_running_deposits` consome. **Estenda-o para o
Layer com hardness e com auto-smooth** — é o único caminho para "bit-idêntico" que
não é uma leitura estática do `.cc`.
⚠️ Uma leitura estática **já enganou esta linha uma vez**: ela previa
`BRUSH_CURVE_CUSTOM` e o Blender a correr reporta `SMOOTH` (*um pincel não nasce
zero-inicializado; ele nasce do arquivo de startup*).

---

## §5 — Armadilhas desta linha (pagas, não repita)

- ⚠️ **A cwd volta ao primário** (§1). Custou-me um veredito errado contra um agente correto.
- ⚠️ **Um gate de kernel é CEGO à fiação** — os 12 gates do Layer passam e não dizem
  nada sobre o chip. E um **gate de seam é cego à LEI**. Precisa dos dois.
- ⚠️ **Fixture que não contém o fenômeno**: o espeto medido sobre a malha inteira lê
  os polos da esfera; a tabela sai com seis linhas iguais e parece um achado.
- ⚠️ **Oráculo byte-a-byte tem de reproduzir a ASSOCIAÇÃO**: `u*u*u*u` diverge de
  `(u*u)*(u*u)` por **um ULP** já em `t = 0,02`.
- ⚠️ **`cargo test -p` NÃO roda `cargo fmt --all -- --check`** — o tip desta linha
  esteve fmt-vermelho em cinco arquivos e **só o ship o via**.
- ⚠️ **Arch-gates que fatiam fonte por índice de BYTE panicam** em prosa portuguesa
  (acento, `⚠️`). Curado hoje em `the_armed_transform_is_shown.rs`, com `read_dir`
  ordenado junto (a ordem dele é *unspecified*).
- ⚠️ **Desfaça mutação com `cp` de um backup, NUNCA `git checkout`.**
- ⚠️ Rode as suítes em **debug além de release** (precedente: pânico só em debug).
- ⚠️ Gates de relógio (`--ignored`) exigem `--test-threads=1` e `load < ~5`.

---

## §6 — Fronteiras

- **NÃO integrar, NÃO `git push`, NÃO rodar `scripts/foundational-integrate.sh`.**
  Integração e ship são **só por ordem explícita do Enio**, via agente integrador
  dedicado (CLAUDE.md §0.7 · DIRETRIZ §1.5.3-1.5.4). A linha fecha a wave, escreve o
  handoff e **PARA**.
- `rayon` novo ⇒ **ADR novo**.
- Contrato congelado (§6 do CLAUDE.md) ⇒ **PARE e reporte**.

---

## §7 — Aberto além do Layer (contexto, não fila)

- Os defaults de fábrica por-tool do Blender moram num **`.blend` binário** ⇒ a W1 e
  o *Draw Sharp* são **decisão de produto do Enio**, não dívida de engenharia.
- Duas pistas de um agente, **ainda não medidas**: o `Draw` em modo `B` ficou
  **5× mais forte por dab**, e quatro verbos (`Blob`, `ClayStrips`, `ClayThumb`,
  `MultiplaneScrape`) nascem em `B` com **metade** da força.
- As duas curvas de domo (`Dome` / `Dome4`) voltaram ao catálogo hoje
  (`ALL` 10 → 12) — **isso é o que eu fiz na sessão, e não era o pedido.**
