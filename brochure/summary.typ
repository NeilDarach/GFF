#let generate_summary(inputFile,suffix: "",filter: ()) = {
let rowOffset=41pt
let screenCol=10%
let pct(mins) = { (((100%-screenCol)/14)*((mins /60))) }
let filmBox(body,start:"10:00",duration:30,color:blue,row:0,id:"")= {
    let (h,m) = start.split(":")
    let st = (int(h)*60)
    let title = context { if (id == "" or id == none or query(label(id)).len() == 0) { 
    text(size:0.75em)[#body] } else {
    link(label(id))[#text(size:0.75em)[#body]]
    }}
    let mybox(body,fill: white) = box(height: 35pt, width: pct(duration),fill: fill,stroke: 1pt+black,clip:true,inset:2pt,radius:3pt,outset:(x:0pt,y:1pt))[#body]
    place(dx:pct((int(h)*60)+int(m)-600)+screenCol+2pt,dy:8pt+(row*rowOffset))[#mybox(fill: white)[]]
    place(dx:pct((int(h)*60)+int(m)-600)+screenCol+2pt,dy:8pt+(row*rowOffset))[#mybox(fill: color)[#title]]
}
let screen(name:"",row:0) = {
      place(dx:0%,dy:10pt+(rowOffset*row))[#box(width: screenCol,height: 10pt,clip:true,outset:(x:1pt))[ #name]]
    }
let mygrid(lines) ={
    let i = 0
    while i < 14 {
      place(dx:pct(i*60)+screenCol+2pt,dy:-4pt)[#(10+i)]
      place(dx:pct(i*60)+screenCol,dy:-4pt)[#line(start:(0pt,-1pt),length: (rowOffset*lines)+10pt, angle: 90deg)]
      i = i+1
      }
}

[= Summary #suffix <summary>]
for (day,showings) in json(inputFile).pairs().sorted(key: it => it.at(0)) {
  let pairs = showings.pairs()
  let filtered = (:)
  for (screen,films) in pairs {
    let to_include = films.filter(it => filter.len() == 0  or it.at("attendees",default:()).any(ea => filter.contains(ea)) )
    if to_include.len() > 0 { filtered.insert(screen,to_include) }
}
  if filtered.len() > 0 {
block(breakable: false)[
    == #day - #filtered.pairs().at(0).at(1).at(0).day #label("s"+day+suffix) 
    #rect(width: 100%, height: (rowOffset*filtered.len()) + 12pt)[
      #mygrid(filtered.len())
      #let row=0
      #for (screen_entry,films) in filtered.pairs().sorted(key: it => it.at(0)) {
       screen(name:screen_entry,row:row)
       for film in films {
      let boxContent = text(size: 1.2em)[#film.at("title")\ #align(right+bottom)[#film.at("attendees",default:()).join(",")]]
       filmBox(start: film.at("start") ,duration: film.at("duration"),color:color.rgb(film.at("color")+ "50"), row: row,id: film.at("id",default:""))[#boxContent]
       }
      row = row +1

}
]]}
}
}
