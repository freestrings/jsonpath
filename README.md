# Jsonpath

[JsonPath](https://goessner.net/articles/JsonPath/) Rust 구현

## 왜?
To enjoy Rust

## 사용법

## With Javascript (WebAssembly)

#### `jsonpath-wasm` 라이브리러

*(not yet published `jsonpath-wasm`)*
```javascript
import * as jsonpath from "jsonpath-wasm";
```

#### `read` 함수

```
jsonpath.read(JSON.parse("{\"a\" : 1}"), "$.a");
jsonpath.read("{\"a\" : 1}", "$.a");
```


#### JsonPath 재사용

```
let template = jsonpath.compile("$.a");

//
// 1. read json string
//
template("{\"a\" : 1}")

//
// 2. read as json object
//
template(JSON.parse("{\"a\" : 1}"));
```

#### Json 재사용

```
//
// 1. read json string
//
let reader1 = jsonpath.reader("{\"a\" : 1}");
reader1("$.a");
reader1("$.b");

//
// 2. read as json object
//
let reader2 = jsonpath.reader(JSON.parse("{\"a\" : 1}"));
reader2("$.a");
reader2("$.b");
```

### 데모

**Demo**: https://freestrings.github.io/jsonpath/

json 데이터 *(참고 사이트: https://github.com/json-path/JsonPath)*

```javascript
{
    "store": {
        "book": [
            {
                "category": "reference",
                "author": "Nigel Rees",
                "title": "Sayings of the Century",
                "price": 8.95
            },
            {
                "category": "fiction",
                "author": "Evelyn Waugh",
                "title": "Sword of Honour",
                "price": 12.99
            },
            {
                "category": "fiction",
                "author": "Herman Melville",
                "title": "Moby Dick",
                "isbn": "0-553-21311-3",
                "price": 8.99
            },
            {
                "category": "fiction",
                "author": "J. R. R. Tolkien",
                "title": "The Lord of the Rings",
                "isbn": "0-395-19395-8",
                "price": 22.99
            }
        ],
        "bicycle": {
            "color": "red",
            "price": 19.95
        }
    },
    "expensive": 10
}
```


| JsonPath (click link to try)| Result |
| :------- | :----- |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.book[*].author" target="_blank">$.store.book[*].author</a>| The authors of all books     |
| <a href="https://freestrings.github.io/jsonpath/?path=$..author" target="_blank">$..author</a>                   | All authors                         |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.*" target="_blank">$.store.*</a>                  | All things, both books and bicycles  |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store..price" target="_blank">$.store..price</a>             | The price of everything         |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2]" target="_blank">$..book[2]</a>                 | The third book                      |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2]" target="_blank">$..book[-2]</a>                 | The second to last book            |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[0,1]" target="_blank">$..book[0,1]</a>               | The first two books               |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[:2]" target="_blank">$..book[:2]</a>                | All books from index 0 (inclusive) until index 2 (exclusive) |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[1:2]" target="_blank">$..book[1:2]</a>                | All books from index 1 (inclusive) until index 2 (exclusive) |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[-2:]" target="_blank">$..book[-2:]</a>                | Last two books                   |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[2:]" target="_blank">$..book[2:]</a>                | Book number two from tail          |
| <a href="https://freestrings.github.io/jsonpath/?path=$..book[?(@.isbn)]" target="_blank">$..book[?(@.isbn)]</a>          | All books with an ISBN number         |
| <a href="https://freestrings.github.io/jsonpath/?path=$.store.book[?(@.price < 10)]" target="_blank">$.store.book[?(@.price < 10)]</a> | All books in store cheaper than 10  |
| $..book[?(@.price <= $['expensive'])] *(not yet supported)* | ~~All books in store that are not "expensive"~~  |
| $..book[?(@.author =~ /.*REES/i)] *(not yet supported)* | ~~All books matching regex (ignore case)~~  |
| <a href="https://freestrings.github.io/jsonpath/?path=$..*" target="_blank">$..*</a>                        | Give me every thing   
| $..book.length() *(not yet supported)* | ~~The number of books~~                      |


## With Rust (as library)

- 

## With AWS API Gateway

-

# 성능테스트

