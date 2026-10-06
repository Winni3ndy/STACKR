use dotenvy::dotenv;
use tracing::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize structured logging
    stackr::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("create-operator") => create_operator_cmd(&args[2..]).await,
        Some("generate-api-key") => generate_api_key_cmd(&args[2..]).await,
        Some(other) => {
            eprintln!("Unknown command: {}", other);
            eprintln!("Usage:");
            eprintln!("  cargo run                     Start the server");
            eprintln!("  cargo run -- create-operator   Create a new operator");
            eprintln!("  cargo run -- generate-api-key  Generate an API key for an operator");
            std::process::exit(1);
        }
        None => {
            info!("Starting Stackr USSD settlement server");

            let app_state = stackr::build_app_state().await?;
            let bind_addr = app_state.config.bind_address();

            info!(addr = %bind_addr, "Listening");

            stackr::run_server(app_state, &bind_addr).await?;

            Ok(())
        }
    }
}

async fn create_operator_cmd(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 12 {
        eprintln!(
            "Usage: cargo run -- create-operator <slug> <name> <fiat_currency> <fiat_country> \\"
        );
        eprintln!("         <wallet_master_seed> <wallet_encryption_key> <fee_payer_secret> \\");
        eprintln!("         <usdc_asset_code> <usdc_issuer> <stellar_horizon_url> \\");
        eprintln!("         <stellar_network_passphrase> <anchor_domain>");
        std::process::exit(1);
    }

    let state = stackr::build_app_state().await?;

    let params = stackr::operator::cli::CreateOperatorParams {
        slug: args[0].clone(),
        name: args[1].clone(),
        fiat_currency: args[2].clone(),
        fiat_country: args[3].clone(),
        wallet_master_seed: args[4].clone(),
        wallet_encryption_key: args[5].clone(),
        fee_payer_secret: args[6].clone(),
        usdc_asset_code: args[7].clone(),
        usdc_issuer: args[8].clone(),
        stellar_horizon_url: args[9].clone(),
        stellar_network_passphrase: args[10].clone(),
        anchor_domain: args[11].clone(),
    };

    let (operator_id, api_key) = stackr::operator::cli::create_operator(&state, &params).await?;

    println!("Operator created successfully!");
    println!("  ID:      {}", operator_id);
    println!("  Slug:    {}", args[0]);
    println!("  API Key: {}", api_key);
    println!();
    println!("IMPORTANT: Save this API key now. It cannot be retrieved later.");

    Ok(())
}

async fn generate_api_key_cmd(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- generate-api-key <operator_id> <label>");
        std::process::exit(1);
    }

    let operator_id: uuid::Uuid = args[0].parse()?;
    let label = &args[1];

    let state = stackr::build_app_state().await?;

    let api_key =
        stackr::operator::cli::generate_api_key_for_operator(&state, operator_id, label).await?;

    println!("API key generated successfully!");
    println!("  Operator: {}", operator_id);
    println!("  Label:    {}", label);
    println!("  API Key:  {}", api_key);
    println!();
    println!("IMPORTANT: Save this API key now. It cannot be retrieved later.");

    Ok(())
}
