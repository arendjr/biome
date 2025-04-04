import React from "react";
import { render } from "react-dom";

import { sharedFoo } from "shared";

import { Hello } from "@components/Hello";
import { bar } from "./bar";

render(<Hello name="World" />, document.getElementById("root"));
