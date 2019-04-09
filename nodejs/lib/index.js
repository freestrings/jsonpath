const { CompileFn, SelectorFn, selectStr, Selector: _Selector } = require('../native');

function compile(path) {
    let compile = new CompileFn(path);
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
    let selector = new SelectorFn(json);
    return (path) => {
        return JSON.parse(selector.select(path));
    }
}

function select(json, path) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    return JSON.parse(selectStr(json, path));
}

class Selector {
    constructor() {
        this._selector = new _Selector();
        return this;
    }

    path(path) {
        this._selector.path(path);
        return this;
    }

    value(json) {
        if(typeof json != 'string') {
            json = JSON.stringify(json)
        }
        this._selector.value_from_str(json);
        return this;
    }

    selectToStr() {
        return this._selector.select_to_str();
    }

    selectTo() {
        return JSON.parse(this.selectToStr());
    }

}

module.exports = {
    compile,
    selector,
    select,
    Selector
};