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

function readPathParam() {
    let params = location.search.substr(1)
        .split('&')
        .map((item) => item.split('='))
        .reduce((acc, param) => {
            acc[param[0]] = decodeURIComponent(param[1]);
            return acc;
        }, {});

    if(params.path) {
        getJsonpathInput().value = params.path;
        let doc = getReadBtn().ownerDocument;
        let event = doc.createEvent('MouseEvents');
        event.initEvent('click', true, true);
        event.synthetic = true;
        getReadBtn().dispatchEvent(event, true);
    }
}

initData('data/example.json').then(initEvent).then(readPathParam);