use async_trait::async_trait;

use storage_testing::harness::support;
use storage_testing::tests::{HarnessContext, HarnessTest};

pub struct ImageLoopSetupValidImage;

#[async_trait]
impl HarnessTest for ImageLoopSetupValidImage {
    fn id(&self) -> &'static str {
        "image.loop_setup.valid_image"
    }

    fn suite(&self) -> &'static str {
        "image"
    }

    fn required_spec(&self) -> &'static str {
        "2disk"
    }

    async fn execute(&self, _ctx: &HarnessContext) -> storage_testing::errors::Result<()> {
        if support::env("STORAGE_TESTING_ENABLE_IMAGE_TESTS").as_deref() != Some("1") {
            return support::skip("set STORAGE_TESTING_ENABLE_IMAGE_TESTS=1 to run image loop setup");
        }
        let client = support::image_client().await?;
        let image_path = support::env("STORAGE_TESTING_IMAGE_SOURCE_PATH")
            .or_else(|| support::env("STORAGE_TESTING_IMAGE_PATH"))
            .ok_or_else(|| storage_testing::errors::TestingError::TestSkipped {
                reason: "set STORAGE_TESTING_IMAGE_SOURCE_PATH or STORAGE_TESTING_IMAGE_PATH"
                    .to_string(),
            })?;
        let _ = client.loop_setup(&image_path).await;
        Ok(())
    }
}
