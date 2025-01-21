#import "in-dexter.typ": *
#set page(
  margin: (inside: 2.0cm, outside: 1.5cm, y: 1.75cm),
  footer: locate(loc => { 
    let headings = query(selector(heading).before(loc),loc)
    if headings != () { 

      if (calc.rem(loc.page(), 2)) != 0 [ #text(size: 0.8em)[GFF 2025 #h(1fr) #counter(page).display("1") ] ] else [ #text(size: 0.8em)[#counter(page).display("1") #h(1fr) GFF 2025 ] ] } else { }}),
  number-align: center,
  )

#v(1fr)
#align(center)[#image("banner.jpg", width: 100%)]
#v(1fr)
#pagebreak()
#pagebreak()
#counter(page).update(1) 


//#set page(
 // flipped: true,
  //)
#set par(justify: false)
#set par(leading: 0.55em)

#let rowOffset=24pt
#let screenCol=10%
#let pct(mins) = { (((100%-screenCol)/14)*((mins /60))) }
#let filmBox(body,start:"10:00",duration:30,color:blue,row:0,id:"")= {
    let (h,m) = start.split(":")
    let st = (int(h)*60)
    place(dx:pct((int(h)*60)+int(m)-600)+screenCol,dy:4pt+(row*rowOffset))[#box(height: 20pt, width: pct(duration),fill: color,stroke: 1pt+black,clip:true,inset:2pt,radius:3pt,outset:(x:0pt,y:1pt))[#link(label(id))[#text(size:0.75em)[#body]]]]
}
#let screen(name:"",row:0) = {
      place(dx:0%,dy:10pt+(rowOffset*row))[#box(width: screenCol,height: 10pt,clip:true,outset:(x:-1pt))[#name]]
    }
#let mygrid(lines) ={
    let i = 0
    while i < 14 {
      place(dx:pct(i*60)+screenCol,dy:-4pt)[#(10+i)]
      place(dx:pct(i*60)+screenCol,dy:-4pt)[#line(start:(0pt,-1pt),length: (rowOffset*lines)+10pt, angle: 90deg)]
      i = i+1
      }
}

= Summary <summary>
#for (day,showings) in json("summary.json") {
block(breakable: false)[
    == #day - #showings.at("GFT 1").at(0).day
    #rect(width: 100%, height: (rowOffset*showings.len()) + 10pt)[
    #mygrid(showings.len())
    #let row=0
    #for (screen_entry,films) in showings {
       screen(name:screen_entry,row:row)
       for film in films {
       filmBox(start: film.at("start") ,duration: film.at("duration"),color:color.rgb(film.at("color")), row: row,id: film.at("id"))[#film.at("title")]
       }
    row = row +1

}
]]
}


#pagebreak()


#set par(justify: true)
#set par(leading: 0.35em)
#columns(2)[
#let showFilm(film) = block(breakable: false)[
#index[#film.name]
#index[#film.sortname]
= #film.name #label(film.id) (#film.rating) \
#if film.strand != "" [ #film.strand\ ]
#text(size: 0.8em)[
#for s in film.showings [
#s.date, #s.time - #s.screen\
]]

        #let sidebar = ()
#if film.starring != "" { 
sidebar.push("Starring:") 
sidebar.push(film.starring) }

#if film.directedBy != "" { 
sidebar.push("Directed By:")
sidebar.push(film.directedBy) }

#if film.genres != "" {
        sidebar.push("Genres:")
        sidebar.push(film.genres) }
#if film.ratingReason != ""  {
        sidebar.push("Rating notes:")
        sidebar.push(film.ratingReason)} 

        #grid(columns:(2fr,3fr), rows: auto, gutter: 3pt, [#align(left)[#image(film.poster,width:100%)]], [
            #text(size: 0.6em)[ #grid(columns:(6em,auto),gutter: 3pt, ..sidebar)]]) 
#let syntext = film.synopsis
#let synmark = eval(syntext,mode:"markup")
#text(size: 0.7em)[ #synmark ]

#v(1em)
]

#for (i,f) in json("brochure.json").enumerate() {
   showFilm(f) }
]

#pagebreak()
#set page(columns:1)
= Index
#columns(3)[
  #make-index(title: none, outlined: true, use-page-counter:true)
]
