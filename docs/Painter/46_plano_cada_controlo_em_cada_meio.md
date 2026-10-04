# 46 — Plano: cada controlo funciona em cada meio (2026-10-04)

> As decisões do dono sobre a tabela do [doc 45](45_censo_dos_controlos.md) §4, verbatim:
> 1. *«Wet Paint, Accumulate e Space Attenuation: (…) vc deve fazer a análise e vc deve escolher o
>    que há de melhor.»*
> 2. *«Os controlos que não fazem nada num meio: tudo que for compatível sem comprometer qualidade e
>    performance, deve funcionar para cada meio. Faça uma análise séria e se for possível coloque no
>    plano de implementação.»*
> 3. *«Dry Time: fazer a tinta da aquarela secar de facto com o tempo.»*
> 4. *«Reset de um meio: deve manter o meio escolhido e só repor os valores dele.»*
> 5. *«Os controlos que dependem de outro: deixá-los esmaecidos.»*
>
> Estado por item: `feito` · `a fazer` · `recusado (com o porquê)`.

## §1 — A análise, item a item

A pergunta de cada linha: **o que o controlo FAZ no Digital, e esse gesto tem sentido no meio?** —
não *«o código do meio o lê?»*. As portas fechadas eram omissões (`solid_owns_the_gesture` e
`threads_own_the_gesture` dizem *«onde o depósito não é pigmento ele não tem o que preencher»*), não
recusas medidas.

| controlo | meio | o que é no Digital | tem sentido no meio? | veredito |
|---|---|---|---|---|
| **Accumulate** | W | sem ele, o traço não passa do **Strength** (o tecto do Blender, `BRUSH_ACCUMULATE`) | **não**: o Wet Paint não tem Strength (não aparece no painel) e o fluido CONSERVA a massa que deposita — não há tecto a desligar | **esconder** no W (a Aquarela e o Impasto já o escondem) |
| **Space Attenuation** | A · I · W | divide a força de cada carimbo pela sobreposição (`space_overlap_factor`), SÓ com Accumulate ligado | não onde o Accumulate não existe; e no W, ligados juntos, **apagam o traço** (`2 428 → 26` texels) porque a lei de alfa do Blender entra como intensidade da água | **esconder** onde o Accumulate não aparece; e o fator deixa de entrar nos meios que não acumulam (defesa contra o *Sync with other tools*, que leva os dois interruptores de um slot para outro) |
| **Blend** (24 modos) | W | como a tinta do traço se junta à tela por baixo | **sim**: a água compõe o pigmento sobre a base pixel a pixel (`wetpaint/composite.rs`) — é ali que o modo entra. O modo é da SESSÃO molhada (a tinta molhada é uma camada), congelado no nascimento dela; trocar o modo fecha a sessão (assa) antes do traço seguinte | **compatível** · custo: uma operação de mistura por pixel composto |
| **Solid** | A | preenche a região que o traço fecha | **sim**: a aguada é COBERTURA — a região fechada entra na cobertura e na cor da sessão como mais um carimbo, e a borda escura, a granulação e o papel agem nela como em qualquer traço | **compatível** · custo: rasterizar o polígono uma vez por quadro |
| **Shape Color Ramp** | A | dá cor a cada pixel do carimbo pela luminância da Shape | **sim**: a aguada já guarda uma COR POR PIXEL na sessão (`stroke_color`, `watercolor_accum_cor.rs`) — a rampa entra no splat da cor | **compatível** · custo: uma consulta de tabela de 256 por texel |
| **Sketchy · Wire · Rungs** (fios) | A | fios finos, de opacidade baixa, que costuram o traço | **sim**: um fio é cobertura fina; entra na aguada como os carimbos | **compatível** · ⚠️ qualidade a confirmar no smoke (a borda escura num fio de 1 px) |
| **Composite Brush** (a pilha) | W | camadas por carimbo: Brush · Smear · Blur · Eraser | **sim**: cada camada tem a ferramenta da água que faz o mesmo gesto — Brush → depósito, Smear → Smear, Blur → Blend, Eraser → Erase (`dispatch_pressure_dab_tool`) | **compatível, grande** · custo: N despachos por carimbo — **mede-se**; kill-criterion: o quadro da pilha cheia no W não pode passar o do Composite no Digital |
| **Solid** | W | (idem) | **sim**: a região fechada vira uma POÇA (pigmento e água depositados na máscara do polígono) | **compatível, grande** · pede uma porta nova no motor (`ph2d-wet-paint`) |
| **Fios** | W | (idem) | sim na física (a grelha nasce com 1 px por célula) | **compatível, a medir** · custo: centenas de carimbos de água por quadro numa teia densa — kill-criterion igual ao da pilha |
| **Shape Color Ramp** | W | (idem) | **não**: a água carrega UMA cor por carimbo para o fluido, e a mistura do pigmento (K–M) homogeneíza o carimbo no primeiro passo — a rampa seria apagada pela física que existe para misturar | **recusado → esconder** no W |

## §2 — O plano (por ordem)

| # | o quê | estado |
|---|---|---|
| 1 | **Reset** de cada meio guarda o meio e repõe os valores DELE (todas as secções; a porta `spec_de_fabrica`) | **feito** (`9c35b6792`) |
| 2 | Wet Paint: esconder **Accumulate** e **Space Attenuation**; Space Attenuation escondido onde o Accumulate não aparece (A · I); o fator de sobreposição só entra onde o Accumulate existe | **feito** (`fedbb192f`) |
| 3 | **Esmaecer** os controlos que dependem de outro (§2.3 do doc 45): a linha pinta-se no tom desabilitado e **continua editável** (o `active = False` do Blender — o artista prepara o valor antes de ligar a pré-condição); o predicado de cada um vive numa tabela única, e o gate é o próprio censo: esmaecido ⇔ a sonda mede inerte | **feito** (`d2e9f7c5d`) — fora dele: o Spread da aquarela, que NÃO é dependente — medido 2026-10-04: na fábrica (raio 10, Spread 7) baixá-lo a 1 muda `1 841` texels e subi-lo a 48 muda `0`, porque o aro usa `core_r = min(Spread, raio/2)` e acima disso só a água o lê; o censo media-o só para cima; o Offset e as ferramentas de esculpir dependem de TINTA na tela, não de outro controlo |
| 4 | O **gate permanente** do censo: `o_censo_dos_controlos_so_encolhe` (`tests/it/o_censo_so_encolhe.rs`) — em cada meio, na fábrica, cada controlo que muda o ajuste e NÃO a tinta tem de estar esmaecido OU na lista `inertes_com_motivo` (o id, ou o id de UMA opção de menu, com o motivo); linha que passa a agir ou sai da tela = vermelho. `25 s` os quatro meios. Mutações: tirar um motivo, pôr um controlo vivo na lista, desfazer a cura do Bug #33 — as três vermelhas. Achou ao nascer: o **Bug #33** (curado) e o **Jitter Rotate** na Aquarela/Wet Paint — inerte na fábrica, mas o gate do esmaecido mediu a tinta a mudar POR ACIDENTE noutros estados (o sorteio do ângulo desloca o fluxo das outras variações; a pegada rodada arredonda diferente), então NÃO esmaece (fica na lista com esse porquê); o **Automatic** não esmaece (é a porta das opções da Shape); os fios **Sketchy no Impasto** só agem com Solid e a porta dos fios não exclui o Impasto — aberto | **feito** |
| 5 | **Dry Time**: a aquarela SECA com o tempo — **cada POÇA seca no seu tempo**: no pen-down de um traço que continua a sessão, a poça que secou (`canvas_wet == 0` em toda ela) e está a mais de 2× o alcance do composite de toda a tinta molhada assa-se na base da sessão e sai da união; o traço novo vela sobre ela e funde onde a tinta está molhada (`watercolor_secagem.rs`, `assa_as_pocas_secas`). Medido antes: a sessão inteira JÁ secava ao fim do Dry Time (o censo leu `0` porque os traços estavam a 67 ms) — o que faltava era a poça velha secar enquanto se pinta noutro canto. ⛔ Secar DENTRO de uma poça (a frente que recua, doc 14 #12b) fica por fazer: o composite lê a VIZINHANÇA da união (aro = borrão da cobertura, dissolução = borrão de raio Spread, warp), e cortar a união dentro de uma poça dá um aro novo no lado molhado e re-deposita no seco. Gates: `watercolor_secagem_por_poca` (4; o B sobre a A seca dá a imagem de referência ao byte; Dry Time 10 s × 60 s mudam a junção; assar não muda um byte; assar nunca corta uma poça meio seca), mutações M1–M5 sangram. Custo a 4096²: `8 ms` uma vez quando a poça assa, `1,2 ms` por pen-down sem nada a assar (load ~25) | **feito** |
| 6 | **Blend** no Wet Paint | a fazer |
| 7 | **Solid**, **Shape Color Ramp** e **fios** na Aquarela | a fazer |
| 8 | **Composite Brush** no Wet Paint (medido contra o kill-criterion) | a fazer |
| 9 | **Solid** e **fios** no Wet Paint (medidos contra o kill-criterion) | a fazer |
| 10 | esconder a **Shape Color Ramp** no Wet Paint — `BrushSettings::shape_ramp_offered` (a secção e a pré-visualização da Shape); gate `a_rampa_da_shape_so_onde_o_meio_a_oferece` (vermelho antes); o censo obrigou a apagar a linha da lista | **feito** |
