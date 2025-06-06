/* should not generate diagnostics */

switch(foo) { case 1: doSomething(); break; case 2: doSomething(); }

function bar(foo) { switch(foo) { case 1: doSomething(); return; case 2: doSomething(); } }

switch(foo) { case 1: doSomething(); throw new Error("Boo!"); case 2: doSomething(); }

switch(foo) { case 1: case 2: doSomething(); }

switch(foo) { case 0:   /* with comments */   case 1: b(); }

switch(foo) { case 1: { doSomething(); break; } case 2: doSomething(); }

function f(input) {
    const a = input * 2;
    switch(a) {
        case 0: {
            if (foo) {
                return 0;
            } else {
                return 1;
            }
        }
        case 2: {
            while(a > 0) {
                a--;
                if (a === DEFAULT) {
                    break;
                }
            }
            if (a < 0) {
                return a;
            } else {
                return 0;
            }
        }
        default:
            f();
    }
}

function f() {
	switch (a) {
		case 0:
			switch (b) {
				case 0:
					return 0;
				default:
					return 1;
			}
		case 2:
		// do nothing
	}
}

function f() {
	switch (a) {
		case 0:
			try {
				return foo();
			} catch {
				return 0;
			} finally {
			}
		case 1:
		// do nothing
	}
}

function f() {
	switch (a) {
		case 0:
			try {
				foo();
			} catch {
			} finally {
				return 0;
			}
		case 1:
		// do nothing
	}
}

function f() {
	switch (a) {
		case 0:
			try {
				foo();
			} finally {
				return 0;
			}
		case 1:
		// do nothing
	}
}
