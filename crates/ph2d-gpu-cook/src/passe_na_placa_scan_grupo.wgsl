// O `scan` do caminho de UM grupo pela MEMORIA DE GRUPO (o de todo dispositivo): cada fio soma um
// troco contiguo, o grupo varre as 256 somas (Hillis-Steele, 17 barreiras) e cada fio reescreve
// o seu troco. A variante por SUBGRUPO e' `passe_na_placa_scan_subgrupo.wgsl` (doc 131 C.7).

struct Ent {
    @builtin(local_invocation_index) lid: u32,
}

var<workgroup> wg_parcial: array<u32, 256>;

// O `scan` exclusivo de `starts[0..m)`.
fn pnp_scan_grupo(e: Ent, m: u32) {
    let lid = e.lid;
    let por = (m + 255u) / 256u;
    let ini = min(lid * por, m);
    let fim = min(ini + por, m);
    var soma = 0u;
    for (var k = ini; k < fim; k = k + 1u) { soma = soma + atomicLoad(&starts[k]); }
    wg_parcial[lid] = soma;
    workgroupBarrier();
    for (var d = 1u; d < 256u; d = d * 2u) {
        var x = 0u;
        if (lid >= d) { x = wg_parcial[lid - d]; }
        workgroupBarrier();
        wg_parcial[lid] = wg_parcial[lid] + x;
        workgroupBarrier();
    }
    var acc = wg_parcial[lid] - soma;
    for (var k = ini; k < fim; k = k + 1u) {
        let c = atomicLoad(&starts[k]);
        atomicStore(&starts[k], acc);
        acc = acc + c;
    }
}
