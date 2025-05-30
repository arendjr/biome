/* should not generate diagnostics */
throw new Error("error");
throw new TypeError("error");
throw new MyCustomError("error");
throw new MyCustomError();
throw generateError();
throw foo();
throw err;
throw 1;
const err = TypeError("error");
throw err;
// Should not check other argument
new Error("message", 0, 0);
// We don't know the value
new Error(foo);
new Error(...foo);

new AggregateError(errors, "message");
new NotAggregateError(errors);
new AggregateError(...foo);
new AggregateError(...foo, "");
new AggregateError(errors, ...foo);
new AggregateError(errors, message, "");
new AggregateError("", message, "");

// Invalid but we don't know the value
const errorMessage1 = Object.freeze({ errorMessage: 1 }).errorMessage;
throw new Error(errorMessage);
throw new Error((lineNumber = 2));
throw new Error({ foo: 0 }.foo);

{
    // Do not fail if Error is shadowed
    const Error = function () {};
    const err1 = new Error({
        name: "Unauthorized",
    });
}
