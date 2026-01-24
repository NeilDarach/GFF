#import "@preview/in-dexter:0.7.2": *
#import "@preview/wrap-it:0.1.1": wrap-content
#import "summary.typ": generate_summary
#set page(
  paper: "a4",
  margin: (inside: 2.0cm, outside: 1.5cm, y: 1.75cm),
  footer: context [
    #let headings = query(selector(heading).before(here()))
    #if headings.len() == 0  { return }
    #let p=counter(page).display("1")
    #let f = if calc.odd(here().page()) {
      "GFF 2026 - " + h(1fr) + p 
    } else {
      p + h(1fr) + " - GFF 2026"
    }
    #text(size: 0.8em)[#f]],
  number-align: center,
    flipped: false,
  )

#v(1fr)
#align(center)[#image("banner.jpg", width: 100%)]
#v(1fr)
#pagebreak()
#set page(flipped: true)
#pagebreak()
#counter(page).update(1) 
#generate_summary("summary.json")

#set page(flipped: true)
#set par(justify: false)
#set par(leading: 0.55em)

#set page(
  flipped: false)
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

    #sidebar.push("")
    #sidebar.push("")
        //#grid(columns:(2fr,3fr), rows: auto, gutter: 3pt, [#align(left)[#image(film.poster,width:100%)]], [
            //#text(size: 0.6em)[ #grid(columns:(6em,auto),gutter: 3pt, ..sidebar)]]) 
#let syntext = film.synopsis
    #let tags = text(size: 0.6em)[#grid(columns: (6em,auto),gutter: 3pt,..sidebar)]
#let synmark = eval(syntext,mode:"markup")
    //#sidebar.push(text(size: 0.7em)[ #synmark ])
    #wrap-content(image(film.poster,width: 75pt),text(size: 0.7em)[#tags #text(size: 1.1em)[#synmark]],column-gutter: 15pt)

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
