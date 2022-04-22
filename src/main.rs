mod stock;
mod portfolio;
mod settings;

use chrono::Utc;

use crate::portfolio::Portfolio;


#[tokio::main]
async fn main() {
    let portfolio = Portfolio::from(
        vec![
            ("GOOG", 0.30), 
            ("AAPL", 0.50), 
            ("AMZN", 0.20)
        ],
        1000.0)
        .await
        .unwrap();
    
    let from = Utc::now() - chrono::Duration::days(200);
    let to = Utc::now();

    let profit = portfolio.profit(from, to).await;
    println!("profit: ${:.2}", profit);

    let annualized_profit = portfolio.annualized_profit(from, to).await;
    println!("annualized profit: ${:.2}", annualized_profit);
}
