# 04 — PROMPT para o agente da linha Painter: «Wet Painter» (só o Wet Paint evolui)

> **Decisão do dono (2026-10-07):** *«não tenho intenção de mudar os modos Impasto, Digital ou Watercolor.
> Apenas aperfeiçoar Wet Paint.»* Este prompt re-aponta o plano do [doc 01 §4](01_o_que_o_rebelle_tem_e_nos_nao.md)
> ao meio `PaintMedia::WetPaint` e às suas crates. Cole-o inteiro na janela nova (ou chame
> `/pd-linha-abrir WetPainter "<objectivo abaixo>" "<leituras abaixo>"`).

---

Abra a linha `line/WetPainter` em Modo L: execute o BLOCO de `docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md`
(FASE 1 inteira: `bash scripts/hw-profile.sh`, worktree em `Worktrees/line-WetPainter`, mergiraf) e as regras
A–J, que valem até ao fim da sessão. ANTES de ler qualquer ficheiro: `cd` na worktree, `pwd`,
`git branch --show-current`.

## Objectivo da linha

Fazer do **Wet Paint** o «Wet Painter»: UM motor (o fluido de `ph2d-wet-paint` + a ponte `wetpaint*.rs` de
`ph2d-tool-painter`) que simula **todas as mídias** — óleo, acrílico, aquarela, guache, tinta (pena e pincel),
lápis/grafite, carvão, pastel seco e oleoso, giz, marcador, airbrush — por **presets em ficheiro**, com o traço
medido contra o Rebelle 8 pela bancada que já existe. **Digital, Impasto e Watercolor não mudam um byte**
(gate abaixo). O Rebelle é ORÁCULO que se corre (CLAUDE.md §0.9): o que ele faz, mede-se pela porta WebSocket
e vira gate; nada dele se lê nem se copia (texturas, Mixbox).

## Leia primeiro (nesta ordem; só isto, depois o código)

1. `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` + DIRETRIZ §1.5 (processo).
2. `docs/Painter/NewFeaturesWetPaint/00_README.md` → `01_o_que_o_rebelle_tem_e_nos_nao.md` (§0, §2, §4, §5)
   → `02_presets_para_wet_painter.md` (§2, §3, §5, §6) → `03_tracos_medidos_rebelle_x_ph2d.md` (tudo).
3. O Wet Paint como está: `docs/Painter/ph2d_wet_paint/SPEC.md`, ADR-0134 e ADR-0147, docs 21, 22, 23,
   44 §1/§5/§6, 46 (itens 8–9), BUGS #35–#46, o handoff `handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-07.md`.
4. Código: `crates/ph2d-wet-paint/src/{brush,trail,flow,solver,drying,paper,render,tuning}.rs`,
   `crates/ph2d-tool-painter/src/tool/paint/{wetpaint.rs,wetpaint_settings.rs,wetpaint_commit.rs,wetpaint/}`,
   `crates/ph2d-panel-wet-tuning/`, a secção Wet Paint de `ph2d-panel-painter-layers`, e a sonda
   `crates/ph2d-tool-painter/tests/it/sonda_tracos_por_meio.rs`.

## A cerca (gate red-first, antes do 1.º commit)

- `the_other_media_are_untouched`: o diff da linha contra o merge-base não toca `stamp_dab`, `watercolor_*.rs`,
  `impasto_*.rs`, `substrate_*.rs`, `dynamics.rs`, `spec.rs` **excepto** campos novos append-only usados só
  pelo Wet Paint; e `PH2D_SONDA_SIZE=28 … sonda_tracos_por_meio` dá PNG **byte-idêntico** para `digital`,
  `watercolor` e `impasto` (guarde os três de hoje como fixture, `docs/Painter/NewFeaturesWetPaint/fixtures/`).
- Os 17 206 testes de `nextest-impacted` continuam verdes; `the_shell_only_shrinks` não sobe.

## As ondas (ordem = dependência). Cada uma: o que entrega · como se prova

**W0 — Bancada.** Ponha a porta do oráculo a correr da worktree (`01 §6`: `arranque.sh`, `gera_eventos.py`,
`ws_eventos.mjs`, `mede_traco.py` — cópias em `NewFeaturesWetPaint/ferramentas/`). Acrescente à sonda os
presets do Wet Paint (um PNG por preset, `PH2D_SONDA_PRESET=<nome>`). Grave como fixture as 12 pranchas do
Rebelle já medidas (`03 §2`) e **meça o Wet Paint de hoje** com os mesmos gestos — é o lado «antes» de
todos os gates. Prova: um teste `#[ignore]` que imprime a tabela dos dois lados e um gate normal que lê a
tabela gravada e reprova se a largura de 40 px deixar de ser 40 ± 2 em TODOS os presets (hoje o mesmo número
dá 28 px no Wet Paint e 58 no Digital — o Wet Painter passa a prometer largura em px).

**W1 — Presets em ficheiro + painel de volumes** (`02 §5`). Crate nova `ph2d-brush-presets`
(`*.ph2d-brush.png`: miniatura + zTXt JSON `schema:1` + texturas NOSSAS; fábrica em `assets/brushes/WetPainter/
<Mídia>/<Grupo>/`, do utilizador em `~/.local/share/ph2d/brushes/`). Painel: grupos = pastas, favoritos,
recentes, *Save as New / Save as Default / Reset*, e os **5 volumes** no topo (Size px · Opacity · Water/
Oiliness · Pressure · Length) com presets rápidos (`1,3,5,10,…,100`) e a política *guardar volumes e modo /
só volumes / não guardar*. Prova: round-trip byte-idêntico de cada preset de fábrica; `the_factory_presets_
are_counted`; campo desconhecido não falha a leitura; miniatura determinística por hash (o corpus de regressão
do motor inteiro).

**W2 — Fios com física própria** (01 L1; o maior ganho). No `brush.rs`/`trail.rs`: cada fio ganha
**pressão própria** (Perlin por fio × `random_ratio`), **largura ondulante** ao longo do traço (`wave_length`
400–1000 px, `jitter`, `min_width`), **oscilação** lateral (0–12 px), três populações (Thin · Thick · Edges
com curvas de pressão e de inclinação), `Brush Form` (0 = os fios reposicionam-se com a pressão; 1 =
escondem-se), **Random Span** e a **cor com ruído HSLuv por fio e por traço**. Prova (a régua do 03): a
40 px, `textura ≥ 5 %` E `CV ≤ 0,02` E fracção do período do resíduo `< 15 %` (Rebelle Bristle Flat: 5,9 %
/ 0,011 / 9 %; Wet Paint hoje: 14 % em carris fixos). Controlo: com oscilação e onda a 0 reproduz o PNG de
hoje ao byte.

**W3 — Espessura e dois tipos de fio** (óleo/acrílico/pastel oleoso dentro do Wet Paint). Um campo de
**altura** (`bump`) no grid do fluido, escrito pelos fios *Add* (cristas) e *Remove* (sulcos), com `Max
Impasto Height`, `Smudge`, `Thick`, e a **oleosidade** como viscosidade do fluido (`oiliness` 0–1 → menos
difusão, mais arrasto) com `length_oiliness_fadeout`. A luz vem da função de iluminação que o Impasto já tem
— **chamada como biblioteca, não alterada** (se precisar de uma assinatura nova, append-only num ficheiro
irmão). Prova: o óleo do Wet Painter a 40 px dá relevo legível (`textura ≥ 5 %` com `borda/miolo 0,8–0,9`
como o Bristle Flat dele) e o acrílico seca sem difusão (largura A = B).

**W4 — A lei do papel** (01 L4). `paper_strength_curve` **por pressão** (cai para 0 com pressão máxima),
`paper_contrast`, `paper_scratch {Auto, Light, Dark}`, **textura de fundo** que fura nos vales/picos
(`INV_MULTIPLY` nos lápis, `HEIGHT` nos óleos), grão com **DPI do documento**, e **comportamento por papel**
(`PaperBehaviour {absorbency, rewet, texture_influence, edge_darkening, diffusion_speed, drips{on,size,
length}, granulation{strength,density,texture}}`, presets Default / Hot Pressed / Cold Pressed / Rough /
Japanese) a mapear nos `WetKnobs` por grupo. Prova: lápis 8B do Wet Painter: `textura ≥ 8 %` e `borda/miolo
< 0,5` (o dele: 9,1 % / 0,32); a mesma aguada em Hot Pressed vs Rough difere em `textura` e `borda` nos
sentidos do Rebelle (`PaperInfluence` 3 vs 6).

**W5 — A lei da tinta** (01 L5). Modos **Paint / Paint & Mix / Paint & Blend / Blend / Erase** por curvas de
pressão com **limiar da queda** relativo ao máximo do traço; **Loading + Length** (a tinta acaba; passa a
Blend); **Dirty Brush** que persiste ENTRE traços (o `Pickup` de hoje é por dab) e **MultiColored**;
**Glaze/Normal** por preset; **Default Color**; **Average Smudge Color** por fio. Prova: PRIMEIRO a bancada
do Rebelle (03 §4: pincel REDONDO a cruzar + 3.º traço fora, `SET_RANDOM_SEED 7`), depois o nosso:
`cor_miolo` do 3.º traço desvia-se do vermelho puro na mesma direcção; com `Length` baixo a cobertura cai ao
longo do traço como a `paint_fadeout_curve (0,1)→(1,0)`.

**W6 — Dinâmicas** (01 L6). Na entrada do Wet Paint (`wetpaint.rs`): **pressão com piso** (`pressure_size_
curve (0,0,6…0,9)→(1,1)`: o traço nunca some), **tecto** (`pressure_max` 0,25…1: o grafite nunca é preto),
**pressure smoothing** (média móvel 0…9) e **Pulled String**, **inclinação em dois eixos** (altura E largura
do feixe de fios, `tip_tilt_curve` e `tip_tilt_width_curve`, só a partir de ~42°), rotação do feixe
`{None, FollowTrajectory, PenTilt, PenRotation}` — o **barrel** entra no `CanvasPointer` como campo
append-only (foundational, Modo L: projecte para isolamento, reporte se a integração conflitar). Prova: rampa
0,15→1→0,15 dá taper `≥ 60 %` nas pontas (Rebelle 35/42/33); 8B a 55° dá largura `≥ 1,2×` e textura menor
(37/30; 5,2 % vs 9,1 %).

> ⚠️ **As gotas e a inclinação da tela saíram desta lista para a W1b** (05 §3, 06 — decisão do dono 08/10):
> gravidade na TELA (direita de fábrica), gotas no PAPEL, água de fábrica baixa, o mecanismo da gota no motor.

**W7 — As mídias secas e a água como volume.** Caminho **água 0** barato (sem solver) para lápis, carvão,
pastel, giz, marcador, airbrush, com a lei do papel e «não difunde sobre tinta pouco molhada» (manual);
**Water** como volume do pincel (0–100) na aquarela/tinta/acrílico com `length_water_reduction`; marcador =
grão preso à tela + glaze 0,3 + modos de efeito; airbrush = fluxo por pressão, cone por tilt; tinta = água
que escorre no molhado. Prova: a prancha dos 8 meios do Wet Painter reproduz a do Rebelle (03 §2) dentro
do vale medido nas sete métricas (barra com os DOIS lados, método §3 passo 6).

**W8 — Painel do Wet Painter.** Preset browser + volumes (W1) + secções *Stroke / Bristles / Paint / Paper*
(os knobs de W2–W6, cada um com dependência esmaecida e sem controlo morto — doc 45/46), **Tilt** (dial +
Lock to Canvas) e **Blow** com Strength. Prova: `censo_dos_controlos` sem controlo morto; cada knob tem gate
de «morde» (mutação).

Antes de codar: **diga-me o plano em waves com o que cada uma entrega e como se prova**, e MEÇA o Wet Paint de
hoje (W0) — a barra de cada gate sai do vale entre o nosso «antes» e o Rebelle, nunca de um número escolhido.

## Regras da linha

- CLAUDE.md §0 inteiro. **Padrão-ouro sem custo** (§0.6): gaps in-scope fecham na sessão; nada «por ora».
- Foundational pode tocar (ADR-0107) projectando para isolamento (ficheiro irmão / ponto append-only).
  Contrato congelado (§6) ou rebase a conflitar fora dos seus ficheiros: PARE e reporte.
- Inner loop = `bash scripts/cargo-check-narrow.sh <crate>`; testes/clippy/auditoria **1× no fecho**
  (`/pd-linha-fechar`); comando pesado por `bash scripts/ph2d-run.sh`.
- **Problemas equivalentes num único bloco** (§0.10): as medições das ondas planeiam-se à partida e correm
  intercaladas numa rodada.
- Cada knob novo: *o painel escreve onde · quem lê · o leitor DECIDE?* (§5.0). Cada tecto: MEÇA antes
  (§0.0), com a tabela ao lado.
- O oráculo corre-se **da worktree por script** (01 §6); as imagens dele sobre os NOSSOS gestos são saída
  livre e vão para `fixtures/` com cabeçalho (gesto, preset, volumes, semente). ⛔ Nunca as texturas dele,
  nunca o Mixbox, nunca o executável.
- Você fecha a linha, escreve o handoff (DIRETRIZ §1.5.9) e PARA. Não integra, não pusha.
- **No handoff, o smoke do dono em PASSOS NUMERADOS** (§0.8): o `cd` + comando, onde clicar com o nome que
  aparece na tela, o que tem de acontecer, como saber que deu errado — e a cena do smoke fotografada antes
  de a mandar. O dono testa óleo, aquarela, lápis e marcador no Wet Painter; os outros três modos têm de
  continuar exactamente iguais.
