#set page(
  flipped: true,
  paper: "a4",
  margin: (inside: 2.5cm, outside: 2cm, y: 1.75cm),
  footer: locate(loc => {
      [ GFF 2025 #h(1fr) #counter(page).display("1") ]   }),
  number-align: center,
  )
#set par(justify: false)
#set text(size: 0.8em)
#set par(leading: 0.55em)

#let rowOffset=24pt
#let screenCol=10%
#let pct(mins) = { (((100%-screenCol)/14)*((mins /60))) }
#let filmBox(body,start:"10:00",duration:30,color:blue,row:0)= {
    let (h,m) = start.split(":")
    let st = (int(h)*60)
    place(dx:pct((int(h)*60)+int(m)-600)+screenCol,dy:4pt+(row*rowOffset))[#box(height: 20pt, width: pct(duration),fill: color,stroke: 1pt+black,clip:true,inset:2pt,radius:3pt,outset:(x:0pt))[#body]]
}
#let screen(name:"",row:0) = {
      place(dx:0%,dy:10pt+(rowOffset*row))[#box(width: screenCol,height: 10pt,clip:true,outset:(x:-1pt))[#name]]
    }
#let grid(lines) ={
    let i = 0
    while i < 14 {
      place(dx:pct(i*60)+screenCol,dy:-4pt)[#(10+i)]
      place(dx:pct(i*60)+screenCol,dy:-4pt)[#line(start:(0pt,-1pt),length: (rowOffset*lines)+10pt, angle: 90deg)]
      i = i+1
      }
}

#for (day,showings) in json("summary.json") {
block(breakable: false)[
    #day
    #rect(width: 100%, height: (rowOffset*showings.len()) + 10pt)[
    #grid(showings.len())
    #let row=0
    #for (screen_entry,films) in showings {
       screen(name:screen_entry,row:row)
       for film in films {
       filmBox(start: film.at("start") ,duration: film.at("duration"),color:color.rgb(film.at("color")), row: row)[#film.at("title")]
       }
    row = row +1

}
]]
}
