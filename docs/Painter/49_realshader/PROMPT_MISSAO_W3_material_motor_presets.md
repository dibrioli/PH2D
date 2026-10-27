# Prompt da janela nova — line/PainterWatercolor: MISSÃO W3 do Óleo Real (autónoma, do início ao fim)

```
Linha line/PainterWatercolor (Modo L) — MISSÃO W3 do Óleo Real, AUTÓNOMA: o dono não está. Não faça perguntas, não peça permissões, não pare à espera dele. Faça tudo por AGENTES PARALELOS, por etapas, a janela principal só decide e integra — guarde o contexto dela.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/49_realshader/PROMPT_MISSAO_W3_material_motor_presets.md e
docs/Painter/49_realshader/HANDOFF_CONTINUACAO_oleo_real_W2_2026-10-27.md inteiros; docs/Painter/49_realshader/ESTADO_MISSAO_W2.md;
docs/Painter/49_realshader/oraculo/README.md e docs/Painter/Pinceis/REGRAS_DE_CUSTO.md. Depois execute as etapas M → K → P deste ficheiro sem parar.
Ao terminar TUDO (fecho incluído, mensagem final escrita), desligue o PC: systemctl poweroff (confirme antes que nada desta linha corre).
```

## Estado ao abrir

- W2 aprovada no smoke (27/10, «MUito bom!»). Backup: tag `backup/oleo-real-W2-2026-10-27`. Confirme `git log -3` e
  `git status`. Se existir `ESTADO_MISSAO_W3.md`, RETOME daí.
- O Rebelle Trial pode estar aberto (porta WebSocket; receita em `project-memory/reference_rebelle_trial_installed_under_wine.md`):
  use-o para fotos novas quando a bancada pedir. Nunca o mate.

## Regras da missão

Valem as regras 1–11 de [`PROMPT_MISSAO_W2_material_ambiente_pinceis.md`](PROMPT_MISSAO_W2_material_ambiente_pinceis.md)
§«Regras da missão autónoma» (sem perguntas · `grep -c aprova-sem-perguntar .claude/settings.local.json` ≥ 1 antes
de lançar agentes · formas proibidas em todo briefing · agente devolve ≤ 20 linhas · ≤ 3 agentes em paralelo · ≤ 2
rondas por item · revisor `sonnet` · ficheiros disjuntos entre agentes paralelos · `ESTADO_MISSAO_W3.md` depois de
cada etapa · testes POR NOME, clippy só no fecho, sem suíte completa · oráculos CORRIDOS · neutro bit-idêntico ·
OLHE a folha · sem push nem integração). Lições da W2 que viram regra:

1. **Reprovado nunca entra na fábrica.** Candidato fica em `docs/Painter/Pinceis/W2_*_candidatos/` + `EM_ESTUDO_W2`
   (gate `os_candidatos_da_w2_sao_as_suas_receitas`); aprovado entra com `Receita` em `familias/oleo_acrilica.rs`,
   regenerado com `PH2D_REGERA_SO`, contagens CONTADAS (fábrica, por meio, aprovados, CLAUDE.md §5) e
   `a_largura_e_a_prometida` sem linha nova (`refina_a_promessa`).
2. **A bancada compara com o Óleo Real LIGADO** (`PH2D_EXAME_OLEO=1`) — o Rebelle foi fotografado com RealShader.
3. **Pasta de trabalho própria por agente** no scratchpad (`<scratchpad>/<família>/`): dois agentes colidiram na W2.
4. Agente que precisa de código fora dos seus ficheiros DIZ no retorno; a janela agenda.

## Etapa M — o material Principled simplificado, COM SSS (decisão do dono 27/10) · 2 agentes

Exactamente 8 controlos, nomes do Blender, no PINCEL (Tuning → Material, por célula) e no cartão Real Oil (tela
inteira), com os mesmos nomes e a mesma ordem: **Metallic · Roughness · IOR Level · Coat · Coat Roughness · Sheen ·
Subsurface · Subsurface Radius**.

- Saem do painel e do preset: Spec tint, Anisotropy, Translucency, Reflectivity, Map scale, Map strength. O mapa de
  reflexo do metal é o do seletor **Environment** (saia o chip «Reflect map», ou fique só se a medição provar que
  Environment ≠ mapa do metal é preciso — escreva-o). ⚠️ Preset ou macro gravados com os campos antigos continuam a
  abrir: leitor tolerante (campo desconhecido ignorado; Translucency → Subsurface se a lei o permitir).
- **Roughness** passa a ser UM número absoluto como no Blender (hoje: Gloss do cartão × `roughnessMul`): mapeie de
  forma que os presets aprovados e a imagem de hoje fiquem bit-idênticos no neutro (gate), e diga a conversão.
- **IOR Level** = o Specular do Blender 4 (0,5 = IOR 1,5 = hoje).
- **Subsurface** (peso) e **Subsurface Radius** (distância em px de tela; a cor do espalhamento = a cor da tinta, como
  o Principled faz por omissão): a luz entra na tinta, espalha-se e sai ao lado — a tinta fina e as bordas ficam
  luminosas e com a cor mais saturada. Padrão-ouro a decidir MEDINDO contra o Cycles (Principled Subsurface, Random
  Walk, MESMA malha da tinta, receita em `oraculo/README.md`): difusão separável em espaço de ecrã com perfil por
  canal (Jimenez) sobre a irradiância, pesada pela ESPESSURA da tinta (`Grid::espessura`/altura) e com o papel por baixo
  a devolver a luz que atravessa a película fina. Neutro (Subsurface 0) = imagem de hoje, bit a bit. Régua: Δ de cor
  nas bordas e na tinta fina, saturação, correlação × Cycles; custo em ns/célula (tecto do hardware, §0.0).
- Combinação pincel × cartão: a de hoje (máximo para neutro 0, produto para o resto) — reescreva-a para os 8.
- Divisão: agente M1 = sombreado + SSS + oráculo (`oleo_real.rs`, `oleo_ambiente.rs`, `material.rs`, `oraculo/`);
  agente M2 = os nomes/controles no pincel e no cartão (painéis, ids, i18n, BrushSettings, knobs, macro, costuras,
  leitor tolerante). Contrato entre eles: escreva-o PRIMEIRO na janela (os campos novos de `MaterialCelula`, molde
  `5464db050b`) e só depois lance os dois.

## Etapa K — o motor que falta para os 13 presets · ≤ 3 agentes, por FAMÍLIA de defeito (§0.10)

A fila medida está nas tabelas de `ESTADO_MISSAO_W2.md` e nos relatórios `docs/Painter/Pinceis/W2_{Dabber_E4,Metallic_exame,Knife_E4}.md`.
Agrupe por família — um bloco cada, um agente cada:

1. **Bancada rápida com o óleo** (PRIMEIRO, antes dos outros: cada foto custa 130–600 s e a W2 parou aí): a foto da
   bancada não precisa de recompor a cada evento — componha o óleo UMA vez no fim de cada foto (o resultado é o mesmo
   bit a bit: gate) ou o que a medição mostrar; alvo: foto com óleo ≤ 3× a sem óleo.
2. **Textura que escala com o tamanho do pincel** (hoje fixa em px de tela): buracos do Dabber 2, lascas angulares do
   Dabber 3, pó do Metallic Splats (tamanho relativo e mais pálido; `spray_count > 1` não pousou nada).
3. **Geometria da faca e da cerda:** faca que NIVELA num planalto em vez de tubo; lâmina fixa que estreita com a
   pressão; campo próprio da crista que alimenta a altura da SOMBRA sem mudar o Oil Bristle (0,44/1,26 × Rebelle
   0,40/1,39 medido); a risca branca vertical a meio do traço recto (Stain, Smooth 3, Textured); fios do Fan
   ATRAVÉS do traço a 200 px; pontas que esfarelam sem fios (Metallic Flat); relevo grumoso de vãos fundos (Metallic
   Round).
4. **Sombreado:** o halo cinza no papel à volta de um traço liso (Metallic Flat, todas as facas: o Rebelle não o tem);
   o metal nas encostas (p90 0,60 × 1,16 do Cycles; a faixa Metallic sai vinho com riscas quase pretas) — depois da M.

Cada item: neutro bit-idêntico para os pincéis que não o usam (gate), medição antes/depois, ≤ 2 rondas.

## Etapa P — os 13 presets · ≤ 3 agentes (Metallic · Dabber · Knife), só depois de M e K

- Os 13: **Metallic** Flat 3 · Round 3 · Splats; **Dabber** 2 · 3 · 4 · Fan; **Knife** Smooth · Smooth 2 · Textured ·
  Stain · Soft · Dry 2. Partem dos candidatos (`W2_*_candidatos/`, valores nos relatórios W2) e dos motores da K.
- Método da W2, igual: bancada (`04_BANCADA.md`, `05_REVISAO_CEGA.md`, `ferramentas/exame.sh`, `exame_prancha.py`,
  `exame_regua.py`) com `PH2D_EXAME_OLEO=1`; oráculo = fotos Rebelle `fixtures/rebelle_bateria/OIL_AND_ACRYLIC/`
  (`INDICE_W2.md`); foto nova pelo Rebelle aberto se faltar um gesto; revisor cego `sonnet`, UM por ronda (pode cobrir
  vários presets), ≤ 2 rondas por preset. Critério: memória `feedback_a_brush_that_passes_the_ruler_can_still_be_useless`
  — distinto dos irmãos (Soft e Dry 2 ≠ Dry; Stain ≠ Wax Scrape; Round ≠ Dabber).
- Os Metallic usam o material da M (Metallic + Roughness + Coat por célula).
- Aprovados entram na fábrica pela regra 1; os outros ficam candidatos com a diferença escrita.

## Fecho (a janela, no fim de tudo)

1. `PH2D_SUITE=fecho` clippy `--all-targets` + fmt das crates tocadas (sem `#[allow]`); testes por NOME de tudo o que
   nasceu; provas de mutação (agente `mutacao`) nas leis do SSS, do Roughness bit-idêntico e do leitor tolerante.
   ⛔ Sem suíte completa. Vermelhos da base: lista no handoff W2 — não são desta onda.
2. Folhas: `out/W3_folha.jpg` (W2 | W3 | Cycles, com uma faixa por controlo do material, Subsurface em destaque) e
   `Pinceis/pranchas/W3_presets.jpg` (os 13 × Rebelle, veredito na etiqueta).
3. `ESTADO_MISSAO_W3.md` completo + `oraculo/README.md` + linha no `plano.md`.
4. `cargo build -p ph2d-host-desktop --profile smoke`; mensagem final ao dono: curta, PT-BR, sem jargão, o que ele
   consegue FAZER agora + smoke em passos numerados (1 comando com o `cd`; 2 onde clicar, com os nomes da tela; 3 o
   que tem de acontecer; 4 como saber que deu errado) + o que ficou por fazer com o porquê.
5. Handoff e prompt seguinte só depois do veredito do smoke. Depois da mensagem: confirmar que nada desta linha corre
   (`pgrep` de cargo/rustc/blender; transcripts das outras sessões sem escrita nos últimos 30 min) e `systemctl poweroff`
   (ordem do dono; polkit `CanPowerOff=yes`, sem senha).
