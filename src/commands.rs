
use secrecy::ExposeSecret;

use crate::utils::{ generate_jwt_token_for_user, get_configuration};




#[tracing::instrument(name = "Generate user token")]
pub async fn generate_user_token(username: &str) {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let token = generate_jwt_token_for_user(username, configuration.secret.jwt.expiry, &configuration.secret.jwt.secret).map_err(|e| anyhow::anyhow!("JWT generation error: {}", e));

    eprint!("Token for {} is: {}", username, token.unwrap().expose_secret())
}



#[tracing::instrument(name = "Run custom command")]
pub async fn run_custom_commands(args: Vec<String>) -> Result<(), anyhow::Error> {
    if args.len() > 1 {
        if args[1] == "generate_token" && args.len() > 2 {
            generate_user_token(&args[2]).await;
        }
    } else {
        eprintln!("Invalid command. Please enter a valid command.");
    }

    Ok(())
}
