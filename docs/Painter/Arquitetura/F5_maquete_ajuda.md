# F5 — maquete do cartão de ajuda ilustrada (estilo Fusion 360)

> 31/10. Paragem do dono: **aprovar o cartão ANTES da primeira tela** (prompt
> [§F5](../49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md), precedente Kanban 27/10). Áreas do painel pelo
> [ADR-0197 §3](../../architecture/decisions/0197-o-painter-e-uma-lei-de-pigmento-em-volumes-e-os-meios-sao-pontos-dela.md):
> Tinta · Pincel · Superfície · Tempo · Luz. Todo o texto da maquete (cartão e painel) está em Plain English (§1). Cores, raios, espaços e tipos = `docs/design/styles/tokens.css`
> (tema `forge-sdf`, acento `magenta`); a fonte Inter não está instalada nesta máquina, por isso o texto cai na
> sans do sistema. Nada de código do produto foi tocado.

## 1. As variantes

| ficheiro | o que é |
|---|---|
| [`variante_A_compacto.png`](fotos/f5_maquete/variante_A_compacto.png) | **A — compacto.** 300 px à direita do controlo: título, UMA frase, 3 miniaturas pequenas com «low · medium · high» e o valor. A miniatura mais próxima do valor actual leva a moldura do acento. Sem nome da área. |
| [`variante_B_largo.png`](fotos/f5_maquete/variante_B_largo.png) | **B — largo, com a área no topo.** 500 px: selo da área («PAINT»), título, 2 frases, 3 imagens grandes lado a lado com «low · medium · high» e o valor, rodapé «Same stroke, only X changes. Painted live by the engine.» e «F1 = full help». |
| [`variante_C_animado.png`](fotos/f5_maquete/variante_C_animado.png) · [`C_animado_3_quadros.png`](fotos/f5_maquete/C_animado_3_quadros.png) | **C — animado.** Uma imagem GRANDE que passa sozinha por low → medium → high (1 s por valor, em ciclo), com um marcador a andar na régua e a fita dos 3 quadros por baixo. A fita mostra os 3 quadros fixos. |
| [`B_pigmento.png`](fotos/f5_maquete/B_pigmento.png) · [`B_pressao.png`](fotos/f5_maquete/B_pressao.png) | B aplicada a «Pigment» (área PAINT) e «Pressure» (área BRUSH). |
| [`comparacao.png`](fotos/f5_maquete/comparacao.png) | A · B · C lado a lado, todas sobre «Water». |

### Requisito do cartão: **Plain English** (dono 31/10: o app é EXCLUSIVAMENTE em inglês)

Inglês fácil para qualquer usuário de qualquer país: **cada frase ≤ 20 palavras**, só vocabulário comum, sem
gíria, sem termos técnicos (nada de «opacity», «Kubelka», «impasto», «viscosity» no cartão). Entra pelo
`ph2d-i18n` como toda string (HR-15). Rótulos sob as imagens: **«low · medium · high»** — escolhidos em vez de
«little · some · lots» porque servem a QUALQUER controlo (Pressure, Depth, Dry time…, não só quantidades), são as
palavras que qualquer interface já usa para um slider, e «some» é vago para quem não é nativo.

Textos finais:

| área | título | frase |
|---|---|---|
| PAINT | Water | How much water is mixed into the paint. Little water: thick paint that covers. Lots of water: loose paint that blends with what is already on the canvas. |
| PAINT | Pigment | How much color is on the brush. Low: a light stroke you can see through. High: strong color that hides what is under it. |
| BRUSH | Pressure | How much your pen pressure changes the stroke. Press harder to put down more paint and make a fuller mark. |

Nomes das áreas: PAINT (Tinta) · BRUSH (Pincel) · PAPER (Superfície) · TIME (Tempo) · LIGHT (Luz). Rodapé:
«Same stroke, only <Control> changes. Painted live by the engine.» · «F1 = full help». Na C: «Plays by itself:
low → medium → high.» Os valores usam ponto decimal (0.5).

## 2. De onde vem cada imagem (pinceladas REAIS do motor de hoje)

Todas são células da varredura F0 (amarelo sobre azul assentado, driver do app na Radeon), copiadas de
`scratchpad/f0full/celulas/` (300×130 px). Nenhuma foi retocada; só redimensionadas pelo navegador.

| controlo | pincel | o que muda | o que fica | células |
|---|---|---|---|---|
| Água | Oil/Flat/Thick Impasto | Water 0 · 0,5 · 1 | Pigment 40, pressão 1, N=3 | `thick_impasto_p40_w{0,0.5,1}_pr1_n3.png` |
| Pigmento | Dabber 4 | Pigment 40 · 600 · 2000 | Water 0, pressão 1, N=1 | `dabber_4_p{40,600,2000}_w0_pr1_n1.png` |
| Pressão | Oil/Flat/Thick Impasto | pressão 0,2 · 0,6 · 1 | Pigment 40, Water 0,5, N=3 | `thick_impasto_p40_w0.5_pr{0.2,0.6,1}_n3.png` |

O Thick Impasto não tem célula P2000 na F0 (só P40/600), por isso o Pigmento usa o Dabber 4, o único com a
tripla completa. **Luz/Relevo: não há imagem** — a F0 não variou nenhum controlo da Luz; o cartão da Luz fica
para quando o gerador do §4 existir.

⛔ **A maquete já apanhou uma mentira:** na tripla da Água, `0,5` e `1` saem quase iguais e `0` sai MAIS mole e
mais misturado — o contrário do que a frase diz. Isto é o motor de hoje (o mesmo canto do `dono_6`: pigmento
baixo, água alta no Thick Impasto). O texto foi escrito para a lei nova (ADR-0197); se a lei nova não pintar o
que a frase diz, **muda-se a lei ou a frase, nunca a imagem** — é exactamente o portão do §5.

## 3. Recomendação: **B** (com a animação de C como opção ligada por omissão só em F1)

- **B** é a única que mostra as 3 pinceladas grandes o bastante para ver a diferença (no A, a 92 px de largura,
  Água 0,5 e 1 são indistinguíveis — a ajuda passa a não ensinar nada), e traz o nome da área (PAINT/BRUSH/PAPER/TIME/LIGHT), que é o que ensina
  as 5 áreas novas ao dono sem um manual.
- **A** cabe em painéis estreitos, mas corta a 2.ª frase e as miniaturas ficam pequenas demais para pinceladas
  (que diferem no relevo e na mistura, não só no tamanho).
- **C** é a mais bonita mas compara MAL: o olho tem de lembrar o quadro anterior; e um cartão que se mexe ao
  pairar distrai quem está só a passar o rato. Fica como a vista da ajuda completa (F1), onde há tempo para ver.
- Regra comum às três: a imagem do valor actual leva a moldura do acento; o cartão aparece após ~600 ms parado
  em cima do controlo e some ao sair (token de duração, não literal).

## 4. Como o motor vai gerar a imagem no produto (proposta)

- **Quem pinta:** o mesmo caminho das miniaturas de fábrica (headless, CPU, determinístico): a pincelada-padrão
  da F0 (amarelo sobre azul assentado, 3 passadas) com o pincel ACTIVO e o seu preset, variando SÓ o controlo
  pairado em 3 pontos: o mínimo útil, o meio e o máximo útil da faixa do controlo (a faixa vem da mesma tabela
  que desenha o slider — um sítio só). A tripla e a pincelada são dados, por controlo.
- **Tamanho:** 300×130 px por célula (a da F0), mostrada a ~150 px no B; ×2 em ecrã HiDPI.
- **Cache:** em disco, chave = hash(versão do motor/lei + preset do pincel + id do controlo + os 3 valores +
  pincelada-padrão). Duas camadas: (1) as triplas dos **pincéis de fábrica** geram-se no build das miniaturas e
  viajam com elas; (2) pincel do utilizador ou preset editado → gera-se em segundo plano à primeira paragem do
  rato (≈ 3 células; enquanto não chega, o cartão mostra o texto e um esqueleto do tamanho da imagem — nunca uma
  imagem de outro pincel).
- **Quando se regera:** sempre que a chave muda — mudar a versão do motor invalida tudo de fábrica (regerado
  junto com as miniaturas); editar um preset invalida só as triplas desse preset.

## 5. O portão «a imagem da ajuda é a que o motor pinta hoje»

Gate proposto (`painter_ajuda_e_o_que_o_motor_pinta`): para cada controlo com ajuda e cada pincel de fábrica,
repinta as 3 células com o motor do HEAD e compara com as imagens guardadas da ajuda; **reprova** se alguma
diferir para além do ruído da máquina (mesmo bit por placa, como as miniaturas). Segunda metade, a que apanha o
caso da Água acima: a tripla tem de ser **monótona na grandeza que a frase promete** (ex.: Pigmento ⇒ a
cobertura/escuridão média sobe de low → medium → high; Água ⇒ a mistura com o fundo sobe) e as 3 células
**distintas entre si** (diferença média acima de um piso medido) — um controlo cuja tripla sai igual é um
controlo morto ou saturado, e a ajuda não o pode esconder.

## 6. Ficheiros

Imagens: `docs/Painter/Arquitetura/fotos/f5_maquete/`. Gerador da maquete (HTML + Firefox headless, tokens do
design system): `scratchpad/f5/gera.py` (rascunho, fora do repo).
