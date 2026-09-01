describe('math', () => {
  test('adds numbers', () => {
    expect(2 + 3).toBe(5);
  });

  test('multiplies numbers', () => {
    expect(4 * 5).toBe(20);
  });

  test('maps arrays', () => {
    const doubled = [1, 2, 3, 4, 5].map((n) => n * 2);
    expect(doubled).toHaveLength(5);
    expect(doubled[0]).toBe(2);
  });
});
