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
        this._selector.valueFromStr(json);
        return this;
    }

    selectToStr() {
        return this.selectAsStr();
    }

    selectTo() {
        return this.selectAs();
    }

    selectAsStr() {
        return this._selector.selectAsStr();
    }

    selectAs() {
        return JSON.parse(this.selectAsStr());
    }

    map(func) {
        this._selector.map((json) => {
            var result = func.call(null, JSON.parse(json));
            if(typeof result !== 'string') {
                result = JSON.stringify(result);
            }
            return result;
        });
        return this;
    }

    get() {
        return JSON.parse(this._selector.get());
    }
}

module.exports = {
    compile,
    selector,
    select,
    Selector
};