# example-actix_web-redis

actix_web-redis example with deadpool

## MGET

```
http://localhost:8080/set/aaaa/1111
http://localhost:8080/set/bbbb/2222
http://localhost:8080/set/cccc/3333
```

send request
```
POST http://localhost:8080/mget HTTP/1.1
content-type: application/json

["aaaa","bbbb", "cccc"]
```

returns
```
[
  "1111",
  "2222",
  "3333"
]
```
