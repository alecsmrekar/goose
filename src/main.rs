use goose::prelude::*;

async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let goose = user.get("bahn").await.expect("TODO: panic message");
    println!("bahn {}", goose.request.response_time.to_string());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions").register_transaction(transaction!(loadtest_index)),
        )
        .execute()
        .await?;

    Ok(())
}
