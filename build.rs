use schema_rust::build::ContractCrateBuild;

fn main() {
    ContractCrateBuild::from_environment(
        "signal-system",
        "0.2.0",
        "SIGNAL_SYSTEM_UPDATE_SCHEMA_ARTIFACTS",
    )
    .expect_fresh();
}
