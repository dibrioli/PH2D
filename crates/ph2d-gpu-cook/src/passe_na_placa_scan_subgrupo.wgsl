// O `scan` do caminho de UM grupo por SUBGRUPO (doc 131 C.7): so' entra onde o dispositivo tem
// `Features::SUBGROUP` (`passe_na_placa.rs`). A soma e' de INTEIROS: o MESMO `starts`, bit a bit,
// com UMA barreira em vez das 17 da memoria de grupo.
//
// O troco de cada fio e' posto pela pista (`subgroup_id * subgroup_size + pista`, a regra do
// `contorno_subgrupo.wgsl`): o prefixo dentro do subgrupo e' o `subgroupExclusiveAdd`, e o dos
// subgrupos anteriores soma-se dos totais na memoria de grupo (no maximo 64: subgrupo >= 4).

struct Ent {
    @builtin(local_invocation_index) lid: u32,
    @builtin(subgroup_id) sid: u32,
    @builtin(subgroup_size) tam: u32,
    @builtin(subgroup_invocation_id) pista: u32,
}

var<workgroup> wg_total_do_subgrupo: array<u32, 64>;

// O `scan` exclusivo de `starts[0..m)`.
fn pnp_scan_grupo(e: Ent, m: u32) {
    let li = e.sid * e.tam + e.pista;
    let por = (m + 255u) / 256u;
    let ini = min(li * por, m);
    let fim = min(ini + por, m);
    var soma = 0u;
    for (var k = ini; k < fim; k = k + 1u) { soma = soma + atomicLoad(&starts[k]); }
    var acc = subgroupExclusiveAdd(soma);
    let total = subgroupAdd(soma);
    if (e.pista == 0u) { wg_total_do_subgrupo[e.sid] = total; }
    workgroupBarrier();
    for (var k = 0u; k < e.sid; k = k + 1u) { acc = acc + wg_total_do_subgrupo[k]; }
    for (var k = ini; k < fim; k = k + 1u) {
        let c = atomicLoad(&starts[k]);
        atomicStore(&starts[k], acc);
        acc = acc + c;
    }
}
