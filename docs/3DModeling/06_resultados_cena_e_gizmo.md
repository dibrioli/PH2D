# 06 — O modelador de campo implícito: o que está ABERTO, e o índice das 144 waves

> ⚠️ **Este doc é um ROTEADOR.** Ele tinha **901 842 B / 14 525 linhas** — `8×` o joelho de
> 80–110 KB, acima do qual um `Read` deixa de o alcançar e o acesso vira raspagem por shell.
> Cortado em 2026-09-10 por `python3 scripts/doc-split.py`, que **prova por `sha256` que as duas
> metades remontam o original** e re-ancorou os 69 links relativos que desciam de pasta.
>
> | | onde | tamanho |
> |---|---|---|
> | **o que está ABERTO** (§13) | aqui em baixo | `57 KB` |
> | **a história, VERBATIM** (§1–§144, menos o §13) | [`docs/archive/3dmodeling-06-2026-09-10/`](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md) | `843 KB` |
>
> ⭐ **O §13.0 é a lista viva** — é para ele que o `CLAUDE.md` §5 aponta, e é ele que se audita
> **contra o código** antes de pegar um item (⚠️ ele já esteve parado catorze waves, e quatro itens
> dele mandaram reconstruir trabalho já pago).

## Índice das waves arquivadas

⚠️ **O padrão de navegação destes docs é «saltar para um endereço»** (`§104`, `W135`), e não ler de
fio a pavio — por isso a história sai daqui mas **continua endereçável por §**. Cada linha aponta
para a secção no arquivo.

| § | O quê |
|---|---|
| [§1](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#1-a-hierarquia-da-cena-é-a-árvore-de-modelagem) | A hierarquia da cena É a árvore de modelagem |
| [§2](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#2-o-gizmo) | O gizmo |
| [§3](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#3-costura-que-é-onde-a-semana-se-perde) | Costura, que é onde a semana se perde |
| [§4](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#4-onde-a-seleção-mora) | Onde a seleção mora |
| [§5](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#5-w6-os-outros-dois-verbos-e-um-undo-que-estava-partido-2008) | W6: os outros dois verbos, e um undo que estava partido (20/08) |
| [§6](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#6-w7-o-clique-escolhe-o-objeto-e-os-eixos-podem-ser-dele-2008) | W7: o clique escolhe o objeto, e os eixos podem ser dele (20/08) |
| [§7](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#7-w8-o-gesto-preso-à-grelha-e-o-número-dele-2008) | W8: o gesto preso à grelha, e o número dele (20/08) |
| [§8](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#8-w9-criar-formas-e-combiná-las-2008) | W9: criar formas e combiná-las (20/08) |
| [§9](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#9-w10-as-dimensões-de-cada-forma-2008) | W10: as dimensões de cada forma (20/08) |
| [§10](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#10-w11-duplicar-e-apagar-2008) | W11: duplicar e apagar (20/08) |
| [§11](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#11-w12-as-mesmas-duas-ações-pela-hierarquia-2008) | W12: as mesmas duas ações, pela Hierarquia (20/08) |
| [§12](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#12-w13-digitar-a-posição-e-dois-gates-que-não-provavam-nada-2008) | W13: digitar a posição, e dois gates que não provavam nada (20/08) |
| [§14](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#14-w14-a-rotação-em-números-e-um-piso-que-faltava-a-toda-linha-2008) | W14: a rotação em números, e um piso que faltava a TODA linha (20/08) |
| [§15](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#15-w141-o-bug-do-eixo-do-meio-e-a-lei-que-faltava-2008) | W14.1: o bug do eixo do meio, e a lei que faltava (20/08) |
| [§16](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#16-w15-a-lente-e-o-raio-que-era-construído-duas-vezes-2008) | W15: a lente, e o raio que era construído duas vezes (20/08) |
| [§17](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#17-w16-a-casca-e-o-afastamento-a-tese-em-duas-linhas-de-aritmética-2008) | W16: a casca e o afastamento — a tese em duas linhas de aritmética (20/08) |
| [§18](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#18-w17-os-padrões-e-a-pilha-aguentou-os-sem-arquitetura-nova-2008) | W17: os PADRÕES — e a pilha aguentou-os sem arquitetura nova (20/08) |
| [§19](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#19-w18-a-inclinação-o-primeiro-operador-que-não-é-exato-e-a-conta-que-a-medição-refutou-2008) | W18: a INCLINAÇÃO — o primeiro operador que não é exato, e a conta que a medição refutou (20/08) |
| [§20](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#20-w19-a-peça-sai-e-a-porta-existia-sem-ninguém-a-abrir-2008) | W19: a peça SAI — e a porta existia, sem ninguém a abrir (20/08) |
| [§21](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#21-w20-a-malha-exportada-e-a-quina-que-estava-aberta-desde-a-w0-2108) | W20: a malha exportada, e a quina que estava aberta desde a W0 (21/08) |
| [§22](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#22-w21-a-ponte-da-escultura-e-o-achado-que-o-plano-não-previa-2108) | W21: a ponte da escultura, e o achado que o plano não previa (21/08) |
| [§23](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#23-w22-a-autoria-uma-escultura-entra-pela-porta-2208) | W22: a AUTORIA — uma escultura entra pela porta (22/08) |
| [§24](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#24-w23-o-regresso-a-escultura-volta-com-o-arquivo-2208) | W23: o REGRESSO — a escultura volta com o arquivo (22/08) |
| [§25](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#25-w24-o-preview-responde-à-mão-e-a-resolução-sai-do-relógio-2208) | W24: o preview responde à mão — e a resolução sai do RELÓGIO (22/08) |
| [§26](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#26-w25-a-peça-que-não-cozinha-diz-porquê-e-o-clique-que-a-partia-deixou-de-existir-2208) | W25: a peça que não cozinha DIZ porquê — e o clique que a partia deixou de existir (22/08) |
| [§27](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#27-w26-o-número-digitado-no-meio-do-gesto-o-g-x-05-2208) | W26: o número digitado no meio do gesto — o `G X 0,5` (22/08) |
| [§28](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#28-w27-a-seleção-é-o-sujeito-do-gesto-e-o-pivô-é-o-meio-dela-2208) | W27: a SELEÇÃO é o sujeito do gesto — e o pivô é o meio dela (22/08) |
| [§29](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#29-w28-o-olho-da-hierarquia-apaga-o-nó-da-peça-2208) | W28: o olho da Hierarquia apaga o nó da peça (22/08) |
| [§30](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#30-w29-o-cadeado-trava-a-peça-de-modelagem-2208) | W29: o cadeado trava a peça de modelagem (22/08) |
| [§31](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#31-w30-arrastar-uma-linha-na-hierarquia-não-teleporta-a-peça-2208) | W30: arrastar uma linha na Hierarquia não teleporta a peça (22/08) |
| [§32](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#32-w31-só-uma-operação-pode-ter-filhos-e-criar-um-grupo-passa-a-ser-um-gesto-2208) | W31: só uma OPERAÇÃO pode ter filhos — e criar um grupo passa a ser um gesto (22/08) |
| [§33](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#33-w32-o-refinamento-cede-à-mão-a-última-espera-do-preview-2208) | W32: o refinamento cede à mão — a última espera do preview (22/08) |
| [§34](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#34-w33-a-caixa-da-grade-é-a-da-peça-e-um-corte-silencioso-a-menos-2208) | W33: a caixa da grade é a da PEÇA — e um corte silencioso a menos (22/08) |
| [§35](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#35-w34-o-gesto-de-criar-grupo-existia-e-ninguém-lhe-chegava-2208) | W34: o gesto de criar grupo existia e ninguém lhe chegava (22/08) |
| [§36](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#36-w35-a-peça-já-atravessava-o-arquivo-o-que-não-atravessava-era-a-memória-2208) | W35: a peça JÁ atravessava o arquivo — o que não atravessava era a memória (22/08) |
| [§37](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#37-w36-a-exportação-diz-o-tamanho-e-havia-dois-números-não-um-2208) | W36: a exportação diz o TAMANHO — e havia dois números, não um (22/08) |
| [§38](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#38-w37-a-mensagem-vivia-um-quadro-o-diálogo-que-a-precede-congela-o-relógio-2208) | W37: a mensagem vivia UM quadro — o diálogo que a precede congela o relógio (22/08) |
| [§39](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#39-w38-isolar-não-precisou-de-lei-nenhuma-e-as-duas-vozes-que-faltavam-2208) | W38: isolar não precisou de lei nenhuma — e as duas vozes que faltavam (22/08) |
| [§40](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#40-w39-a-escultura-da-cena-entra-sem-passar-pelo-disco-e-o-vivo-era-impossível-2208) | W39: a escultura da cena entra sem passar pelo disco — e o «vivo» era impossível (22/08) |
| [§41](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#41-w40-o-modelador-não-cedia-o-canvas-a-ninguém-2208) | W40: o modelador não cedia o canvas a ninguém (22/08) |
| [§42](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#42-w41-o-crash-que-o-smoke-da-w40-encontrou-na-escultura-2208) | W41: o crash que o smoke da W40 encontrou na escultura (22/08) |
| [§43](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#43-w42-desarmar-não-desarmava-a-cerca-cuja-razão-dissolveu-2208) | W42: desarmar não desarmava — a cerca cuja razão dissolveu (22/08) |
| [§44](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#44-w43-a-vista-sobrevive-ao-fecho-e-a-categoria-já-estava-escrita-em-três-sítios-2308) | W43: a VISTA sobrevive ao fecho — e a categoria já estava escrita em três sítios (23/08) |
| [§45](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#45-w44-isolado-é-um-estado-ele-diz-se-e-sai-se-dele-de-qualquer-sítio-2308) | W44: «isolado» é um ESTADO — ele diz-se, e sai-se dele de qualquer sítio (23/08) |
| [§46](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#46-w45-a-porta-estava-trancada-por-dentro-um-projeto-que-traz-uma-peça-abre-o-painel-dela-2308) | W45: a porta estava trancada por dentro — um projeto que traz uma peça abre o painel dela (23/08) |
| [§47](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#47-w46-a-peça-nasce-enquadrada-e-o-home-não-sabia-onde-ela-estava-2308) | W46: a peça nasce ENQUADRADA — e o `Home` não sabia onde ela estava (23/08) |
| [§48](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#48-w47-as-seis-vistas-existem-e-a-câmera-passou-a-ser-alcançável-2308) | W47: as SEIS VISTAS existem, e a câmera passou a ser alcançável (23/08) |
| [§49](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#49-w48-nenhum-botão-funcionou-o-quinto-sítio-e-o-gate-que-eu-escrevi-a-cometer-o-pecado-que-ele-condena-2308) | W48: «nenhum botão funcionou» — o quinto sítio, e o gate que eu escrevi a cometer o pecado que ele condena (23/08) |
| [§50](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#50-w49-o-gizmo-de-navegação-e-a-pesquisa-que-o-enio-mandou-fazer-antes-2308) | W49: o GIZMO DE NAVEGAÇÃO — e a pesquisa que o Enio mandou fazer antes (23/08) |
| [§51](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#51-w50-a-moldura-do-app-empurra-o-gizmo-e-o-arnês-de-mutação-mentiu-com-um-vermelho-2308) | W50: a moldura do app EMPURRA o gizmo — e o arnês de mutação mentiu com um vermelho (23/08) |
| [§52](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#52-w51-a-viagem-entre-vistas-e-a-curva-não-é-minha-2308) | W51: a VIAGEM entre vistas — e a curva não é minha (23/08) |
| [§53](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#53-w52-a-viagem-não-é-do-reduced-motion-e-o-papel-dela-nasceu-disso-2308) | W52: a viagem NÃO é do *reduced motion* — e o papel dela nasceu disso (23/08) |
| [§54](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#54-w53-o-perfil-desenhado-vira-peça-uma-família-de-features-completa-e-invisível-2308) | W53: o PERFIL DESENHADO vira peça — uma família de features completa e invisível (23/08) |
| [§55](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#55-w54-a-régua-da-suavidade-é-a-normal-não-a-silhueta-e-a-tabela-velha-estava-desmentida-por-24-2308) | W54: a régua da suavidade é a NORMAL, não a silhueta — e a tabela velha estava desmentida por 2,4× (23/08) |
| [§56](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#56-w55-o-contorno-continua-a-ser-a-fonte-e-o-knob-que-faltava-era-a-mesma-ausência-2308) | W55: o contorno continua a ser a FONTE — e o knob que faltava era a mesma ausência (23/08) |
| [§57](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#57-w56-o-perfil-deixa-de-ser-uma-fita-e-passa-a-ser-uma-consulta-o-alicerce-e-a-receita-que-foi-refutada-2408) | W56: o perfil deixa de ser uma FITA e passa a ser uma CONSULTA — o alicerce, e a receita que foi refutada (24/08) |
| [§58](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#58-w56e-a-profundidade-e-o-que-ela-obrigou-a-admitir-2408) | W56e: a PROFUNDIDADE, e o que ela obrigou a admitir (24/08) |
| [§59](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#59-w56f-o-passo-da-marcha-é-do-documento-não-uma-constante-2408) | W56f: o passo da marcha é do DOCUMENTO, não uma constante (24/08) |
| [§60](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#60-w57-o-vínculo-ao-desenho-vê-se-e-solta-se-e-o-item-que-não-precisava-de-wave-2408) | W57: o vínculo ao desenho VÊ-SE e SOLTA-SE — e o item que não precisava de wave (24/08) |
| [§61](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#61-w58-a-seleção-múltipla-nasce-no-canvas-2408) | W58: a seleção múltipla nasce no CANVAS (24/08) |
| [§62](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#62-w58b-não-seleciona-mais-de-2-a-causa-não-era-um-teto-era-a-pergunta-2408) | W58b: «não seleciona mais de 2» — a causa não era um teto, era a PERGUNTA (24/08) |
| [§63](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#63-w58c-a-moldura-do-laço-estava-do-lado-errado-de-uma-lei-escrita-uma-linha-acima-2408) | W58c: a moldura do laço estava do lado ERRADO de uma lei escrita uma linha acima (24/08) |
| [§64](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#64-w58d-o-laço-soma-o-clique-alterna-e-a-assimetria-é-a-lei-2408) | W58d: o laço SOMA, o clique alterna — e a assimetria é a lei (24/08) |
| [§65](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#65-w59-a-região-do-corte-é-o-casco-não-a-caixa-dele-2408) | W59: a região do corte é o CASCO, não a caixa dele (24/08) |
| [§66](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#66-w60-reconferir-a-nota-que-o-custo-tornava-inalcançável-e-o-eixo-do-zoom-não-existe-2408) | W60: reconferir a nota que o custo tornava inalcançável — e o eixo do zoom não existe (24/08) |
| [§67](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#67-w61-o-placar-da-malha-extraída-contra-o-estado-da-arte-2408) | W61: o PLACAR da malha extraída, contra o estado da arte (24/08) |
| [§68](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#68-w61b-a-exportação-passa-pela-cadeia-de-quads-e-ela-bate-o-oráculo-2408) | W61b: a exportação passa pela cadeia de quads — e ela bate o oráculo (24/08) |
| [§69](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#69-a-nota-do-traçado-24-mais-caro-estava-errada-por-4-e-o-suspeito-era-inocente) | ⛔⛔ A nota do «traçado 2,4× mais caro» estava errada por 4×, e o suspeito era inocente |
| [§70](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#70-as-duas-decisões-do-enio-2608-o-teto-sobe-a-escada-de-densidade-é-recusada-por-medição) | As duas decisões do Enio (26/08): o teto SOBE, a escada de densidade é RECUSADA por medição |
| [§71](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#71-w70-três-em-cada-quatro-fitas-do-quadro-não-eram-avaliadas-por-ninguém-2608) | W70: três em cada quatro fitas do quadro não eram avaliadas por ninguém (26/08) |
| [§72](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#72-w71-a-montagem-é-20-do-quadro-e-a-fatia-mudou-de-número-2608) | W71: a montagem é `20 %` do quadro, e a fatia mudou de número (26/08) |
| [§73](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#73-w72-o-quadro-de-movimento-também-não-paga-o-anti-serrilhado-2608) | W72: o quadro de movimento também não paga o anti-serrilhado (26/08) |
| [§74](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#74-w73-ao-parar-ficou-mais-lento-para-alisar-o-assentar-vira-uma-escada-2608) | W73: «ao parar ficou mais lento para alisar» — o assentar vira uma ESCADA (26/08) |
| [§75](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#75-w74-com-duas-formas-escolhidas-a-segunda-desaparecia-em-silêncio-2608) | W74: com duas formas escolhidas, a segunda desaparecia em silêncio (26/08) |
| [§76](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#76-w75-a-cerca-do-passo-da-marcha-estava-errada-e-a-cena-1-do-smoke-marchava-acima-do-seguro-2608) | W75: a cerca do passo da marcha estava ERRADA, e a cena 1 do smoke marchava acima do seguro (26/08) |
| [§77](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#77-w76-a-escultura-que-perdeu-o-arquivo-pode-ser-religada-2608) | W76: a escultura que perdeu o arquivo pode ser RELIGADA (26/08) |
| [§78](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#78-w77-a-segunda-cerca-do-passo-e-a-nota-dizia-ninguém-mediu-com-um-gate-a-medir-2608) | W77: a segunda cerca do passo — e a nota dizia «ninguém mediu» com um gate a medir (26/08) |
| [§79](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#79-w78-a-auditoria-da-lista-viva-duas-entradas-eram-trabalho-já-feito-2608) | W78: a auditoria da lista viva — duas entradas eram trabalho JÁ FEITO (26/08) |
| [§80](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#80-w79-o-espelho-passa-a-ter-três-botões-e-a-cerca-que-dizia-roda-o-nó-era-falsa-2608) | W79: o espelho passa a ter TRÊS botões — e a cerca que dizia «roda o nó» era falsa (26/08) |
| [§81](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#81-w80-a-caça-às-listas-que-se-dizem-exaustivas-e-a-segunda-estava-na-lei-mais-cara-do-módulo-2608) | W80: a caça às listas que se dizem exaustivas — e a segunda estava na lei mais cara do módulo (26/08) |
| [§82](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#82-w81-a-marcha-ganha-um-contador-e-a-normal-era-um-quinto-do-quadro-2708) | W81: a marcha ganha um contador — e a normal era um quinto do quadro (27/08) |
| [§82.8](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#828-a-máquina-calma-a-montagem-não-escala-e-é-ela-a-parede-2708) | ⭐⭐⭐ A máquina calma: a MONTAGEM não escala, e é ela a parede (27/08) |
| [§83](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#83-w82-a-cache-de-fitas-entre-quadros-a-cura-da-parede-da-829-2708) | W82: a cache de fitas entre quadros — a cura da parede da §82.9 (27/08) |
| [§84](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#84-w83-o-assentar-e-o-que-sobrava-a-compilar-era-o-anti-serrilhado-2708) | W83: o assentar — e o que sobrava a compilar era o ANTI-SERRILHADO (27/08) |
| [§85](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#85-w84-o-decimador-do-preview-apagava-quinas-e-quem-sobrevivia-era-uma-lotaria-2708) | W84: o decimador do preview apagava QUINAS, e quem sobrevivia era uma lotaria (27/08) |
| [§86](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#86-w85-o-preview-pede-um-erro-e-a-contagem-de-arestas-sai-da-forma-2708) | W85: o preview pede um ERRO, e a contagem de arestas sai da forma (27/08) |
| [§87](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#87-w86-recusa-confirmada-o-perfil-como-consulta-perde-para-a-fita-especializada-2708) | W86: ⛔ RECUSA CONFIRMADA — o perfil como CONSULTA perde para a fita especializada (27/08) |
| [§88](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#88-w86-onde-o-quadro-está-e-uma-atribuição-minha-que-caiu-2708) | W86: onde o quadro está — e uma atribuição minha que caiu (27/08) |
| [§89](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#89-w87-a-perda-é-a-decomposição-e-a-minha-cura-para-ela-foi-medida-e-recusada-2708) | W87: a perda é a DECOMPOSIÇÃO — e a minha cura para ela foi medida e recusada (27/08) |
| [§90](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#90-w88-o-quadro-de-movimento-entrou-no-orçamento-2708) | W88: ⭐⭐⭐ O QUADRO DE MOVIMENTO ENTROU NO ORÇAMENTO (27/08) |
| [§91](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#91-w89-a-travadinha-tinha-nome-a-cache-despejava-1-700-fitas-debaixo-do-cadeado-2708) | W89: ⭐⭐⭐ A TRAVADINHA TINHA NOME — a cache despejava 1 700 fitas debaixo do cadeado (27/08) |
| [§92](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#92-w90-o-canvas-divide-se-em-quatro-vistas-2708) | W90: ⭐⭐⭐ O CANVAS DIVIDE-SE EM QUATRO VISTAS (27/08) |
| [§93](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#93-w97-um-verbo-por-forma-a-operação-sai-do-grupo-e-entra-em-cada-objeto-2808) | W97: ⭐⭐⭐ UM VERBO POR FORMA — a operação sai do grupo e entra em cada objeto (28/08) |
| [§94](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#94-w98-um-raio-de-junção-por-forma-e-as-duas-palavras-que-ele-obrigou-2808) | W98: ⭐⭐⭐ UM RAIO DE JUNÇÃO POR FORMA — e as duas palavras que ele obrigou (28/08) |
| [§95](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#95-w99-o-chanfro-e-as-duas-réguas-que-ele-obrigou-a-separar-2808) | W99: ⭐⭐⭐ O CHANFRO — e as DUAS RÉGUAS que ele obrigou a separar (28/08) |
| [§96](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#96-w100-a-paleta-de-formas-a-fileira-cortava-em-8-e-já-tinha-8-2808) | W100: ⭐⭐⭐ A PALETA DE FORMAS — a fileira cortava em 8 e já tinha 8 (28/08) |
| [§97](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#97-w101-cone-cápsula-e-prisma-uma-lei-só-e-o-max-que-a-torna-segura-2908) | W101: ⭐⭐⭐ CONE, CÁPSULA E PRISMA — uma lei só, e o `max` que a torna segura (29/08) |
| [§98](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#98-o-smoke-da-w100-o-modal-não-fechava-e-não-criava-nada-um-mecanismo-dois-sintomas-2908) | ⛔⛔ O SMOKE DA W100: o modal não fechava e não criava nada — UM mecanismo, dois sintomas (29/08) |
| [§99](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#99-o-smoke-da-w98-as-juntas-mudavam-de-aspecto-ao-rotacionar-a-marcha-lia-o-filete-do-grupo-2908) | ⛔⛔⛔ O SMOKE DA W98: as juntas mudavam de aspecto ao ROTACIONAR — a marcha lia o filete do GRUPO (29/08) |
| [§100](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#100-w102-pirâmide-cunha-arco-de-toro-e-a-auditoria-que-encolheu-a-fila-de-47-para-7-2908) | W102: ⭐⭐⭐ PIRÂMIDE · CUNHA · ARCO DE TORO — e a auditoria que encolheu a fila de 47 para 7 (29/08) |
| [§101](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#101-w103-estrela-gaiola-elipsóide-a-fila-fecha-e-o-filete-de-quatro-formas-era-mentira-2908) | W103: ⭐⭐⭐ ESTRELA · GAIOLA · ELIPSÓIDE — a fila fecha, e o filete de QUATRO formas era mentira (29/08) |
| [§102](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#102-w104-toda-aresta-arredonda-e-o-filete-só-é-um-arco-a-90-2908) | W104: ⭐⭐⭐ TODA ARESTA ARREDONDA — e o filete só é um arco a 90° (29/08) |
| [§103](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#103-w104-bis-as-meias-luas-nos-vales-da-estrela-o-corte-do-sector-passava-pelo-vale-2908) | W104-bis: ⭐⭐ AS MEIAS-LUAS NOS VALES DA ESTRELA — o corte do sector passava PELO vale (29/08) |
| [§104](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#104-w104-ter-a-compensação-do-ângulo-mas-só-nas-quinas-agudas-2908) | W104-ter: ⭐⭐⭐ A COMPENSAÇÃO DO ÂNGULO, mas SÓ nas quinas AGUDAS (29/08) |
| [§105](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#105-w105-o-atlas-de-imagem-persistente-8438-mbs-de-pixels-que-não-mudaram-3008) | W105: ⭐⭐⭐ O ATLAS DE IMAGEM PERSISTENTE — 843,8 MB/s de pixels que não mudaram (30/08) |
| [§106](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#106-w106-as-catorze-formas-que-a-fila-nunca-contou-e-a-auditoria-que-fechou-contra-a-lista-errada-3008) | W106: ⭐⭐⭐ AS CATORZE FORMAS QUE A FILA NUNCA CONTOU — e a auditoria que fechou contra a lista errada (30/08) |
| [§107](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#107-w106-bis-o-arco-preto-na-cruz-a-caixa-do-mundo-era-41-menor-que-a-peça-3008) | W106-bis: ⛔⛔⛔ O ARCO PRETO na cruz — a caixa do mundo era `4,1 %` menor que a peça (30/08) |
| [§108](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#108-w107-o-filete-passa-a-ser-um-arco-em-qualquer-quina-o-operador-aprende-o-ângulo-0209) | W107: ⭐⭐⭐ O FILETE PASSA A SER UM ARCO EM QUALQUER QUINA — o operador aprende o ângulo (02/09) |
| [§109](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#109-w108-o-divisor-por-região-de-marcha-medido-e-refutado-por-dois-mecanismos-0209) | W108: ⛔⛔⛔ O DIVISOR POR REGIÃO DE MARCHA — medido e REFUTADO, por dois mecanismos (02/09) |
| [§110](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#110-w109-o-cabeçalho-de-cada-vista-é-clicável-o-último-do-canvas-0209) | W109: ⭐⭐ O CABEÇALHO DE CADA VISTA É CLICÁVEL — o último ⏳ do canvas (02/09) |
| [§111](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#111-w110-o-chanfro-honesto-a-terceira-saída-medida-e-a-causa-não-era-a-que-eu-previ-0209) | W110: ⛔⛔⛔ O CHANFRO HONESTO — a terceira saída MEDIDA, e a causa não era a que eu previ (02/09) |
| [§112](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#112-w111-o-chanfro-honesto-a-célula-que-faltava-tinha-quatro-peças-e-nenhuma-anda-sozinha-0309) | W111: ⭐⭐⭐ O CHANFRO HONESTO — a célula que faltava tinha QUATRO peças, e nenhuma anda sozinha (03/09) |
| [§113](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#113-w112-o-laço-que-subtrai-e-o-preço-da-fileira-mudou-o-desenho-0309) | W112: ⭐⭐⭐ O LAÇO QUE SUBTRAI — e o preço da fileira mudou o desenho (03/09) |
| [§114](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#114-w113-o-undo-não-obedecia-cada-etapa-e-eram-duas-causas-uma-delas-em-todo-ctrlz-0309) | W113: ⛔⛔⛔ O UNDO NÃO OBEDECIA CADA ETAPA — e eram DUAS causas, uma delas em todo Ctrl+Z (03/09) |
| [§115](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#115-w114-ainda-pula-etapas-as-duas-curas-da-w113-eram-reais-e-não-eram-a-causa-0309) | W114: ⛔ *«ainda pula etapas»* — as duas curas da W113 eram reais e NÃO eram a causa (03/09) |
| [§116](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#116-w115-a-causa-com-o-log-do-próprio-app-um-pedido-servido-num-quadro-sem-evento-0309) | W115: ⭐⭐⭐ A CAUSA, com o log do próprio app: um PEDIDO servido num quadro SEM EVENTO (03/09) |
| [§117](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#117-w116-o-undoredo-está-completamente-destruído-a-causa-era-a-mão-não-a-fila-0409) | W116: ⭐⭐⭐ *«o undo/redo está completamente destruído»* — a causa era a MÃO, não a fila (04/09) |
| [§118](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#118-w117-mirror-não-funcionou-ele-era-um-controlo-morto-e-mediu-se-0000000-0409) | W117: ⛔⛔⛔ *«Mirror não funcionou»* — ele era um controlo MORTO, e mediu-se `0.000000` (04/09) |
| [§119](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#119-w118-o-undo-é-um-bosta-não-melhorou-nada-auditoria-completa-a-coluna-que-nunca-foi-vigiada-0409) | W118: ⛔⛔⛔ *«o undo é um bosta, não melhorou nada — auditoria completa»* — a coluna que nunca foi vigiada (04/09) |
| [§120](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#120-w119-o-lote-da-seta-nove-portas-seis-formas-e-a-régua-do-chanfro-supunha-que-toda-aresta-é-convexa-0509) | W119: ⭐⭐⭐ O LOTE DA SETA — nove portas, seis formas, e a régua do chanfro supunha que toda aresta é convexa (05/09) |
| [§121](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#121-w120-o-lote-dos-símbolos-dez-portas-e-dois-gates-novos-que-apanharam-um-defeito-de-2026-08) | W120: ⭐⭐⭐ O LOTE DOS SÍMBOLOS — dez portas, e DOIS gates novos que apanharam um defeito de 2026-08 |
| [§122](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#122-w121-cloud-completamente-bugado-três-sintomas-e-o-gate-que-faltava-era-maior-que-os-três-0509) | W121: ⛔⛔⛔ *«cloud completamente bugado»* — três sintomas, e o gate que faltava era MAIOR que os três (05/09) |
| [§123](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#123-w122-o-lote-3-fluxograma-e-a-cerca-que-a-medição-desmentiu-0509) | W122: o LOTE 3 (fluxograma), e a cerca que a medição DESMENTIU (05/09) |
| [§124](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#124-w123-usando-fórmulas-não-ficam-mais-leves-implemente-e-a-recusa-respondia-a-outra-pergunta-0509) | W123: ⭐⭐⭐ *«usando fórmulas não ficam mais leves? Implemente»* — e a recusa respondia a OUTRA pergunta (05/09) |
| [§125](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#125-w124-a-mola-e-o-gyroid-e-o-preço-de-uma-fórmula-não-é-103-a-esfera-0609) | W124: a MOLA e o GYROID — e o preço de uma fórmula **não** é `1,03×` a esfera (06/09) |
| [§126](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#126-w125-o-lote-5-e-o-levantamento-da-semana-passada-estava-inflado-em-quatro-0609) | W125: o lote 5 — e o levantamento da semana passada estava INFLADO em quatro (06/09) |
| [§127](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#127-w126-o-ecrã-em-branco-e-o-report-do-enio-apontava-31-pares-não-um-0609) | W126: o ecrã em branco — e o report do Enio apontava **31** pares, não um (06/09) |
| [§128](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#128-w127-a-superquadrática-um-knob-que-atravessa-a-família-inteira-0609) | W127: a SUPERQUADRÁTICA — um knob que atravessa a família inteira (06/09) |
| [§129](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#129-w128-a-superfórmula-de-gielis-uma-fórmula-um-catálogo-0609) | W128: a SUPERFÓRMULA de Gielis — uma fórmula, um catálogo (06/09) |
| [§130](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#130-auditoria-da-w128-performance-menor-que-as-outras-formas-é-esperado-0609) | AUDITORIA da W128: *«performance menor que as outras formas? é esperado?»* (06/09) |
| [§131](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#131-a-revisão-adiada-por-que-a-cura-de-42-não-se-sente-0609) | A REVISÃO adiada: por que a cura de `−42 %` não se SENTE (06/09) |
| [§132](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#132-w131-o-triângulo-que-o-prisma-não-faz-e-a-lei-da-casa-que-eu-violei-0609) | W131: o TRIÂNGULO que o prisma não faz — e a lei da casa que eu violei (06/09) |
| [§133](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#133-w132-o-polígono-de-n-vértices-e-a-nota-que-eu-ia-acreditar-0609) | W132: o POLÍGONO de `N` vértices — e a nota que eu ia acreditar (06/09) |
| [§134](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#134-w133-os-vértices-no-canvas-e-o-arrasto-pela-alça-0709) | W133: os VÉRTICES no canvas, e o arrasto pela alça (07/09) |
| [§135](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#135-w134-o-nó-de-toro-p-q-e-as-três-réguas-que-eu-tive-de-corrigir-antes-do-algoritmo-0709) | W134: o NÓ DE TORO `(p, q)`, e as TRÊS réguas que eu tive de corrigir antes do algoritmo (07/09) |
| [§136](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#136-w135-a-rosca-e-o-serrilhado-e-o-campo-que-era-uma-cunha-infinita-0709) | W135: a ROSCA e o SERRILHADO, e o campo que era uma cunha INFINITA (07/09) |
| [§137](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#137-w136-as-duas-curvas-com-espessura-e-as-quatro-construções-da-onda-0709) | W136: as DUAS CURVAS COM ESPESSURA, e as QUATRO construções da onda (07/09) |
| [§138](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#138-w137-o-campo-que-saturava-longe-da-peça-e-as-duas-metades-do-mesmo-report-0709) | W137: o campo que SATURAVA longe da peça, e as duas metades do mesmo report (07/09) |
| [§139](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#139-w138-sai-e-w139-entra-as-duas-últimas-formas-agora-como-peças-0809) | W138 SAI e W139 ENTRA: as duas últimas formas, agora como PEÇAS (08/09) |
| [§140](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#140-o-plane-sai-por-medição-e-o-tubo-da-mola-passa-a-ter-a-espessura-que-o-painel-diz-0809) | O PLANE sai por MEDIÇÃO, e o tubo da mola passa a ter a espessura que o painel diz (08/09) |
| [§141](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#141-o-chanfro-da-estrela-entrava-no-miolo-e-a-raiz-é-o-campo-das-paredes-0809) | O CHANFRO DA ESTRELA entrava no MIOLO, e a raiz é o campo das paredes (08/09) |
| [§142](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#142-o-filete-não-pegava-as-arestas-depois-do-chanfro-e-a-raiz-é-o-eixo-medial-0809) | O FILETE não pegava as arestas depois do chanfro, e a raiz é o EIXO MEDIAL (08/09) |
| [§143](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#143-dois-controlos-de-chanfro-na-estrela-as-faces-e-as-pontas-0909) | DOIS controlos de chanfro na estrela: as faces e as PONTAS (09/09) |
| [§144](../archive/3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md#144-o-vale-é-uma-aresta-do-contorno-e-o-tecto-do-controlo-tem-um-preço-medido-0909) | O VALE é uma aresta do CONTORNO, e o tecto do controlo tem um preço medido (09/09) |

---

## §13 — Aberto

> ⚠️ **Esta lista parou na W56 durante catorze waves** (auditada em 2026-08-26). O que se seguiu
> viveu só em mensagens de commit — e *uma lista de aberto que envelhece manda reconstruir o que já
> está pago*, que foi exactamente o que aconteceu com quatro itens dela em 25/08.

### §13.0 — O que está ABERTO agora (o resto desta seção é o histórico por wave)

| O quê | Estado | Onde |
|---|---|---|
| ✅⛔⛔ **O undo não obedecia cada etapa** (report do Enio, 03/09) | **DUAS** causas: um gesto de gizmo tem **três** saídas e só uma marcava o quadro como autorado (acabar com `Enter` registava **passo nenhum**) · e **todo `Ctrl+Z` apagava a seleção 3D**, porque o `apply_project` só devolvia a vetorial — a cura é por `StableId`, nunca pelos bits | §114 |
| ✅⭐⭐⭐ **A PALETA DE FORMAS** (`+ Add shape…` / tecla `A`) — o catálogo sai do painel e entra no modal genérico da casa | a fileira de chips cortava em **8** e já tinha 8; ⛔ morreram as 4 constantes derivadas do fim da lista, que faziam *Extrude* abrir o diálogo de escultura ao acrescentar no fim | §96 |
| ✅⭐⭐ **CONE · TRONCO DE CONE · CÁPSULA · PRISMA** — o 1.º lote da fila do Enio | uma lei só (`max` de meias-fatias, 1-Lipschitz por definição); ⚠️ **5 mutações sobreviveram à 1.ª ronda**, todas por eu gatear o campo e não a API do documento | §97 |
| ✅ **A FILA DE FORMAS FECHOU** (era *«as 47 do catálogo vetorial + 11 sólidas»*) | ⚠️ esta linha ficou **obsoleta** durante uma jornada inteira: a auditoria da §100 encolheu a fila de 47 para 7, e a W103 fechou-a com a estrela, a gaiola e o elipsóide. A `Family::Plates` **não** nasce vazia — a estrela vive lá | §100, §101 |
| ✅⭐⭐⭐ **O ATLAS DE IMAGEM PERSISTENTE da `vello` 0.10** — o módulo enviava `843,8 MB/s` de pixels que não tinham mudado, e punha o atlas no tecto de `8192²` (256 MB de VRAM) | ⛔ **não era quebra: nada chegava a ser descartado** (`não coube = 0`, mesmo com um vizinho de `4096²`). A cura é o `StableImage`, que já existia; a picture é **byte-idêntica por gate** | §105 |
| ✅⭐⭐⭐ **A pré-visualização ALCANÇA 60 Hz** — mediana `~12 ms` contra `16,7`, e independente do `Resolution` | o item nº 1 desde a §70. ⛔ **o `14,2 ms` da §90 foi medido com a câmera PARADA** (corrigido na §91.1); num arrasto real o quadro custa `10`–`27 ms` | §90.4, §91.1 |
| ✅⛔⛔ **E a BANCADA continuava a medir a pose PARADA** — a §91.1 corrigiu-a **por escrito** em 27/08 e ninguém lhe tocou no CÓDIGO (14/09) | a `measure_where_the_frame_stands_after_all_of_it` aquecia com um arrasto e cronometrava `Orbit::default()` **repetido**, que é o único caso em que a cache de fitas acerta `100 %`. Hoje ela cronometra **uma pose nova por amostra** e imprime **mediana e p90**, com a pose repetida ao lado como CONTROLO. ⭐ Reconferido depois da W147 e das três waves de material: **`10,7`–`12,5 ms` de mediana** e `p90 13`–`20`, a `load 78`–`84` — o item continua fechado, e agora com a régua que o mede. ⚠️ **O `CLAUDE.md` §5 ainda diz `26,7 ms`**, que é o número de ANTES da W88 | §90.1, §91.1 |
| ✅⭐⭐⭐ **A TRAVADINHA do Enio: a cache despejava `1 700` fitas debaixo do cadeado** | ⭐ `94 %` do preço era a **árvore**, que na rota do produto é lastro. Máximo do regime `364,6 → 21,7 ms` (`17×`), despejo no cadeado `~3 000×` mais barato | §91.2–§91.4 |
| ✅⭐ **A lei do cancelamento perguntava ao TAMANHO** desde a W73 | ⭐ passa a perguntar à **espécie**: numa hesitação de um quadro o erro angular vai de `2,97°` para `1,50°` | §91.7 |
| ✅ **`FRAMES_KEPT = 3` era derivado; agora é MEDIDO** | o joelho está lá: `1` e `2` piores, `4` e `6` compram `≤0,4 ms` por `1,4`–`2,5×` a memória | §91.8 |
| ✅⭐⭐⭐ **O CANVAS DIVIDE-SE EM QUATRO VISTAS** (`Ctrl+Alt+Q` ou o chip *Quad View*) | o item que o plano chama *«o produto»* desde a W2; falta a outra metade da frase dele, o **cabeçalho** | §92 |
| ✅ **E só a vista ACTIVA ficava lisa** (smoke do Enio, 27/08) | ⭐ cada viewport comparava-se com o pedido do **activo**; os 5 gates da wave mediam a GEOMETRIA e passaram todos | §92.8 |
| ✅⭐⭐ **Cada vista diz o NOME dela**, derivado da câmera (orbitar a *Top* fá-la *User*) | a metade do cabeçalho que não rouba pixels ao traçado | §92.10 |
| ✅⭐⭐ **O cabeçalho é CLICÁVEL** — um menu com as seis vistas, por quadrante | ⛔ ele **não** pedia a faixa reservada: um menu precisa de um alvo de clique, e o rótulo já tinha posição · ⚠️ o gate da costura apanhou a precedência (a costura roubava metade das linhas ao menu) | §110 |
| ✅⭐⭐⭐ **As divisórias ARRASTAM-SE** (o cruzamento move as duas) | ⚠️ e a nota que dizia depender do cabeçalho estava **errada** | §92.12 |
| ✅⭐⭐ **O custo de uma EDIÇÃO com a divisão aberta** — medido: quatro juntas custam `3,93×` uma (elas só se fatiam) | ⭐ curado por **ORDEM**: a activa tem prioridade e chega em `64 ms` em vez de `254` | §92.9 |
| ⛔ A **varredura linear** do `TapeCache::get` — **MEDIDA**: `3,0 %` do quadro de movimento, `10,7 %` a `640×360`, e cresce ~quadraticamente | não paga um índice **ainda**; o gatilho está nomeado | §92.11 |
| ✅ **W82: a cache de fitas entre quadros EXISTE** | ⭐ `1,15×`–`1,23×` no quadro de movimento, com `84 %`–`93 %` de acerto e `226` compilações/quadro a cair para `16`–`44`. ⛔ **A estimativa de `1,7×` estava errada por dois motivos nomeados** | §83.7, §83.8 |
| ✅⛔⛔ **Uma folha de perfil debaixo de uma OPERAÇÃO que dobra desenhava errado** (achado na auditoria da W148; defeito da W56 + W79) — `84` px (espelho) e `586` px (matriz) contra `0` do gémeo | o mapa mundo→local compunha só poses; ⭐ a descida pára debaixo de um modificador que remapeia, e a folha fica certa e **não especializada** · ⏳ a pré-imagem de cada dobra é a wave que a W56 já nomeava | [doc 12 §12.9-bis](12_a_cache_contra_o_casco.md) |
| ✅⭐⭐⭐ **A cache contra o CASCO** (W148) — a nota punha `1,11×` na mesa, e a mesa tinha **até `~2×` da marcha**: a caixa servia `1,9×`–`2,4×` as arestas do caminho sem cache a `TILE = 24` | ⛔ o desenho da nota (o casco **escalado**) **triplica as compilações**; o que shipa é o casco crescido por uma **DISTÂNCIA** (`0,08 × alcance` a partir do alvo), melhor que a caixa nas duas colunas em órbita, pan e zoom · ⚠️ uma mutação (servir por caixa) sobreviveu aos três gates de imagem, e a propriedade ganhou gate próprio · ⏳ o relógio entre `0,06` e `0,08` | [doc 12](12_a_cache_contra_o_casco.md) |
| ✅ **O assentar: o que sobrava a compilar era o ANTI-SERRILHADO** | ⭐ `29` fitas por degrau → **`1`**; o custo dele caiu de `1,34×` para **`1,11×`**. A recusa da W70 dissolveu porque a W82 apagou a premissa dela | §84 |
| ✅ O contorno cheio era `3,39×` no assentar de uma peça de resolução ALTA | ⭐ curado pela §86: o assente engrossa até ao erro que a imagem mostra | §84.4, §86.1 |
| ✅ **O decimador do preview apagava QUINAS** — e quem sobrevivia era uma lotaria de índice | ⭐ decima por **GIRO**: a estrela vai de `134` pontos partidos para **`10` exactos**, imagem idêntica | §85 |
| ✅ **O preview pede um ERRO, e a contagem sai da forma** | ⭐ o assente engrossa até `0,5°` de erro de normal: peça de omissão **intocada**, peça de `Resolution` alto **`2×`–`3×`** mais barata a assentar, com `≤3/255` de mudança | §86 |
| ⏳ **A MARCHA** — o que sobra do quadro; custo **por aresta tocada** | ⛔ sobre-relaxação fora (`8,0` amostras por raio) · ⛔ **o perfil como CONSULTA fora**: a fita especializada ganha `2×`–`8×` na região real | §73, §82.1, §87 |
| ⭐⭐⭐ **O quadro de movimento usa `~30 %` da máquina** — o buraco até 60 Hz é de ESCALAMENTO, não de algoritmo | ⛔ o ladrilho **não** é a alavanca (`48 ≈ 64`) · ⛔⛔ **e o JIT também não era**: tirá-lo não mudou a forma da curva (§88.2). A causa está por achar | §82.8, §88.2 |
| ⛔ A **ordenação dos ladrilhos** — **MEDIDA e fechada**: o tecto caiu de `1,76×` para `1,14×` com o ladrilho a `24`, e o custo do quadro **anterior** atinge-o (`1,00×`) | recusada: `1,01×` a 8 threads, e o preço é uma tabela por ladrilho entre quadros. Gatilho nomeado | §92.16 |
| ✅⭐⭐ **O `TILE` estava a ser escolhido pelo TECTO DA MINHA CACHE** | ⭐ tecto **derivado** do que o quadro pede ⇒ `TILE` de `64` para **`24`**, `1,44×`–`1,51×` | §90.2, §90.3 |
| ⏳ O `SLABS` — **reconferido**: a imagem é idêntica nas 5, mas o óptimo MOVE-SE com o tamanho do quadro (`2`–`3` a `426×240`, `4`–`6` a `640×360`) | fica em `4` (o compromisso); decide-se com uma corrida a `load < 5`, e a resposta pode ser **derivá-lo** do tamanho | §92.14 |
| ⛔ O estêncil de **quatro** amostras para a normal (`7 %` de todas as amostras) | **RECUSA MEDIDA** — numa quina de navalha move a normal `14°`–`35°` | §82.6 |
| ⏸️ As duas fatias de FORA: `8,7 %` da montagem por `0,18 %` da marcha | as três saídas medidas, nenhuma se paga | §82.3 |
| ⏸️ Baixar as arestas do contorno a mexer (`PREVIEW_MAX_EDGES`) | preço medido na tabela; muda a FORMA, decisão de quem vê | §73.1 |
| ⏸️ O 2.º degrau do assentar custa `504 ms` numa peça densa | a escada tirou-o do caminho; o número fica | §74.2 |
| ⛔ Reaproveitar o avaliador na **2.ª passagem** (anti-serrilhado) | construído 2×, medido `0,97×`–`1,01×`, **revertido** | §71.4 |
| ✅ Ladrilhar em `(u, v)` contra o paralelogramo | ⭐ **já feito na W59** (o casco); apertar mais está fora do vale | §79.1 |
| ✅⭐⭐ **O laço que SUBTRAI** — um chip *Add / Subtract* no painel | ⛔ a 5.ª saída (o laço a ALTERNAR) é **recusa de produto de 24/08**, não uma tecla em falta · ⭐⭐ o **preço** da fileira (`+66 px`, `+11,9 %`) mudou o desenho: ela só aparece com **2+** escolhidas e o modo **volta a somar** quando ela some · ⭐⭐⭐ uma mutação que SOBREVIVEU achou o buraco do 2.º vão da costura no painel INTEIRO | §113 |
| ✅ Vários `VecPath` separados | ⭐ era um defeito MUDO, curado: uma peça por forma, todas ligadas | §75 |
| ✅ Religar uma escultura que mudou de sítio | ⭐ `Relink Sculpture…`, com a chave nova escrita no nó | §77 |
| ✅ O `Mirror` «não se consegue demonstrar» | ⭐ **demonstra-se** — o modificador na OPERAÇÃO dobra um filho fora do eixo | §79.2 |
| ✅ A composição de dois `Exact` encadeados | ⛔ **medida: eles COMPÕEM** — a cerca estava errada e a marcha furava | §76 |
| ✅ O gradiente de uma **escultura** | medido: máx `1,0852` (cubo), `30 %` de folga para o `√2` | §78 |
| ⏸️ A barra **demonstrável** da interpolação trilinear é `√3`, e ship-se o `√2` medido | dívida nomeada | §78.3 |
| ⛔ Os níveis de exportação **não** podem mandar na densidade dos quads | recusa MEDIDA, revertida | §70 |
| ⛔ Dois `panic` do `ph2d-gridmap` com reprodutor | **dono: `line/quadextract`** | §68, §70 |
| ✅⭐⭐⭐ **UM VERBO POR FORMA** — a operação sai do grupo e entra em cada objeto (etapa **1** de 3) | a receita lê-se na Hierarquia (`UNI`/`SUB`/`INT`/`BSE`, os selos do vetorial) · ausência = **herança** · a base **semeia** e guarda o verbo dela | §93 |
| ✅⭐⭐⭐ **(2) O RAIO POR OBJETO** — linha **Joint**, derivada, e escrever nela **materializa** o verbo | ⭐ o painel não mudou (as linhas saem do `params_of`) · ⚠️ **Fillet** = as arestas da forma · **Joint** = o encontro, e o grupo passa a dizer `Joint` (é o padrão) | §94 |
| ✅⭐⭐⭐ **(3) O CHANFRO** — `Fillet · Chamfer · Organic`, **três** chips (a aresta viva é o raio zero) | ⭐ uma fórmula só, exacta · ⚠️ **DUAS réguas** (recuo · mordida) e nenhum carácter bate as duas: o orgânico calibra-se pela **mordida**, o chanfro **não se calibra** · ⛔ 2 mutantes sobreviveram e um era defeito VIVO | §95 |
| ✅⭐⭐⭐ **O FILETE É UM ARCO EM QUALQUER QUINA** — o operador deixa de supor que as duas faces são perpendiculares | ⭐ o recuo passa a ser `r·(1/sin α − 1)` em todo ângulo; a estrela vai de `3,71` para **`0,66`** de quebra de curvatura · ⚠️ a lei antiga errava nos DOIS sentidos (`2,29×` a menos numa ponta, `2,19×` a MAIS numa parede de hexágono) · ⛔ a 1.ª versão deslocava a FACE | §108 |
| ✅⭐⭐⭐ **O CHANFRO recua o que o slider DIZ** — a mentira de `1,61×` numa ponta de estrela fechou | ⭐ a célula que faltava tinha **quatro** peças e nenhuma anda sozinha (recuo · normalização · filete por PARES · **limite da faceta**) · ⭐⭐ acima de `r_max = c·sin α(1+sin α)/cos α` o filete come o chanfro, e a transição **não tem degrau** porque no limite os três planos deslocados são concorrentes · ⚠️ o pior giro da estrela SOBE para `82,4°` e isso é o **vértice** que o corte antigo escondia (`arccos((κ+1)/2) = 83,81°`) — o gate trocou o tecto em graus por uma **igualdade analítica** | §112 |
| ⏳ Pela mistura **N-ÁRIA** o corte ainda desce `c/sin 2α` — `1,15×` num prisma hexagonal, `1,06×` num octaedro | ⛔ **bloqueio NOMEADO**: generalizar a `intersection_round_n` pede a matriz de Gram inteira, cujo recorte não tem forma fechada em `N ≥ 3` · ⛔ e o plano honesto sozinho ali está **medido e recusado** (octaedro `10,03 → 15,12 %`) | §112.4 |
| ⏳ O teto de `round` da **estrela** é `12,3 %` do bordo, contra `43`–`60 %` das outras | ela é a única em que a mistura é uma faixa estreita a atravessar uma face grande | §104.1 |

- ⭐⭐⭐ **W97 (§93): UM VERBO POR FORMA — e o desenho já era LEI na metade 2D deste app.** Pedido do
  Enio (*«a hierarquia fica mais confusa criando vários parentescos… colocar a operação dentro de cada
  objeto»*), que é **o mesmo** que ele desenhou para o vetorial em 22/08 — logo esta wave é as duas
  metades do app passarem a falar a mesma língua, com os **mesmos selos**. ⭐ Foi barata porque os dois
  avaliadores **já eram uma dobra à esquerda**: o que estava fixo era só o verbo. ⭐⭐ **As duas idéias
  do pedido são UMA:** uma mistura pertence a uma JUNÇÃO, e é a dobra que dá a cada forma exactamente
  uma — *«um raio por objeto» não tinha onde existir antes*. ⚠️ **Ausência é HERANÇA**, então toda peça
  anterior avalia igual e o seletor do pai vira o **padrão**; ⛔ e o acumulado **não** começa vazio
  (uma subtração no topo apagaria a peça). ⛔⛔ **Um mutante SOBREVIVEU e o achado é maior que o gate:**
  apagar a herança inteira passou em todos, porque eles **comparavam duas construções** e a mutação
  afectava as duas igual — *um controlo que partilha o defeito não é um controlo*; a cura é medir
  contra **oráculo**, e aí os 5 morrem. ⚠️ E o gate de alcance apanhou um defeito **meu** na tabela
  dele: as asserções endereçavam as fileiras por **índice**, e inserir uma no meio re-apontou-as em
  silêncio.
- ⭐⭐⭐ **W89 (§91): A TRAVADINHA TINHA NOME.** De `~12` em `12` quadros de arrasto a cache chegava ao
  tecto e despejava `1 738` fitas **debaixo do cadeado de escrita**: `274,8 ms` num quadro cujo
  orçamento é `16,7`, com as outras 31 threads à porta. ⭐⭐⭐ **`94 %` desse preço era a ÁRVORE que
  cada fita guardava** — e o único leitor dela é o `fork` da rota de bissecção, desligada por omissão.
  *Guardar «para o caso de» tem preço, e aqui ele era um terço de segundo de imagem congelada.*
  Máximo do regime **`364,6 → 21,7 ms`**. ⛔ Três hipóteses caíram por medição (a hesitação da mão · a
  contenção · o despejo em fatia de `1/8`, **pior nos três números**), e ⚠️ **a régua corrigiu-se três
  vezes**: uma sonda que media o boot, outra que media milissegundos onde a pergunta é angular, e a
  varredura da fase que correu com a tempestade ainda ligada. ⛔⛔ **E a §90 estava optimista: o
  `14,2 ms` que reportei ao Enio foi medido com a câmera PARADA.**
- ⭐⭐⭐ **W88 (§90): O QUADRO DE MOVIMENTO ENTROU NO ORÇAMENTO — `14,2 ms` contra `16,7` (⛔ ver a
  correcção da §91.1: esse número é de câmera parada).** Primeiro o
  **oráculo**: gravar o custo verdadeiro de cada ladrilho e **simular** o escalonamento (*simule antes
  de construir*) — a ordem perfeita dá `1,00×` a 8 threads e `1,02×` a 16, logo **a ordem é o
  mecanismo**; e a 32 sobra um piso de `1,52×` que nenhuma ordem passa, porque *uma ordem não parte um
  ladrilho*. ⛔⛔ **E aí achou-se que o `TILE` estava a ser escolhido pelo TECTO DA MINHA CACHE**: com
  `CAPACITY = 2048` fixo, um ladrilho de `32` a `1600×900` pedia `~5 800` regiões e a cache despejava
  metade a cada quadro (`677 ms` num quadro!) — *o «óptimo» era o maior ladrilho que ainda cabia no
  meu tecto*. Com o tecto **derivado do que o quadro pede**, a resposta inverteu-se e o `TILE` foi de
  `64` para **`24`** (`1,44×`–`1,51×`). ⚠️ *Um limite que não diz de que recurso é acaba a escolher a
  constante do lado.*
- ⛔⛔ **W87 (§89): a perda de escala é a DECOMPOSIÇÃO — e a minha cura foi medida e recusada.** O
  discriminador clássico (`T` quadros **independentes**, um por thread e cada um serial, contra **um**
  repartido por `T`) dá `16,99×` contra `11,65×` a 32 threads ⇒ **a decomposição custa `1,47×`**, e o
  resto é o chão honesto da máquina. ⭐⭐ E isso bate o **`1,52×`** que a §82.5 já tinha previsto por
  **contagem** — *o número certo estava na página ao lado, e a §82.8.2 deu a culpa ao JIT porque era o
  mecanismo que estava à mão.* ⛔⛔ **Ordenar os ladrilhos caros primeiro é neutro a pior**: a régua
  que usei (a profundidade da peça sob o ladrilho) está anti-correlacionada — o caro é o da
  **silhueta**, onde os raios passam rasantes, e não o do meio, onde eles acertam cedo.
- ⛔⛔ **W86 (§88): o ciclo medido de ponta a ponta — e uma atribuição MINHA caiu.** O quadro de
  movimento é hoje **`24 ms` seja qual for o `Resolution`** (era isso o objectivo da W84/W85), e
  continua `1,45×` acima do orçamento. ⛔⛔ **E o JIT NÃO era a causa da má escala:** a curva de
  eficiência é a **mesma** com e sem cache (`31 %` contra `30 %` a 32 threads) — a cache desloca o
  nível e não muda a forma. *Um mecanismo confirmado em isolamento não é, por isso, a causa do que se
  via*, e o erro foi dar por causa o mecanismo que estava à mão.
- ⛔⛔ **W86 (§87): a RECUSA da W56 confirmada, e por outra razão — o perfil como CONSULTA perde.**
  Com a montagem quase eliminada, o quadro é quase só marcha, e a direcção nomeada era trocar a fita
  por um BVH (*«`40 ns` contra `155`»*). Medido na região que de facto ocorre: **a fita especializada
  ganha `2×`–`8×`**, e o BVH só ganha na peça inteira a `672` arestas — o regime que a própria W56
  eliminou. ⭐ O mecanismo é geral: *uma estrutura de aceleração amortiza-se sobre o trabalho que ela
  poda; quando outro mecanismo já o podou, ela paga a própria descida por nada* (o cruzamento é a
  ~`150` arestas guardadas, e a especialização guarda `24`–`202`). ⚠️ E a sonda corrigiu-se **duas**
  vezes antes de dizer isto: media a função errada (a linear, não o BVH) e usava regiões **centradas**,
  em que o corte não corta nada.
- ⭐⭐⭐ **W85 (§86): o preview pede um ERRO, e a contagem de arestas sai da forma.** A decimação por
  giro (W84) tornou a contagem uma consequência: o que o orçamento fixa é o **erro da normal**, que é
  o que a luz mostra. ⇒ dois orçamentos — `1,0°` a mexer (que **reproduz o que já shipava**: `168`
  arestas num círculo dão `1,056°`) e `0,5°` ao assentar, que é onde a §85.1 mediu a imagem parar de
  mudar. ⭐⭐ **Uma peça de omissão não muda nada; uma de `Resolution` alto paga `2×`–`3×` menos no
  assentar**, que é exactamente o custo de que o Enio se queixou. ⚠️ E a pergunta que isso abre
  (*«o `Resolution` ainda serve?»*) tem gate: ele governa a **malha exportada**, e o engrossamento do
  preview tem de continuar com **um** chamador.
- ⭐⭐⭐ **W84 (§85): o decimador do preview apagava QUINAS, e quem sobrevivia era uma lotaria.** Ele
  tirava um em cada `k` vértices — certo para **curvatura**, que é distribuída, e errado para uma
  **quina**, que é um vértice só com todo o ângulo dentro. Medido numa estrela: com o
  `PREVIEW_MAX_EDGES = 168` que ship (passo `3`) a normal saltava **`126,8°`**; com passo `2` ou `5`
  não mudava nada — *as quinas caíam em múltiplos de `40`*. ⭐ A cura é decimar por **GIRO**: `400`
  pontos passam a **`10`** (as quinas exactas) e a imagem sai **idêntica**. ⭐⭐ E a §85.1 mediu o que
  o `Resolution` alto compra: a silhueta quase não mexe, a **normal** mexe `∝ 1/n`, e acima de `~336`
  arestas o pixel muda `≤3` níveis de `255`. ⛔ Isso **refuta** derivar o tecto do tamanho do pixel —
  o erro que se vê é angular, e um ângulo não encolhe com a resolução da tela.
- ⭐⭐⭐ **W83 (§84): o que sobrava a compilar no assentar era o ANTI-SERRILHADO — e eram cinco
  linhas.** A 2.ª passagem constrói um avaliador por lote de 64 pixels de borda, e cada um
  **recompilava a árvore inteira**: num assentar a `640×360` isso era `29` das `29` fitas do quadro,
  com a passagem primária a **100 %** de acerto na cache. ⭐ O `fork` passa a **clonar** a fita
  (`Arc<Mmap>`) em vez de a recompilar ⇒ `29 → 1`, e o anti-serrilhado passou de `1,34×` para
  **`1,11×`**. ⚠️ **A W70 já tinha medido isto e achado NEUTRO** — a nota dela dizia porquê («as
  dezenas de fitas desta passagem são ruído ao lado das `917` regiões»), e **a W82 apagou aquele
  `917` no dia anterior**. ⛔ E duas suposições minhas caíram antes do primeiro número: uma peça na
  resolução de omissão **não alterna documento nenhum**, e a minha sonda **avançava a câmera** no
  assentar, que acontece precisamente porque ela parou.
- ⭐⭐⭐ **W82 (§83): a cache de fitas entre quadros — e a parede da §82.9 cedeu `1,2×`, não `1,7×`.**
  Uma fita construída para a região `R` serve toda a sub-região de `R` ⇒ construída para `R`
  **inflada**, ela serve o quadro seguinte: `84 %`–`93 %` de acerto, `226` compilações por quadro a
  cair para `16`–`44`. ⛔⛔ **E o defeito mais caro foi meu:** a 1.ª versão carimbava a idade de uso
  com o cadeado de **escrita** — o acerto era `87 %`, as compilações caíam `7,5×` e o quadro **não
  mexia** (`0,74×` num caso). *Uma cache que serializa os leitores devolve na trava o que poupou no
  JIT.* ⛔ E a minha estimativa de `14 ms` estava errada por **dois** motivos medidos: montagem e
  marcha **sobrepõem-se** (não se somam), e a fita da cache é **mais gorda** (caixa em vez de casco).
  ⭐ O `INFLATE = 1,25` é um **mínimo medido**: `1,00` e `2,00` os dois **perdem**.
- ⭐⭐⭐ **W81 (§82): a marcha ganhou um contador — e a NORMAL era um quinto do quadro.** `21,1 %` de
  todas as amostras de campo são a diferença central (seis por acerto), e ela não estava em conta
  nenhuma — o doc dizia que elas *«saem noutro sítio»* e o sítio não existia. ⛔ **O estêncil de
  quatro foi medido e RECUSADO** (grátis em tudo menos numa quina de navalha, onde move a normal
  `14°`–`35°`). ⭐⭐ **Ladrilhar e fatiar não custam UMA amostra** — `2 121 060` em todos os
  tamanhos, ao dígito, e eu tinha escrito o contrário. ⭐⭐⭐ E o **ladrilho mais caro** vale `1,52×`
  a fatia perfeita de todo o quadro. ⭐⭐⭐ **E a máquina calma disse onde a perda está (§82.8): a
  MONTAGEM não escala** — a mesma fita custa `1,93×` mais CPU a 32 threads que a 1 (o JIT mapeia
  memória executável, e `mmap` é do kernel) —, ela é **`39 %`** do quadro serial e não os `20 %`
  publicados (aquele número foi medido **com** anti-serrilhado, que a W72 tirou no dia seguinte). O
  quadro usa **`36 %`** da máquina; a `76 %` ele caberia no orçamento. ⛔ **O tamanho do ladrilho
  está fechado** — `48 ≈ 64` e a minha própria receita da §82.5 caiu. ⚠️ **Quatro hipóteses minhas
  caíram antes de custarem código, e uma quinta depois de a escrever.**
- ⭐⭐ **W80 (§81): a caça às listas que se dizem exaustivas — a segunda estava na lei mais cara do
  módulo.** O gate que a W53 escreveu para impedir *«uma família de features completa e invisível»*
  percorria uma **lista literal**: uma primitiva nova ficava sem botão e ele ficava **verde**. ⭐ A
  corrente fecha-se agora no compilador (`PrimitiveKind`), e a mutação achou a metade que faltava —
  **duas famílias a partilhar um botão** passavam.
- ⭐⭐ **W79 (§80): o espelho passa a ter TRÊS botões** (`Mirror X/Y/Z`, pedido do Enio) — e a cerca
  que dizia *«roda o nó»* era falsa: o modificador age **antes** da pose, então rodar exigiria um nó
  intermédio. Variantes **append-only** ⇒ zero migração. ⛔ E **duas mutações sobreviveram** aos
  primeiros gates, nos dois sítios que **cortam** (a caixa) e **furam** (a pré-imagem) a peça — mais
  um gate que prometia *«erro de compilação»* sobre uma lista escrita à mão.
- ⭐ **W78 (§79): a auditoria da lista viva — DUAS entradas eram trabalho já feito.** Ladrilhar em
  `(u, v)` estava feito desde a W59 (o casco), e o **`Mirror`** — que o Enio adiou porque *«não se
  consegue demonstrar»* — **demonstra-se**: o modificador vai na **operação** e dobra um filho fora
  do eixo. ⚠️ Sexta nota velha desta sessão.
- ⭐ **W77 (§78): a segunda cerca do passo — e a nota mentia sobre si mesma.** Ela dizia «ninguém
  mediu» e havia um gate a medir: **numa esfera, numa banda**. A generalização (formas com **vinco**,
  a caixa inteira, a barra da marcha) dá `1,0852` no pior caso — `30 %` de folga para o `√2`.
  ⚠️ E a barra **demonstrável** é `√3`: a dívida fica escrita.
- ⭐ **W76 (§77): a escultura que perdeu o arquivo pode ser RELIGADA** — o aviso da W23 era um beco
  (a única cura era pôr o arquivo de volta no caminho exacto). O verbo aparece **só a quem perdeu**,
  a chave **nova** é escrita no nó (senão a peça abre hoje e falha amanhã), e a **pose fica**.
- ⭐⭐⭐ **W75 (§76): a cerca do passo da marcha estava ERRADA — e a cena 1 do smoke marchava acima
  do seguro desde que existe.** Arredondamentos exactos **encadeados** compõem o factor (`1,96` a
  três níveis contra o `√2` que o passo supunha), e ⭐ **um nó de `n` filhos já é uma corrente de
  `n − 1`** (o lowering dobra aos pares) — que é a forma da cena 1. O passo passa a ser `1/√2^k`;
  preço medido: `1,01×`/`1,06×`/`1,23×` a um, dois e três níveis.
- ⭐⭐ **W74 (§75): com duas formas escolhidas, a segunda desaparecia em silêncio.** Duas perdas
  em série (a função cozia só a primeira; a caixa de correio era um slot que a segunda apagava).
  ⭐ Uma peça **por forma**, cada uma ligada ao seu desenho — e não uma peça com todas, porque o
  vínculo vivo aponta para **um** desenho e o componente viaja no arquivo.
- ⭐⭐ **W73 (§74): «ao parar ficou mais lento para alisar» — o assentar vira uma ESCADA.** O
  traçado assente nunca mudou; o que mudou foi **onde o alisamento vive**. Dois degraus: primeiro o
  **mesmo tamanho** com o contorno inteiro e o anti-serrilhado (`131 ms`), depois o cheio (`504`) ⇒
  o que ele espera chega **`3,8×` mais cedo**. ⚠️ E a `wants_antialias` da W72 **morreu com um dia**:
  ela perguntava pelo tamanho, e o degrau que alisa pede o mesmo tamanho do de movimento.
- ⭐ **W72 (§73): o quadro de movimento também não paga o anti-serrilhado** — `35,7 → 26,7 ms`
  (`1,34×`), e o corte muda a **borda de um pixel** em vez da forma. ⭐ E a marcha foi medida antes:
  `8,7` amostras por pixel ⇒ **a sobre-relaxação não tem de onde tirar**; o custo é *por aresta
  tocada*.
- ⭐ **W71 (§72): a montagem é `20 %` do quadro — medida, não dividida** (o produto passou a contar
  o tempo dela). ⛔ Isso **fecha** as duas direcções que a W70 tinha nomeado e manda o alvo para a
  **marcha**. ⭐ E o `SLABS` foi de `2` para **`4`**: ele fora escolhido quando montar custava o
  dobro (`1,09×` no caso do preview, `1,19×` no mais pesado).
- ⭐⭐⭐ **W70 (§71): a montagem de fitas era o quadro inteiro — e três em cada quatro fitas não
  eram avaliadas por ninguém.** Por região especializada pagavam-se **quatro** compilações (árvore ·
  fita float · fita de **gradiente**, que só a exportação consome · e um **`fork`** que recompilava
  as duas) e o traçado avalia **uma**. ⭐ `1,65×` a 168 arestas e `1,92×` a 672, com as fitas de
  gradiente a caírem de `293` para **zero** por quadro. ⛔ E a terceira cura — a 2.ª passagem — foi
  construída duas vezes, medida e **revertida**.
- ⭐⭐⭐ **W69: a CURA do report de fps — o contorno também ENGROSSA enquanto a mão mexe.** A lei que
  o módulo já ship (*grosso a mexer, nítido ao assentar*) aplicada onde o custo estava. Medido a
  `640×360`: `168` arestas `55,3 → 52,1 ms` · `472` `133,4 → 54,6` · `940` **`266,1 → 53,7`**
  (**4,96×**) ⇒ ⭐ **o custo de movimento passou a ser CONSTANTE** (~`53 ms`) qualquer que seja o
  nível, e o teto de `Resolution` voltou a **64** com o recurso certo (o quadro **assente**).
  ⚠️ Ela **DECIMA, não recoze** — recozer exigiria a curva de origem, que vive na cena vetorial —, e
  tem três metades que a impedem de mentir: ao parar volta inteiro · um **furo pequeno fica intacto**
  (⛔ um furo de 6 lados desligava a cura para a peça inteira, e foi uma **prova de mutação** que o
  mostrou) · o laço compara o documento **real**. ⏸️ Fica a base, que é a W70
- ⛔⛔ **W68 (§70): a lentidão que o Enio viu é PRÉ-EXISTENTE, e o teto só a tornou visível.** O
  traçado paga **`0,22 ms` por aresta, CEGO AOS PIXELS** (4× menos pixels ⇒ `1,3×` menos tempo) —
  assinatura de **montar**, não de marchar. ⛔ Mesmo na resolução de omissão o piso é `39 ms` contra
  um orçamento de `16,7`. ⚠️ **A minha nota da W67 media o relógio ERRADO** (o quadro assente, pago
  uma vez, em vez do de movimento, pago sempre). Quatro hipóteses refutadas por medição: o
  recozimento (`23 µs`) · o preview mais grosso (tem **piso**, e volta a subir depois de `D≈6`) · o
  teto sozinho · a montagem **base** (`Hybrid::new`) — ⏸️ e o suspeito que ficou (a especialização
  por ladrilho) foi **ilibado** na W70: sem ela o traçado vai de `58` para `565 ms`
- ✅ **W67 (§70): as duas decisões do Enio, e elas deram respostas OPOSTAS.** O teto de `Resolution`
  sobe **16 → 64** (a régua que faltava é um contorno de curvatura **variável**; `θ ≈ √(8·tol/R)`
  confirma-se em quatro pontos ⇒ *dobrar o nível divide o salto de normal por `√2`*) · e a **escada
  de densidade dos quads é RECUSADA por medição**: `Fine` `49 691 ms` com **42 arestas de bordo**,
  `Max` **`27 min 29 s`** com `316` bordo e `6` não-manifold — *o limite não é o tempo, é a
  TOPOLOGIA da extracção*. ⛔ A 1.ª medição atravessou a própria trava (o `clamp` da
  `tolerance_ratio_for`) e leu «o achatamento saturou»
- ✅ **W66: a exportação diz ONDE a peça está** (só quando a origem cai fora da caixa dela — uma peça
  centrada continua calada), e a auditoria do roteador do módulo achou **quatro** itens que já
  estavam fechados ou desactualizados
- ✅ **W65: a Hierarquia diz qual linha está ISOLADA** (selo `ISO`). ⚠️ A decisão foi a
  **PRECEDÊNCIA** — o selo é um por linha e `ISO`/`LNK` caem na mesma: ganha o `ISO`, que é um estado
  da **VISTA** a explicar por que o resto desapareceu, contra uma propriedade do nó
- ⛔ **W64 (§69): a nota do «traçado 2,4× mais caro» estava errada por 4×, e o suspeito era
  inocente.** O anti-serrilhado custa **22–34 %** (não `2,4×`), e especializar a **segunda passagem**
  por ladrilho é **neutro a pior** — *a especialização paga-se por AMORTIZAÇÃO*: `4 096` raios por
  ladrilho na primária contra `~256` na de borda. **Revertido**
- ✅ **W63: a exportação SAI da thread que desenha** — *«o linux fica cinza»*: a 12 s o loop não
  responde ao ping do compositor e o KDE oferece **forçar o encerramento**. ⚠️ **Declarar o
  congelamento cura a MENSAGEM e não cura o congelamento** — são dois observadores. Bancada com uma
  de cada vez, recusa do segundo em alto, e um sentinela que a liberta no `Drop`. ⭐ E tirar o
  trabalho da thread **abriu a porta que o congelamento fechava** (fechar o app a meio deixava meio
  arquivo com o nome certo) ⇒ gravação por temporário + `rename`, **na pasta do destino**
- ⭐⭐⭐ **W62: a exportação caiu de 8 min 17 s para 6,4 s (77×), e o arquivo que sai é o MESMO.**
  *O alvo da cadeia de quads sai da CAIXA, nunca da densidade* (`target_edge = alpha · diagonal`) —
  a grade fina era mastigada pela fase zero e **deitada fora depois de paga**. ⛔ E não é só preço:
  nas profundidades 7-8 a fidelidade medida no campo *piora* e a esfera é **destruída** (`55,5°`)
- ✅ **W61b (§68): a exportação passa pela cadeia de quads** — e ela **bate o oráculo** na esfera
- ✅ **W61 (§67): o PLACAR da malha extraída** contra o estado da arte — topologia e geometria no
  nível, a **forma da face** não
- ✅ **W60 (§66): reconferir a nota que o custo tornava inalcançável** — e o eixo do zoom **não
  existe**
- ✅ **W59 (§65): a região do corte é o CASCO, não a caixa dele** — `1,21×` menos arestas
- ✅ **W58/58b/58c/58d (§61-§64): a selecção múltipla nasce no CANVAS** — `Shift`+clique **alterna**,
  `Shift`+arrasto **SOMA** (e a assimetria é a lei), apanhando também o que está tapado. ⚠️ «não
  seleciona mais de 2» não era um teto: era a **pergunta**
- ✅ **W57 (§60): o vínculo desenho→peça VÊ-SE e SOLTA-SE** (selo `LNK`, `Unlink` / `Link Drawing`)
- ✅ **W56e/W56f (§58, §59): a PROFUNDIDADE** (`SLABS = 2`, medido) e **o passo da marcha é do
  DOCUMENTO**, não uma constante

- 🔶 **W56 (§57): o perfil deixa de ser uma FITA e passa a ser uma CONSULTA — o ALICERCE está posto,
  o produto ainda não o vê.** ⭐ O gatilho que o `04_resultados_perfis` §7 deixou armado em 19/08
  disparou — e **quem o disparou foi a W55** (168 arestas por omissão, 664 no teto do knob). ⛔ **E a
  cura que aquela nota prescrevia foi REFUTADA por leitura**: ela pedia poda por intervalo, e
  **ninguém avalia intervalos neste caminho** — a fita é ponto-a-ponto, sem ladrilho e sem
  `simplify`. ⭐⭐ O tecto foi medido primeiro (Amdahl): o perfil é **92 %** do quadro no default e
  **98 %** no teto ⇒ uma cura perfeita vale **12,5×** e **49,8×**. ⚠️ E a barra é alta: a fita custa
  **0,95 ns por ponto por aresta** — oito faixas de SIMD com JIT, quase óptima *por aresta*; o que se
  ganha é **tocar menos arestas**. A consulta nova (BVH para a distância + grelha com o enrolamento
  pré-somado para o sinal, ambas exactas, com a fita como **juiz**) dá 1,9× sozinha e **3,8×/5,3×**
  quando o lote é compacto ⇒ **3,1×/4,9× no quadro**. ⛔ Duas metades faltam para o produto ver isto:
  ⭐⭐⭐ **e a saída não era sair da árvore, era ESPECIALIZÁ-LA** (§57.12): a mesma lei com uma
  fracção das arestas — distância pelas que podem ganhar o `min`, sinal por uma **constante** mais os
  atravessamentos da região — dá **32,5×** (168) e **42,7×** (664) e **satura o tecto**, mantendo
  fita única, JIT, gradiente exacto, modificadores e poses. ⛔ A folha nativa foi **recusada por
  produto**, não por velocidade: ela perderia os modificadores e a quina viva. ⏸️ Falta **uma** metade
  — a marcha por região —, e a região é em espaço **local**, logo independente da câmera.
  ⭐⭐ **E o CONSUMIDOR existe** (§57.14): a marcha passa a ser por **ladrilho**, com o raio preso à
  caixa da peça — **1,8×** no quadro (167 → 92 ms a 168 arestas). ⚠️ Não os 5–6× projectados, e o
  mecanismo está medido: um raio de viés varre em `(u, v)` muito mais do que a largura do ladrilho ⇒
  a pegada efectiva é ~`0,4` e não `0,125`. ⏸️ O degrau seguinte é ladrilhar em **profundidade**.
  ⛔⛔ **Três defeitos só o gate de IMAGEM os apanhou** (§57.15) — a região que era a peça inteira
  (lento, e invisível a um gate de paridade), as sondas da normal a saírem da região (90 pixels
  apagados), e ⭐ a regra do atravessamento que tinha de ser **semiaberta**: um caminho que passa por
  um **vértice** contava zero em vez de um, o sinal invertia-se numa cunha fina e a marcha **inventava
  uma superfície**. ⚠️ **E os três sobreviveram a gates verdes por causa da FIXTURE** (§57.16): 21 das
  22 mutações ficaram vermelhas à primeira, e as três sobreviventes eram exactamente estas.
  ⚠️ **E metade da wave foi escrita na árvore ERRADA** — a cwd
  escorregou para o primário e tudo compilou lá; quem o apanhou foi o caminho absoluto do arnês de
  mutação.

- ✅ **W55 (§56): o contorno continua a ser a FONTE — e o knob que faltava era a mesma ausência.** ⭐
  Os dois ⏸️ que a W54 deixou (*"não há knob de resolução"* e *"o nó não se religa ao contorno"*) eram
  **um só**: o `+ Extrude` cozia uma vez e a peça deixava de conhecer o desenho, então afinar a
  conversão era **inexprimível**. O `FieldProfileSource { path, level }` resolve os dois — editar a
  curva remodela a peça, e a linha **Resolution** aparece exactamente onde tem onde escrever. ⚠️ Sem
  cache, de propósito: recozer custa **7 µs** e comparar **0,2 µs** contra um quadro de 16,7 ms, e um
  resumo guardado seria estado derivado a envenenar o undo. ⚠️ O nível guarda a **intenção**, nunca a
  tolerância cozida. ⚠️ O teto do nível (**16**) sai de uma tabela medida, e ela trouxe de graça a
  medida do **próprio instrumento**: o mesmo traçado deu **184,1 ms** a `load ≈ 4,7` e **139,3 ms** a
  `load < 3` — ⭐ *32 % só de carga*. ⛔ E a leitura desenterrou um defeito de W26: `copy_subtree`
  copiava uma lista escrita à mão e **largava o `FieldMods`** — duplicar um cilindro oco devolvia-o
  maciço, em silêncio; a cura leva uma **censura** presa ao registo de componentes. ⛔⛔ E o fecho
  correu a suíte **inteira** do shell pela primeira vez em waves: o gate de **LOC** estava vermelho
  com **quatro** arquivos, três deles antes desta wave (§56.8) — *um gate de árvore não é alcançado
  por um filtro de nome*, que é a irmã da lição do clippy da W44. Curado por **quatro cortes para o
  irmão**, cada um numa fronteira que já existia por dentro. ⏸️ Fica: a tabela do teto pede uma
  corrida com a máquina parada · nada na Hierarquia mostra que uma forma está ligada · não há gesto
  para largar nem para religar o vínculo · um contorno de cada vez
- ✅ **W54 (§55): a régua da suavidade é a NORMAL** — Enio, com duas fotos: *"sem ajustes de
  resolução"*. ⭐ A minha primeira hipótese (a polilinha na silhueta) foi **refutada pela aritmética**:
  ela erra **0,079 % da peça**, invisível. As bandas estão na **LUZ** — o campo de um polígono tem
  gradiente constante por segmento, e a normal salta **6,43°**. A tolerância passa a `1e-4` pelo
  **joelho medido** (degraus **56×** menores; o passo seguinte custa +70 % para nada). ⛔⛔ E a tabela
  de 19/08 estava **desmentida por 2,4×** — o traçado engordou desde a W3 e ninguém reconferiu
  (suspeito nomeado: o anti-serrilhado, que re-amostra a borda 4×). ⛔ O gate que lá estava media
  **arestas** (o custo) e defendia o número velho. ✅ **O knob FECHOU na W55** (§56), e pela ligação
  que esta nota previa. ⏸️ Fica: o traçado 2,4× mais caro por explicar · a normal suavizada, nomeada
  e recusada
- ✅ **W53 (§54): o PERFIL DESENHADO vira peça** — ⛔ `Extrude` e `Revolve` existiam no motor **desde
  a W3**, medidos contra oráculos, e **nenhum botão os alcançava**: uma família de features completa
  e invisível, e o plano chama-lhes a razão de existir do módulo (*"é aqui que o fluxo do MoI
  renasce"*). ⚠️ O gate da W34 não a apanhava por uma **exclusão correta**: ele pergunta *"o painel
  oferece o que a seleção permite?"*, e o que faltava é *"o painel oferece tudo o que o MOTOR sabe
  fazer?"*. ⭐ A ponte já existia inteira (`cook_path_auto`) — a wave escreveu o **gesto**, não
  geometria. Dois gates existentes reprovaram e os dois estavam a trabalhar (um deles **previa** isto
  no próprio comentário). ✅ **O religar FECHOU na W55** (§56). ⏸️ Fica: um contorno de cada vez ·
  nada mostra o eixo do *Revolve* antes do clique
- ✅ **W52 (§53): a viagem NÃO é do *reduced motion*** — Enio: *"o lerp não deve estar vinculado ao
  Reduced Motion. Mas deve ser o único modo."* ⚠️ O smoke da W51 leu *"não funcionou, está como
  antes"* e **o código estava certo**: a preferência dele diz `reduced_motion=1`, e o papel de então
  (`Surface`) morre ali. ⛔ A armadilha estava anotada no `CLAUDE.md` §5 — *"um `reduced_motion=1`
  esquecido reprova smokes sobre produto correto"* — e eu não li o arquivo antes de pedir um juízo
  **sobre movimento**. ⭐ A cura é um **papel novo** (`Role::Viewpoint`), não uma excepção, com o
  critério escrito: *aqui o CORTE é pior do que o movimento*; e ele é estreito, com gate a exigir que
  um percurso comum continue a morrer. ⏸️ Fica: a janela 3D da escultura tem o mesmo problema, e
  ligá-la é decisão da linha dela
- ✅ **W51 (§52): a VIAGEM entre vistas** — pedido do Enio (*"falta um Lerp() rápido […] como no
  blender"*). ⭐ **A curva e a duração são as da CASA**, não minhas: `Role::Surface`, cujo doc
  descreve este caso à letra (*"viaja… e **nunca ultrapassa**; uma roda nomeia um destino, e passar
  dele e voltar lê como a régua a mentir"*) — uma vista nomeada é um destino. O *reduced motion* sai
  de graça. Slerp pelo **caminho curto** (com gate a medir o comprimento do percurso), alvo linear,
  enquadramento **geométrico**. ⭐ **UMA porta** (`fly_to`) em vez das cinco escritas de câmera, e a
  **mão cancela**. ⚠️ O `RefCell` re-entrante mordeu **duas vezes no mesmo dia** — a cura agora é
  estrutural (corpo separado da porta), não memória. ⏸️ Fica: a linha do laço que serve o progresso
  não é alcançável de um gate · o custo do traçado durante a viagem não foi medido
- ✅ **W50 (§51): a MOLDURA do app empurra o gizmo** — Enio, no smoke da W49: *"fica escondido entre
  botões"*. A área é o viewport inteiro e a moldura pinta por cima; o gizmo passa a viver na **parte
  livre**. ⭐ A lei é a **fuga mais barata** (um painel alto sai pela direita, uma faixa larga sai
  pelo topo) — a primeira, por *«toca a aresta»*, contava um painel da altura toda como faixa do topo
  e baixava o gizmo 600 px. Iterativa, com gate de ordem, e **local** (a tira do Flip não o move).
  ⚠️ Acessor novo `WidgetStore::panel_rects()` em vez de uma segunda lista de ids. ⛔⛔ **E o arnês de
  mutação deu por apanhada uma mutação que não apanhou**: o gate rebentava sozinho (`RefCell`
  re-entrante) e os dois controles positivos passaram — faltava o **verde antes do vermelho**, que a
  memória do projeto já exigia e o arnês não. ⏸️ Fica: a chamada que publica a parte livre não é
  alcançável de um gate
- ✅ **W49 (§50): o GIZMO DE NAVEGAÇÃO (bolas de eixo)** — pedido do Enio, e ele mandou **pesquisar
  antes de construir**. ⛔ O ViewCube do Fusion está sob patente **viva até 2029-03-06** (Autodesk,
  US 7.782.319). ⭐ E a própria pesquisa da Autodesk mede que o ganho vem do **arrasto**, não do
  cubo (*"quase 2× mais rápidos, independentemente das representações examinadas"*) — por isso
  arrastar orbita é o gesto principal. Decisão dele: bolas de eixo. Números **derivados** do gizmo
  3D (raio de agarre, espessura, cores dos eixos). ⚠️ Duas mutações sobreviventes acharam buracos
  reais: **espelhar o widget na vertical passava em tudo**, e a guarda `nav.is_none()` era **código
  morto** (apagada, não gateada). ⏸️ Fica: sem letras nas bolas · sem cantos/arestas (as 26 direções
  do cubo) · o salto não é animado · sem gate de pintura
- ✅ **W47 (§48): as SEIS VISTAS existem, e a câmera passou a ser alcançável** — o item que o plano
  chama **⭐ «É o produto»** desde 19/08. ⭐ A medição mudou o entregável: o módulo **já pinta no
  viewport inteiro**, então um cabeçalho dentro do canvas seria uma **segunda superfície de UI** do
  mesmo módulo (hit-test, ids e lei de alcançabilidade próprios) — os controles vão para o **painel**,
  que é a decisão que o plano já tinha tomado. ⛔ O buraco real: **a câmera nunca passou pela lei da
  W34** — as vistas não existiam, e a lente e o enquadrar eram alcançáveis só por tecla. ⚠️ Lemos as
  **teclas** do Blender, não os **eixos** (ele é Z-up, nós Y-up), e o gate confere o **eixo do olho**,
  não a aritmética. ⭐ A vista é **derivada** da orientação, nunca guardada. ⏸️ Fica: sem vista oposta
  rápida nem *enquadrar a seleção* · as vistas não forçam a lente paralela (produto) · quad-view fora
- ✅ **W46 (§47): a peça nasce ENQUADRADA, e o `Home` passou a encontrá-la** — o ⏸️ que a W45 deixou
  uma hora antes. ⚠️ Uma peça longe da origem abria **fora do quadro**, e a tela voltava a ficar
  vazia: *o mesmo sintoma que a W45 existiu para curar, por outro caminho*. ⛔ E o `Home` **não** era
  a saída (eu disse ao Enio que era): ele punha o alvo na **origem**. A lei é a da referência — no
  Blender `Home` é *View All*; tínhamos herdado a tecla e metade do significado. ⭐ O bordo é o
  **mesmo** do exportador (W33). ⭐ A folga saiu de uma varredura: `1,00` deixa **144 pixels** da
  peça na moldura (a lente é convergente e o lado virado à câmera projeta maior que o raio), e
  **1,10** zera. ⚠️ A varredura só o disse depois de a fixtura mudar para uma esfera **sozinha** —
  com a união de duas, todas as folgas davam zero. ⏸️ Fica: enquadrar a **seleção** · o salto não é
  animado · o teto de `half_extent` trunca uma peça enorme **em silêncio**
- ✅ **W45 (§46): um projeto que traz uma PEÇA abre o painel dela** — o ⏸️ da W35. ⛔ **A porta estava
  trancada por dentro:** o pedido de abrir só era aceite com o módulo **armado**, e o único caminho
  que o arma é a visibilidade do painel — *para pedir a abertura era preciso já estar aberto*. A obra
  atravessava o arquivo (W35) e a tela ficava vazia, indistinguível de a ter perdido. ⚠️ O load
  deixa a **pergunta** e o quadro responde-a (o mundo vive no `gfx`, e o load corre sem janela — a
  forma é a do `sculpt3d_install_pending`). ⭐ E a pergunta certa foi escrita por uma **mutação
  sobrevivente**: *"há raiz"* e *"há nó"* são a mesma coisa (o `spawn_doc` dá `FieldNode` à raiz
  sempre), e o que separa é o **cozimento** — *«há alguma coisa PARA VER?»*. ⏸️ Fica: um projeto com
  peça **e** escultura abre a escultura e nada diz que também há peça · o painel abre mas não
  **enquadra** a peça
- ✅ **W44 (§45): o isolamento DIZ-SE e SAI-SE de qualquer sítio** — os dois ⏸️ da W38, e a medição
  encontrou um terceiro item que era o defeito: ⛔ **o único sinal estava preso à SELEÇÃO**
  (`isolated == selection.first()`), então isolar `A` e escolher `B` apagava-o, e com a **raiz**
  escolhida a fileira inteira desaparece — nem indicador, nem porta. ⚠️ E o *toggle* fora escolhido
  com a razão *"a porta que o artista não acha quando a cena some"*: **o chip não a cumpria**, e é
  justamente para a raiz que se vai quando sumiu tudo. ⭐ `Shift+I` (a tecla do módulo irmão, lida)
  responde de qualquer sítio, com lei **própria** — a tecla é global (*dentro ou fora*), o chip é de
  uma linha (*mostra-me ESTE*), e há `assert_ne!` a prender a divergência. ⏸️ Fica: a **Hierarquia**
  não tem marca própria (quem anuncia é o painel) · isolar **vários** nós
- ✅ **W43 (§44): a VISTA sobrevive ao fechar o painel** — o ⏸️ que a W42 deixou. ⭐ E não era «a
  câmera»: o próprio `Smoke` já classificava **cinco** campos como *estado de vista*, em três
  doc-comments distintos (*"é estado de **vista**, e não do documento"*) — *um doc-comment repetido
  em N campos é uma estrutura por nascer*. A `cam` viaja com o `manual`, ou o prato desfaz o ângulo
  restaurado no quadro seguinte. Um campo novo no `Smoke` é **erro de compilação** no sítio onde a
  pergunta *"vista ou cache?"* tem de ser respondida (destructuring sem `..`). ⭐ E a W42 tornou
  escrevível o gate comportamental que a W38 declarara impossível: desarmar limpa-se a si mesmo.
  ⏸️ Fica: a vista é de **processo** (não é salva no arquivo), e a Hierarquia continua sem mostrar
  que há um isolamento em curso — agora com mais alcance, porque ele também atravessa um fecho
- ✅ **W33 (§34): a caixa da grade do EXPORTADOR passou a ser a da peça** — uma peça fora de
  `[-1,1]` era cortada em silêncio, e uma peça pequena gastava resolução em vazio (**>3×** de
  ganho medido). ⏸️ Fica: a exportação **não diz o tamanho** da peça, e agora que o bordo existe
  isso é uma linha
- ✅ **o OLHO da Hierarquia passou a valer na W28** (§29) — esconder um nó tira-o da peça, e um
  grupo leva a subárvore consigo; um nó escondido não tem gizmo nem anda com a seleção.
  ⏸️ Fica **isolar** (mostrar só o escolhido), que é o gesto irmão
- ✅ **o CADEADO passou a valer na W29** (§30) — pelo predicado da casa, com a metade do
  `GroupedChildren`. ⛔ Ele **não** tranca os números do painel, e isso é lido do doc do componente,
  não decidido aqui
- ✅ **arrastar uma linha na Hierarquia deixou de TELEPORTAR a peça na W30** (§31) — a lei do
  mundo-preservado da casa não alcançava o tipo da pose deste módulo. ⏸️ Fica: re-parentar muda
  a **peça** (um cilindro dentro de uma subtração passa a cortar) e ninguém o diz
- ✅ **W42 (§43): DESARMAR não desarmava** — a W40 fechava o painel e o módulo continuava a comer o
  ponteiro. Duas notas a mentir: o doc do `with_smoke` prometia inércia que o código não fazia (o
  estado de armado só era lido **antes** de o smoke nascer), e a bandeira **travava ligada** por uma
  razão que **dissolveu na W5** (o medo era perder a peça; ela é uma árvore de entidades desde
  então). ⭐ E a ordem do despacho explicava a assimetria: a escultura vê o clique **antes** da
  modelagem, o Vector **depois**. ⏸️ Fica: fechar o painel larga a **câmera** (a peça não)
- ✅ **W41 (§42): o CRASH que o smoke da W40 encontrou na escultura** — apagar a última peça e
  clicar derrubava o app (`index out of bounds`, `sculpt3d_input.rs:173`). A cena vazia é um estado
  **legítimo** (o `delete` promete o Ctrl+Z) e os caminhos de gesto supunham-na impossível. ⛔ A cura
  completa são **42** indexações sem guarda em 9 arquivos da `line/sculpt3d`; esta linha fecha **a
  porta que o artista bateu** e nomeia o resto. ⏸️ Fica: as outras 4 portas · e o pânico terminou em
  **SIGSEGV** (o app crasha ao crashar, e perde o relatório)
- ✅ **W40 (§41): o modelador CEDE o canvas** — Enio, 22/08: *"não consigo esculpir nada pois o modo
  de modelagem permanece interferindo"*. ⚠️ **É o mesmo report que a escultura já pagou duas vezes**
  (09/08 e 17/08), um nível acima: lá *"o ponteiro cedia e o teclado não"*; aqui **nenhum dos dois**.
  A lei é *tomar o canvas liberta quem o tinha*, em duas metades simétricas e **de borda** (contínua
  criaria impasse: não há gesto de largar uma ferramenta). ⏸️ Fica: reabrir o MODEL com ferramenta em
  mãos deixa as duas de pé · o modelador não tem pill próprio (o painel É o interruptor)
- ✅ **W39 (§40): a escultura da cena entra SEM passar pelo disco** — botão `+ Sculpt from scene`,
  oferecido só quando há uma. ⛔ **E a medição proibiu o «vivo» contínuo**: voxelizar custa
  **229–389 ms** a 128³ (1,5 s a 256), contra um quadro de 16,7 — são 14 a 23 quadros por pincelada.
  A decisão já estava escrita no doc da `DEFAULT_RESOLUTION` (*"o custo é pago uma vez, na
  importação"*). ⏸️ Fica: a escultura **não se atualiza sozinha** e nada o diz · **uma** escultura
  por peça (a chave é fixa) · o reencontro ao reabrir o projeto **não foi smokado**
- ✅ **W38 (§39): ISOLAR (mostrar só o escolhido) e o grupo que nascia MUDO** — os dois itens da
  fila. ⭐ A lei do isolar foi **lida** no módulo irmão (toggle · não entra na história · isolar
  «nada» é recusado), e o mecanismo era **zero**: o `cook` já cozia *"a subárvore de `root`"*, então
  isolar é cozer a partir daquele nó. Faltava **uma linha** — a pose da cadeia acima, senão a peça
  isolada salta para a origem. ⏸️ Fica: sem tecla · sem gate comportamental da costura painel↔smoke
  (o estado do smoke é `thread_local` e armá-lo contaminaria os vizinhos) · nada mostra na
  **Hierarquia** que há um isolamento em curso
- ✅ **W37 (§38): a mensagem deixou de viver UM quadro** — o diálogo de arquivo congela o loop, e o
  `wall_dt` do quadro seguinte cobrava esse congelamento ao relógio do chrome, matando o toast antes
  de alguém o ver. ⛔ **É defeito da CASA:** 25 chamadas de diálogo em 12 arquivos, e esta wave liga
  as 2 do módulo. ⏸️ Fica: as outras **23** (tabela no §38.3), e se um resultado de exportação devia
  durar mais do que os 3 s de um aviso passageiro (produto)
- ✅ **W36 (§37): a exportação diz o TAMANHO da peça** — e havia **dois** números disponíveis, não
  um: o bordo é **andaime** (cúbico e conservador; até **28×** o eixo curto de uma peça fina), e o
  que se diz é a caixa da **malha que saiu**. ⛔ Numa esfera os dois coincidem, então uma conferência
  feita só nela teria confirmado o número errado. ⏸️ Fica: nada diz **onde** a peça está (o canto
  mínimo), que importa a quem monta várias peças no mesmo arquivo
- ✅ **W35 (§36): a peça ATRAVESSA o arquivo — e a nota que dizia o contrário era velha.** Ela é uma
  árvore de entidades, o `ProjectState` é o mundo inteiro, e o `PROJECT_SCHEMA` **não se mexe**. O
  que faltava era estreito: a memória de *"já tentei ler esta escultura"* era do **processo**, então
  um arquivo consertado no disco nunca era relido — e o segundo silêncio era idêntico ao de quando
  estava certo. ⏸️ Fica: o Ctrl+S de verdade **não é alcançável de um gate** (o save exige `gfx`), e
  o módulo **não se abre sozinho** quando o projeto carregado traz uma peça
- ✅ **W34 (§35): o painel passou a oferecer EXATAMENTE o que o gesto faz** — a fileira de operações
  aparece com **uma** forma escolhida (o gesto de criar grupo, que a W31 escreveu e ninguém
  alcançava), e a de *Duplicar/Apagar* deixa de ser pintada sobre a **raiz**, que as recusa. A lei
  vale para as três fileiras que dependem da seleção. ⏸️ Fica: um controle que dependa de outra coisa
  que não a seleção não é apanhado por ela
- ✅ **W31 (§32): um objeto largado em cima de outro deixou de SUMIR** — só uma operação pode ter
  filhos, e a forma anfitriã é promovida a grupo (a peça na tela não muda). E **criar grupo** passa
  a ser um gesto: uma forma sozinha + um botão de operação — ⚠️ **alcançável só desde a W34**, que é
  quando o painel passou a pintar a fileira nesse caso. ⏸️ Fica: ninguém **diz** que um grupo
  nasceu, e não há grupo VAZIO
- ✅ **orientação Global/Local FECHOU** na W7 (§6)
- ✅ **rotacionar e escalar FECHARAM** na W6 (§5)
- ✅ **clicar na peça para selecionar FECHOU** na W7 (§6) — e o custo está medido: **0,10 ms**
- ✅ **snap e leitura numérica FECHARAM** na W8 (§7)
- ✅ **o undo de um arrasto FECHOU** na W6 (§5) — e a nota que estava aqui estava **errada**: a lei
  do shell já existia e o que faltava era o módulo dizer-se. *Meça o mecanismo antes de construir o
  que a nota prescreve.*
- ✅ **duplicar e apagar FECHARAM** na W11 (§10)
- ✅ **a rotação em números FECHOU** na W14 (§14) — e com ela o piso de toda linha
- ⏸️ **o ESPELHO não se consegue demonstrar** (Enio, smoke da W17): ele dobra em torno do centro do
  nó, e toda peça das cenas — folhas *e* grupos — é simétrica. O verbo está correto e gateado; o que
  falta é um alvo descentrado, ou um pivô de espelho autorado. Adiado por decisão dele
- ✅ **draft/taper FECHOU** na W18 (§19), e a W4 do plano com ele — o primeiro operador não-exato do
  módulo, com as duas tabelas ao lado do número
- ✅ **a W5 FECHOU**: o motor na W21 (§22), a **autoria** na W22 (§23) e o **regresso** na W23 (§24) —
  o botão `+ Sculpt…` traz um arquivo de malha para dentro da peça, a booleana corta-o, e reabrir o
  projeto **regenera-o do arquivo que o nomeia** (o que não voltar **fala**, com o nome do arquivo).
  ⏸️ Fica: um arquivo que **mudou de sítio** não se reencontra (religar pede UI, e é a pergunta que o
  app ainda não faz por asset nenhum), a ligação à escultura **viva** do módulo 3D (hoje o vínculo
  passa pelo disco e acorda ao **abrir**), e os modificadores sobre uma escultura
- ✅ **a LENTIDÃO que o Enio nomeou no smoke da cena 6 FECHOU na W24** (§25): a resolução do preview
  passa a sair da **medição** — 4,2× e 7,3× mais rápido em movimento, sem um segundo motor. ✅ E a
  espera que sobrava (até 121 ms) **FECHOU na W32** (§33): um refinamento cede à mão, um traçado de
  movimento nunca. ⏸️ Fica: o trabalho abandonado é deitado fora, não reaproveitado
- ✅ **o ERRO na UI FECHOU na W25** (§26) — a peça que não cozinha diz porquê, e o clique que a
  apagava (um modificador sobre uma escultura) deixou de existir. ⏸️ Fica: o aviso não aponta **qual**
  nó é o culpado
- ✅ **digitar o número durante o arrasto FECHOU na W26** (§27) — a ficha passou a aceitar, o número
  é o **total** e `Esc` desfaz o gesto inteiro. ⏸️ Fica: escolher o eixo por tecla (exige gesto modal)
  e contas na entrada
- ✅ **o gesto passou a ser da SELEÇÃO INTEIRA na W27** (§28), e com ela o **pivô do centro da
  seleção**. ⏸️ Fica o pivô **escolhido** (o cursor 3D do Blender), que é produto e pede a UI que o
  põe lá — e um laço de seleção na janela 3D (hoje escolhe-se na Hierarquia com `Ctrl`)
- ✅ **perspectiva FECHOU** na W15 (§16) — entrou num sítio, como a nota previa, e revelou que o
  raio era construído em dois. `Numpad5` alterna as lentes

