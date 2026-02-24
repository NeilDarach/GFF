#import "summary.typ": person_summary
#let version = read("summary-version.txt").trim()
#let current_filter=state("current_filter","")
#set page(
  flipped: true,
  paper: "a4",
  margin: (inside: 2.5cm, outside: 2cm, y: 1.75cm),
  footer: context [ GFF 2026 v#version#current_filter.get() #h(1fr) #counter(page).display("1") ],
  number-align: center,
  )
#set par(justify: false)
#set text(size: 0.8em)
#set par(leading: 0.55em)
#let names=(M: "Marion",
N:"Neil",
Pt:"Patrick",
Pm:"Pam",
V:"Vanessa")
#counter(page).update(1)
#person_summary("filter-summary.json",names: names)
}
