/* should not generate diagnostics */

// Custom implementations (should NOT trigger the rule)
const alert = myCustomLib.customAlert;
alert("custom alert");

function alert(message) {
    console.log(message);
}
alert("shadowed function");

// Object methods with same names (should NOT trigger the rule)
const myObject = {
    alert: function(msg) { console.log(msg); },
    confirm: function(msg) { return true; },
    prompt: function(msg) { return "input"; }
};

myObject.alert("object method");
myObject.confirm("object method");
myObject.prompt("object method");

// Different function names (should NOT trigger the rule)
customAlert("Something happened!");

customConfirm("Are you sure?");

customPrompt("Who are you?");

// Scoped variables (should NOT trigger the rule)
function foo() {
    const alert = myCustomLib.customAlert;
    alert();
}

// Variables with same name (should NOT trigger the rule)
let prompt = "this is a string";
console.log(prompt);

// Other global functions (should NOT trigger the rule)
console.log("hello");
setTimeout(() => {}, 1000);

// Comments containing the words (should NOT trigger the rule)
// This alert should not trigger
/* confirm this works */
const msg = "prompt the user"; // not a function call