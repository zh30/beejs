// Stage 56.4 - Basic Test Suite
// Testing the new Beejs test runner

describe('Basic Math Tests', () => {
  test('1 + 1 should equal 2', () => {
    expect(1 + 1).toBe(2);
  });

  test('2 * 3 should equal 6', () => {
    expect(2 * 3).toBe(6);
  });

  test('10 / 2 should equal 5', () => {
    expect(10 / 2).toBe(5);
  });
});

describe('String Tests', () => {
  test('hello should contain ell', () => {
    expect('hello').toContain('ell');
  });

  test('world should match /wor/', () => {
    expect('world').toMatch('wor');
  });
});

describe('Truthy/Falsy Tests', () => {
  test('true should be truthy', () => {
    expect(true).toBeTruthy();
  });

  test('false should be falsy', () => {
    expect(false).toBeFalsy();
  });

  test('0 should be falsy', () => {
    expect(0).toBeFalsy();
  });
});
