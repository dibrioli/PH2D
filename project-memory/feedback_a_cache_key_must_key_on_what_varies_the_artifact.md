---
name: feedback_a_cache_key_must_key_on_what_varies_the_artifact
description: Chave de cache derivada do RESULTADO colide; derive da ENTRADA que varia o artefato
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 62ac077f-09f4-41be-9a44-14a0a85668a9
---

Uma chave de cache tem de sair da **entrada que determina o artefato**, não de uma
propriedade *derivada* dele. Se você resume o resultado, dois resultados diferentes
podem resumir igual — e aí o cache entrega o objeto errado.

**Why:** o `presence_signature` do `ph2d-gpu-cook` usava 1 bit por binding = "liga
buffer de leitura OU de escrita". Num `ColumnAccess::ReadWrite` esse bit é 1 **nos
dois casos** (coluna ausente ainda escreve), então *presente* e *ausente*
compartilhavam a chave — e os módulos WGSL diferem por um binding de leitura
inteiro. wgpu rejeita o bind group contra o layout que compilou pro outro:
**não é número errado, é crash**. A entrada real era um `bool` só (`here` = "achei
minha coluna?"), que determina o texto do módulo exatamente.

Ficou latente **duas fases inteiras** porque nada mudava a presença de uma coluna:
só aparece quando o MESMO tipo de nó ocorre duas vezes com presenças diferentes
(`grid → scale → scale`) ou quando o estado de uma simulação nasce vazio no tick 0
e cheio no tick 1. Um cache que nunca vê a chave variar não prova que a chave
discrimina.

**How to apply:** ao escrever a chave, pergunte *"que entrada faz o artefato mudar
de forma?"* e derive dela — nunca de uma característica do artefato. E gateie com
um fixture onde a chave **de fato varia** (o mesmo tipo, duas presenças, numa
cadeia só): sem isso o gate mede um cache com uma entrada só.
Ver [[feedback_a_gate_only_proves_what_its_fixture_contains]] e
[[reference_topic_mutation_proofs]].
