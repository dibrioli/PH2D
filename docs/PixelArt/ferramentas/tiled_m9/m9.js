var out=[];
function run(){
 function dump(tag,m){ var l=m.layerAt(0); var c=[]; for(var y=-2;y<8;y++)for(var x=-2;x<8;x++){var k=l.cellAt(x,y); if(k&&k.tileId>=0)c.push(x+','+y+':'+k.tileId);} out.push(tag+' map='+m.width+'x'+m.height+' layer='+l.width+'x'+l.height+' offset='+l.offset.x+','+l.offset.y+' cells='+c.join(' ')+' tilesets='+m.tilesets.length);}
var p=tiled.open('M9DIR/m.tmx');
dump('antes',p);
p.resize(Qt.size(2,2)); dump('resize2x2_so_tamanho',p);
p.resize(Qt.size(4,4)); dump('volta4x4',p);
p.undo(); p.undo(); dump('undo2',p);
p.resize(Qt.size(6,6),Qt.point(2,2)); dump('resize6x6_of2',p); p.undo(); dump('undo',p);
p.resize(Qt.size(2,2),Qt.point(-1,-1)); dump('resize2x2_of-1',p);
p.resize(Qt.size(4,4),Qt.point(1,1)); dump('volta_of+1',p); p.undo();p.undo();
p.removeLayerAt(0); out.push('layer_removida tilesets='+p.tilesets.length+' n='+p.tilesets[0].tileCount);
var w=tiled.mapFormat('tmx').write(p,'M9DIR/semcamada.tmx'); out.push('grava='+w);

}
try{run()}catch(e){out.push('ERRO '+e+' '+e.stack)}
var f2=new TextFile('M9DIR/res.txt',TextFile.WriteOnly); f2.write(out.join('\n')); f2.commit();
