
use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "PlaceOrder WebSocket REST API", description = "PlaceOrder WebSocket API Endpoints")
    ),
    info(
        title = "PlaceOrder WebSocket API",
        description = "PlaceOrder WebSocket API Endpoints",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
)]
pub struct ApiDoc {}