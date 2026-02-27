#import sys: inputs
#let make_dict(arr) = {
arr.fold((:),(d,assoc) => { d.insert(assoc.at("key"),assoc.at("value")); d})
}
#let person_summary(names: ()) = {
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

  let colours =inputs.colours

[= Summary <summary>]
for (date,showings) in inputs.summary.pairs().sorted(key: it => it.at(0)) {
  let by_person = names.pairs().fold((:),(dict,(init,name)) => {dict.insert(name,()); dict})
  let day = ""
  for (screen,films) in showings.pairs() {
    for film in films {
      day = film.day
  film.insert("screen", screen)
  film.insert("color", colours.at(screen,default:"ffffff"))
       for person in film.attendees {
          let name = names.at(person)
          if name in by_person { by_person.at(name).push(film) }
        }
    }
  }
block(breakable: false)[
    == #date - #day #label("s"+date) 
    #rect(width: 100%, height: (rowOffset*by_person.len()) + 12pt)[
      #mygrid(by_person.len())
      #let row=0
      #for (name,films) in by_person.pairs().sorted(key: it => it.at(0)) {
       screen(name:name,row:row)
       for film in films {
      let boxContent = text(size: 1.2em)[#film.at("title")\ #align(right+bottom)[#film.at("screen",default:"")]]
       filmBox(start: film.at("start") ,duration: film.at("duration"),color:color.rgb(film.at("color")+ "50"), row: row,id: film.at("id",default:""))[#boxContent]
       }
      row = row +1

}
]]}
}
#let version = inputs.version
#let current_filter=state("current_filter","")
#set page(
  flipped: true,
  paper: "a4",
  margin: (inside: 2.5cm, outside: 2cm, y: 1.75cm),
  footer: context [ GFF 2026 v#version#current_filter.get() #h(1fr) #counter(page).display("1") ],
  number-align: center,
  )
#set par(justify: false,leading: 0.55em)
#set text(font: "Source Serif 4",size: 0.8em)
#let names=inputs.names
#counter(page).update(1)
#person_summary(names: names)
