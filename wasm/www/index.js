import * as jsonpath from "rs-jsonpath";

let jsonString = "{\"a\" : 1}";

let template = jsonpath.compile("$.a");
console.log(template(jsonString));
console.log(template(JSON.parse(jsonString)));

let reader1 = jsonpath.reader(jsonString);
console.log(reader1("$.a"));

let reader2 = jsonpath.reader(JSON.parse(jsonString));
console.log(reader2("$.a"));

console.log(jsonpath.read(JSON.parse(jsonString), "$.a"));
console.log(jsonpath.read(jsonString, "$.a"));