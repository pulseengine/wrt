use anyhow::Result;
use dagger_sdk::Query;
use tracing::info;

use crate::dagger_pipelines::docs_pipeline::check_docs_strict_pipeline;

#[tracing::instrument(name = "check_docs_strict_task", skip_all, err)]
pub async fn check_docs_strict(client: &Query) -> Result<()> {
    info!("Executing strict documentation check via Dagger pipeline...");
    check_docs_strict_pipeline(client).await
}
