import * as jsonpath from "jsonpath-wasm";

function getTextarea() {
    return document.querySelector('#json-example');
}

function getJsonpathInput() {
    return document.querySelector('#jsonpath-input');
}

function getReadBtn() {
    return document.querySelector('#read-json');
}

function getReadResult() {
    return document.querySelector('#read-result');
}

function initData(url) {
    return fetch(url)
        .then((res) => res.text())
        .then((jsonStr) => getTextarea().value = jsonStr)
        .catch(console.error);
}

function initEvent() {
    getJsonpathInput().onkeyup = function(e) {
        var charCode = (typeof e.which === "number") ? e.which : e.keyCode;
        if(charCode == 13) {
            read();
        }
    }

    getReadBtn().onclick = function() {
        read();
    }

    function read() {
        let ret = jsonpath.read(getTextarea().value, getJsonpathInput().value);
        if(typeof ret === 'string') {
            getReadResult().innerText = ret;
        } else {
            getReadResult().innerText = JSON.stringify(ret, null, 2);
        }
    }
}

initData('data/example.json').then(initEvent)