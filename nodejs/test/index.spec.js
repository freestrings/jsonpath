const jsonpath = require('../lib/index.js');

describe('compile test', () => {
    it('basic', (done) => {
        let template = jsonpath.compile('$.a');
        let result = template({'a': 1});
        if (result === 1) {
            done();
        }
    });
});

describe('selector test', () => {
    it('basic', (done) => {
        let selector = jsonpath.selector({'a': 1});
        let result = selector('$.a');
        if (result === 1) {
            done();
        }
    });
});

describe('select test', () => {
    it('basic', (done) => {
        let result = jsonpath.select({'a': 1}, '$.a');
        if (result === 1) {
            done();
        }
    });
});