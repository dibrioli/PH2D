TROCA DE AGENTE — line/PainterWatercolor: o **plano 27** — a seção Shape vira a CASA dos pincéis em todas as
mídias. Esta janela: **W0 (medir, num lote)** e **W1 (Bristles como tipo do Shape, ao bit)**. Ordem do dono (13/10):
*«a seção Shape [é] a casa onde se cria e configura os pincéis … as texturas dos pincéis Wet Paint devem estar no
dropdown de textura e as configurações devem sair de Tuning e ir para Shape»*.

Leia SÓ: [27_plano_shape_casa_dos_pinceis.md](27_plano_shape_casa_dos_pinceis.md) inteiro (os fatos com endereço,
D1–D8, as ondas) · [26_ABERTOS.md](26_ABERTOS.md) (A1, C1, C2) · `docs/Painter/BUGS_painter.md` §Bug #47 · o topo
(R1–R9) de `docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG52_PONTA_CORTADA.md`.

⚠️ Ordem do dono (11/10): «faça como achar melhor e pare de me fazer perguntas». Só o Wet Paint evolui (07/10);
Digital/Impasto ganham o tipo Bristles mas o traço de hoje deles fica AO BIT. O Watercolor não se toca.

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
→ `line/PainterWatercolor`; `git status --short` limpo; `git cherry main HEAD` e `git rebase main` se o `main` andou;
`bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10`. Fish: laços em `bash -c`.

## W0 — medir primeiro, TUDO num lote (§0.10), sem mudar o produto
1. **A tabela do #47 inteira:** os 29 tipos do Shape (+ None) × Wet Paint e Digital, o mesmo traço, tinta por tipo
   (a sonda do #47 é a base: tela branca, o pincel de fábrica, um traço de 220 px, `Σ(255 − min(rgb))`). E as cerdas de
   hoje (Shape None no Wet Paint) como a linha de referência. Sonda `measure_*`/`sonda_*` `#[ignore]`.
2. **O oráculo:** o Rebelle (porta WebSocket sob Wine, `ferramentas/xtree.py`, `reinicia.sh`; memória
   `reference_rebelle_trial_installed_under_wine`) com 3 pontas texturadas sobre os NOSSOS gestos — gravar como
   fixture com cabeçalho em `fixtures/rebelle/`; medir como a textura da ponta e o dente do papel se combinam (D6) e o
   Shape × Grain (D7).
3. **A lei candidata do #47** (D6) medida no MESMO processo contra a de hoje (override em execução, intercalado, o
   mínimo): o Noise passa a pintar? quanto muda o traço redondo de cada preset de fábrica?
4. **Censo:** `BrushShape::{Flat, Fan}` do motor tem leitor fora do motor? Os 4 knobs das cerdas — quem os lê.
Escreva a tabela e a decisão de cada D no 27 (§6 «W0 medido»), com os números.

## W1 — Bristles como tipo do Shape (se a W0 não o mudar)
- `TextureKind::Bristles` (o 30.º) com os 4 parâmetros (Count, Strength, Size, Felt) como os de qualquer procedural;
  UM gerador (sai do `ph2d-wet-paint/src/brush.rs:61` para onde as duas crates o alcancem — sem duas cópias);
  amostrado em **píxeis** (ladrilho de 128 px, o `sample_bristle`), não na escala da ponta.
- Portões: o ladrilho AO BIT com o do motor de hoje para os mesmos knobs · Digital/Impasto pintam com Bristles ·
  todo traço de hoje (sem Bristles) ao bit · i18n do nome e dos parâmetros (zero string solta, HR-15) · o dropdown
  conta 30.
- ⛔ A W2 (o Wet Paint ler as cerdas do Shape, o Tuning esconder os 4, a migração dos presets) é a PRÓXIMA janela —
  deixe-a pronta no handoff, com o prompt 29.

R1: ~150 chamadas. Fecho: bateria pelo `verificador` (fmt, clippy `-D warnings --all-targets`, `nextest-impacted`),
handoff ≤ 8 KB com R1–R9, atualizar o 26 e o 27, commit por caminho, nunca push; smoke ao dono em passos numerados
(Digital: Shape ▸ Texture ▸ Bristles, mexer em Count/Size — a ponta de cerdas; e o que a W0 mediu, numa frase).
