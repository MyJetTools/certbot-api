# HTTP Actions Design Pattern

This document describes the HTTP action architecture used in this project. Follow this pattern when creating new HTTP endpoints or building similar projects.

## Overview

HTTP actions are organized using a controller-based architecture where each action is:
- A self-contained struct with its own route, input model, and handler
- Registered through a centralized builder
- Automatically documented via Swagger/OpenAPI through macro annotations
- Separated from business logic (which lives in `scripts/`)

## Architecture Components

### 1. Directory Structure

```
src/
├── http/
│   ├── mod.rs                 # HTTP module exports
│   ├── builder.rs             # Controller registration
│   ├── start_up.rs            # HTTP server initialization
│   ├── errors.rs              # HTTP error types (if needed)
│   └── controllers/
│       ├── mod.rs             # Controller module exports
│       └── {controller_group}/
│           ├── mod.rs         # Group module exports
│           └── {action_name}_action.rs  # Individual action
├── scripts/                   # Business logic (called by actions)
└── app/
    └── app_ctx.rs             # Application context
```

### 2. Action Structure

Each HTTP action follows this pattern:

```rust
use std::sync::Arc;
use my_http_server::macros::*;
use my_http_server::*;
use crate::app::AppContext;

#[http_route(
    method: "GET" | "POST" | "PUT" | "DELETE",
    route: "/api/{controller}/v1/{action-name}",
    summary: "Brief summary",
    description: "Detailed description",
    controller: "ControllerName",
    input_data: "InputModelName",
    result: [
        {status_code: 200, description: "Success description", model: "OptionalModel"},
        {status_code: 404, description: "Not found description"},
        {status_code: 500, description: "Error description"},
    ]
)]
pub struct ActionName {
    _app: Arc<AppContext>,
}

impl ActionName {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &ActionName,
    input_data: InputModelName,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    // Call business logic from scripts/
    let result = crate::scripts::business_function(input_data.field).await;

    match result {
        Ok(output) => {
            // Return success response
            HttpOutput::as_json(output_model).into_ok_result(true).into()
            // OR for text:
            // HttpOutput::as_text(output).into_ok_result(true).into()
        }
        Err(error) => {
            // Handle different error types
            if error.contains("not found") {
                return HttpFailResult::as_not_found(error, false).into_err();
            }
            return HttpFailResult::as_fatal_error(error).into_err();
        }
    }
}
```

### 3. Input Models

Input models use the `MyHttpInput` derive macro and specify where data comes from:

**For POST/PUT requests (body data):**
```rust
#[derive(MyHttpInput)]
pub struct AddDomainInputModel {
    #[http_body(name = "domain", description = "Domain name to add certificate for")]
    pub domain: String,

    #[http_body(name = "email", description = "Email address for certificate registration")]
    pub email: String,
}
```

**For GET requests (query parameters):**
```rust
#[derive(MyHttpInput)]
pub struct GetCertInfoInputModel {
    #[http_query(name = "domain", description = "Domain name")]
    pub domain: String,
}
```

### 4. Output Models

Output models use `Serialize`, `Deserialize`, and `MyHttpObjectStructure`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, MyHttpObjectStructure)]
pub struct CertificateInfoHttpModel {
    pub cn: String,
    pub expires: String,
}
```

### 5. Response Types

**JSON Response:**
```rust
HttpOutput::as_json(result_model).into_ok_result(true).into()
```

**Text Response:**
```rust
HttpOutput::as_text(output_string).into_ok_result(true).into()
```

### 6. Error Handling

Use appropriate error types based on the failure:

```rust
// Not Found (404)
HttpFailResult::as_not_found(error_message, false).into_err()

// Fatal Error (500)
HttpFailResult::as_fatal_error(error_message).into_err()

// Bad Request (400) - if needed
HttpFailResult::as_bad_request(error_message, false).into_err()
```

### 7. Controller Registration

Actions are registered in `src/http/builder.rs`:

```rust
use std::sync::Arc;
use my_http_server::controllers::ControllersMiddleware;
use crate::app::AppContext;

pub fn build_controllers(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    // Register POST actions
    result.register_post_action(Arc::new(
        crate::http::controllers::controller_group::ActionName::new(app.clone()),
    ));

    // Register GET actions
    result.register_get_action(Arc::new(
        crate::http::controllers::controller_group::ActionName::new(app.clone()),
    ));

    result
}
```

### 8. Module Organization

**Controller group module (`controllers/{group}/mod.rs`):**
```rust
pub mod action_name_action;
pub use action_name_action::*;
```

**Main controllers module (`controllers/mod.rs`):**
```rust
pub mod controller_group;
```

### 9. Server Startup

HTTP server is initialized in `src/http/start_up.rs`:

```rust
use std::{net::SocketAddr, sync::Arc};
use my_http_server::controllers::swagger::SwaggerMiddleware;
use my_http_server::MyHttpServer;
use crate::app::AppContext;

pub fn start(app: &Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 8000)));

    let controllers = Arc::new(super::builder::build_controllers(&app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        crate::app::APP_NAME.to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);
    http_server.start(app.app_states.clone(), my_logger::LOGGER.clone());
}
```

## Design Principles

1. **Separation of Concerns**: HTTP actions are thin wrappers that delegate to business logic in `scripts/`
2. **Type Safety**: Use strongly-typed input/output models
3. **Documentation**: All routes are auto-documented via `http_route` macro
4. **Consistency**: Follow the same pattern for all actions
5. **Error Handling**: Use appropriate HTTP status codes and error types
6. **Modularity**: Group related actions in controller modules

## Creating a New Action

1. **Create the action file**: `src/http/controllers/{group}/{action_name}_action.rs`
2. **Define the action struct** with `#[http_route]` macro
3. **Create input model** with `#[derive(MyHttpInput)]`
4. **Create output model** (if returning JSON) with `Serialize`, `Deserialize`, `MyHttpObjectStructure`
5. **Implement `handle_request`** function that calls business logic
6. **Export in module**: Add to `{group}/mod.rs`
7. **Register in builder**: Add registration call in `builder.rs`

## Example: Complete Action

See `src/http/controllers/certbot/add_domain_action.rs` for a complete POST action example.

See `src/http/controllers/certificates/get_cert_info_action.rs` for a complete GET action example.

## Dependencies

This pattern requires:
- `my_http_server` crate with macros support
- `serde` for serialization
- `tokio` for async runtime
- Application context (`AppContext`) for shared state

## Notes

- Actions receive `Arc<AppContext>` for shared application state
- The `_app` field is prefixed with `_` if not directly used in the handler
- Swagger documentation is automatically generated from `http_route` annotations
- Business logic should be implemented in `scripts/` module, not directly in actions
