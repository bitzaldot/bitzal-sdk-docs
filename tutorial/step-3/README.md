# Step 3

In this step, we will expand our barrel to live in a basic runtime. We will do this one step at a
time, and initially our runtime is not compiling to wasm.

- [Step 3](#step-3)
		- [State Transition, Background Knowledge](#state-transition-background-knowledge)
		- [In the code](#in-the-code)

### State Transition, Background Knowledge

* The main realization of this section is that a runtime is really just what we built in the
  previous section for tests. The main differences are:

* We use some of the types generated by `construct_runtime!` macro, to create `Executive`.
* We use this `Executive` to fulfill a a set of trait implementations, wrapped in
  `impl_runtime_apis`.
* This macro will essentially mean that these trait functions are "exported functions" from our wasm
  function, such that the client can call into them.

### In the code

- the implementation of `frame_system::Config` for `Runtime` should be simple:
  - Most items are the same as before.
  - We use a real `RuntimeVersion`. See its docs for more info.
  - we use a real account id type given to us by `runtime_types_common`.
- we bring in our sibling `frame-barrels` crate that contain the currency barrel we wrote.
  - implement its config as well for runtime.
- to create an `Executive`, we need primitive types like `Block` and such to be defined. Again,
  `runtime_types_common` gives us one examples set of these that are reasonable be use. You would be
  free to to build custom versions of this if need be.
- Finally, you will see that a number of traits are implemented at the bottom of the file, all
  wrapped in `impl_runtime_apis!`. See the reference docs section on client/runtime communication,
  and runtime APIs for more info.
