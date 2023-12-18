This module is a simple implementation of hierarchical [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection),
inspired by [Angular](https://angular.io/guide/hierarchical-dependency-injection),
based on [this talk](https://www.youtube.com/watch?v=6o5zaKHedTE) by Polymer/Lit
engineer Justin Fagnani, and adapted from [this StackBlitz experiment](https://stackblitz.com/edit/vitejs-vite-6tenu8?file=src%2Fapp.ts).

### Design Goals

* Enable [dependency inversion](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
  by declaring dependencies as abstract interfaces which can be swapped out with
  different concrete implementations without necessitating downstream code
  changes
* Enable bottom-up _injection_ of dependencies (i.e., descendants can inject
  dependencies provided by ancestors)
* Enable top-down _querying_ for dependencies (i.e., ancestors can query for
  dependencies provided by descendants)
* Enable these features through [water-tight](https://en.wikipedia.org/wiki/Leaky_abstraction),
  intuitive abstractions

### Core Concepts

* Abstract dependencies are represented by a "token," which serves as a runtime
  representation of a dependency described by a TypeScript interface
* Dependency tokens are "provided" by custom elements annotated with a `provide`
  decorator
* Dependencies can be "injected" by descendants of a provider (using the
  `inject` decorator to annotate a custom element property) or "queried" by
  ancestors of a provider (using the `queryProviders` decorator)

### Implementation

These features are implemented through an exchange of [`CustomEvent`](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent)
extensions.

* Providers dispatch a `DependencyProvision` event just before their
  `connectedCallback` method is invoked, and a `ProviderRemoval` event just
  after `disconnectedCallback`.
* Elements with `queryProviders`-decorated properties listen for
  `DependencyProvision` and `ProviderRemoval` events, updating the decorated
  property's array accordingly.
* Elements with `inject`-decorated properties dispatch an `InjectionRequest`
  event just before their `connectedCallback` method is invoked. Providers
  listen for this event. If they provide a token matching the one specified
  by the `inject` decorator, the provider will:

  - Mutate the event's `detail` property to insert the provided `value`
  - `stopImmediatePropagation` of the event to prevent it from bubbling further
    up the DOM tree

  Because JavaScript event bubbling and event listener invocations happen
  synchronously, the injector can check the `detail` property of the dispatched
  event immediately after dispatching it to retrieve the injected value. This
  value is then assigned to a field returned by the decorated property's `get`
  accessor.
