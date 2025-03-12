Pet project under development.

Crates:
    libs:
        lib-core: data management, tokio tasks, etc.
        lib-dto: data models
        lib-load: load functionality for tests and benchmarks
        lib-utils: utils
        lib-web: axum app, handlers, middleware
    services:
        auth-server: used for emulating OAUTH2 authentication
        load-server: for tests and benchmarks
        web-server: main server for serving requests
    tests:
        it: integration tests

The main functionality can be tested by running test: crates/tests/it/src/dev/web/scenario.rs