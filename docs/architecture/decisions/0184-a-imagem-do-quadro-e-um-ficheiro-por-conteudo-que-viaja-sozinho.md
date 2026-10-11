# ADR-0184 — A imagem do quadro é um FICHEIRO por conteúdo, guardado no quadro e que viaja sozinho

- **Status:** Accepted (decisão técnica delegada — `feedback_architecture_decisions_are_delegated_to_the_gold_standard`;
  produto do dono 11/10: *«ainda não podemos arrastar imagens nem para os quadros nem para o canvas»*)
- **Data:** 2026-10-11
- **Linha:** `line/MiroClone` (W14, [continuação](../../MiroClone/handoffs/HANDOFF_CONTINUACAO_W14_IMAGEM_NO_QUADRO_2026-10-11.md))
- **Amparo:** [ADR-0181](0181-o-quadro-ao-vivo-funde-por-campo-e-o-transporte-e-uma-porta.md) (registos por campo,
  o transporte com `MAX_FRAME`) · o oráculo CORRIDO Excalidraw 0.18.1
  ([`excalidraw_oracle/`](../../MiroClone/ferramentas/excalidraw_oracle/README.md) §«W14», `saidas/w14/`)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — contado contra o máximo desta linha (`0183`, que o `main`
> de 11/10 ainda não tem). Quem integrar reconta-o.

## Contexto

O quadro não tinha imagens (`ElementKind`: forma, seta, traço, frame); um `.excalidraw` com `type:"image"` saltava
com aviso, e largar ficheiros num quadro era recusado. Uma imagem são MEGABYTES; um elemento do quadro custa ~200 B
e é reescrito inteiro a cada gesto (o desfazer guarda o elemento anterior, ao vivo o `Write::Born`), as versões
guardam o documento comprimido (14,5 ms e 2,1 MB a 100 mil elementos, W8), e uma mensagem ao vivo tem o tecto
`MAX_FRAME` = 256 MiB. O Excalidraw (medido): o elemento leva `fileId`, `scale`, `crop` e `status`; os bytes vão
uma vez em `files`, num `dataURL`; sem o ficheiro, o SVG omite a imagem e o ecrã pinta um cinzento.

## Decisão

1. **O elemento leva a CHAVE, nunca os bytes:** `ElementKind::Image(Box<Image { file: FileId, style, crop, flip }>)`,
   `FileId` = sha256 dos bytes como chegaram. O registo ao vivo é `Field::Image` (ficheiro, recorte e espelho juntos).
2. **Os bytes vivem no QUADRO, uma vez por conteúdo** (`Board::files: BTreeMap<FileId, ImageFile { bytes, size }>`),
   imutáveis: juntar os de dois pares é a união; não há desfazer de um ficheiro (tirar a imagem é apagar o elemento).
3. **Gravar leva só o que está em uso** (`Board::files_in_use`: elementos não apagados ∪ `Version::files`); uma
   versão NÃO copia bytes — guarda a lista das chaves que mostra, e restaurá-la copia-os do quadro de origem.
4. **A porta de entrada é uma só** (`Ingest`, implementada em `ph2d-app-board::images::ingest` pela descodificação
   da CENA, `ph2d_asset::decode_image`): o que a cena lê, o quadro lê; o que um navegador não mostra (TIFF, PSD,
   ORA, QOI…) entra já em PNG — o SVG, o `.excalidraw` e o par ao vivo mostram sempre o que recebem.
5. **Ao vivo, cada ficheiro viaja SOZINHO:** `Write::File` (uma vez por par), numa mensagem só dele (`by_file`); o
   retrato de quem entra é MAGRO (`BoardSet::board_bytes_lean`) e o anfitrião manda a seguir um ficheiro por
   mensagem. Uma imagem cujo ficheiro ainda não chegou desenha o cinzento. `PROTOCOL` 4 → 5.
6. **O desenho não descodifica no fio dele:** `Images` (por `RenderCache`) descodifica em fios (metade da máquina),
   guarda uma pirâmide de metades (o 1.º nível cabe no atlas do Vello, `MAX_ATLAS_SIZE` 8192) e desenha o nível com
   a resolução do ECRÃ; o export descodifica já. O descodificador é do app (`install_decoder`, ao arrancar).

## Alternativas recusadas

| alternativa | porquê não |
|---|---|
| os bytes no elemento | cada gesto reescrevia megabytes no desfazer e ao vivo (o `Born`/`Put` leva o elemento inteiro) |
| os bytes no `BoardDoc` | a versão serializa o documento: cada versão copiava todas as fotografias (comprimir PNG/JPEG não ganha nada) |
| os bytes no `BoardSet` (partilhados entre quadros) | o quadro deixava de ser auto-suficiente: o retrato ao vivo, a cópia, a exportação e a versão precisavam do conjunto inteiro; o ganho é só quando se duplica um quadro (em memória o `Arc` já partilha) |
| um `BoardOp::File` | o `BoardOp` é a porta do desfazer e da fusão POR ELEMENTO (`op.id()`); um ficheiro não tem id de elemento nem inverso |
| o retrato com os bytes | 30 fotografias de 8 MB passavam o `MAX_FRAME` e quem entrava era recusado |
| guardar os bytes num formato próprio (re-codificar tudo em PNG) | um JPEG de 3 MB virava um PNG de 30 MB; só se converte o que o navegador não mostra |
| SVG como imagem | precisa de rasterizar por nível (vector); fica aberto — hoje um `.svg` largado num quadro conta como «não é imagem» |

## Consequências

- `FORMAT_VERSION` 11 → 12 (`legacy_v11.rs`, fixture `format_v11.bin` gravada pelo build `72b26c071`).
- Um `.excalidraw` com imagens abre com elas (`read_with`); o nosso sai com `files` e volta igual pelo `restore` dele.
- Abertos: editar o recorte (o modelo e o desenho já o têm), espelhar pela barra, imagens SVG, PDF com JPEG por
  passagem directa (`DCTDecode`), GIF animado (mostra o 1.º quadro).
