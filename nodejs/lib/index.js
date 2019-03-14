const { Compile, Selector, selectStr } = require('../native');

function compile(path) {
    let compile = new Compile(path);
    return (json) => {
        if(typeof json != 'string') {
            json = JSON.stringify(json)
        }
        return compile.template(json);
    };
}

function selector(json) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    let selector = new Selector(json);
    return (path) => {
        return selector.selector(path);
    }
}

function select(json, path) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    return selectStr(json, path);
}

module.exports = {
    compile,
    selector,
    select
};