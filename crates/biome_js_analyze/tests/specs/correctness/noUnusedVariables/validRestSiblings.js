/* should not generate diagnostics */
// With ignoreRestSiblings: true, unused variables should be ignored
const car = { brand: "Tesla", year: 2019, countryCode: "US" };
const { brand, year, ...other } = car;
console.log(other);

// Multiple rest siblings
const person = { name: "John", age: 30, city: "New York", country: "US" };
const { name, age, city, ...rest } = person;
console.log(rest);

// Renamed properties
const data = { foo: 1, bar: 2, baz: 3 };
const { foo: renamedFoo, bar: renamedBar, ...remaining } = data;
console.log(remaining);
