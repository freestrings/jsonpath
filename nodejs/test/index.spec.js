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

describe('filter test', () => {
    it('complex filter1', (done) => {
        let json = {
            'store': {
                'book': [
                    {
                        'category': 'reference',
                        'author': 'Nigel Rees',
                        'title': 'Sayings of the Century',
                        'price': 8.95,
                    },
                    {
                        'category': 'fiction',
                        'author': 'Evelyn Waugh',
                        'title': 'Sword of Honour',
                        'price': 12.99,
                    },
                    {
                        'category': 'fiction',
                        'author': 'Herman Melville',
                        'title': 'Moby Dick',
                        'isbn': '0-553-21311-3',
                        'price': 8.99,
                    },
                    {
                        'category': 'fiction',
                        'author': 'J. R. R. Tolkien',
                        'title': 'The Lord of the Rings',
                        'isbn': '0-395-19395-8',
                        'price': 22.99,
                    },
                ],
                'bicycle': {
                    'color': 'red',
                    'price': 19.95,
                },
            },
            'expensive': 10,
        };

        let target = [
            {
                category: 'fiction',
                author: 'Evelyn Waugh',
                title: 'Sword of Honour',
                price: 12.99,
            },
            {
                category: 'fiction',
                author: 'J. R. R. Tolkien',
                title: 'The Lord of the Rings',
                isbn: '0-395-19395-8',
                price: 22.99,
            },
            {
                category: 'reference',
                author: 'Nigel Rees',
                title: 'Sayings of the Century',
                price: 8.95,
            }]
        ;

        let result = jsonpath.select(json, '$..book[?((@.price == 12.99 || $.store.bicycle.price < @.price) || @.category == "reference")]');
        if (JSON.stringify(result) === JSON.stringify(target)) {
            done();
        }
    });
});