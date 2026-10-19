# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W10 (a marca do artista é cobertura: sem resquícios nos cruzamentos) → `main` (2026-10-19)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, decisões e o porquê: [plano W10](../08_plano_W10_cobertura.md) (§ «W10 — resultado» no fim).
> ⚠️ **W8 e W9 ainda NÃO estão no `main`** (mesma branch, integram juntas): [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md)
> · [W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) continuam válidos para a parte delas. Este handoff só
> acrescenta a W10; a lei de W9 que ele substitui é o §2 item 1 e 11 do W9 (marca binária).
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono para esta janela: «não faça reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base com o `main`: `ad5f8fe39c`
  (`git merge-base main HEAD`). O `main` andou **3** commits de processo desde então (`git log --oneline
  $(git merge-base main HEAD)..main`): `451c3ad14a` (a vez da placa é de todo uso), `bdc7a38639` (runbook da vez),
  `100d34c8e0` (a pausa da régua da Luz). Em comum com a linha: só o `CLAUDE.md` (§2 lá, §5.1 cá).
- HEAD `d7abe5bab0`. Commits da W10: `08d607c660` (plano) · `2544ee3cc0` (test: a fixtura dos cruzamentos, vermelho
  antes) · `b144365664` (fix: a marca passa de bit a cobertura) · `593eff1452` (fix: a `=6 cruza` pinta em Digital,
  frase da consola, fotos) · `d7abe5bab0` (clippy do fecho).
- `git diff --shortstat main...HEAD` (W8+W9+W10): **131 ficheiros, +9 360 / −1 567**. Só a W10
  (`08d607c660..HEAD`): **35 ficheiros, +1 596 / −350**.

## §2 — O que a W10 entrega (pedido do dono: smoke de 19/10 reprovado, «resquícios onde os traços cruzaram»)

1. **A lei.** O Play mostra `atual + (1 − c)·(palco − B)`, premultiplicado: `c` = cobertura da tinta do artista (`u16`,
   CHEIA = 65 535 — em `u8` só, somava até ½ nível), `B` = a imagem POR BAIXO no 1.º toque (RGBA8 recto). Exacta para a
   composição «por cima» (associativa). `c = 0` ⇒ palco; `c` cheia ⇒ atual (a lei da W9).
2. **A força é MEDIDA pelo carimbo.** Um plano `u16` opcional atravessa as 3 rotas do Digital em `ph2d-painter-brush`
   (`stamp_band` por-pixel e em bandas, `blit_band` com carimbo em cache, `canvas_blit_band` com grão fixo à tela);
   cada dab compõe a sua alfa (`stroke_cover::compoe_forca`; sem Accumulate o tecto `a = add/(1−m)` telescopa para
   `m`). O carimbo da placa (GPU) não devolve a força ⇒ com o plano armado o lote vai pela rota CPU em bandas (mesma lei).
3. **Armar** (`stamp_dabs` → `a_forca_mede_se`): Digital, `PaintMode::Paint`, Mix ou borracha (EraseAlpha = por
   cima com A=0; no papel A = a·papel), sem mistura de pigmento, camada única (+ papel opaco: `B` = camada sobre o
   papel), sem vidro da aguada, sem luz do impasto, sem máscara de protecção/selecção, sem enchimento Solid. Fora
   disto, ou se nenhuma rota mediu, o passo fica com a lei da W9 (`c` cheia). Wet Paint, Watercolor, Impasto e outros
   modos de mistura ficam cheios (decisão do plano §5).
4. **`MarcaDoArtista`**: `c` `u16` + `B` em ladrilhos esparsos 64² (`Arc`); o delta do desfazer = ladrilhos antes/depois
   (run coalescido = 1.º antes, último depois). **FORMATO 2** no blob do projecto; o **FORMATO 1** (bits da W9) abre
   com `c` cheia. **Sem degrau de `PROJECT_SCHEMA`** (blob opaco; o formato do contentor das marcas não muda).
5. **A mistura na app** (`mistura_na_caixa`) por ladrilho; `B` premultiplicado em linha, `(1−c)` como `·resta >> 16`.
6. **Reescrever uma etapa editada** passa o fim novo POR BAIXO da tinta parcial (imagem `x + (1−c)(para − de)` e
   `B ← para` sobre o papel), desfazível (`comeca_a_reescrita` → delta de ladrilhos). Sprite sem Painter: sem mudança
   (o diff ⇒ `c` cheia); os texels parciais lá são saltados (§8 item 3).
7. **A ponta achatada e o grão deixam de cair em «o que mudou»** (medidos pelo carimbo) — fecha o plano §5 item 1.
8. **A cena `=6 cruza` pintava em Wet Paint dizendo «dois traços Digital»** ⇒ passa a Digital (como a `arrasta`); a
   frase da consola diz o meio real de cada variante.

### §2-bis — Medições (tabelas; o mecanismo está no plano)

**Perfil da força contra o byte** (sonda, Digital preto sobre branco, 128², ANTES de o plano existir): pincel redondo
padrão / duro / esparso contínuo ±1 nível; Accumulate desligado (força ½) 62 níveis; grão / forma / achatada não
cobertos ⇒ daí o plano no carimbo.

**Gate `a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio` (app), desvio máximo em níveis a 0,1 s / 0,6 s:**

| Pincel | W9 (antes) | W10 (depois) |
|---|---|---|
| macio | 199–231 | 2 / 2 |
| duro esfera | 199–231 | 1 / 1 |
| duro disco | (já ok) | 0 / 0 |
| grão a seguir o pincel | 199–231 | 3 / 3 |
| grão fixo à tela | 199–231 | 5 / 3 |
| força ½ | 199–231 | 3 / 3 |
| contínuo esparso | 199–231 | 1 / 1 |
| fim do clipe | — | 0 em todos |

Tolerância = piso medido do carimbo (o mesmo traço preto escalado sobre o branco contra sobre o fundo: 0–3 níveis) + 2,
tecto 6. Grão de mapeamento aleatório EXCLUÍDO: o sorteio depende da história do carimbo e a régua (ferramenta nova)
sorteia outro grão — 52 níveis medidos entre as duas pinceladas.

**Custo** (A3 `sonda_o_custo_da_marca`, 1024², `load` ~25): água sem marca **0,034 ms** média; água + traço Wet
marcado **0,747 ms** média / 2,989 máx (W9: 0,704 / 4,873). Mistura (`sonda_o_custo_da_mistura_parcial`, caixa 512²,
intercalada, mínimo): cheia 0,57 ms, parcial **1,57 ms** depois de optimizada (5,34 ms antes: premultiplicação por
texel + divisão). Memória: ladrilho 24 KB (`c` `u16` + `B`) só onde se tocou.

**Mutação 14/14 sangra** (controlos verdes, âncoras 1×): ignorar o plano (volta a lei da W9) · tirar `compoe_forca` em
cada um dos 3 núcleos · sobrescrever `B` no 2.º toque · ignorar `B` na mistura · ignorar o papel em `B` · sem
reescrita parcial · reescrita sem delta · `junta` guarda o 1.º depois · FORMATO 1 não saturado · fórmula do `compoe` ·
plano não zerado entre passos · papel não guardado. Os 3 sítios dos núcleos são apanhados pelo MESMO gate da app
(nenhum tem teste isolado).

**Gates reescritos (a lei mudou de propósito):** `marca_tests` (4) · `marca_cobertura_tests` (cobertura = a força que o
pincel pôs, piso ±3; achatada/grão medidos; sobreposição + desfazer/refazer) · `marca_painter_tests`
(`um_traco_digital_marca_o_que_mudou_e_nada_longe_dele`; NOVO `a_reescrita_passa_o_fim_novo_por_baixo_da_tinta_parcial`) ·
`palcos_marca_tests` (oráculo `esperado_pela_marca` em vírgula flutuante, ±1).

## §3 — Foundational / partilhado tocado (tudo ADITIVO; muda assinaturas)

| Crate | O quê |
|---|---|
| `ph2d-painter-brush` | as `pub fn` `stamp_dab_textured_masked`, `stamp_dab_textured_masked_with`, `blit_stamp`, `blit_canvas_cached` ganharam um parâmetro `forca: Option<&mut [u16]>` (chamadores no teste de paridade do `paint-gpu`, nos testes do pincel e na ferramenta passam `None`); `pub fn stroke_cover::compoe_forca` NOVO (o módulo é `pub(crate)`); rotas `stamp_band` / `blit_band` / `canvas_blit_band` |
| `ph2d-tool-painter` | `macro_track/marca.rs` (c `u16` + `B` em ladrilhos, FORMATO 2), `undo_marca.rs` (delta de ladrilhos, `comeca_a_reescrita`), `tool/paint/stamp_route.rs` (`a_forca_mede_se`, plano armado), reescrita; o módulo `tool::papel` passou a `pub(crate)` |
| `ph2d-app-painter` | `macro_track/mod.rs` (`mistura_na_caixa`), `marcas.rs` (FORMATO 2 / leitura do 1), `reescrita.rs`, `macro_track/caminho.rs` (a `=6 cruza` em Digital, frase por variante) |

**Contratos congelados (§6) — NÃO tocados.** Prova (sai vazio, exit 1; corrida em `d7abe5bab0`):

```
git diff --name-only main...HEAD | grep -E 'ph2d-editor-core/src/tool|ph2d-vec|ph2d-vector|ph2d-nodegraph/src/(op|manifest|cook)'
```

Sem i18n, sem `Cargo.toml`, sem `project_schema` na W10 (`git diff --name-only 08d607c660..HEAD | grep -E
'Cargo|i18n|project_schema'` sai vazio).

## §4 — O que um merge pode partir

- **Outra linha que chame as 4 `pub fn` do `ph2d-painter-brush` do §3 deixa de compilar**: acrescente `None`.
- Linhas do Painter que toquem `stamp_route.rs`, `stamp_cache`, `stamp_banded`, `bands.rs`, `stamp.rs`: conflito
  provável; a rota é aditiva (o plano `forca` é opcional), a resolução é manter os dois.
- `MarcaDoArtista`/`DeltaDaMarca` mudaram de forma (bits → ladrilhos): quem tocou `marca.rs`, `undo_marca.rs`,
  `marcas.rs` na W8/W9 está nesta mesma branch; outra linha nunca lá tocou.
- `CLAUDE.md`: o `main` mudou a §2; a W10 só muda a entrada das Macros na §5.1 (ver «Para o integrador»).

## §5 — Contadores (DELTA sobre o merge-base)

Crates novas **0** · chaves i18n **0** (na W10; +2 da W9) · ADR 0 · `PROJECT_SCHEMA`: **a W10 não mexe** (o 192 é da W9;
recontar contra o `main` do dia com `python3 scripts/schema-recount.py`) · contratos 0 · cenas de smoke das Macros
`=1..6` (a próxima é `=7`).

## §6 — Prova de fecho

Bateria do fecho sobre `d7abe5bab0` (sem rebase, ordem do dono para esta janela):

| Portão | Resultado |
|---|---|
| `cargo fmt --all` | ✓ limpo |
| `PH2D_SUITE=fecho … cargo clippy --workspace --all-targets -- -D warnings` | ✓ (a 1.ª corrida reprovou 3 + 3 lints de teste, curados em `d7abe5bab0`) |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ · ✓ (em `593eff1452`; `d7abe5bab0` só mexeu em 6 `.rs` já cobertos pelo clippy) |
| `file_loc_caps` (shell) | ✓ 4/4 |
| `censos-da-arvore-combinada.sh` | ✓ 121/121, 12 de 12 censos |
| `PH2D_SUITE=fecho BASE=$(git merge-base main HEAD) nextest-impacted.sh` | ✓ **18 757 / 18 757**, 11 163 saltados, 834 s (load ~97 no início) |
| Prova de mutação | 14/14 sangraram (§2-bis) |

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`): «Finished `smoke` profile [optimized] target(s) in 0.25s», zero `Compiling`.

`agent-loop-profile.sh` (20 sessões):
```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                164   alvo: <= 800
  ✗ cargo test : cargo check                459 : 159   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.0%   alvo: <= 2%
  ✗ contexto relido por passo (media)         283 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               64 mil   alvo: <= 80 mil
```

## §7 — Smoke do dono e fotos

Binário: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=cruza ./target/smoke/ph2d-host-desktop`
2. A cena corre sozinha: duas pinceladas gravadas, **Parar** (aos ~5,2 s), uma pincelada vertical por cima das duas,
   **⏮** (~6,9 s) e **▷** (~7,4 s). Não clique em nada.
3. O que tem de acontecer: ao tocar, a vertical aparece inteira desde o início e as duas gravadas nascem depois;
   nos pontos onde se cruzam não fica nenhum «nó» ou mancha escura — só a vertical limpa, e a gravada a nascer
   por baixo dela.
4. Deu errado se, onde a vertical cruza uma gravada ainda por nascer, se vir um pedaço dessa gravada antes do tempo
   (um «nó» ou «x») ou uma faixa branca no miolo da vertical.

Fotos (`smoke_2026-10-19_W10/`; comando acima, Parar 5,21 s, vertical, ⏮ 6,91 s, ▷ 7,41 s; limpas, sem nós):
[0,25 s: só a vertical](smoke_2026-10-19_W10/cruza_digital_play_0.25s_so_a_vertical.png) ·
[1,67 s: a de cima nascida](smoke_2026-10-19_W10/cruza_digital_play_1.67s_a_de_cima_nascida.png) ·
[3,27 s: as duas](smoke_2026-10-19_W10/cruza_digital_play_3.27s_as_duas.png) ·
[zoom do cruzamento de baixo aos 0,25 / 1,67 / 3,27 s](smoke_2026-10-19_W10/cruza_digital_zoom_cruzamento_de_baixo_0.25_1.67_3.27s.png).
Antes da cura da cena, a mesma cena em Wet Paint ainda mostrava os nós (o Wet Paint mantém a marca cheia, plano §5).
Smoke do dono depois da W10: **por fazer**.

## §8 — ABERTO

1. **Wet Paint, Watercolor, Impasto, outros modos de mistura e pilhas de várias camadas ficam com `c` cheia** ⇒ os
   cruzamentos em Wet Paint ainda mostram o nó (decisão do plano §5). Se a foto do dono do «pincel macio com grão»
   era Wet Paint, pede onda própria (o compósito molhado é `blend_over(base, wet, pa)` — medível, mas `pa` continua a
   mudar depois do passo).
2. O carimbo da placa não devolve a força: com uma sprite marcada os lotes do pincel simples vão pela rota CPU em
   bandas (custo não medido na app; os testes não têm dispositivo, o ramo não é exercido no CI).
3. Sprite sem Painter + reescrita: os texels parciais são saltados (o desfazer global não leva `B`) — um halo fraco do
   fim antigo sob tinta parcial, só no documento; o Play está certo (o palco é a gravação nova).
4. Passo misto: um texel escrito por uma rota que mede E por um escritor que não mede, no mesmo passo (p. ex. o acorde
   de fecho do Solid) ficaria sub-contado; o Solid está excluído do armar.
5. A pincelada em voo mostra `c` cheia (disco provisório) até o passo fechar.
6. As fotos do dono de 19/10 não estavam em disco; a fixtura reproduz a cena descrita.
7. Seguem de pé os itens 1–3 e 5–8 do [§8 da W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md) (o 9 foi feito
   aqui, para o Digital).

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff. Frase da natureza: «a marca do artista (onde se pintou depois de gravar fica por cima;
a água não marca)» → «a marca do artista é cobertura (a tinta pintada depois de gravar por cima do clipe; a água não
marca)». Mantenha a entrada ≤ 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
