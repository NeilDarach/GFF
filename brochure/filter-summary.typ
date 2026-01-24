#import "summary.typ": generate_summary
#let current_filter=state("current_filter","")
#set page(
  flipped: true,
  paper: "a4",
  margin: (inside: 2.5cm, outside: 2cm, y: 1.75cm),
  footer: context [ GFF 2026 #current_filter.get() #h(1fr) #counter(page).display("1") ],
  number-align: center,
  )
#set par(justify: false)
#set text(size: 0.8em)
#set par(leading: 0.55em)
#let summaries=("Marion": ("M"),
"Marion and Neil": ("M","N"),
"Neil": ("N"),
"Patrick": ("Pt"),
"Vanessa": ("V"),
"Everyone": ("M", "N", "Pt", "V"))
#for(title,filter) in summaries.pairs() {
current_filter.update("for "+ title)
counter(page).update(1)
generate_summary("filter-summary.json",suffix: ("for "+ title),filter: filter)
pagebreak()
}
