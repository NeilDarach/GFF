#import sys: inputs
#let make_dict(arr) = {
  arr.fold((:),(d,assoc) => { d.insert(assoc.at("key"),assoc.at("value")); d})
}
#set page(
  flipped: true,
  paper: "a4",
  margin: (inside: 2.5cm, outside: 2cm, y: 1.75cm),
  )
#set text(font: "Source Serif 4",size: 0.8em)
#json.encode(inputs)
