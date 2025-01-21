mkdir -p brochure/posters
for id in $(jq -r '(.. | .posterImage?)//""' showings.json | sort -u) ; do 
  if [[ ! -f "brochure/posters/${id}.jpg" ]] ; then 
    curl -s "https://indy-systems.imgix.net/${id}" > "brochure/posters/${id}"
    convert -resize 300x300 "brochure/posters/${id}" "brochure/posters/${id}.jpg"
    rm "brochure/posters/${id}"
  fi
done
