#import "in-dexter.typ": *
#set page(
  margin: (inside: 2.0cm, outside: 1.5cm, y: 1.75cm),
  footer: locate(loc => { 
    let headings = query(selector(heading).before(loc),loc)
    if headings != () { 
      if (calc.rem(loc.page(), 2)) != 0 [ GFF 2025 #h(1fr) #counter(page).display("1") ] else [ #counter(page).display("1") #h(1fr) GFF 2025 ] } else { }}),
  number-align: center,
  )
#set par(justify: true)
#set text(size: 0.8em)
#set par(leading: 0.55em)

#v(1fr)
#align(center)[#image("banner.jpg", width: 100%)]
#v(1fr)
#pagebreak()
#pagebreak()
#counter(page).update(1) 

#columns(2)[
#let showFilm(film) = block(breakable: false)[
#index[#film.name]
#index[#film.sortname]
= #film.name (#film.rating) \
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
            #text(size: 0.8em)[ #grid(columns:(6em,auto),gutter: 3pt, ..sidebar)]]) 
#let syntext = film.synopsis
#let synmark = eval(syntext,mode:"markup")
#text(size: 0.8em)[ #synmark ]
#label("synopsis")

#text(size: 0.8em)[
]
#v(1em)
]

#label("synopsis")
#for (i,f) in json("brochure.json").enumerate() {
    if i < 300 {
   showFilm(f) } }
]

#pagebreak()
#set page(columns:1)
= Index
#columns(3)[
  #make-index(title: none, outlined: true, use-page-counter:true)
]
