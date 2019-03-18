const { Compile, Selector, selectStr } = require('../native');

function compile(path) {
    let compile = new Compile(path);
    return (json) => {
        if(typeof json != 'string') {
            json = JSON.stringify(json)
        }
        return JSON.parse(compile.template(json));
    };
}

function selector(json) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    let selector = new Selector(json);
    return (path) => {
        return JSON.parse(selector.selector(path));
    }
}

function select(json, path) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    return JSON.parse(selectStr(json, path));
}

module.exports = {
    compile,
    selector,
    select
};