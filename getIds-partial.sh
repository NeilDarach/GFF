curl 'https://www.glasgowfilm.org/graphql' --compressed -X POST -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:122.0) Gecko/20100101 Firefox/122.0' -H 'Accept: */*' -H 'Accept-Language: en-GB,en;q=0.5' -H 'Accept-Encoding: gzip, deflate' -H 'Referer: https://www.glasgowfilm.org/glasgow-film-festival/' -H 'content-type: application/json'  -H 'client-type: consumer' -H 'Cookie:    site_id=eyJfcmFpbHMiOnsibWVzc2FnZSI6IkJBZ2lDREV3TXc9PSIsImV4cCI6bnVsbCwicHVyIjoiY29va2llLnNpdGVfaWQifX0%3D--3ddffe33619f88c10c860650564ce545dd2481e3;' -H 'Sec-Fetch-Dest: empty' --data-raw '{"operationName":null,"variables":{"type":"all-published","orderBy":"magic","titleClassIds":[196,211],"limit":130,"resultVersion":null},"query":"query ($limit: Int, $orderBy: String, $titleClassId: ID, $type: String) {
 movies(
 limit: $limit
 orderBy: $orderBy
 titleClassId: $titleClassId
 type: $type
 ) {
 data {
 id
 name
 }
 count
 resultVersion
 __typename
}
}
"}'
