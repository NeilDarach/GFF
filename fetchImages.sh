for id in $(jq -r '(.. | .posterImage?)//""' showings.json | sort -u) ; do 
  curl https://indy-systems.imgix.net/$id > posters/$id.jpg
done
