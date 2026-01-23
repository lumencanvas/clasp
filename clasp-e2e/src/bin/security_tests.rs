//! Standalone Security Model Test Runner

use clasp_e2e::tests;
use clasp_e2e::TestSuite;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    println!("Running Security Model Tests...\n");

    let mut suite = TestSuite::new();
    tests::security::run_tests(&mut suite).await;
    suite.print_summary();

    std::process::exit(if suite.all_passed() { 0 } else { 1 });
}
