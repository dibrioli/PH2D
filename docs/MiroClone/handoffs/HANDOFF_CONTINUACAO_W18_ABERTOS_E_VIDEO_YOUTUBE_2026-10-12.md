# CONTINUAÇÃO — `line/MiroClone` W18: os abertos das imagens + o VÍDEO do YouTube no quadro (2026-10-12)

✅ FECHADO em 12/10 — ver [handoff de integração W18](HANDOFF_INTEGRACAO_line_MiroClone_W18_2026-10-12.md)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W17 FECHOU com **smoke OK do dono (12/10)** —
> [handoff de integração W17](HANDOFF_INTEGRACAO_line_MiroClone_W17_2026-10-12.md) (integra-se JUNTO com W16, W15, W14 e
> W12+W13; **não integre, não pushe**: integrar é só por ordem do dono). Ordem do dono (12/10): *«vamos continuar
> implementando o que está em aberto mais 1 coisa: quadro youtube (através de um link o quadro exibe um vídeo youtube
> como o navegador)»*.

## O que já existe e NÃO se reconstrói

Tudo o que os handoffs W14–W17 §1 listam: a imagem como ficheiro por conteúdo (ADR-0184), a pirâmide e os fios de
descodificação (`ph2d-board-render/src/image.rs`), o GIF que toca (`anim.rs`), o recorte, o espelho, o `ExportCache`, o
«Send to board…», o aviso da imagem estragada (`Images::take_failed` → `live.notice`), as cenas 15–18. O oráculo
Excalidraw 0.18.1 está instalado (`ferramentas/excalidraw_oracle/`; README §W16 e §W17 têm as sondas das imagens —
reutilize a infra-estrutura da `sonda_imagem_w17.mjs`). ⛔ Recusas medidas de pé: descodificar no fio da janela; a
feature `text` do usvg (uma 2.ª stack de fontes ao lado do `parley`, `ph2d-vec-svg/Cargo.toml`).

## PARTE A — os abertos, num bloco só (§0.10): confirme cada um no CÓDIGO e reproduza-o com um teste VERMELHO antes de curar

1. **O GIF na APRESENTAÇÃO mostra só o 1.º quadro** (`board_team_show.rs` desenha com o `ExportLook` Png). O dono pediu GIF
   animado (W16) e a apresentação é um ECRÃ, não um export ⇒ toca lá também (o relógio dos `Players`). Diga-o no
   relatório como escolha derivada do pedido dele, para ele confirmar no smoke.
2. **Um fio de tocador por GIF à vista, sem tecto** (W16 (d)). §0.0: MEÇA antes de limitar — régua com 1, 10, 50, 200 GIFs
   à vista (fios vivos, RSS, CPU, tempo de quadro), as variantes no MESMO processo. O tecto, se houver, diz DE QUE RECURSO
   é, com a tabela ao lado; a cura natural é partilhar o pool dos fios de descodificação, não cortar GIFs.
3. **Os `<text>` de um SVG não se desenham.** A recusa é a 2.ª stack de fontes, não o texto. Rota a medir: o NOSSO motor
   (`ph2d-text`/`parley`) molda os `<text>` (o `roxmltree` que o usvg re-exporta já os lê — é quem os conta hoje) e
   desenha-os como contornos por cima/dentro do raster de cada nível e no vectorial do export. Corra antes o oráculo
   (Chromium a mostrar um SVG nosso com `<text>`: família presente, ausente, `text-anchor`, `tspan`, `transform`) e grave
   a fixture; o aviso fica para o que não se exprime (família ausente → diga qual). Se a fidelidade não fechar, a
   recusa ganha a MEDIDA e o porquê (tabela `⛔ Recusas MEDIDAS`), não um meio-feito.
4. **Um ficheiro CORTADO não mostra nada do que se lê** (W17: o Chromium desenha as linhas de cima). Descodificação
   tolerante: o que o `png`/o JPEG entregarem até ao erro entra, o resto transparente — e continua a ser «estragada»
   (o aviso dispara igual; o desenho mostra as linhas lidas + a marca de estragada num canto). Gate com as entradas
   `entradas/w17_*` e os números do `saidas/w17` (onde a imagem acaba em `y`).
5. **O nome em branco no «Send to board…»** só chega por objectos nascidos sem `Name`: confirme no código se o
   renomear da Hierarquia recusa o vazio. Se recusar, nada a fazer (diga-o no handoff); se aceitar, é o mesmo caminho já
   curado.

## PARTE B — o VÍDEO do YouTube no quadro

**Produto (o dono):** colar/largar/escrever um link do YouTube num quadro faz nascer um elemento que mostra o vídeo e o
TOCA ali, como o navegador (play/pausa, som, avançar, ecrã inteiro), e que se move, redimensiona, copia e desfaz como
uma imagem. ⚠️ Não é «abrir no navegador».

1. **ORÁCULOS primeiro (§0.9), e grave as saídas com cabeçalho.**
   - Excalidraw 0.18.1 (instalado): o `embeddable`. Sonda: colar `youtube.com/watch?v=`, `youtu.be/`, `/shorts/`,
     `/embed/`, `/live/`, com `t=`/`start=`, playlist, e um link que não é vídeo; que elemento nasce (`type`, `link`,
     tamanho, proporção, `validated`), o que se desenha parado e a tocar, como se activa (clique? selecção?), o
     redimensionar, o export PNG/SVG (que lugar fica), o `.excalidraw` gravado. ⚠️ O corredor BLOQUEIA a rede: corra uma
     vez sem rede (o que ele mostra offline) e uma com o domínio do YouTube permitido.
   - O Miro é a referência de produto do quadro (`01_pesquisa_miro_excalidraw.md`): o cartão de vídeo (miniatura, botão
     play, o tocador no lugar). Sem ficheiro de instalação: a pesquisa já feita + o que o oráculo correr.
2. **ADR antes do código** (`/pd-adr`; o número CONTA-SE contra o máximo do `main` e das linhas — hoje a linha vai em
   0184). A pergunta: *como um app Rust/Vello/winit mostra o tocador do YouTube dentro da tela?* Alternativas a MEDIR,
   não a opinar:
   - (i) **webview do sistema como janela-filha** por cima do retângulo do elemento (o tocador oficial do YouTube por
     `youtube-nocookie.com/embed/<id>` — dentro dos termos dele): retângulo sem rotação, sempre por cima dos overlays,
     reposicionado a cada pan/zoom. Linux primeiro (CachyOS, KDE Wayland — o ecrã do dono): subsuperfície Wayland ou
     X11? Triagem de licença POR ARTEFACTO (`pacman -Qi webkit2gtk-4.1`, o crate `wry`) e a compatibilidade com o nosso
     `winit` (`bash scripts/stack-audit.sh --tetos`, `STACK_VERSOES.md` — nunca uma versão de memória).
   - (ii) **renderizar a web fora do ecrã para uma textura** (CEF/Servo) — o custo de build e de memória medido.
   - (iii) ⛔ **tirar o fluxo do vídeo** (yt-dlp + descodificar): viola os termos do YouTube — registe a recusa no ADR.
   - O desenho provável (confirme por medição): **a miniatura no canvas SEMPRE** (export, pan, zoom, rotação, ao vivo,
     sem rede) e o **tocador vivo quando activado**, ajustado ao retângulo do elemento enquanto ele está de pé, visível e
     não tapado; ao rodar, tapar ou sair da vista, volta à miniatura no instante em que parou. Critério de morte ANTES do
     build (DIRETIVA §5): p.ex. se o tocador não seguir o pan sem atraso de mais de 1 quadro no KWin do dono depois da 2.ª
     tentativa, a forma (i) não entra e a decisão volta ao dono.
3. **Modelo (append-only):** um elemento novo no `ph2d-board-model` (o link canónico, o id do vídeo, o início em
   segundos, a proporção 16:9 por omissão); desfazer, copiar/colar, duplicar; **ao vivo** viaja o LINK (cada par carrega
   o seu tocador; o estado de reprodução NÃO se sincroniza — diga-o ao dono como escolha); `.excalidraw` lê e escreve o
   `embeddable` com `link` (hoje SALTA com aviso, X3 — o contador do aviso muda); export: PNG/PDF com a miniatura + o
   símbolo de play + a ligação (`/URI` no PDF, `<a>` no SVG).
4. **Entradas:** `Ctrl+V` de um texto que é link do YouTube → vídeo (outro link continua como hoje); largar um link do
   navegador (`text/uri-list`); um item na barra do quadro. A lei de reconhecer o link é UMA função com gate sobre a
   tabela de formas medidas no passo 1.
5. **Rede:** a miniatura e o tocador só com rede, nunca no fio da janela, com prazo (o molde do `clipboard.rs` da W16);
   sem rede, o lugar com o título/link e uma frase na tela. A miniatura entra na cache do quadro (não a pedir a cada
   quadro).
6. Código da família em `crates/ph2d-app-board` / crates novas (`drop-crate`); a shell só COMPÕE (catraca
   `the_shell_only_shrinks`); UI só tokens + `ph2d-i18n` (HR-15). O PH2D terá versão no navegador mais tarde (decisão do
   dono, 06/10): desenhe o elemento para que lá seja um `iframe` sem mudar o modelo.

## Fecho

Portão batched 1× (`nextest-impacted` com `BASE=merge-base`, `CARGO_BUILD_WARNINGS=deny cargo check --workspace
--all-targets`, clippy do `ship.sh`, `fmt`, `typos`, `machete`, `deny`, `file_loc_caps`, censos da árvore combinada),
auditoria de 2 lentes, mutação nas leis novas. Smoke: a cena 19 (conte no roteador `smoke.rs`, hoje `max_level: 18`)
com um vídeo do YouTube a tocar, o GIF na apresentação, um SVG com texto e a imagem cortada que mostra o que se lê —
lições em notas que CABEM (fotografe e leia a foto: a W16 e a W17 cortaram lições), fotografada com
`PH2D_GPU=1 bash scripts/ph2d-run.sh env PH2D_GPU=0 bash docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome>
PH2D_BOARD_SMOKE=19`. Handoff de integração W18 (≤ 15 KB), binário `smoke` compilado 2×, `rm -rf target/*/incremental`.
Relatório ao dono curto e sem jargão, smoke em passos numerados.
