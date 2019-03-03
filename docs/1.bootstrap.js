(window["webpackJsonp"] = window["webpackJsonp"] || []).push([[1],{

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! rs-jsonpath */ \"./node_modules/rs-jsonpath/wasm.js\");\n\n\nlet jsonString = \"{\\\"a\\\" : 1}\";\n\nlet template = rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__[\"compile\"](\"$.a\");\nconsole.log(template(jsonString));\nconsole.log(template(JSON.parse(jsonString)));\n\nlet reader1 = rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__[\"reader\"](jsonString);\nconsole.log(reader1(\"$.a\"));\n\nlet reader2 = rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__[\"reader\"](JSON.parse(jsonString));\nconsole.log(reader2(\"$.a\"));\n\nconsole.log(rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__[\"read\"](JSON.parse(jsonString), \"$.a\"));\nconsole.log(rs_jsonpath__WEBPACK_IMPORTED_MODULE_0__[\"read\"](jsonString, \"$.a\"));\n\n//# sourceURL=webpack:///./index.js?");

/***/ })

}]);