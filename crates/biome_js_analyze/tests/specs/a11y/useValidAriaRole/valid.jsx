/* should not generate diagnostics */
<>
  <div role="button"></div>
  <div role={role}></div>
  <div role="switch" />
  <div role></div>
  <div></div>
  <div role={role || "button"} />
  <Bar baz />
  <Foo role="button" />
  <div role={role || "foobar"} />
  <div role="switch row" />
  <div />
  <div role="tabpanel row" />
</>