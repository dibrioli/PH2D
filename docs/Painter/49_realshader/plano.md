# 49 — Plano: o «Óleo Real» no Wet Paint

> Especificação e medições: [`README.md`](README.md). Linha: `line/PainterWatercolor` (Modo L).
> Alvo: o Wet Paint ilumina a tinta como o RealShader + SoftShadows do Rebelle 8, com a direcção da luz
> que eles não têm. Critério de fecho: **render-and-look do dono** lado a lado com as capturas de
> `capturas/` (a régua é o produto do dono — memória `feedback_the_owners_reference_product_decides_the_law`).

## Ondas

| # | Onda | Entrega | Abre com | Cancela/Recua se | Gates |
|---|---|---|---|---|---|
| **W0** | **Medir antes de limitar** | (a) custo do passe de luz actual e de uma marcha `k=8/16/32` × `N=4/8` direcções a 1080p/4K no NVIDIA e na Radeon onboard (memória `project_onboard_radeon_runs_gpu_tests`); (b) **M10** do README: sombra a `84°/42°/15°` no Rebelle (a sessão `ferramentas/sessao.sh` já faz) ⇒ comprimento × altitude; (c) a escala de altura do Wet Paint (`sett+susp` em px de relevo) contra a espessura que o Rebelle mostra na mesma pincelada | — | — | tabela no doc, nenhuma constante sem ela (§0.0) |
| **W0b** | **As espátulas do Wet Paint** (dono 26/10: «rebelle tem várias espátulas») | o Wet Paint não tem nenhuma (W0(c)); o Rebelle tem 10 em `Oil/Knife`. Medir as 10 na folha `ferramentas/tracos_facas.py` (sessão `F42`): largura, arrasto/alisamento, degrau da borda, rasto; construir primeiro Knife Smooth e Knife Textured (as das capturas), depois Oily e as restantes, como ferramenta + presets do Wet Paint | W0 | a lâmina não se exprime com o motor de fios — mede-se se a composição já a exprime (CLAUDE.md §5.0) antes de lei nova | régua de forma contra a folha `F42`; olho do dono |
| **W1** | **Passe de óptica do Wet Paint na placa** | o campo `h = massa·escala + altura dos fios` sobe como textura; um compute pass novo (`ph2d-render`, irmão do `ImpastoLightPass`) devolve a cor iluminada; **Óleo Real desligado = o render de hoje ao byte** | W0 | paridade CPU↔GPU não fecha no ε documentado | neutro byte-idêntico; CPU de referência + `#[ignore]` de placa; mesmo bit por placa |
| **W2** | **Linear + reflexo de ambiente** | sRGB↔linear; normal Sobel; IBL: irradiância SH9 + especular pré-filtrado (split-sum, LUT BRDF), Fresnel `F0=0.04`; 2 ambientes CC0 (estúdio, janela) + «Clássico» (a luz de 4 lâmpadas de hoje); rotação do ambiente = direcção da luz | W1 | o look do dono reprova contra `A_base` | triagem de licença do HDRI por ficheiro; assets pré-filtrados gravados com sha256 |
| **W3** | **Cavidade e cor da fenda** | AO por horizonte (`N` dir × `k` passos de W0) + a lei medida `lerp(1, s·albedoᵏ, ao)` (`k≈2`, `s≈0,25`, M2); ajuste por régua: fenda/mediana por canal dentro de ±0,05 do Rebelle na *Knife Textured* | W2 | custo dobra o passe sem ganho visível | régua M1–M3 automatizada sobre render nosso × captura |
| **W4** | **Sombra macia** | marcha na direcção da luz, penumbra por cone; cai na tinta **e no papel** (excepção explícita à lei «papel nu não recebe nada», só para este termo, com gate); slider linear (M6) | W3 | perfil no papel não casa M6 (decaimento ~4 px/e-fold a 42°) | perfil M6 automatizado; neutro em `Sombra=0` ao byte |
| **W5** | **Controlos e smoke** | cartão do Wet Paint: Óleo Real · Ambiente · Profundidade · Brilho · Sombra · Altura da luz · Direcção; i18n, tokens; preset de fábrica = o look aprovado; cena de smoke `PH2D_COMPOSITE_SMOKE` nova com espátula/pincel grosso em claro, médio e escuro | W4 | — | registo de painel/knob; censo de controlo morto (quem lê, o leitor DECIDE?) |
| **W6** | **Exportação ampliada (o nosso NanoPixel)** | pesquisa com medição: re-simular o traço a N× reproduz a pincelada? se sim, export N× ilumina o campo ampliado; se não, amostragem bicúbica do campo + *detail normal* de fotos NOSSAS de tinta real | W5 | a reprodução a N× diverge do traço a 1× (régua de forma) ⇒ negativo escrito, export fica 1× | régua de forma 1× vs N×/N |

**Série justificada (§0.10):** W1→W4 são uma pilha sobre o MESMO passe (cada termo multiplica o anterior),
mas o lote de medições de W0 é UM só e cobre os parâmetros de W2–W4. W6 é independente e pode correr em
paralelo a partir de W1.

## Riscos e decisões já tomadas

- **Onde vive:** óptica em `ph2d-render` (o passe), lei de cor/AO em CPU de referência em `ph2d-wet-paint`;
  a shell só compõe (catraca `the_shell_only_shrinks`).
- **Custo:** o passe corre só na região suja e na vista; o tecto de `k`/`N` sai de W0 e diz de que
  recurso é (§0.0) — nada de «por ora».
- **Impasto (modo antigo):** congelado por decisão do dono (07/10); ganha o mesmo passe de graça só se a
  porta `relief` servir sem tocar no modo — não é objectivo desta linha.
- **Oráculo:** a sessão de captura (`ferramentas/`) precisa do ecrã REAL com a placa e dos monitores
  ligados (`kscreen-doctor --dpms on`; com DPMS desligado a janela fica `_NET_WM_STATE_HIDDEN` e o
  `import` falha com «missing an image filename»). O trial acaba ~06/11/2026: capturas a mais para as
  réguas de W3/W4 fazem-se ANTES disso.

## Estado 26/10 e correcção do plano

Smoke do dono 26/10: «Resultado interessante», mas (a) pixels brancos nas bordas, (b) sombra de *drop shadow*
(nenhuma DENTRO da tinta), (c) o relevo não sobe com o acúmulo. Detalhe: [handoff](HANDOFF_CONTINUACAO_oleo_real_W0_W1cpu_2026-10-26.md).

- **Estado:** W0 **feita** · W0b **parcial** (lâmina H3 só na sonda, falta rebordo e presets) · W1 **CPU feita e ligada, rejeitada no olho**; passe de placa por fazer.
- **Causa de (c), medida:** tecto duro de 3000 por célula em `trail/transfer.rs` (`:268–275`, `:285–288`, `:178–181`): a massa pára na 2.ª passagem; auto-sombra interior 0–2 %. A altura do Rebelle acumula sem tecto ([pesquisa](pesquisa_tecnologia.md)). Logo o campo da W1 não tem o que sombrear.
- **Nova onda inserida antes de W2:**

| # | Onda | Entrega | Gates |
|---|---|---|---|
| **W1b** | **Espessura sem tecto + campo sem suavizar + cobertura** | (1) `Grid::espessura` escrita antes do tecto, levada por arrasto/empurrão/levantar, com undo e gravação (modelo `grid/altura.rs`), `px_por_massa` recalibrado; (2) altura de sombra/oclusão sem suavizar, textura ∝ espessura, rebordo; (3) altura e normais pesadas pela cobertura + especular × cobertura + AA (cura dos pixels brancos); (4) cavidade tipo GTAO com rebote pelo albedo; (5) horizonte com pirâmide de máximos (sombra longa dentro da tinta barata) | `medida_oleo_real_acumulo` cresce com as passagens; neutro ao byte; olho do dono |

- **W1b 26/10 (noite): itens 1–3 feitos, aprovados no rumo pelo dono** («sombras interessantes entre a tinta! Bom!») — [handoff](HANDOFF_CONTINUACAO_oleo_real_W1b_2026-10-26.md). Itens 4–5 passam para a W1c.
- **W1c (nova, antes da GPU):** «porque parece Eevee antigo e não Cycles?» — oráculo Blender/Cycles corrido por script sobre o nosso campo; rebote (GTAO multi-ricochete), ambiente (a W2 sobe para aqui), penumbra que cresce com a distância, rolagem dos realces. [Prompt](PROMPT_NOVA_JANELA_W1c.md).
- **W1c 27/10: aprovada no smoke** («Realmente muito bom»; backup `backup/oleo-real-W1c-2026-10-27`) — [handoff](HANDOFF_CONTINUACAO_oleo_real_W1c_2026-10-27.md).
- **MISSÃO W2 (autónoma, 27/10):** E1 contraste da fenda · E2 HDRI como meia-esfera (difuso da tinta e do papel) · E3 material Principled por célula (metal, verniz, sheen, anisotropia, translucidez) · E4 18 presets parecidos com o Rebelle (Metallic Flat 3/Round 3/Splats, 5 Dabber, 10 Knife). [Prompt](PROMPT_MISSAO_W2_material_ambiente_pinceis.md).
- **W2 27/10: aprovada no smoke («MUito bom!», backup `backup/oleo-real-W2-2026-10-27`); dono decidiu o material = Principled simplificado (8 nomes do Blender, com SSS) → [MISSÃO W3](PROMPT_MISSAO_W3_material_motor_presets.md)** — E1 força/alcance da fenda · E2 ambiente no difuso por sector de fatia (corr. tinta estúdio 0,56→0,77) · E3 material por célula + Principled × Cycles (brilho do Estúdio curado) · E4 faca com rebordo, 4 facas aprovadas (fábrica 49 / 22); Metallic e Dabber 2–4/Fan candidatos. Custo do óleo a escorrer 28→17 s (bit-idêntico). [ESTADO](ESTADO_MISSAO_W2.md).
- **Ordem restante:** W1 GPU (`ph2d-render`, paridade com a CPU, K/N por placa) → facas H3 → presets Knife Smooth/Textured e lei do rebordo → W2 (IBL de foto nossa/CC0) → W3..W6 como na tabela (W3 absorve o GTAO se o item 4 de W1b o fizer).
