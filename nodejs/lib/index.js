const {
    CompileFn,
    SelectorFn,
    selectStr,
    deleteValue: _deleteValue,
    replaceWith: _replaceWith,
    Selector: _Selector,
    SelectorMut: _SelectorMut
} = require('../native');

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

function deleteValue(json, path) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    return JSON.parse(_deleteValue(json, path));
}

function replaceWith(json, path, fun) {
    if(typeof json != 'string') {
        json = JSON.stringify(json)
    }
    let result = _replaceWith(json, path, (v) => {
        let result = fun(JSON.parse(v));
        if(typeof result != 'string') {
            result = JSON.stringify(result)
        }
        return result;
    });
    if(typeof result == 'string') {
        result = JSON.parse(result);
    }
    return result;
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
        this._selector.value(json);
        return this;
    }

    select() {
        return JSON.parse(this._selector.select());
    }
}

class SelectorMut {
    constructor() {
        return this;
    }

    path(path) {
        this.path = path;
        return this;
    }

    value(json) {
        if(typeof json != 'string') {
            json = JSON.stringify(json)
        }
        this.json = json;
        return this;
    }

    deleteValue() {
        let selector = new _SelectorMut();
        if(!this.path) {
            selector.emptyPathError();
            return;
        }

        if(!this.json) {
            selector.emptyValueError();
            return;
        }

        this.json = deleteValue(this.json, this.path);
        return this;
    }

    replaceWith(fun) {
        let selector = new _SelectorMut();
        if(!this.path) {
            selector.emptyPathError();
            return;
        }
        if(!this.json) {
            selector.emptyValueError();
            return;
        }
        this.json = replaceWith(this.json, this.path, fun);
        return this;
    }

    take() {
        let json = this.json;
        delete this.json;
        return json;
    }
}

module.exports = {
    compile,
    selector,
    select,
    deleteValue,
    replaceWith,
    Selector,
    SelectorMut
};