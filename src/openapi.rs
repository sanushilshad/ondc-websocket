
use utoipa::OpenApi;
use utoipauto::utoipauto;

#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "ONDC WebSocket REST API", description = "ONDC WebSocket API Endpoints")
    ),
    info(
        title = "ONDC WebSocket API",
        description = "ONDC WebSocket API Endpoints",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
)]
pub struct ApiDoc {}